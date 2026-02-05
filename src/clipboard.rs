use crate::error::{CliError, Result};
use sha2::{Sha256, Digest};
use std::process::Command;

/// Get the current clipboard content as a string
pub fn get_clipboard_content() -> Result<Option<String>> {
    let output = Command::new("pbpaste")
        .output()
        .map_err(|e| CliError::ClipboardError(format!("pbpaste error: {}", e)))?;

    if !output.status.success() {
        return Ok(None);
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();

    if content.is_empty() {
        Ok(None)
    } else {
        Ok(Some(content))
    }
}

/// Get the current clipboard change count (using file modification time as a proxy)
pub fn get_clipboard_change_count() -> Result<i64> {
    // Since we can't easily track clipboard changes without GUI frameworks,
    // we'll use the hash of the current content as our "change count"
    // In practice, this still works because we compare hashes in the daemon
    match get_clipboard_content() {
        Ok(Some(content)) => {
            let hash = hash_content(&content);
            // Convert hash to i64 for change count (use first 16 chars)
            Ok(i64::from_str_radix(&hash[..16], 16).unwrap_or(0))
        }
        Ok(None) => Ok(0),
        Err(_) => Ok(0),
    }
}

/// Copy content to clipboard
pub fn set_clipboard_content(content: &str) -> Result<()> {
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| CliError::ClipboardError(format!("Failed to run pbcopy: {}", e)))?;

    {
        use std::io::Write;
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| CliError::ClipboardError("Failed to open stdin".to_string()))?;
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| CliError::ClipboardError(format!("Failed to write to pbcopy: {}", e)))?;
    }

    let status = child
        .wait()
        .map_err(|e| CliError::ClipboardError(format!("pbcopy error: {}", e)))?;

    if status.success() {
        Ok(())
    } else {
        Err(CliError::ClipboardError("pbcopy failed".to_string()))
    }
}

/// Compute SHA256 hash of content
pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content() {
        let content = "test content";
        let hash = hash_content(content);
        // SHA256 should produce 64 hex characters
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_consistency() {
        let content = "test";
        let hash1 = hash_content(content);
        let hash2 = hash_content(content);
        assert_eq!(hash1, hash2);
    }
}
