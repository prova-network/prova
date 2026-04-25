// `prova put <file>` — upload, get a piece-cid back.

import { readFile, stat } from 'node:fs/promises';
import { basename } from 'node:path';
import { computeCid } from '../util/hash.mjs';
import { api } from '../util/api.mjs';
import { loadConfig, DEFAULT_API } from '../util/config.mjs';
import { c, formatSize } from '../util/format.mjs';

export async function putCmd(args) {
  const file = args[0];
  if (!file) {
    console.error(c.red('Usage: prova put <file>'));
    process.exit(2);
  }

  const cfg = await loadConfig();
  const auth = cfg.token ? cfg : null;

  let info;
  try {
    info = await stat(file);
  } catch {
    console.error(c.red(`File not found: ${file}`));
    process.exit(1);
  }

  if (!info.isFile()) {
    console.error(c.red(`Not a file: ${file}`));
    process.exit(1);
  }

  const sizeBytes = info.size;
  const tier = auth ? 'authed' : 'sponsored';
  const fileLimit = auth ? 5 * 1024 * 1024 * 1024 : 100 * 1024 * 1024;
  if (sizeBytes > fileLimit) {
    console.error(c.red(`File is ${formatSize(sizeBytes)}; ${tier} tier caps at ${formatSize(fileLimit)}.`));
    if (!auth) console.error(c.dim('Sign up for a free token: ') + c.cyan('prova auth'));
    process.exit(1);
  }

  const filename = basename(file);
  process.stdout.write(c.dim(`hashing  ${filename} (${formatSize(sizeBytes)})… `));
  const buf = await readFile(file);
  const cid = await computeCid(buf);
  console.log(c.green('✓') + ' ' + c.dim(cid));

  process.stdout.write(c.dim('staging  bytes for prover…             '));
  const t0 = Date.now();
  const res = await api(`/api/upload?cid=${encodeURIComponent(cid)}`, {
    method: 'POST',
    headers: {
      'content-type': 'application/octet-stream',
      'x-filename': encodeURIComponent(filename),
    },
    body: buf,
    token: auth?.token,
  });
  console.log(c.green('✓') + ' ' + c.dim(`${Date.now() - t0} ms`));

  console.log();
  console.log(c.bold('Stored.'));
  console.log(c.ash('  piece-cid : ') + c.cyan(res.cid));
  console.log(c.ash('  deal-id   : ') + res.dealId);
  console.log(c.ash('  size      : ') + formatSize(res.size));
  console.log(c.ash('  paid      : ') + (res.sponsored ? 'sponsored (free)' : 'free quota'));
  console.log(c.ash('  term      : ') + res.term);
  console.log(c.ash('  retrieve  : ') + c.cyan(res.retrievalUrl));
  console.log();
  console.log(c.dim('Verify with: ') + c.cyan(`curl -O ${res.retrievalUrl}`));
}
