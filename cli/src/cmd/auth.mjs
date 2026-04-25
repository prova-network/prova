// `prova auth` — interactive sign-in.
// Asks for an email, calls /api/auth/signup, saves the token to ~/.prova/config.json

import { stdin as input, stdout as output } from 'node:process';
import { createInterface } from 'node:readline/promises';
import { saveConfig, loadConfig } from '../util/config.mjs';
import { api } from '../util/api.mjs';
import { c, formatSize } from '../util/format.mjs';
import { hostname } from 'node:os';

export async function authCmd() {
  const existing = await loadConfig();
  if (existing.token) {
    console.log(c.yellow('You are already signed in as') + ' ' + c.bold(existing.email));
    console.log(c.dim('Run `prova logout` to switch accounts.'));
    return;
  }

  const rl = createInterface({ input, output });
  const email = (await rl.question(c.cyan('Email: '))).trim().toLowerCase();
  rl.close();

  if (!email) {
    console.error(c.red('Email is required.'));
    process.exit(1);
  }

  const label = `cli@${hostname()}`;
  const res = await api('/api/auth/signup', {
    method: 'POST',
    json: { email, label },
  });

  await saveConfig({
    token: res.token,
    email: res.email,
    userId: res.userId,
    quotaMb: res.quotaMb,
    expiresAt: res.expiresAt,
  });

  console.log();
  console.log(c.green('✓') + ' Signed in as ' + c.bold(res.email));
  console.log(c.ash('  user-id : ') + res.userId);
  console.log(c.ash('  quota   : ') + formatSize(res.quotaMb * 1024 * 1024) + ' / day');
  console.log(c.ash('  expires : ') + new Date(res.expiresAt).toLocaleString());
  console.log();
  console.log(c.dim('Token saved to ~/.prova/config.json'));
}
