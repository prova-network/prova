// `prova logout` — clear local creds.

import { clearConfig, loadConfig } from '../util/config.mjs';
import { c } from '../util/format.mjs';

export async function logoutCmd() {
  const cfg = await loadConfig();
  if (!cfg.token) {
    console.log(c.dim('Not signed in.'));
    return;
  }
  await clearConfig();
  console.log(c.green('✓') + ' Signed out.');
}
