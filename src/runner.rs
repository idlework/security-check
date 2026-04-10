use std::process::Command;
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
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;

    let (tx, rx) = mpsc::channel();
    let child_id = child.id();

    std::thread::spawn(move || {
        std::thread::sleep(timeout);
        let _ = tx.send(child_id);
    });

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = {
                    use std::io::Read;
                    let mut buf = String::new();
                    if let Some(mut out) = child.stdout.take() {
                        let _ = out.read_to_string(&mut buf);
                    }
                    buf
                };
                let stderr = {
                    use std::io::Read;
                    let mut buf = String::new();
                    if let Some(mut err) = child.stderr.take() {
                        let _ = err.read_to_string(&mut buf);
                    }
                    buf
                };

                if status.success() || !stdout.is_empty() {
                    return Ok(stdout);
                } else {
                    return Err(stderr);
                }
            }
            Ok(None) => {
                if rx.try_recv().is_ok() {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("{} timed out after {:?}", cmd, timeout));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(format!("Failed to wait for {}: {}", cmd, e)),
        }
    }
}

pub fn run_defaults_read(domain: &str, key: &str) -> Result<String, String> {
    run_command("defaults", &["read", domain, key])
}
