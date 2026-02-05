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
    $ clippy              Launch the clipboard history browser
    $ clippy setup        Configure database location

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

// Handle setup command
if (cli.input[0] === 'setup') {
  await runSetup();
  process.exit(0);
}

// Check if config exists
if (!configExists()) {
  console.error("Error: Clippy not configured.");
  console.error("Run 'clippy setup' to configure the database location.");
  process.exit(1);
}

if (!dbExists()) {
  console.error("Error: Clipboard history database not found.");
  console.error("Make sure the daemon is running: clippy-start");
  console.error("Or configure a valid database location: clippy setup");
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
