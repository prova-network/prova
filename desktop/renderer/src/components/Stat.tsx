type StatProps = {
  label: string
  value: string | number
  sub?: string
  tone?: 'default' | 'ok' | 'warn' | 'bad'
}

// Color tokens. `default` reads as a calm ink; status tones override
// with their own semantic color for the value text. Sub-text stays
// muted across all tones so the eye rests on the headline number.
const valueClass: Record<NonNullable<StatProps['tone']>, string> = {
  default: 'text-ink dark:text-cream',
  ok: 'text-emerald-700 dark:text-emerald-300',
  warn: 'text-amber-700 dark:text-amber-300',
  bad: 'text-red-700 dark:text-red-300',
}

// Left-edge accent bar mirrors the SectionHeading bar treatment. Default
// uses the brand teal; status tones swap to their semantic color so
// 'something needs attention' reads at a glance.
const toneBar: Record<NonNullable<StatProps['tone']>, string> = {
  default: 'border-l-2 border-l-teal-cyan/40 dark:border-l-teal-cyan/40',
  ok: 'border-l-2 border-l-emerald-400',
  warn: 'border-l-2 border-l-amber-400',
  bad: 'border-l-2 border-l-red-400',
}

export function Stat({ label, value, sub, tone = 'default' }: StatProps) {
  return (
    <div className={`surface-card p-4 flex flex-col gap-1 ${toneBar[tone]}`}>
      <span className="text-[11px] uppercase tracking-wider text-ink/60 dark:text-cream/55">
        {label}
      </span>
      <span className={`text-2xl font-semibold ${valueClass[tone]}`}>{value}</span>
      {sub && (
        <span className="text-[11px] text-ink/50 dark:text-cream/45">{sub}</span>
      )}
    </div>
  )
}
