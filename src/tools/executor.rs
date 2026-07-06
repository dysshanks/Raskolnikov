use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::sync::watch;

async fn terminate_child(child: &mut Child) {
    #[cfg(unix)]
    {
        if let Some(pid) = child.id() {
            let _ = std::process::Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .spawn();
        }
        tokio::select! {
            _ = child.wait() => return,
            _ = tokio::time::sleep(Duration::from_secs(5)) => {}
        }
    }
    let _ = child.start_kill();
}

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

    let spawn_err = |msg: &str| ToolRunResult {
        exit_code: None,
        stdout: String::new(),
        stderr: msg.to_string(),
        duration: start.elapsed(),
        was_interrupted: false,
    };

    let mut child: Child = match Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return spawn_err(&format!("Failed to spawn tool: {}", cmd)),
    };

    let mut stdout = match child.stdout.take() {
        Some(s) => s,
        None => return spawn_err("Failed to capture stdout"),
    };
    let mut stderr = match child.stderr.take() {
        Some(s) => s,
        None => return spawn_err("Failed to capture stderr"),
    };

    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();

    let mut interrupt_rx_clone = interrupt_rx;

    let status = loop {
        tokio::select! {
            status = child.wait() => {
                break status.ok();
            }
            _ = tokio::time::sleep(Duration::from_secs(30 * 60)) => {
                terminate_child(&mut child).await;
                was_interrupted = true;
                break None;
            }
            _ = interrupt_rx_clone.changed() => {
                if *interrupt_rx_clone.borrow() {
                    terminate_child(&mut child).await;
                    was_interrupted = true;
                    break None;
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
        stdout: String::from_utf8(stdout_buf)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()),
        stderr: String::from_utf8(stderr_buf)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()),
        duration,
        was_interrupted,
    }
}

pub async fn run_tool_streaming(
    cmd: &str,
    args: &[&str],
    interrupt_rx: watch::Receiver<bool>,
    line_tx: tokio::sync::mpsc::UnboundedSender<String>,
) -> ToolRunResult {
    let start = std::time::Instant::now();
    let mut was_interrupted = false;

    let spawn_err = |msg: &str| ToolRunResult {
        exit_code: None,
        stdout: String::new(),
        stderr: msg.to_string(),
        duration: start.elapsed(),
        was_interrupted: false,
    };

    let mut child: Child = match Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return spawn_err(&format!("Failed to spawn tool: {}", cmd)),
    };

    let mut stdout = match child.stdout.take() {
        Some(s) => s,
        None => return spawn_err("Failed to capture stdout"),
    };
    let mut stderr = match child.stderr.take() {
        Some(s) => s,
        None => return spawn_err("Failed to capture stderr"),
    };

    let mut interrupt_rx_clone = interrupt_rx;

    let tx_out = line_tx.clone();
    let stdout_handle = tokio::spawn(async move {
        let reader = tokio::io::BufReader::new(&mut stdout);
        let mut lines = reader.lines();
        let mut acc = String::new();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = tx_out.send(line.clone());
            acc.push_str(&line);
            acc.push('\n');
        }
        acc
    });

    let tx_err = line_tx;
    let stderr_handle = tokio::spawn(async move {
        let reader = tokio::io::BufReader::new(&mut stderr);
        let mut lines = reader.lines();
        let mut acc = String::new();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = tx_err.send(format!("[stderr] {}", line));
            acc.push_str(&line);
            acc.push('\n');
        }
        acc
    });

    let status = loop {
        tokio::select! {
            status = child.wait() => break status.ok(),
            _ = tokio::time::sleep(Duration::from_secs(30 * 60)) => {
                terminate_child(&mut child).await;
                was_interrupted = true;
                break None;
            }
            _ = interrupt_rx_clone.changed() => {
                if *interrupt_rx_clone.borrow() {
                    terminate_child(&mut child).await;
                    was_interrupted = true;
                    break None;
                }
            }
        }
    };

    let full_stdout = stdout_handle.await.unwrap_or_default();
    let full_stderr = stderr_handle.await.unwrap_or_default();

    let duration = start.elapsed();

    ToolRunResult {
        exit_code: status.map(|s| s.code().unwrap_or(-1)),
        stdout: full_stdout,
        stderr: full_stderr,
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
