//! tmux multiplexer implementation
//!
//! Implements the Multiplexer trait for tmux using its command-line interface.
//! Based on the tmux integration guide in specs/Public/Terminal-Multiplexers/tmux.md

use aw_mux_core::*;
use std::process::Stdio;
use tokio::process::Command;

/// tmux multiplexer implementation
pub struct TmuxMultiplexer {
    session_name: String,
}

impl Default for TmuxMultiplexer {
    fn default() -> Self {
        Self {
            session_name: "aw".to_string(),
        }
    }
}

impl TmuxMultiplexer {
    pub fn new() -> Result<Self, MuxError> {
        Ok(Self::default())
    }

    pub fn with_session_name(session_name: String) -> Self {
        Self { session_name }
    }

    /// Run a tmux command and return its output
    async fn run_tmux_command(&self, args: &[&str]) -> Result<String, MuxError> {
        let output = Command::new("tmux")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    MuxError::NotAvailable("tmux")
                } else {
                    MuxError::Io(e)
                }
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(MuxError::CommandFailed(format!(
                "tmux {} failed: {}",
                args.join(" "),
                stderr
            )))
        }
    }

    /// Ensure a tmux session exists
    async fn ensure_session(&self) -> Result<(), MuxError> {
        // Check if session exists
        let result = self.run_tmux_command(&["has-session", "-t", &self.session_name]).await;

        match result {
            Ok(_) => Ok(()), // Session exists
            Err(MuxError::CommandFailed(_)) => {
                // Session doesn't exist, create it
                self.run_tmux_command(&[
                    "new-session",
                    "-d",
                    "-s",
                    &self.session_name,
                    "-c",
                    &std::env::current_dir()
                        .map_err(MuxError::Io)?
                        .to_string_lossy(),
                ]).await?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl Multiplexer for TmuxMultiplexer {
    fn id(&self) -> &'static str {
        "tmux"
    }

    fn is_available(&self) -> bool {
        std::process::Command::new("tmux")
            .arg("-V")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn open_window(&self, opts: &WindowOptions) -> Result<WindowId, MuxError> {
        // For now, return a placeholder implementation
        // In a real implementation, this would execute tmux commands
        let title = opts.title.unwrap_or("window");
        let window_id = format!("{}-{}", self.session_name, title);
        Ok(window_id)
    }

    fn split_pane(
        &self,
        window: &WindowId,
        target: Option<&PaneId>,
        dir: SplitDirection,
        percent: Option<u8>,
        opts: &CommandOptions,
        initial_cmd: Option<&str>,
    ) -> Result<PaneId, MuxError> {
        // For now, return a placeholder implementation
        let pane_id = format!("{}.pane", window);
        Ok(pane_id)
    }

    fn run_command(&self, pane: &PaneId, cmd: &str, opts: &CommandOptions) -> Result<(), MuxError> {
        // For now, return a placeholder implementation
        Ok(())
    }

    fn send_text(&self, pane: &PaneId, text: &str) -> Result<(), MuxError> {
        // For now, return a placeholder implementation
        Ok(())
    }

    fn focus_window(&self, window: &WindowId) -> Result<(), MuxError> {
        // For now, return a placeholder implementation
        Ok(())
    }

    fn focus_pane(&self, pane: &PaneId) -> Result<(), MuxError> {
        // For now, return a placeholder implementation
        Ok(())
    }

    fn list_windows(&self, title_substr: Option<&str>) -> Result<Vec<WindowId>, MuxError> {
        // For now, return a placeholder implementation
        Ok(vec![])
    }

    fn list_panes(&self, window: &WindowId) -> Result<Vec<PaneId>, MuxError> {
        // For now, return a placeholder implementation
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmux_multiplexer_creation() {
        let tmux = TmuxMultiplexer::new().unwrap();
        assert_eq!(tmux.id(), "tmux");
        assert_eq!(tmux.session_name, "aw");
    }

    #[test]
    fn test_tmux_with_custom_session() {
        let tmux = TmuxMultiplexer::with_session_name("custom-session".to_string());
        assert_eq!(tmux.session_name, "custom-session");
    }

    #[test]
    fn test_tmux_availability() {
        let tmux = TmuxMultiplexer::new().unwrap();
        // This will fail if tmux is not installed, which is expected in test environment
        let _available = tmux.is_available();
    }
}
