use tokio::process::Command;

pub async fn run_cmd(cmd: &str, args: &[&str]) -> String {
    match Command::new(cmd).args(args).output().await {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(_) => "Error.".to_string(),
    }
}