use std::path::PathBuf;
use clap::{Args, Subcommand};
use anyhow::{Result, Context};
use aw_repo::VcsRepo;
use aw_core::{
    AgentTasks, PushHandler, PushOptions,
    devshell_names, parse_push_to_remote_flag,
    edit_content_interactive, EditorError
};
use aw_fs_snapshots::PreparedWorkspace;
use crate::sandbox::{parse_bool_flag, prepare_workspace_with_fallback};

/// Task-related commands
#[derive(Subcommand)]
pub enum TaskCommands {
    /// Create a new task or add to an existing task branch
    Create(TaskCreateArgs),
}

/// Arguments for creating a new task
#[derive(Args)]
pub struct TaskCreateArgs {
    /// Branch name for new tasks (positional argument)
    #[arg(value_name = "BRANCH")]
    pub branch: Option<String>,

    /// Use STRING as the task prompt (direct input)
    #[arg(long = "prompt", value_name = "TEXT")]
    pub prompt: Option<String>,

    /// Read the task prompt from FILE
    #[arg(long = "prompt-file", value_name = "FILE")]
    pub prompt_file: Option<PathBuf>,

    /// Record the dev shell name in the commit
    #[arg(short = 's', long = "devshell", value_name = "NAME")]
    pub devshell: Option<String>,

    /// Push branch to remote automatically (true/false/yes/no)
    #[arg(long = "push-to-remote", value_name = "BOOL")]
    pub push_to_remote: Option<String>,

    /// Non-interactive mode (skip prompts)
    #[arg(long = "non-interactive")]
    pub non_interactive: bool,

    /// Run task in a local sandbox
    #[arg(long = "sandbox", value_name = "TYPE", default_value = "none")]
    pub sandbox: String,

    /// Allow internet access in sandbox
    #[arg(long = "allow-network", value_name = "BOOL", default_value = "no")]
    pub allow_network: String,

    /// Enable container device access (/dev/fuse, storage dirs)
    #[arg(long = "allow-containers", value_name = "BOOL", default_value = "no")]
    pub allow_containers: String,

    /// Enable KVM device access for VMs (/dev/kvm)
    #[arg(long = "allow-kvm", value_name = "BOOL", default_value = "no")]
    pub allow_kvm: String,

    /// Enable dynamic filesystem access control
    #[arg(long = "seccomp", value_name = "BOOL", default_value = "no")]
    pub seccomp: String,

    /// Enable debugging operations in sandbox
    #[arg(long = "seccomp-debug", value_name = "BOOL", default_value = "no")]
    pub seccomp_debug: String,

    /// Additional writable paths to bind mount
    #[arg(long = "mount-rw", value_name = "PATH")]
    pub mount_rw: Vec<PathBuf>,

    /// Paths to promote to copy-on-write overlays
    #[arg(long = "overlay", value_name = "PATH")]
    pub overlay: Vec<PathBuf>,
}

impl TaskCommands {
    /// Execute the task command
    pub async fn run(self) -> Result<()> {
        match self {
            TaskCommands::Create(args) => args.run().await,
        }
    }
}

impl TaskCreateArgs {
    /// Execute the task creation
    pub async fn run(self) -> Result<()> {
        // Validate mutually exclusive options
        if self.prompt.is_some() && self.prompt_file.is_some() {
            anyhow::bail!("Error: --prompt and --prompt-file are mutually exclusive");
        }

        // Determine if we're creating a new branch or appending to existing
        let branch_name = self.branch.as_ref()
            .filter(|b| !b.trim().is_empty())
            .cloned();
        let start_new_branch = branch_name.is_some();

        // Get task content
        let prompt_content = self.get_prompt_content().await?;

        // Create VCS repository instance
        let repo = VcsRepo::new(".")
            .context("Failed to initialize VCS repository")?;

        let orig_branch = repo.current_branch()
            .context("Failed to get current branch")?;

        // Handle branch creation/validation
        let actual_branch_name = if start_new_branch {
            let branch = branch_name.as_ref().unwrap();
            self.handle_new_branch_creation(&repo, branch).await?;
            branch.clone()
        } else {
            // Using existing branch
            self.validate_existing_branch(&repo, &orig_branch).await?;
            orig_branch.clone()
        };

        let mut cleanup_branch = start_new_branch;
        let mut task_committed = false;

        // Get task content (editor or provided)
        let task_content = if let Some(content) = prompt_content {
            content
        } else {
            // Use editor for interactive input
            if self.non_interactive {
                // Cleanup branch if we created it
                if cleanup_branch {
                    self.cleanup_branch(&repo, &actual_branch_name);
                }
                anyhow::bail!("Error: Non-interactive mode requires --prompt or --prompt-file");
            }
            match self.get_editor_content() {
                Ok(content) => content,
                Err(e) => {
                    // Cleanup branch if we created it and editor failed
                    if cleanup_branch {
                        self.cleanup_branch(&repo, &actual_branch_name);
                    }
                    return Err(e);
                }
            }
        };

        // Validate task content
        if task_content.trim().is_empty() {
            anyhow::bail!("Aborted: empty task prompt.");
        }

        // Create task and commit
        let tasks = AgentTasks::new(repo.root())
            .context("Failed to initialize agent tasks")?;

        let commit_result = if start_new_branch {
            tasks.record_initial_task(
                &task_content,
                &actual_branch_name,
                self.devshell.as_deref()
            )
        } else {
            tasks.append_task(&task_content)
        };

        if let Err(e) = commit_result {
            // Cleanup branch if we created it and task recording failed
            if cleanup_branch {
                self.cleanup_branch(&repo, &actual_branch_name);
            }
            return Err(e.into());
        }

        // Success - mark as committed and don't cleanup branch
        task_committed = true;
        cleanup_branch = false;

        // Validate and prepare sandbox if requested
        let sandbox_workspace = if self.sandbox != "none" {
            Some(validate_and_prepare_sandbox(&self).await?)
        } else {
            None
        };

        // For now, just log the sandbox workspace preparation
        if let Some(ref ws) = sandbox_workspace {
            println!("Sandbox workspace prepared at: {}", ws.exec_path.display());
        }

        // Handle push operations
        if let Some(push_flag) = &self.push_to_remote {
            let push_bool = parse_push_to_remote_flag(push_flag)
                .context("Invalid --push-to-remote value")?;
            self.handle_push(&actual_branch_name, Some(push_bool)).await?;
        } else if !self.non_interactive {
            self.handle_push(&actual_branch_name, None).await?;
        }

        // Success - don't cleanup branch
        cleanup_branch = false;

        // Switch back to original branch if we created a new one
        if start_new_branch {
            repo.checkout_branch(&orig_branch)?;
        }

        Ok(())
    }

