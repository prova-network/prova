// `prova get <cid> [-o out]` — download a stored file.

import { writeFile } from 'node:fs/promises';
import { DEFAULT_API } from '../util/config.mjs';
import { c, formatSize } from '../util/format.mjs';

export async function getCmd(args) {
  let cid;
  let out;
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '-o' || args[i] === '--output') out = args[++i];
    else if (!cid) cid = args[i];
  }
  if (!cid) {
    console.error(c.red('Usage: prova get <cid> [-o output-file]'));
    process.exit(2);
  }

  const url = `${DEFAULT_API}/p/${encodeURIComponent(cid)}`;
  process.stderr.write(c.dim(`fetching ${url}… `));
  const res = await fetch(url);
  if (!res.ok) {
    console.error(c.red(`failed (${res.status} ${res.statusText})`));
    process.exit(1);
  }
  const ab = await res.arrayBuffer();
  process.stderr.write(c.green('✓') + ' ' + c.dim(formatSize(ab.byteLength)) + '\n');

  if (out) {
    await writeFile(out, Buffer.from(ab));
    console.log(c.green('✓') + ' wrote ' + c.bold(out));
  } else {
    process.stdout.write(Buffer.from(ab));
  }
}
