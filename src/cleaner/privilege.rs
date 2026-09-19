use tokio::process::Command;

/// Executes a command with elevated privileges using PolicyKit (`pkexec`) or `sudo`.
pub async fn run_elevated_command(program: &str, args: &[&str]) -> Result<String, String> {
    // 1. Try pkexec (PolicyKit standard graphical authentication)
    if which::which("pkexec").is_ok() {
        let mut cmd = Command::new("pkexec");
        cmd.arg(program);
        cmd.args(args);

        match cmd.output().await {
            Ok(output) if output.status.success() => {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                // If user dismissed auth dialog (exit code 126 or 127 in pkexec), report clearly
                if output.status.code() == Some(126) || output.status.code() == Some(127) {
                    return Err("Authentication dismissed or permission denied by user".to_string());
                }
                if !stderr.is_empty() {
                    return Err(format!("pkexec error: {}", stderr));
                }
                return Err(format!("Command exited with status {:?}: {}", output.status.code(), stdout));
            }
            Err(e) => {
                eprintln!("Failed to spawn pkexec: {}, falling back to direct run/sudo", e);
            }
        }
    }

    // 2. Fallback to sudo if pkexec is unavailable
    if which::which("sudo").is_ok() {
        let mut cmd = Command::new("sudo");
        cmd.arg("-n"); // non-interactive check
        cmd.arg(program);
        cmd.args(args);

        match cmd.output().await {
            Ok(output) if output.status.success() => {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
            _ => {
                return Err(format!(
                    "Root privileges required for '{}'. Please run with pkexec or sudo.",
                    program
                ));
            }
        }
    }

    // 3. Fallback to running directly if user is already root
    let output = Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(|e| format!("Failed to execute '{}': {}", program, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
