use tokio::process::Command;
use tokio::time::{timeout, Duration};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("command timed out after {timeout_secs}s: {cmd}")]
    Timeout { cmd: String, timeout_secs: u64 },
    #[error("failed to execute command {cmd}: {source}")]
    Io { cmd: String, source: std::io::Error },
}

pub async fn run_cmd(cmd: &str, args: &[&str], timeout_secs: u64) -> Result<CommandOutput, CommandError> {
    let mut child = Command::new(cmd);
    child.args(args);

    let output = timeout(Duration::from_secs(timeout_secs), child.output())
        .await
        .map_err(|_| CommandError::Timeout {
            cmd: cmd.to_string(),
            timeout_secs,
        })?
        .map_err(|source| CommandError::Io {
            cmd: cmd.to_string(),
            source,
        })?;

    Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        status: output.status.code().unwrap_or(-1),
    })
}