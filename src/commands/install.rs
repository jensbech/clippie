use crate::error::Result;
use std::fs;
use std::process::Command;

const PLIST_NAME: &str = "no.bechsor.clippie-daemon.plist";

pub async fn run_install() -> Result<()> {
    println!("\n⚙️  Installing Clippie Daemon\n");

    let home = dirs::home_dir().ok_or_else(|| {
        crate::error::CliError::ConfigError("Could not determine home directory".to_string())
    })?;

    let plist_dir = home.join("Library/LaunchAgents");
    let plist_path = plist_dir.join(PLIST_NAME);
    let binary_path = std::env::current_exe()?;
    let log_dir = home.join(".clippie");

    fs::create_dir_all(&plist_dir)?;
    fs::create_dir_all(&log_dir)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&log_dir, fs::Permissions::from_mode(0o700));
    }

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>no.bechsor.clippie-daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>daemon</string>
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
        log_dir.join("daemon.log").display(),
        log_dir.join("daemon.err").display()
    );

    fs::write(&plist_path, plist_content)?;
    println!("✓ Created LaunchAgent plist at {}", plist_path.display());

    let output = Command::new("launchctl")
        .args(["load", &plist_path.to_string_lossy()])
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
