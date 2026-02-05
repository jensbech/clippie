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

/// Get native macOS pasteboard change count for reliable change detection
pub fn get_pasteboard_change_count() -> i64 {
    use objc2::{msg_send, runtime::AnyClass};
    use objc2::runtime::AnyObject;

    unsafe {
        let Some(pasteboard_class) = AnyClass::get("NSPasteboard") else {
            return 0;
        };

        let pasteboard: *mut AnyObject = msg_send![pasteboard_class, generalPasteboard];

        if pasteboard.is_null() {
            return 0;
        }

        let change_count: i64 = msg_send![pasteboard, changeCount];
        change_count
    }
}

/// Get the current clipboard change count (using pasteboard API)
pub fn get_clipboard_change_count() -> Result<i64> {
    Ok(get_pasteboard_change_count())
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
