use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

pub struct Context {
    pub is_root: bool,
}

impl Context {
    pub fn detect() -> Self {
        let is_root = Command::new("id")
            .arg("-u")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
            .unwrap_or(false);
        Self { is_root }
    }
}

pub fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    run_command_timeout(cmd, args, Duration::from_secs(10))
}

pub fn run_command_timeout(cmd: &str, args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;

    // Take pipes before moving child into the wait thread
    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait());
    });

    let status = rx
        .recv_timeout(timeout)
        .map_err(|_| format!("{} timed out after {:?}", cmd, timeout))?
        .map_err(|e| format!("Failed to wait for {}: {}", cmd, e))?;

    let stdout = read_pipe(&mut stdout_pipe);
    let stderr = read_pipe(&mut stderr_pipe);

    // Prefer stdout; fall back to stderr (some macOS tools write results to stderr)
    let output = if !stdout.is_empty() { stdout } else { stderr };

    if status.success() || !output.is_empty() {
        Ok(output)
    } else {
        Err(format!("{} exited with {}", cmd, status))
    }
}

pub fn run_defaults_read(domain: &str, key: &str) -> Result<String, String> {
    run_command("defaults", &["read", domain, key])
}

fn read_pipe(pipe: &mut Option<impl Read>) -> String {
    let mut buf = String::new();
    if let Some(p) = pipe.as_mut() {
        let _ = p.read_to_string(&mut buf);
    }
    buf
}
