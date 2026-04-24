const STATUS_COLORS: Record<string, string> = {
  proposed: 'bg-amber-100 text-amber-900 border-amber-200',
  downloading: 'bg-amber-100 text-amber-900 border-amber-200',
  verifying: 'bg-amber-100 text-amber-900 border-amber-200',
  accepting: 'bg-amber-100 text-amber-900 border-amber-200',
  active: 'bg-emerald-100 text-emerald-900 border-emerald-200',
  completed: 'bg-slate-100 text-slate-900 border-slate-200',
  cancelled: 'bg-slate-100 text-slate-700 border-slate-200',
  failed: 'bg-red-100 text-red-900 border-red-200',
  slashed: 'bg-red-200 text-red-950 border-red-300',
}

export function StatusBadge({ status }: { status: string }) {
  const cls = STATUS_COLORS[status] ?? 'bg-slate-100 text-slate-800 border-slate-200'
  return (
    <span
      className={`inline-block px-2 py-0.5 text-xs font-medium rounded border ${cls}`}
      title={`Status: ${status}`}
    >
      {status}
    </span>
  )
}
