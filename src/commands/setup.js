import readline from 'readline';
import { mkdirSync, existsSync } from 'fs';
import { dirname } from 'path';
import { getConfigPath, getDefaultDbPath, saveConfig, ensureConfigDir } from '../lib/config.js';

function createInterface() {
  return readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: process.stdin.isTTY,
  });
}

function question(rl, prompt) {
  return new Promise(resolve => {
    rl.question(prompt, resolve);
  });
}

export async function runSetup() {
  const rl = createInterface();

  console.log('\n╔════════════════════════════════════════╗');
  console.log('║        Clippy Setup Wizard             ║');
  console.log('╚════════════════════════════════════════╝\n');

  console.log('Clippy monitors your clipboard and stores history in a SQLite database.');
  console.log('This wizard will help you configure the database location.\n');

  const defaultPath = getDefaultDbPath();
  const configPath = getConfigPath();

  console.log(`Configuration file will be saved to: ${configPath}\n`);

  const useDefault = await question(rl, `Use default database location (${defaultPath})? [Y/n] `);

  let dbPath = defaultPath;
  if (useDefault.toLowerCase() === 'n') {
    while (true) {
      dbPath = await question(rl, 'Enter full path to clipboard database: ');
      if (dbPath.startsWith('/')) {
        break;
      }
      console.log('Error: Path must be an absolute path (start with /)');
    }
  }

  // Ensure database directory exists
  const dbDir = dirname(dbPath);
  if (!existsSync(dbDir)) {
    try {
      mkdirSync(dbDir, { recursive: true });
      console.log(`Created directory: ${dbDir}`);
    } catch (e) {
      console.error(`Error creating directory: ${e.message}`);
      rl.close();
      process.exit(1);
    }
  }

  // Save configuration
  ensureConfigDir();
  const config = {
    dbPath: dbPath,
    version: '1.0.0',
  };

  if (saveConfig(config)) {
    console.log(`✓ Configuration saved to: ${configPath}\n`);
    console.log('Database configuration:');
    console.log(`  Location: ${dbPath}\n`);
  } else {
    console.error('Failed to save configuration');
    rl.close();
    process.exit(1);
  }

  // Optional: Ask about daemon installation on macOS
  if (process.platform === 'darwin') {
    const installDaemon = await question(rl, 'Install and start the clipboard monitoring daemon? [Y/n] ');
    if (installDaemon.toLowerCase() !== 'n') {
      console.log('\nNext steps:');
      console.log('  1. Run: clippy-install');
      console.log('  2. Run: clippy-start');
      console.log('  3. Verify: clippy-status');
      console.log('');
    }
  }

  console.log('Setup complete! You can now:');
  console.log('  • Launch the browser: clippy');
  console.log('  • Manage daemon: clippy-{start,stop,status,install}');
  console.log('  • View history: clippy');
  console.log('');
  console.log('For terminal integration, add to ~/.zshrc:');
  console.log('  cb() {');
  console.log('    local result');
  console.log('    result=$(clippy)');
  console.log('    if [[ -n "$result" ]]; then');
  console.log('      print -z "$result"');
  console.log('    fi');
  console.log('  }');
  console.log('');

  rl.close();
}
