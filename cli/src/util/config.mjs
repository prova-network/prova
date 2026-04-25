// Local config file at ~/.prova/config.json
import { homedir } from 'node:os';
import { join } from 'node:path';
import { mkdir, readFile, writeFile, unlink } from 'node:fs/promises';

const CONFIG_DIR  = join(homedir(), '.prova');
const CONFIG_PATH = join(CONFIG_DIR, 'config.json');

export const DEFAULT_API = process.env.PROVA_API || 'https://prova.network';

export async function loadConfig() {
  try {
    const raw = await readFile(CONFIG_PATH, 'utf8');
    return JSON.parse(raw);
  } catch {
    return {};
  }
}

export async function saveConfig(cfg) {
  await mkdir(CONFIG_DIR, { recursive: true, mode: 0o700 });
  await writeFile(CONFIG_PATH, JSON.stringify(cfg, null, 2), { mode: 0o600 });
}

export async function clearConfig() {
  try { await unlink(CONFIG_PATH); } catch {}
}

export async function requireToken() {
  const cfg = await loadConfig();
  if (!cfg.token) {
    throw new Error('Not signed in. Run: \x1b[36mprova auth\x1b[0m');
  }
  return cfg;
}
