use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::sync::watch;

pub struct ToolRunResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub was_interrupted: bool,
}

pub async fn run_tool(
    cmd: &str,
    args: &[&str],
    interrupt_rx: watch::Receiver<bool>,
) -> ToolRunResult {
    let start = std::time::Instant::now();
    let mut was_interrupted = false;

    let mut child: Child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn tool");

    let mut stdout = child.stdout.take().expect("Failed to capture stdout");
    let mut stderr = child.stderr.take().expect("Failed to capture stderr");

    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();

    let mut interrupt_rx_clone = interrupt_rx;

    let status = loop {
        tokio::select! {
            status = child.wait() => {
                break status.ok();
            }
            _ = tokio::time::sleep(Duration::from_secs(30 * 60)) => {
                let _ = child.start_kill();
                was_interrupted = true;
                break None;
            }
            _ = interrupt_rx_clone.changed() => {
                if *interrupt_rx_clone.borrow() {
                    let _ = child.start_kill();
                    was_interrupted = true;
                }
            }
        }
    };

    let _ = tokio::io::BufReader::new(&mut stdout)
        .read_to_end(&mut stdout_buf)
        .await;
    let _ = tokio::io::BufReader::new(&mut stderr)
        .read_to_end(&mut stderr_buf)
        .await;

    let duration = start.elapsed();

    ToolRunResult {
        exit_code: status.map(|s| s.code().unwrap_or(-1)),
        stdout: String::from_utf8_lossy(&stdout_buf).into_owned(),
        stderr: String::from_utf8_lossy(&stderr_buf).into_owned(),
        duration,
        was_interrupted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::watch;

    #[tokio::test]
    async fn test_run_tool_success() {
        let (_tx, rx) = watch::channel(false);
        let result = run_tool("echo", &["hello"], rx).await;
        assert_eq!(result.exit_code, Some(0));
        assert!(!result.stdout.is_empty());
    }

    #[tokio::test]
    async fn test_run_tool_not_found() {
        let (_, rx) = watch::channel(false);
        let result = tokio::spawn(async { run_tool("nonexistent-tool-xyz", &[], rx).await }).await;
        assert!(result.is_err() || result.unwrap().exit_code.is_none());
    }
}
