use std::io::Write;
use std::process::{Command, Stdio};
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone)]
pub struct KeyInfo {
    pub key_id: String,
    pub uid: String,
}

impl std::fmt::Display for KeyInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}  –  {}", self.key_id, self.uid)
    }
}

/// List all public keys in the GPG keyring.
pub fn list_public_keys() -> Vec<KeyInfo> {
    let output = Command::new("gpg")
        .args(["--list-keys", "--with-colons"])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut keys = Vec::new();
    let mut current_key_id = String::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 10 {
            continue;
        }
        match parts[0] {
            "pub" => {
                // Long key ID is the last 16 chars of field 4
                current_key_id = parts[4].chars().rev().take(16).collect::<String>()
                    .chars().rev().collect();
            }
            "uid" if !current_key_id.is_empty() => {
                let uid = parts[9].to_string();
                if !uid.is_empty() {
                    keys.push(KeyInfo {
                        key_id: current_key_id.clone(),
                        uid,
                    });
                    current_key_id.clear();
                }
            }
            _ => {}
        }
    }

    keys
}

/// Encrypt plaintext with the given key ID.
/// Returns ASCII-armored ciphertext or an error message.
pub fn encrypt(plaintext: &str, key_id: &str) -> Result<String, String> {
    let mut child = Command::new("gpg")
        .args([
            "--batch", "--yes", "--armor",
            "--trust-model", "always",
            "--recipient", key_id,
            "--encrypt",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn gpg: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(plaintext.as_bytes())
            .map_err(|e| format!("Failed to write to gpg stdin: {e}"))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("GPG process error: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Clear all cached passphrases from the GPG agent.
pub fn clear_agent_cache() {
    let _ = std::process::Command::new("gpg-connect-agent")
        .args(["reloadagent", "/bye"])
        .output();
}

/// Returns plaintext wrapped in Zeroizing<String> so it is zeroed on drop.
pub fn decrypt(ciphertext: &str) -> Result<Zeroizing<String>, String> {
    let mut child = Command::new("gpg")
        .args(["--batch", "--decrypt"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn gpg: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(ciphertext.as_bytes())
            .map_err(|e| format!("Failed to write to gpg stdin: {e}"))?;
    }

    let output = child.wait_with_output()
        .map_err(|e| format!("GPG process error: {e}"))?;

    if output.status.success() {
        // Collect into Zeroizing so the buffer is zeroed when dropped
        let plain = Zeroizing::new(
            String::from_utf8(output.stdout)
                .map_err(|e| format!("GPG output was not valid UTF-8: {e}"))?
        );
        // Explicitly zero the raw output bytes
        let mut raw = output.stderr;
        raw.zeroize();
        Ok(plain)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}