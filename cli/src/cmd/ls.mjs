// `prova ls` — list your stored files.

import { requireToken } from '../util/config.mjs';
import { api } from '../util/api.mjs';
import { c, formatSize } from '../util/format.mjs';

export async function lsCmd() {
  const cfg = await requireToken();
  const res = await api('/api/files', { token: cfg.token });

  if (!res.files || !res.files.length) {
    console.log(c.dim('No files yet. Run: ') + c.cyan('prova put <file>'));
    return;
  }
  console.log(c.bold(`${res.count} file(s):`));
  console.log();
  for (const f of res.files) {
    const date = new Date(f.uploadedAt).toLocaleString();
    console.log(`  ${c.cyan(f.cid)}  ${c.dim(formatSize(f.size).padStart(8))}  ${c.ash(date)}  ${f.filename}`);
  }
}
