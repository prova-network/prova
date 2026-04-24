type StatProps = {
  label: string
  value: string | number
  sub?: string
  tone?: 'default' | 'ok' | 'warn' | 'bad'
}

const toneClass: Record<NonNullable<StatProps['tone']>, string> = {
  default: 'text-ink',
  ok: 'text-emerald-700',
  warn: 'text-amber-700',
  bad: 'text-red-700',
}

// Stat is the standard-sized KPI tile. Subtle left-border touch of gold
// for default tone ties these to the HeroTile's more prominent gold bar;
// status-sensitive tones override with their own semantic color.
const toneBar: Record<NonNullable<StatProps['tone']>, string> = {
  default: 'border-l-2 border-l-gold/30',
  ok: 'border-l-2 border-l-emerald-400',
  warn: 'border-l-2 border-l-amber-400',
  bad: 'border-l-2 border-l-red-400',
}

export function Stat({ label, value, sub, tone = 'default' }: StatProps) {
  return (
    <div className={`surface-card p-4 flex flex-col gap-1 ${toneBar[tone]}`}>
      <span className="text-xs uppercase tracking-wider text-ink/60">{label}</span>
      <span className={`text-2xl font-semibold ${toneClass[tone]}`}>{value}</span>
      {sub && <span className="text-xs text-ink/50">{sub}</span>}
    </div>
  )
}