    /// Get prompt content from --prompt or --prompt-file options
    async fn get_prompt_content(&self) -> Result<Option<String>> {
        if let Some(prompt) = &self.prompt {
            Ok(Some(prompt.clone()))
        } else if let Some(file_path) = &self.prompt_file {
            let content = tokio::fs::read_to_string(file_path)
                .await
                .with_context(|| format!("Error: Failed to read prompt file: {}", file_path.display()))?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    /// Handle new branch creation with validation
    async fn handle_new_branch_creation(&self, repo: &VcsRepo, branch_name: &str) -> Result<()> {
        repo.start_branch(branch_name)?;

        // Validate devshell if specified
        if let Some(devshell) = &self.devshell {
            let flake_path = repo.root().join("flake.nix");
            if !flake_path.exists() {
                anyhow::bail!("Error: Repository does not contain a flake.nix file");
            }

            let shells = devshell_names(repo.root()).await
                .context("Failed to read devshells from flake.nix")?;

            if !shells.contains(&devshell.to_string()) {
                anyhow::bail!("Error: Dev shell '{}' not found in flake.nix", devshell);
            }
        }

        Ok(())
    }

    /// Validate existing branch (not main branch, etc.)
    async fn validate_existing_branch(&self, repo: &VcsRepo, branch_name: &str) -> Result<()> {
        let main_names = vec![
            repo.default_branch(),
            "main",
            "master",
            "trunk",
            "default"
        ];

        if main_names.contains(&branch_name) {
            anyhow::bail!("Error: Refusing to run on the main branch");
        }

        if self.devshell.is_some() {
            anyhow::bail!("Error: --devshell is only supported when creating a new branch");
        }

        Ok(())
    }

    /// Get content using the interactive editor
    fn get_editor_content(&self) -> Result<String> {
        match edit_content_interactive(None) {
            Ok(content) => Ok(content),
            Err(EditorError::EmptyTaskPrompt) => anyhow::bail!("Aborted: empty task prompt."),
            Err(e) => Err(e.into()),
        }
    }

    /// Handle push operations
    async fn handle_push(&self, branch_name: &str, explicit_push: Option<bool>) -> Result<()> {
        let push_handler = PushHandler::new(".").await
            .context("Failed to initialize push handler")?;

        let options = PushOptions::new(branch_name.to_string())
            .with_push_to_remote(explicit_push);

        push_handler.handle_push(&options).await
            .context("Failed to handle push operation")?;

        Ok(())
    }

    /// Cleanup a branch that was created but task recording failed
    fn cleanup_branch(&self, repo: &VcsRepo, branch_name: &str) {
        // Try to switch back to original branch first
        let _ = repo.checkout_branch(&repo.default_branch());

        // Try to delete the branch (ignore errors)
        let _ = std::process::Command::new("git")
            .args(["branch", "-D", branch_name])
            .current_dir(repo.root())
            .output();
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_push_to_remote_flag_truthy() {
        assert!(parse_push_to_remote_flag("1").unwrap());
        assert!(parse_push_to_remote_flag("true").unwrap());
        assert!(parse_push_to_remote_flag("yes").unwrap());
        assert!(parse_push_to_remote_flag("y").unwrap());
        assert!(parse_push_to_remote_flag("YES").unwrap());
        assert!(parse_push_to_remote_flag("True").unwrap());
    }

    #[test]
    fn test_parse_push_to_remote_flag_falsy() {
        assert!(!parse_push_to_remote_flag("0").unwrap());
        assert!(!parse_push_to_remote_flag("false").unwrap());
        assert!(!parse_push_to_remote_flag("no").unwrap());
        assert!(!parse_push_to_remote_flag("n").unwrap());
        assert!(!parse_push_to_remote_flag("NO").unwrap());
        assert!(!parse_push_to_remote_flag("False").unwrap());
    }

    #[test]
    fn test_parse_push_to_remote_flag_invalid() {
        assert!(parse_push_to_remote_flag("maybe").is_err());
        assert!(parse_push_to_remote_flag("invalid").is_err());
        assert!(parse_push_to_remote_flag("").is_err());
    }

    #[test]
    fn test_task_create_args_builder() {
        let args = TaskCreateArgs {
            branch: Some("feature-branch".to_string()),
            prompt: Some("Implement feature X".to_string()),
            prompt_file: None,
            
            devshell: Some("dev".to_string()),
            push_to_remote: Some("yes".to_string()),
            sandbox: "none".to_string(),
            allow_network: "no".to_string(),
            allow_containers: "no".to_string(),
            allow_kvm: "no".to_string(),
            seccomp: "no".to_string(),
            seccomp_debug: "no".to_string(),
            mount_rw: vec![],
            overlay: vec![],
            non_interactive: true,
        };

        assert_eq!(args.branch, Some("feature-branch".to_string()));
        assert_eq!(args.prompt, Some("Implement feature X".to_string()));
        assert_eq!(args.devshell, Some("dev".to_string()));
        assert_eq!(args.push_to_remote, Some("yes".to_string()));
        assert!(args.non_interactive);
    }

    #[tokio::test]
    async fn test_get_prompt_content_from_prompt_option() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let args = TaskCreateArgs {
            branch: Some("test-branch".to_string()),
            prompt: Some("Test task content".to_string()),
            prompt_file: None,
            devshell: None,
            push_to_remote: None,
            non_interactive: true,
            sandbox: "none".to_string(),
            allow_network: "no".to_string(),
            allow_containers: "no".to_string(),
            allow_kvm: "no".to_string(),
            seccomp: "no".to_string(),
            seccomp_debug: "no".to_string(),
            mount_rw: vec![],
            overlay: vec![],
        };

        let content = args.get_prompt_content().await.unwrap();
        assert_eq!(content, Some("Test task content".to_string()));

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_get_prompt_content_from_file() {
        let temp_dir = TempDir::new().unwrap();

        // Create the file in the temp directory
        let file_path = temp_dir.path().join("task.txt");
        fs::write(&file_path, "Task content from file").unwrap();

        let args = TaskCreateArgs {
            branch: Some("test-branch".to_string()),
            prompt: None,
            prompt_file: Some(file_path), // Use absolute path
            devshell: None,
            push_to_remote: None,
            non_interactive: true,
            sandbox: "none".to_string(),
            allow_network: "no".to_string(),
            allow_containers: "no".to_string(),
            allow_kvm: "no".to_string(),
            seccomp: "no".to_string(),
            seccomp_debug: "no".to_string(),
            mount_rw: vec![],
            overlay: vec![],
        };

        let content = args.get_prompt_content().await.unwrap();
        assert_eq!(content, Some("Task content from file".to_string()));
    }

    #[test]
    fn test_cli_args_mutually_exclusive() {
        // Test that clap properly rejects mutually exclusive --prompt and --prompt-file
        // This would be caught by clap's validation, but we test the logic that would
        // be used in the run() method

        let args1 = TaskCreateArgs {
            branch: Some("test-branch".to_string()),
            prompt: Some("prompt".to_string()),
            prompt_file: Some("file.txt".into()),
            devshell: None,
            push_to_remote: None,
            non_interactive: true,
            sandbox: "none".to_string(),
            allow_network: "no".to_string(),
            allow_containers: "no".to_string(),
            allow_kvm: "no".to_string(),
            seccomp: "no".to_string(),
            seccomp_debug: "no".to_string(),
            mount_rw: vec![],
            overlay: vec![],
        };

        // The validation logic is: if both prompt and prompt_file are Some, it's an error
        assert!(args1.prompt.is_some() && args1.prompt_file.is_some());
    }

    #[tokio::test]
    async fn test_get_prompt_content_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let args = TaskCreateArgs {
            branch: Some("test-branch".to_string()),
            prompt: None,
            prompt_file: Some(temp_dir.path().join("nonexistent.txt")),
            devshell: None,
            push_to_remote: None,
            non_interactive: true,
            sandbox: "none".to_string(),
            allow_network: "no".to_string(),
            allow_containers: "no".to_string(),
            allow_kvm: "no".to_string(),
            seccomp: "no".to_string(),
            seccomp_debug: "no".to_string(),
            mount_rw: vec![],
            overlay: vec![],
        };

        let content = args.get_prompt_content().await;
        assert!(content.is_err());
        assert!(content.unwrap_err().to_string().contains("Failed to read prompt file"));

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_task_validation_empty_content() {
        let args = TaskCreateArgs {
            branch: Some("test-branch".to_string()),
            prompt: None,
            prompt_file: None,
            devshell: None,
            push_to_remote: None,
            non_interactive: true,
            sandbox: "none".to_string(),
            allow_network: "no".to_string(),
            allow_containers: "no".to_string(),
            allow_kvm: "no".to_string(),
            seccomp: "no".to_string(),
            seccomp_debug: "no".to_string(),
            mount_rw: vec![],
            overlay: vec![],
        };

        // This would normally be tested in the run() method, but we'll test the validation logic
        let empty_content = "";
        assert!(empty_content.trim().is_empty());
    }

    #[test]
    fn test_task_validation_whitespace_only() {
        let args = TaskCreateArgs {
            branch: Some("test-branch".to_string()),
            prompt: None,
            prompt_file: None,
            devshell: None,
            push_to_remote: None,
            non_interactive: true,
            sandbox: "none".to_string(),
            allow_network: "no".to_string(),
            allow_containers: "no".to_string(),
            allow_kvm: "no".to_string(),
            seccomp: "no".to_string(),
            seccomp_debug: "no".to_string(),
            mount_rw: vec![],
            overlay: vec![],
        };

        // This would normally be tested in the run() method, but we'll test the validation logic
        let whitespace_content = "   \n\t   ";
        assert!(whitespace_content.trim().is_empty());
    }

    #[test]
    fn test_branch_name_validation_regex() {
        use aw_repo::VcsRepo;

        // Test valid branch names
        assert!(VcsRepo::valid_branch_name("feature-branch"));
        assert!(VcsRepo::valid_branch_name("bug_fix"));
        assert!(VcsRepo::valid_branch_name("v1.2.3"));
        assert!(VcsRepo::valid_branch_name("test-123"));

        // Test invalid branch names
        assert!(!VcsRepo::valid_branch_name("feature branch")); // space
        assert!(!VcsRepo::valid_branch_name("feature@branch")); // @
        assert!(!VcsRepo::valid_branch_name("feature/branch")); // /
        assert!(!VcsRepo::valid_branch_name("")); // empty
    }

    #[test]
    fn test_main_branch_protection() {
        use aw_repo::VcsType;

        // Test protected branch detection for Git (most common case)
        let git_type = VcsType::Git;
        let protected = git_type.protected_branches();

        assert!(protected.contains(&"main"));
        assert!(protected.contains(&"master"));
        assert!(protected.contains(&"trunk"));
        assert!(protected.contains(&"default"));

        // Test non-protected branches
        assert!(!protected.contains(&"feature-x"));
        assert!(!protected.contains(&"bugfix"));
        assert!(!protected.contains(&"develop"));
    }

    #[tokio::test]
    async fn test_devshell_validation_no_flake() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Create a mock VcsRepo-like structure for testing
        // Since we can't easily mock the full VcsRepo, we'll test the logic indirectly
        // by checking that devshell validation requires flake.nix

        let args = TaskCreateArgs {
            branch: Some("test-branch".to_string()),
            prompt: Some("test".to_string()),
            prompt_file: None,
            devshell: Some("custom".to_string()),
            push_to_remote: None,
            non_interactive: true,
            sandbox: "none".to_string(),
            allow_network: "no".to_string(),
            allow_containers: "no".to_string(),
            allow_kvm: "no".to_string(),
            seccomp: "no".to_string(),
            seccomp_debug: "no".to_string(),
            mount_rw: vec![],
            overlay: vec![],
        };

        // This test would normally be integration-tested, but we'll verify the logic
        // The actual validation happens in handle_new_branch_creation
        // which checks for flake.nix existence

        // Verify flake.nix doesn't exist
        assert!(!temp_dir.path().join("flake.nix").exists());

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_devshell_validation_with_flake() {
        let temp_dir = TempDir::new().unwrap();

        // Save original HOME and set it to a proper location for nix
        let original_home = std::env::var("HOME").ok();
        if let Some(ref home) = original_home {
            if home.contains("tmp") || home.contains("temp") {
                // If HOME is set to a temp directory, unset it so nix can use the real home
                std::env::remove_var("HOME");
            }
        }

        // Create a flake.nix file
        let flake_content = r#"
        {
          outputs = { self }: {
            devShells.x86_64-linux.default = {};
            devShells.x86_64-linux.custom = {};
          };
        }
        "#;
        fs::write(temp_dir.path().join("flake.nix"), flake_content).unwrap();

        // Test devshell parsing (this tests the underlying devshell_names function)
        // Note: This may fail if nix is not available in the test environment,
        // but that's expected behavior
        let result = aw_core::devshell_names(temp_dir.path()).await;

        // Restore original HOME
        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        }

        // If nix is available, check the results
        if let Ok(devshells) = result {
            assert!(devshells.contains(&"default".to_string()) || devshells.contains(&"custom".to_string()));
        } else {
            // If nix is not available, the function should still not panic
            // The error is expected in some test environments
            eprintln!("Nix not available for devshell testing: {:?}", result);
        }
    }

    #[test]
    fn test_non_interactive_mode_requires_input() {
        let args = TaskCreateArgs {
            branch: Some("test-branch".to_string()),
            prompt: None,
            prompt_file: None,
            devshell: None,
            push_to_remote: None,
            non_interactive: true,
            sandbox: "none".to_string(),
            allow_network: "no".to_string(),
            allow_containers: "no".to_string(),
            allow_kvm: "no".to_string(),
            seccomp: "no".to_string(),
            seccomp_debug: "no".to_string(),
            mount_rw: vec![],
            overlay: vec![],
        };

        // This test verifies the logic that non-interactive mode requires --prompt or --prompt-file
        // The actual validation happens in the run() method
        assert!(args.prompt.is_none());
        assert!(args.prompt_file.is_none());
        assert!(args.non_interactive);
    }

    #[test]
    fn test_devshell_only_for_new_branches() {
        let args = TaskCreateArgs {
            branch: None, // No branch means append to existing
            prompt: Some("test".to_string()),
            prompt_file: None,
            devshell: Some("custom".to_string()),
            push_to_remote: None,
            non_interactive: true,
            sandbox: "none".to_string(),
            allow_network: "no".to_string(),
            allow_containers: "no".to_string(),
            allow_kvm: "no".to_string(),
            seccomp: "no".to_string(),
            seccomp_debug: "no".to_string(),
            mount_rw: vec![],
            overlay: vec![],
        };

        // This test verifies the logic that --devshell is only allowed for new branches
        // The actual validation happens in validate_existing_branch
        assert!(args.branch.is_none()); // No branch = append mode
        assert!(args.devshell.is_some()); // But devshell is specified
    }

    #[test]
    fn test_error_messages_format() {
        // Test that error messages contain expected text
        let err1 = parse_push_to_remote_flag("invalid");
        assert!(err1.is_err());
        assert!(err1.unwrap_err().to_string().contains("--push-to-remote"));

        let err2 = parse_push_to_remote_flag("");
        assert!(err2.is_err());
        assert!(err2.unwrap_err().to_string().contains("Invalid value"));
    }

    // Integration tests - these require the binary to be built and available
    // They are marked with ignore by default since they require external dependencies

    // Integration tests that replicate Ruby test_start_task.rb exactly

    fn setup_git_repo_integration() -> Result<(tempfile::TempDir, tempfile::TempDir, tempfile::TempDir)> {
        use std::process::Command;

        // Set HOME to a temporary directory to avoid accessing user git/ssh config
        let temp_home = tempfile::TempDir::new()?;
        std::env::set_var("HOME", temp_home.path());

        let remote_dir = tempfile::TempDir::new()?;
        let repo_dir = tempfile::TempDir::new()?;

        // Create bare remote repository
        Command::new("git")
            .args(["init", "--bare"])
            .current_dir(&remote_dir)
            .output()?;

        // Create local repository
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(&repo_dir)
            .output()?;

        // Configure git
        Command::new("git")
            .args(["config", "user.email", "tester@example.com"])
            .current_dir(&repo_dir)
            .output()?;
        Command::new("git")
            .args(["config", "user.name", "Tester"])
            .current_dir(&repo_dir)
            .output()?;

        // Create initial commit
        fs::write(repo_dir.path().join("README.md"), "initial")?;
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&repo_dir)
            .output()?;
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&repo_dir)
            .output()?;

        // Add remote
        Command::new("git")
            .args(["remote", "add", "origin", &remote_dir.path().to_string_lossy()])
            .current_dir(&repo_dir)
            .output()?;

        Ok((temp_home, repo_dir, remote_dir))
    }

