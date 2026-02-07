use crate::error::{CliError, Result};
use sha2::{Digest, Sha256};
use std::process::Command;

pub fn get_clipboard_content() -> Result<Option<String>> {
    let output = Command::new("pbpaste")
        .output()
        .map_err(|e| CliError::ClipboardError(format!("pbpaste error: {}", e)))?;

    if !output.status.success() {
        return Ok(None);
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(if content.is_empty() { None } else { Some(content) })
}

#[allow(dead_code)]
pub fn get_pasteboard_change_count() -> i64 {
    use objc2::runtime::{AnyClass, AnyObject};
    use objc2::msg_send;

    unsafe {
        let Some(pasteboard_class) = AnyClass::get("NSPasteboard") else {
            return 0;
        };
        let pasteboard: *mut AnyObject = msg_send![pasteboard_class, generalPasteboard];
        if pasteboard.is_null() {
            return 0;
        }
        msg_send![pasteboard, changeCount]
    }
}

pub fn set_clipboard_content(content: &str) -> Result<()> {
    use std::io::Write;

    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| CliError::ClipboardError(format!("Failed to run pbcopy: {}", e)))?;

    child
        .stdin
        .as_mut()
        .ok_or_else(|| CliError::ClipboardError("Failed to open stdin".to_string()))?
        .write_all(content.as_bytes())
        .map_err(|e| CliError::ClipboardError(format!("Failed to write to pbcopy: {}", e)))?;

    let status = child
        .wait()
        .map_err(|e| CliError::ClipboardError(format!("pbcopy error: {}", e)))?;

    if status.success() {
        Ok(())
    } else {
        Err(CliError::ClipboardError("pbcopy failed".to_string()))
    }
}

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
        let hash = hash_content("test content");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_consistency() {
        let hash1 = hash_content("test");
        let hash2 = hash_content("test");
        assert_eq!(hash1, hash2);
    }
}
