use crate::error::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub async fn run_install() -> Result<()> {
    println!("\n⚙️  Installing Clippie Daemon\n");

    let home = dirs::home_dir().ok_or_else(|| {
        crate::error::CliError::ConfigError("Could not determine home directory".to_string())
    })?;

    let plist_dir = home.join("Library/LaunchAgents");
    let plist_path = plist_dir.join("no.bechsor.clippy-daemon.plist");

    // Create LaunchAgents directory if needed
    fs::create_dir_all(&plist_dir)?;

    // Get the path to the clippie binary
    let binary_path = std::env::current_exe()?;

    // Create plist content
    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>no.bechsor.clippy-daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>start</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}</string>
    <key>StandardErrorPath</key>
    <string>{}</string>
</dict>
</plist>"#,
        binary_path.display(),
        home.join(".local/share/clippy/daemon.log").display(),
        home.join(".local/share/clippy/daemon.err").display()
    );

    // Write plist file
    fs::write(&plist_path, plist_content)?;
    println!("✓ Created LaunchAgent plist at {}", plist_path.display());

    // Load the plist
    let output = Command::new("launchctl")
        .arg("load")
        .arg(&plist_path)
        .output()?;

    if output.status.success() {
        println!("✓ Loaded daemon with launchctl");
        println!("\nDaemon installed successfully! 🎉\n");
        println!("The daemon will start automatically on next login.");
        println!("To start it now, run: 'clippie start'\n");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("⚠️  Failed to load daemon: {}", stderr);
        println!("\nYou may need to check the plist or launchctl configuration.\n");
    }

    Ok(())
}