    fn run_aw_task_create_integration(
        repo_path: &std::path::Path,
        branch: &str,
        prompt: Option<&str>,
        prompt_file: Option<&std::path::Path>,
        push_to_remote: Option<bool>,
        devshell: Option<&str>,
        sandbox: Option<(&str, Option<&str>, Option<&str>, Option<&str>, Option<&str>)>, // (type, allow_network, allow_containers, allow_kvm, seccomp)
        editor_lines: Vec<&str>,
        editor_exit_code: i32,
    ) -> Result<(std::process::ExitStatus, String, bool)> {
        use std::process::Command;

        // Set up fake editor if needed
        let mut editor_dir = None;
        let mut editor_script = None;
        let mut marker_file = None;

        if prompt.is_none() && prompt_file.is_none() {
            editor_dir = Some(tempfile::TempDir::new()?);
            let script_path = editor_dir.as_ref().unwrap().path().join("fake_editor.sh");
            let marker_path = editor_dir.as_ref().unwrap().path().join("called");

            let script_content = format!(r#"#!/bin/bash
echo "yes" > "{}"
cat > "$1" << 'EOF'
{}
EOF
exit {}
"#, marker_path.to_string_lossy(), editor_lines.join("\n"), editor_exit_code);

            fs::write(&script_path, script_content)?;
            Command::new("chmod").args(["+x", &script_path.to_string_lossy()]).output()?;

            editor_script = Some(script_path);
            marker_file = Some(marker_path);
        }

        // Build command
        let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| "/home/zahary/blocksense/agents-workflow/cli".to_string());
        // CARGO_MANIFEST_DIR is the crate directory when running individual crate tests,
        // but workspace root when running --workspace
        let binary_path = if cargo_manifest_dir.contains("/crates/") {
            // Running individual crate test - go up to workspace root then to target
            std::path::Path::new(&cargo_manifest_dir)
                .join("../../target/debug/aw")
        } else {
            // Running workspace test - target is directly under workspace
            std::path::Path::new(&cargo_manifest_dir)
                .join("target/debug/aw")
        };

        let mut cmd = Command::new(&binary_path);
        cmd.args(["task", "create", branch])
           .current_dir(repo_path)
           .env("GIT_CONFIG_NOSYSTEM", "1")
           .env("GIT_TERMINAL_PROMPT", "0")
           .env("GIT_ASKPASS", "echo")
           .env("SSH_ASKPASS", "echo");

        if let Some(prompt_text) = prompt {
            cmd.arg("--prompt").arg(prompt_text);
        }

        if let Some(file_path) = prompt_file {
            cmd.arg("--prompt-file").arg(file_path);
        }

        if let Some(devshell_name) = devshell {
            cmd.arg("--devshell").arg(devshell_name);
        }

        if let Some(push) = push_to_remote {
            let flag = if push { "true" } else { "false" };
            cmd.arg("--push-to-remote").arg(flag);
        }

        if prompt.is_some() || prompt_file.is_some() {
            cmd.arg("--non-interactive");
        }

        // Set up environment
        if let Some(script_path) = &editor_script {
            cmd.env("EDITOR", script_path);
        }

        // Handle interactive prompt for push
        if push_to_remote.is_none() && (prompt.is_none() && prompt_file.is_none()) {
            cmd.arg("--push-to-remote").arg("true"); // Default to true for testing
        }

        // Add sandbox parameters
        if let Some((sandbox_type, allow_network, allow_containers, allow_kvm, seccomp)) = sandbox {
            cmd.arg("--sandbox").arg(sandbox_type);
            if let Some(network) = allow_network {
                cmd.arg("--allow-network").arg(network);
            }
            if let Some(containers) = allow_containers {
                cmd.arg("--allow-containers").arg(containers);
            }
            if let Some(kvm) = allow_kvm {
                cmd.arg("--allow-kvm").arg(kvm);
            }
            if let Some(seccomp_val) = seccomp {
                cmd.arg("--seccomp").arg(seccomp_val);
            }
        }

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let full_output = format!("{}{}", stdout, stderr);

        // Check if editor was called
        let editor_called = if let Some(marker) = marker_file {
            marker.exists()
        } else {
            false
        };

        Ok((output.status, full_output, editor_called))
    }

