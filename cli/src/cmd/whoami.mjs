// `prova whoami` — show signed-in identity + usage.

import { requireToken } from '../util/config.mjs';
import { api } from '../util/api.mjs';
import { c, formatSize } from '../util/format.mjs';

export async function whoamiCmd() {
  const cfg = await requireToken();
  let usage;
  try {
    usage = await api('/api/usage', { token: cfg.token });
  } catch (err) {
    console.error(c.red('Could not fetch usage: ') + err.message);
    return;
  }

  const todayBytes = usage.today?.bytes || 0;
  const quotaBytes = usage.quotaBytes;
  const pct = quotaBytes ? Math.min(100, (todayBytes / quotaBytes) * 100) : 0;
  const bar = renderBar(pct);

  console.log(c.bold(usage.email));
  console.log(c.ash('  user-id : ') + usage.userId);
  console.log();
  console.log(c.ash('  today   : ') + formatSize(todayBytes) + ' / ' + formatSize(quotaBytes));
  console.log('            ' + bar + ' ' + c.dim(pct.toFixed(1) + '%'));
  console.log();
  console.log(c.ash('  last 7d : ') + formatSize(usage.last7DaysTotalBytes));
}

function renderBar(pct, width = 30) {
  const fill = Math.round((pct / 100) * width);
  return c.cyan('█'.repeat(fill)) + c.ash('░'.repeat(width - fill));
}
