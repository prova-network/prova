// Small formatters shared by components.

export function formatBytes(n: number | undefined): string {
  if (!n || n <= 0) return '0 B'
  const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB']
  let i = 0
  let x = n
  while (x >= 1024 && i < units.length - 1) {
    x /= 1024
    i++
  }
  const precision = x >= 100 || i === 0 ? 0 : x >= 10 ? 1 : 2
  return `${x.toFixed(precision)} ${units[i]}`
}

export function formatDuration(seconds: number | undefined): string {
  if (!seconds || seconds <= 0) return '0s'
  const d = Math.floor(seconds / 86400)
  const h = Math.floor((seconds % 86400) / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (d > 0) return `${d}d ${h}h`
  if (h > 0) return `${h}h ${m}m`
  if (m > 0) return `${m}m ${s}s`
  return `${s}s`
}

export function shortAddr(addr: string | undefined): string {
  if (!addr || addr.length < 10) return addr ?? ''
  return `${addr.slice(0, 6)}…${addr.slice(-4)}`
}

export function shortHash(hex: string | undefined, keep = 10): string {
  if (!hex) return ''
  if (hex.length <= keep + 3) return hex
  return `${hex.slice(0, keep)}…`
}

// Renders e.g. "5s ago", "3m ago", "2h ago", "1d ago". Falls back to
// an absolute timestamp for anything older than a week.
export function relativeTime(iso: string | undefined): string {
  if (!iso) return ''
  const then = new Date(iso).getTime()
  if (!Number.isFinite(then)) return ''
  const secs = Math.max(0, Math.floor((Date.now() - then) / 1000))
  if (secs < 5) return 'just now'
  if (secs < 60) return `${secs}s ago`
  const mins = Math.floor(secs / 60)
  if (mins < 60) return `${mins}m ago`
  const hrs = Math.floor(mins / 60)
  if (hrs < 48) return `${hrs}h ago`
  const days = Math.floor(hrs / 24)
  if (days < 7) return `${days}d ago`
  return new Date(iso).toLocaleDateString()
}
