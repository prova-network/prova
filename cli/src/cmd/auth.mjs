// `prova auth` — interactive sign-in via magic-link + 6-digit code.
//
// Flow:
//   1. Prompt for email
//   2. POST /api/auth/start  → server emails a code
//   3. Prompt for code
//   4. POST /api/auth/verify → server returns the API token
//   5. Save to ~/.prova/config.json

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
  if (!email || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
    rl.close();
    console.error(c.red('A valid email is required.'));
    process.exit(1);
  }

  const label = `cli@${hostname()}`;

  // Step 1: ask the server to email a code
  process.stdout.write(c.dim('Sending sign-in code…'));
  try {
    await api('/api/auth/start', {
      method: 'POST',
      json: { email, label, returnUrl: '' },
    });
  } catch (err) {
    rl.close();
    process.stdout.write('\r' + ' '.repeat(40) + '\r');
    console.error(c.red('Could not send sign-in email: ') + (err.body?.detail || err.message));
    process.exit(1);
  }
  process.stdout.write('\r' + ' '.repeat(40) + '\r');
  console.log(c.green('✓') + ' Sent. Check your inbox for a 6-digit code from Prova.');
  console.log(c.dim('  (Code expires in 15 minutes.)'));
  console.log();

  // Step 2: prompt for the code
  let res;
  for (let attempt = 0; attempt < 5; attempt++) {
    const raw = (await rl.question(c.cyan('Code (6 digits): '))).trim().replace(/\s+/g, '');
    if (!/^[0-9]{6}$/.test(raw)) {
      console.log(c.yellow('  Codes are 6 digits. Try again.'));
      continue;
    }
    try {
      res = await api('/api/auth/verify', {
        method: 'POST',
        json: { email, code: raw, label },
      });
      break;
    } catch (err) {
      const code = err.body?.error || '';
      if (code === 'invalid_code') {
        console.log(c.yellow('  That code didn\'t match. Try again.'));
        continue;
      }
      if (code === 'too_many_attempts') {
        console.error(c.red('Too many attempts on that code. Run `prova auth` again.'));
        rl.close();
        process.exit(1);
      }
      if (code === 'expired_or_unknown') {
        console.error(c.red('Code expired. Run `prova auth` again.'));
        rl.close();
        process.exit(1);
      }
      console.error(c.red('Verify failed: ') + (err.body?.detail || err.message));
      rl.close();
      process.exit(1);
    }
  }
  rl.close();

  if (!res) {
    console.error(c.red('Too many bad codes. Run `prova auth` again.'));
    process.exit(1);
  }

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
