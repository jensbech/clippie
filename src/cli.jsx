import React from 'react';
import { render } from 'ink';
import { MouseProvider } from '@ink-tools/ink-mouse';
import meow from 'meow';
import { execSync } from 'child_process';
import { dbExists, closeDb } from './lib/db.js';
import { configExists } from './lib/config.js';
import { runSetup } from './commands/setup.js';
import App from './App.jsx';

const cli = meow(`
  Usage
    $ clippie                   Launch the clipboard history browser
    $ clippie setup             Configure database location
    $ clippie start             Start the daemon
    $ clippie stop              Stop the daemon
    $ clippie status            Show daemon status
    $ clippie db <path>         Switch to a database
    $ clippie clear [--all]     Clear history entries

  Navigation
    j/k or arrows   Move up/down
    Enter           Copy selected entry to clipboard and exit
    /               Filter entries
    r               Refresh from database
    q / Esc         Quit
`, {
  importMeta: import.meta,
  flags: {},
});

// Check for help flag
if (process.argv.includes('-h') || process.argv.includes('--help')) {
  console.log(cli.help);
  process.exit(0);
}

// Helper to run shell commands
function runCommand(cmd) {
  try {
    execSync(cmd, { stdio: 'inherit' });
    process.exit(0);
  } catch (e) {
    process.exit(1);
  }
}

// Get the bin directory path
const { dirname } = await import('path');
const { fileURLToPath } = await import('url');
const currentDir = dirname(fileURLToPath(import.meta.url));
const binDir = `${currentDir}/../bin`;

// Handle subcommands
const command = cli.input[0];

if (command === 'setup') {
  (async () => {
    await runSetup();
    process.exit(0);
  })();
} else if (command === 'start') {
  runCommand(`${binDir}/clippy-start`);
} else if (command === 'stop') {
  runCommand(`${binDir}/clippy-stop`);
} else if (command === 'status') {
  runCommand(`${binDir}/clippy-status`);
} else if (command === 'db') {
  if (!cli.input[1]) {
    console.error('Error: database path required');
    console.error('Usage: clippy db <path>');
    process.exit(1);
  }
  runCommand(`${binDir}/clippy-db ${cli.input.slice(1).join(' ')}`);
} else if (command === 'clear') {
  runCommand(`${binDir}/clippy-clear ${cli.input.slice(1).join(' ')}`);
} else if (command === 'install') {
  runCommand(`${binDir}/clippy-install`);
} else if (command) {
  console.error(`Error: unknown command '${command}'`);
  process.exit(1);
} else {
  // Check if config exists
  if (!configExists()) {
    console.error("Error: Clippy not configured.");
    console.error("Run 'clippie setup' to configure the database location.");
    process.exit(1);
  }

  if (!dbExists()) {
    console.error("Error: Clipboard history database not found.");
    console.error("Make sure the daemon is running: clippy-start");
    console.error("Or configure a valid database location: clippie setup");
    process.exit(1);
  }

  // Alternate screen buffer
  process.stdout.write('\x1B[?1049h');
  process.stdout.write('\x1B[H');

  const restoreScreen = () => {
    process.stdout.write('\x1B[?1049l');
  };
  process.on('SIGINT', () => { restoreScreen(); process.exit(0); });
  process.on('SIGTERM', () => { restoreScreen(); process.exit(0); });

  let selectedContent = null;

  const instance = render(
    React.createElement(MouseProvider, { cacheInvalidationMs: 0 },
      React.createElement(App, {
        onSelect: (content) => {
          selectedContent = content;
          try {
            execSync('pbcopy', { input: content });
          } catch (e) {
            // pbcopy not available
          }
        },
      })
    )
  );

  instance.waitUntilExit().then(() => {
    closeDb();
    restoreScreen();
    if (selectedContent) {
      process.stdout.write(selectedContent + '\n');
    }
  });
}
