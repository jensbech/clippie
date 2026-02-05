import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'fs';
import { homedir } from 'os';
import { join, dirname } from 'path';

export function getConfigPath() {
  return join(homedir(), '.config', 'clippy', 'config.json');
}

export function getDefaultDbPath() {
  return join(homedir(), '.local', 'share', 'clippy', 'clipboard-history.db');
}

export function loadConfig() {
  const configPath = getConfigPath();
  if (!existsSync(configPath)) {
    return null;
  }
  try {
    const content = readFileSync(configPath, 'utf-8');
    return JSON.parse(content);
  } catch (e) {
    console.error(`Error reading config file: ${e.message}`);
    return null;
  }
}

export function saveConfig(config) {
  const configPath = getConfigPath();
  ensureConfigDir();
  try {
    writeFileSync(configPath, JSON.stringify(config, null, 2), 'utf-8');
    return true;
  } catch (e) {
    console.error(`Error writing config file: ${e.message}`);
    return false;
  }
}

export function ensureConfigDir() {
  const configPath = getConfigPath();
  const configDir = dirname(configPath);
  if (!existsSync(configDir)) {
    mkdirSync(configDir, { recursive: true });
  }
}

export function getDbPath() {
  // Priority: env var > config file > default
  if (process.env.CLIPPY_DB_PATH) {
    return process.env.CLIPPY_DB_PATH;
  }

  const config = loadConfig();
  if (config && config.dbPath) {
    return config.dbPath;
  }

  return getDefaultDbPath();
}

export function configExists() {
  return existsSync(getConfigPath());
}