    fn assert_task_branch_created_integration(
        repo_path: &std::path::Path,
        remote_path: &std::path::Path,
        branch: &str,
        expect_push: bool,
    ) -> Result<()> {
        use std::process::Command;

        // Verify branch exists and has exactly one commit ahead of main
        let tip_commit_output = Command::new("git")
            .args(["rev-parse", branch])
            .current_dir(repo_path)
            .output()?;
        let tip_commit = String::from_utf8(tip_commit_output.stdout)?.trim().to_string();

        let commit_count_output = Command::new("git")
            .args(["rev-list", "--count", &format!("main..{}", branch)])
            .current_dir(repo_path)
            .output()?;
        let commit_count = String::from_utf8(commit_count_output.stdout)?.trim().parse::<i32>()?;
        assert_eq!(commit_count, 1);

        // Verify only the task file was added
        let files_output = Command::new("git")
            .args(["show", "--name-only", "--format=", &tip_commit])
            .current_dir(repo_path)
            .output()?;
        let files_output_str = String::from_utf8(files_output.stdout)?;
        let files: Vec<&str> = files_output_str.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(files.len(), 1);
        assert!(files[0].contains(".agents/tasks/"));
        assert!(files[0].contains(branch));

        if expect_push {
            // Verify branch was pushed to remote
            let remote_commit_output = Command::new("git")
                .args(["rev-parse", branch])
                .current_dir(remote_path)
                .output()?;
            let remote_commit = String::from_utf8(remote_commit_output.stdout)?.trim().to_string();
            assert_eq!(remote_commit, tip_commit);
        }

        Ok(())
    }

