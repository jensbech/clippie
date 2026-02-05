import readline from 'readline';
import { mkdirSync, existsSync } from 'fs';
import { dirname, resolve } from 'path';
import { homedir } from 'os';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';
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
      const userInput = await question(rl, 'Enter path to clipboard database (absolute or relative to home): ');

      // Convert relative paths to absolute (relative to home directory)
      if (userInput.startsWith('~/')) {
        dbPath = resolve(homedir(), userInput.slice(2));
      } else if (userInput.startsWith('/')) {
        dbPath = userInput;
      } else {
        // Treat as relative to home directory
        dbPath = resolve(homedir(), userInput);
      }

      console.log(`\nUsing database path: ${dbPath}`);
      const confirm = await question(rl, 'Is this correct? [Y/n] ');

      if (confirm.toLowerCase() !== 'n') {
        break;
      }
      console.log('Please try again.\n');
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
      try {
        // Get bin directory path
        const currentDir = dirname(fileURLToPath(import.meta.url));
        const scriptDir = resolve(currentDir, '../../bin');

        console.log('\nInstalling daemon...');
        execSync(`${scriptDir}/clippy-install`, { stdio: 'inherit' });

        console.log('\nStarting daemon...');
        execSync(`${scriptDir}/clippy-start`, { stdio: 'inherit' });

        // Small delay for daemon to initialize
        await new Promise(resolve => setTimeout(resolve, 2000));

        console.log('\nDaemon status:');
        execSync(`${scriptDir}/clippy-status`, { stdio: 'inherit' });

      } catch (e) {
        console.error(`\nError setting up daemon: ${e.message}`);
        console.log('You can manually run: clippy-install && clippy-start');
      }
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
