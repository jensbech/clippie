use crate::error::{CliError, Result};
use objc2::rc::autoreleasepool;
use objc2::{class, msg_send, sel};
use objc2_foundation::NSString;
use sha2::{Sha256, Digest};

/// Get the current clipboard content as a string
pub fn get_clipboard_content() -> Result<Option<String>> {
    unsafe {
        autoreleasepool(|pool| {
            // Get the general pasteboard
            let pasteboard: *const objc2::runtime::AnyObject = msg_send![
                class!(NSPasteboard),
                generalPasteboard
            ];

            if pasteboard.is_null() {
                return Ok(None);
            }

            // Try to get string content using legacy method
            let string_type_ns = NSString::from_str("NSStringPboardType");
            let string_obj: *const NSString = msg_send![
                pasteboard,
                stringForType: &*string_type_ns
            ];

            if !string_obj.is_null() {
                let content = NSString::as_str(&*string_obj, pool).to_string();
                return Ok(Some(content));
            }

            Ok(None)
        })
    }
}

/// Get the current clipboard change count (for efficient change detection)
pub fn get_clipboard_change_count() -> Result<i64> {
    unsafe {
        autoreleasepool(|_pool| {
            let pasteboard: *const objc2::runtime::AnyObject = msg_send![
                class!(NSPasteboard),
                generalPasteboard
            ];

            if pasteboard.is_null() {
                return Ok(0);
            }

            let count: i64 = msg_send![pasteboard, changeCount];
            Ok(count)
        })
    }
}

/// Copy content to clipboard
pub fn set_clipboard_content(content: &str) -> Result<()> {
    unsafe {
        autoreleasepool(|_pool| {
            let pasteboard: *const objc2::runtime::AnyObject = msg_send![
                class!(NSPasteboard),
                generalPasteboard
            ];

            if pasteboard.is_null() {
                return Err(CliError::ClipboardError("Failed to get pasteboard".to_string()));
            }

            let _: () = msg_send![pasteboard, clearContents];

            let string = NSString::from_str(content);
            let string_type = NSString::from_str("NSStringPboardType");

            let success: bool = msg_send![
                pasteboard,
                setString: &*string
                forType: &*string_type
            ];

            if success {
                Ok(())
            } else {
                Err(CliError::ClipboardError("Failed to set clipboard content".to_string()))
            }
        })
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