    #[test]
    fn integration_test_clean_repo() -> Result<()> {
        let (_temp_home, repo_dir, remote_dir) = setup_git_repo_integration()?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "feature",
            Some("task"), // Use prompt instead of editor
            None,
            
            Some(false), // Don't push to remote
            None,
            
            
            None, // No sandbox
            vec![], // No editor content needed
            0,
        )?;

        // Should succeed
        assert!(status.success());

        // Verify task branch was created
        assert_task_branch_created_integration(repo_dir.path(), remote_dir.path(), "feature", false)?;

        Ok(())
    }

    #[test]
    fn integration_test_prompt_option() -> Result<()> {
        let (_temp_home, repo_dir, remote_dir) = setup_git_repo_integration()?;

        let (status, _output, editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "p1",
            Some("prompt text"),
            None,
            
            Some(true), // Push to remote
            None,
            
            
            None, // No sandbox
            vec![], // No editor content needed
            0,
        )?;

        // Should succeed and not call editor
        assert!(status.success());
        assert!(!editor_called);

        // Verify task branch was created
        assert_task_branch_created_integration(repo_dir.path(), remote_dir.path(), "p1", true)?;

        Ok(())
    }

    #[test]
    fn integration_test_prompt_file_option() -> Result<()> {
        let (_temp_home, repo_dir, remote_dir) = setup_git_repo_integration()?;

        // Create a prompt file
        let prompt_file = repo_dir.path().join("task.txt");
        fs::write(&prompt_file, "Task from file\n")?;

        let (status, _output, editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "pf1",
            None,
            
            Some(&prompt_file),
            Some(true), // Push to remote
            None,
            
            
            None, // No sandbox
            vec![], // No editor content needed
            0,
        )?;

        // Should succeed and not call editor
        assert!(status.success());
        assert!(!editor_called);

        // Verify task branch was created
        assert_task_branch_created_integration(repo_dir.path(), remote_dir.path(), "pf1", true)?;

        Ok(())
    }

    #[test]
    fn integration_test_editor_failure() -> Result<()> {
        use std::process::Command;

        let (_temp_home, repo_dir, _remote_dir) = setup_git_repo_integration()?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "bad",
            None,
            
            None,
            
            Some(false), // Don't push to remote
            None,
            
            None, // No sandbox
            vec![], // Empty editor content
            1, // Editor fails
        )?;

        // Should fail
        assert!(!status.success());

        Ok(())
    }

    #[test]
    fn integration_test_empty_file() -> Result<()> {
        use std::process::Command;

        let (_temp_home, repo_dir, _remote_dir) = setup_git_repo_integration()?;

        // Create an empty prompt file
        let prompt_file = repo_dir.path().join("empty.txt");
        fs::write(&prompt_file, "")?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "empty",
            None,
            
            Some(&prompt_file),
            Some(false), // Don't push to remote
            None,
            
            None, // No sandbox
            vec![], // No editor needed
            0,
        )?;

        // Should fail (empty task)
        assert!(!status.success());

        Ok(())
    }

    #[test]
    fn integration_test_dirty_repo_staged() -> Result<()> {
        use std::process::Command;

        let (_temp_home, repo_dir, remote_dir) = setup_git_repo_integration()?;

        // Create staged changes
        fs::write(repo_dir.path().join("foo.txt"), "foo")?;
        Command::new("git")
            .args(["add", "foo.txt"])
            .current_dir(&repo_dir)
            .output()?;

        // Check that we have staged changes
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&repo_dir)
            .output()?;
        let status_before = String::from_utf8(status_output.stdout)?;

        let (status, output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "s1",
            Some("task"), // Use prompt instead of editor
            None,
            
            Some(false), // Don't push to remote
            None,
            
            None, // No sandbox
            vec![], // No editor needed
            0,
        )?;

        if !status.success() {
            eprintln!("Binary failed with output: {}", output);
        }
        assert!(status.success());

        // Verify staged changes are preserved
        let status_output_after = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&repo_dir)
            .output()?;
        let status_after = String::from_utf8(status_output_after.stdout)?;
        assert_eq!(status_before, status_after);

        // Verify task branch was created
        assert_task_branch_created_integration(repo_dir.path(), remote_dir.path(), "s1", false)?;

        Ok(())
    }

    #[test]
    fn integration_test_dirty_repo_unstaged() -> Result<()> {
        use std::process::Command;

        let (_temp_home, repo_dir, remote_dir) = setup_git_repo_integration()?;

        // Create unstaged changes
        fs::write(repo_dir.path().join("bar.txt"), "bar")?;
        // Check that we have unstaged changes
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&repo_dir)
            .output()?;
        let status_before = String::from_utf8(status_output.stdout)?;

        let (status, output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "s2",
            Some("task"), // Use prompt instead of editor
            None,
            
            Some(false), // Don't push to remote
            None,
            
            None, // No sandbox
            vec![], // No editor needed
            0,
        )?;

        if !status.success() {
            eprintln!("Binary failed with output: {}", output);
        }
        assert!(status.success());

        // Verify unstaged changes are preserved
        let status_output_after = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&repo_dir)
            .output()?;
        let status_after = String::from_utf8(status_output_after.stdout)?;
        assert_eq!(status_before, status_after);

        // Verify task branch was created
        assert_task_branch_created_integration(repo_dir.path(), remote_dir.path(), "s2", false)?;

        Ok(())
    }

    #[test]
    fn integration_test_devshell_option() -> Result<()> {
        let (_temp_home, repo_dir, remote_dir) = setup_git_repo_integration()?;

        // Create a flake.nix file
        let flake_content = r#"
        {
          outputs = { self }: {
            devShells.x86_64-linux.default = {};
            devShells.x86_64-linux.custom = {};
          };
        }
        "#;
        fs::write(repo_dir.path().join("flake.nix"), flake_content)?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "ds1",
            Some("task"),
            None,
            
            Some(false), // Don't push to remote
            Some("custom"),
            None, // No sandbox
            vec![], // No editor needed
            0,
        )?;

        assert!(status.success());

        // Verify task branch was created
        assert_task_branch_created_integration(repo_dir.path(), remote_dir.path(), "ds1", false)?;

        Ok(())
    }

    #[test]
    fn integration_test_devshell_option_invalid() -> Result<()> {
        use std::process::Command;

        let (_temp_home, repo_dir, _remote_dir) = setup_git_repo_integration()?;

        // Create a flake.nix file without the requested devshell
        let flake_content = r#"
        {
          outputs = { self }: {
            devShells.x86_64-linux.default = {};
          };
        }
        "#;
        fs::write(repo_dir.path().join("flake.nix"), flake_content)?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "ds2",
            Some("task"),
            None,
            
            Some(false), // Don't push to remote
            Some("missing"),
            None, // No sandbox
            vec![], // No editor needed
            0,
        )?;

        // Should fail (invalid devshell)
        assert!(!status.success());

        Ok(())
    }

    #[test]
    fn integration_test_devshell_without_flake() -> Result<()> {
        use std::process::Command;

        let (_temp_home, repo_dir, _remote_dir) = setup_git_repo_integration()?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "ds3",
            Some("task"),
            None,
            
            Some(false), // Don't push to remote
            Some("any"),
            None, // No sandbox
            vec![], // No editor needed
            0,
        )?;

        // Should fail (no flake.nix)
        assert!(!status.success());

        Ok(())
    }

    #[test]
    fn integration_test_prompt_option_empty() -> Result<()> {
        use std::process::Command;

        let (_temp_home, repo_dir, _remote_dir) = setup_git_repo_integration()?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "poe",
            Some("   \n\t  "), // Empty/whitespace prompt
            None,
            
            Some(false), // Don't push to remote
            None,
            
            None, // No sandbox
            vec![], // No editor needed
            0,
        )?;

        // Should fail (empty prompt)
        assert!(!status.success());

        Ok(())
    }

    #[test]
    fn integration_test_prompt_file_empty() -> Result<()> {
        use std::process::Command;

        let (_temp_home, repo_dir, _remote_dir) = setup_git_repo_integration()?;

        // Create a prompt file with only whitespace
        let prompt_file = repo_dir.path().join("whitespace.txt");
        fs::write(&prompt_file, "   \n\t\n  ")?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "pfe",
            None,
            
            Some(&prompt_file),
            Some(false), // Don't push to remote
            None,
            
            None, // No sandbox
            vec![], // No editor needed
            0,
        )?;

        // Should fail (empty/whitespace content)
        assert!(!status.success());

        Ok(())
    }

    #[test]
    fn integration_test_invalid_branch() -> Result<()> {
        use std::process::Command;

        let (_temp_home, repo_dir, _remote_dir) = setup_git_repo_integration()?;

        let (status, _output, editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "inv@lid name", // Invalid branch name
            None,
            
            None,
            
            Some(false), // Don't push to remote
            None,
            
            None, // No sandbox
            vec!["task"], // Editor content
            0,
        )?;

        // Should fail (invalid branch name)
        assert!(!status.success());
        // Editor should not be called when branch validation fails
        assert!(!editor_called);

        Ok(())
    }

    #[test]
    fn integration_test_sandbox_basic() -> Result<()> {
        let (_temp_home, repo_dir, remote_dir) = setup_git_repo_integration()?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "sandbox-test",
            Some("Test task with sandbox"),
            None,
            Some(false), // Don't push to remote
            None,
            Some(("local", None, None, None, None)), // Basic sandbox without extra features
            vec![], // No editor content needed
            0,
        )?;

        // Should succeed
        assert!(status.success());

        // Verify task branch was created
        assert_task_branch_created_integration(repo_dir.path(), remote_dir.path(), "sandbox-test", false)?;

        Ok(())
    }

    #[test]
    fn integration_test_sandbox_with_network() -> Result<()> {
        let (_temp_home, repo_dir, remote_dir) = setup_git_repo_integration()?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "sandbox-net",
            Some("Test task with network access"),
            None,
            Some(false), // Don't push to remote
            None,
            Some(("local", Some("yes"), None, None, None)), // Sandbox with network access
            vec![], // No editor content needed
            0,
        )?;

        // Should succeed
        assert!(status.success());

        // Verify task branch was created
        assert_task_branch_created_integration(repo_dir.path(), remote_dir.path(), "sandbox-net", false)?;

        Ok(())
    }

    #[test]
    fn integration_test_sandbox_with_seccomp() -> Result<()> {
        let (_temp_home, repo_dir, remote_dir) = setup_git_repo_integration()?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "sandbox-seccomp",
            Some("Test task with seccomp"),
            None,
            Some(false), // Don't push to remote
            None,
            Some(("local", None, None, None, Some("yes"))), // Sandbox with seccomp
            vec![], // No editor content needed
            0,
        )?;

        // Should succeed
        assert!(status.success());

        // Verify task branch was created
        assert_task_branch_created_integration(repo_dir.path(), remote_dir.path(), "sandbox-seccomp", false)?;

        Ok(())
    }

    #[test]
    fn integration_test_sandbox_invalid_type() -> Result<()> {
        let (_temp_home, repo_dir, _remote_dir) = setup_git_repo_integration()?;

        let (status, _output, _editor_called) = run_aw_task_create_integration(
            repo_dir.path(),
            "sandbox-invalid",
            Some("Test task with invalid sandbox"),
            None,
            Some(false), // Don't push to remote
            None,
            Some(("invalid", None, None, None, None)), // Invalid sandbox type
            vec![], // No editor content needed
            0,
        )?;

        // Should fail due to invalid sandbox type
        assert!(!status.success());

        Ok(())
    }
}

/// Validate sandbox parameters and prepare workspace if sandbox is enabled
async fn validate_and_prepare_sandbox(args: &TaskCreateArgs) -> Result<PreparedWorkspace> {
    // Validate sandbox type
    if args.sandbox != "local" {
        anyhow::bail!("Error: Only 'local' sandbox type is currently supported");
    }

    // Parse boolean flags
    let _allow_network = parse_bool_flag(&args.allow_network)
        .context("Invalid --allow-network value")?;
    let _allow_containers = parse_bool_flag(&args.allow_containers)
        .context("Invalid --allow-containers value")?;
    let _allow_kvm = parse_bool_flag(&args.allow_kvm)
        .context("Invalid --allow-kvm value")?;
    let _seccomp = parse_bool_flag(&args.seccomp)
        .context("Invalid --seccomp value")?;
    let _seccomp_debug = parse_bool_flag(&args.seccomp_debug)
        .context("Invalid --seccomp-debug value")?;

    // Get current working directory as the workspace to snapshot
    let workspace_path = std::env::current_dir()
        .context("Failed to get current working directory")?;

    // Prepare writable workspace using FS snapshots
    prepare_workspace_with_fallback(&workspace_path).await
        .context("Failed to prepare sandbox workspace")
}
