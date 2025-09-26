//! tui-test runner CLI

use clap::{Parser, Subcommand};
use aw_test_scenarios::{Scenario, Step};
use aw_rest_client_mock::MockClient;
use aw_tui::{execute_scenario, TestRuntime};
use std::sync::Arc;
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[command(name = "tui-test")]
#[command(about = "TUI testing runner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the TUI against a scenario file and validate assertions
    Run {
        /// Path to JSON/YAML scenario
        scenario_path: String,
        /// Start from step index
        #[arg(long)]
        start_step: Option<usize>,
        /// Stop after step index (inclusive)
        #[arg(long)]
        until_step: Option<usize>,
        /// Update golden snapshots instead of asserting
        #[arg(long)]
        update_snapshots: bool,
        /// Override terminal width
        #[arg(long)]
        terminal_width: Option<u16>,
        /// Override terminal height
        #[arg(long)]
        terminal_height: Option<u16>,
        /// Seed for randomized components
        #[arg(long)]
        seed: Option<u64>,
        /// Write JSON report to path
        #[arg(long)]
        report: Option<String>,
    },
    /// Interactive player for stepping through a scenario
    Play {
        /// Path to JSON/YAML scenario
        scenario_path: String,
        /// Start from step index
        #[arg(long)]
        start_step: Option<usize>,
        /// Jump to step index
        #[arg(long)]
        jump: Option<usize>,
        /// Run without opening a real TTY (uses TestBackend)
        #[arg(long)]
        headless: bool,
        /// Record per-step VM state and buffer
        #[arg(long)]
        trace: bool,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { scenario_path, .. } => {
            let data = std::fs::read_to_string(&scenario_path)?;
            let scenario = Scenario::from_str(&data)?;

            // Create mock client
            let client = Arc::new(MockClient::from_scenario_name(&scenario.name));

            // Execute the scenario
            match execute_scenario(client, &scenario).await {
                Ok(()) => {
                    println!("✅ Scenario '{}' executed successfully", scenario.name);
                }
                Err(e) => {
                    eprintln!("❌ Scenario '{}' failed: {}", scenario.name, e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Play {
            scenario_path,
            start_step,
            jump,
            headless: _,
            trace,
        } => {
            let data = std::fs::read_to_string(&scenario_path)?;
            let scenario = Scenario::from_str(&data)?;

            // Create mock client
            let client = Arc::new(MockClient::from_scenario_name(&scenario.name));

            // Initialize test runtime
            let (width, height) = scenario
                .terminal
                .as_ref()
                .map(|t| (t.width.unwrap_or(80), t.height.unwrap_or(24)))
                .unwrap_or((80, 24));

            let mut runtime = TestRuntime::new(client.clone(), width, height);

            // Load initial data
            runtime.load_initial_data().await.map_err(|e| anyhow::anyhow!("Failed to load initial data: {}", e))?;

            // Set up interactive session
            let mut current_step = start_step.unwrap_or(0);
            if let Some(jump_to) = jump {
                current_step = jump_to.min(scenario.steps.len());
            }

            println!("🎬 Interactive Scenario Player");
            println!("==============================");
            println!("Scenario: {}", scenario.name);
            println!("Terminal: {}x{}", width, height);
            println!("Steps: {}", scenario.steps.len());
            println!();

            // Freeze time for deterministic execution
            tokio::time::pause();

            // Interactive loop
            run_interactive_player(&scenario, &mut runtime, current_step, trace).await?;
        }
    }
    Ok(())
}

/// Run the interactive scenario player
async fn run_interactive_player(
    scenario: &Scenario,
    runtime: &mut TestRuntime<MockClient>,
    mut current_step: usize,
    trace: bool,
) -> anyhow::Result<()> {
    let mut executed_steps = 0; // Track how many steps we've actually executed

    loop {
        // Show current state
        show_current_state(scenario, runtime, current_step, executed_steps, trace);

        // Show commands
        println!();
        println!("Commands:");
        println!("  n/next       - Execute next step");
        println!("  p/prev       - Go back one step (reset state)");
        println!("  j <n>/jump <n> - Jump to step <n>");
        println!("  r/reset      - Reset to beginning");
        println!("  s/show       - Show current TUI buffer");
        println!("  v/view       - Show detailed ViewModel state");
        println!("  q/quit       - Exit");
        print!("> ");
        io::stdout().flush()?;

        // Read command
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "n" | "next" => {
                if current_step < scenario.steps.len() {
                    // Execute the current step
                    match runtime.execute_step(&scenario.steps[current_step]).await {
                        Ok(()) => {
                            executed_steps += 1;
                            current_step += 1;
                            println!("✅ Step {} executed successfully", current_step);
                        }
                        Err(e) => {
                            println!("❌ Step {} failed: {}", current_step + 1, e);
                            // Don't advance if step failed
                        }
                    }
                } else {
                    println!("🎯 End of scenario reached!");
                }
            }
            "p" | "prev" => {
                if executed_steps > 0 {
                    // Reset and replay up to previous step
                    executed_steps = executed_steps.saturating_sub(1);
                    current_step = executed_steps;
                    reset_and_replay(scenario, runtime, executed_steps).await?;
                    println!("↶ Reset to step {}", executed_steps);
                } else {
                    println!("⚠️  Already at beginning");
                }
            }
            "r" | "reset" => {
                executed_steps = 0;
                current_step = 0;
                reset_and_replay(scenario, runtime, 0).await?;
                println!("🔄 Reset to beginning");
            }
            "s" | "show" => {
                show_tui_buffer(runtime);
            }
            "v" | "view" => {
                show_viewmodel_details(runtime);
            }
            "q" | "quit" => {
                println!("👋 Goodbye!");
                break;
            }
            input => {
                // Check if it's a jump command
                if let Some(cmd) = input.strip_prefix("j ") {
                    if let Ok(step_num) = cmd.parse::<usize>() {
                        if step_num <= scenario.steps.len() {
                            executed_steps = step_num;
                            current_step = step_num;
                            reset_and_replay(scenario, runtime, step_num).await?;
                            println!("⏭️  Jumped to step {}", step_num);
                        } else {
                            println!("❌ Invalid step number: {}", step_num);
                        }
                    } else {
                        println!("❌ Invalid jump command. Use: j <number>");
                    }
                } else if let Some(cmd) = input.strip_prefix("jump ") {
                    if let Ok(step_num) = cmd.parse::<usize>() {
                        if step_num <= scenario.steps.len() {
                            executed_steps = step_num;
                            current_step = step_num;
                            reset_and_replay(scenario, runtime, step_num).await?;
                            println!("⏭️  Jumped to step {}", step_num);
                        } else {
                            println!("❌ Invalid step number: {}", step_num);
                        }
                    } else {
                        println!("❌ Invalid jump command. Use: jump <number>");
                    }
                } else {
                    println!("❌ Unknown command: {}", input);
                }
            }
        }

        println!();
    }

    Ok(())
}

/// Reset the runtime and replay steps up to the specified step
async fn reset_and_replay(
    scenario: &Scenario,
    runtime: &mut TestRuntime<MockClient>,
    up_to_step: usize,
) -> anyhow::Result<()> {
    // Create fresh runtime
    let (width, height) = scenario
        .terminal
        .as_ref()
        .map(|t| (t.width.unwrap_or(80), t.height.unwrap_or(24)))
        .unwrap_or((80, 24));

    let client = Arc::new(MockClient::from_scenario_name(&scenario.name));
    *runtime = TestRuntime::new(client, width, height);

    // Reload initial data
    runtime.load_initial_data().await.map_err(|e| anyhow::anyhow!("Failed to load initial data: {}", e))?;

    // Replay steps up to the target
    for i in 0..up_to_step {
        runtime.execute_step(&scenario.steps[i]).await.map_err(|e| anyhow::anyhow!("Failed to execute step {}: {}", i, e))?;
    }

    Ok(())
}

/// Show current state information
fn show_current_state(
    scenario: &Scenario,
    runtime: &TestRuntime<MockClient>,
    current_step: usize,
    executed_steps: usize,
    trace: bool,
) {
    println!("📊 Current State");
    println!("===============");
    println!("Step: {}/{} (executed: {})", current_step, scenario.steps.len(), executed_steps);

    if current_step < scenario.steps.len() {
        println!("Next: {}", describe_step(&scenario.steps[current_step]));
    } else {
        println!("Next: [END OF SCENARIO]");
    }

    // Show ViewModel summary
    let vm = runtime.view_model();
    println!("Focus: {}", vm.focus_string());
    if let Some(selected) = vm.selected_item_name() {
        println!("Selected: {}", selected);
    }

    if trace {
        println!("Buffer size: {} chars", runtime.buffer_content().len());
    }
}

/// Describe a step in human-readable format
fn describe_step(step: &Step) -> String {
    match step {
        Step::AdvanceMs { ms } => format!("Advance {}ms", ms),
        Step::Key { key } => format!("Press key '{}'", key),
        Step::Sse { .. } => "Receive SSE event".to_string(),
        Step::AssertVm { focus, selected } => format!("Assert focus='{}' selected={:?}", focus, selected),
        Step::Snapshot { name } => format!("Take snapshot '{}'", name),
    }
}

/// Show the current TUI buffer in a simplified text format
fn show_tui_buffer(runtime: &TestRuntime<MockClient>) {
    println!("🖥️  TUI Buffer");
    println!("============");

    let buffer = runtime.buffer_content();
    let lines: Vec<&str> = buffer.split('\n').collect();

    // Show first 20 lines with line numbers
    for (i, line) in lines.iter().enumerate().take(20) {
        if line.trim().is_empty() {
            continue;
        }
        println!("{:2}: {}", i + 1, line);
    }

    if lines.len() > 20 {
        println!("... ({} more lines)", lines.len() - 20);
    }
}

/// Show detailed ViewModel state
fn show_viewmodel_details(runtime: &TestRuntime<MockClient>) {
    println!("🔍 ViewModel Details");
    println!("===================");

    let vm = runtime.view_model();

    println!("Focus: {}", vm.focus_string());
    println!("Editor Height: {}", vm.editor_height);
    println!("Loading: {}", vm.loading);
    if let Some(error) = &vm.error_message {
        println!("Error: {}", error);
    }

    println!();
    println!("Projects ({}):", vm.visible_projects.len());
    for (i, project) in vm.visible_projects.iter().enumerate() {
        let marker = if i == vm.selected_project { "▶" } else { " " };
        println!("  {} {}", marker, project.display_name);
    }

    println!();
    println!("Repositories ({}):", vm.visible_repositories.len());
    for (i, repo) in vm.visible_repositories.iter().enumerate() {
        let marker = if i == vm.selected_repository { "▶" } else { " " };
        println!("  {} {}", marker, repo.display_name);
    }

    println!();
    println!("Agents ({}):", vm.visible_agents.len());
    for (i, agent) in vm.visible_agents.iter().enumerate() {
        let marker = if i == vm.selected_agent { "▶" } else { " " };
        println!("  {} {}", marker, agent.agent_type);
    }

    if !vm.task_description.is_empty() {
        println!();
        println!("Task Description:");
        println!("  {}", vm.task_description.replace('\n', "\n  "));
    }
}
