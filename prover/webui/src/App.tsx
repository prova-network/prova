import type { ReactNode } from 'react'
import { useEffect, useState } from 'react'
import { api, type Deal, type Overview } from './api'
import { Logo } from './components/Logo'
import { Stat } from './components/Stat'
import { StatusBadge } from './components/StatusBadge'
import {
  formatBytes,
  formatDuration,
  relativeTime,
  shortAddr,
  shortHash,
} from './util'

// Poll interval for the live dashboard. The endpoints are cheap (local
// reads against in-memory state), so 5s feels live without thrashing.
const POLL_MS = 5000

type FetchState<T> =
  | { kind: 'loading' }
  | { kind: 'ok'; data: T; fetchedAt: number }
  | { kind: 'error'; error: string }

function useOverview() {
  const [s, setS] = useState<FetchState<Overview>>({ kind: 'loading' })
  useEffect(() => {
    let stop = false
    async function tick() {
      try {
        const data = await api.overview()
        if (!stop) setS({ kind: 'ok', data, fetchedAt: Date.now() })
      } catch (e: unknown) {
        if (!stop)
          setS({
            kind: 'error',
            error: e instanceof Error ? e.message : String(e),
          })
      }
    }
    tick()
    const h = setInterval(tick, POLL_MS)
    return () => {
      stop = true
      clearInterval(h)
    }
  }, [])
  return s
}

function useDeals(status: string) {
  const [s, setS] = useState<FetchState<Deal[]>>({ kind: 'loading' })
  useEffect(() => {
    let stop = false
    async function tick() {
      try {
        const resp = await api.deals(status === 'all' ? undefined : status)
        if (!stop) setS({ kind: 'ok', data: resp.deals, fetchedAt: Date.now() })
      } catch (e: unknown) {
        if (!stop)
          setS({
            kind: 'error',
            error: e instanceof Error ? e.message : String(e),
          })
      }
    }
    tick()
    const h = setInterval(tick, POLL_MS)
    return () => {
      stop = true
      clearInterval(h)
    }
  }, [status])
  return s
}

export default function App() {
  const [statusFilter, setStatusFilter] = useState<string>('all')
  const ov = useOverview()
  const dl = useDeals(statusFilter)

  return (
    <div className="min-h-screen bg-cream flex flex-col">
      <Header ov={ov} />
      <main className="max-w-6xl w-full mx-auto px-4 py-6 space-y-10 flex-1">
        <section>
          {ov.kind === 'loading' && (
            <div className="text-ink/50">Loading dashboard…</div>
          )}
          {ov.kind === 'error' && (
            <div className="surface-card p-4 border-red-300 bg-red-50">
              <div className="font-semibold text-red-900">
                Dashboard API unreachable
              </div>
              <div className="text-sm text-red-800 mt-1">{ov.error}</div>
              <div className="text-xs text-red-700 mt-2">
                Check that <code className="mono">provad</code> is running
                and that <code className="mono">[dashboard] enabled = true</code>
                is set in prover.toml.
              </div>
            </div>
          )}
          {ov.kind === 'ok' && <HeroStrip ov={ov.data} />}
        </section>

        <section>
          {ov.kind === 'ok' && <SecondaryStats ov={ov.data} />}
        </section>

        <section>
          <SectionHeading
            title="Deals"
            sub="Local view of deals this prover has seen or accepted."
            right={<DealsFilter value={statusFilter} onChange={setStatusFilter} />}
          />
          {dl.kind === 'loading' && <div className="text-ink/50">Loading deals…</div>}
          {dl.kind === 'error' && (
            <div className="surface-card p-3 border-red-300 bg-red-50 text-sm text-red-800">
              {dl.error}
            </div>
          )}
          {dl.kind === 'ok' && <DealsTable deals={dl.data} />}
        </section>

        <section>
          <SectionHeading
            title="Chain &amp; stake"
            sub="On-chain state of this prover's registration and staked collateral."
          />
          {ov.kind === 'ok' && <ChainPanel ov={ov.data} />}
        </section>
      </main>
      <Footer ov={ov} />
    </div>
  )
}

function Header({ ov }: { ov: FetchState<Overview> }) {
  const liveOK = ov.kind === 'ok'
  const chainId = ov.kind === 'ok' ? ov.data.chain?.chain_id : undefined
  return (
    <header className="bg-ink text-cream">
      <div className="max-w-6xl mx-auto px-4 py-4 flex items-center gap-3">
        <div className="text-gold">
          <Logo size={40} />
        </div>
        <div className="flex-1">
          <h1 className="font-semibold text-lg leading-none tracking-tight">
            Prova <span className="text-gold">Prover</span>
          </h1>
          <div className="text-xs text-cream/60 mt-1">
            verifiable storage on Base, read-only operator view
          </div>
        </div>

        {/* Network identifier (neutral styling, not a status). */}
        {chainId !== undefined && (
          <div className="hidden sm:flex items-center gap-2 px-3 py-1.5 border border-cream/15 rounded-full bg-white/5 text-xs text-cream/80">
            <span className="h-2 w-2 rounded-full bg-cream/40" aria-hidden />
            {chainName(chainId)}
          </div>
        )}

        {/* Live indicator */}
        <div
          className="flex items-center gap-2 px-3 py-1.5 border rounded-full text-xs"
          style={{
            borderColor: liveOK ? 'rgba(113, 176, 116, 0.4)' : 'rgba(184, 73, 24, 0.5)',
            background: liveOK ? 'rgba(44, 92, 46, 0.25)' : 'rgba(184, 73, 24, 0.2)',
            color: liveOK ? '#A5DEA9' : '#F8B79A',
          }}
        >
          <span
            className={`h-2 w-2 rounded-full ${liveOK ? 'bg-emerald-400 animate-pulse' : 'bg-red-400'}`}
            aria-hidden
          />
          {liveOK ? 'Live' : 'Offline'}
        </div>
      </div>
      {/* Hair-thin gold underline for brand continuity down to the hero row */}
      <div className="h-[2px] bg-gradient-to-r from-transparent via-gold to-transparent" />
    </header>
  )
}

function SectionHeading({
  title,
  sub,
  right,
}: {
  title: string
  sub?: string
  right?: ReactNode
}) {
  return (
    <div className="flex items-end justify-between mb-3 gap-3">
      <div className="flex items-stretch gap-2.5">
        <span aria-hidden className="w-[3px] bg-gold rounded-full" />
        <div>
          <h2 className="text-sm font-semibold uppercase tracking-wider text-ink/70">
            {title}
          </h2>
          {sub && <p className="text-xs text-ink/40 mt-0.5">{sub}</p>}
        </div>
      </div>
      {right}
    </div>
  )
}

function Footer({ ov }: { ov: FetchState<Overview> }) {
  const fetchedAt = ov.kind === 'ok' ? ov.fetchedAt : undefined
  return (
    <footer className="border-t border-ink/10 bg-white/60 mt-12">
      <div className="max-w-6xl mx-auto px-4 py-4 text-xs text-ink/50 flex items-center justify-between">
        <div>
          Read-only dashboard,{' '}
          <a href="/api/overview" className="underline hover:text-gold-dark">
            raw JSON
          </a>
        </div>
        <div>
          {fetchedAt
            ? `updated ${relativeTime(new Date(fetchedAt).toISOString())}`
            : 'no data yet'}
          {' '}· auto-refresh {POLL_MS / 1000}s
        </div>
      </div>
    </footer>
  )
}

// HeroStrip is the top row of the dashboard: 3 oversized, primary KPIs.
function HeroStrip({ ov }: { ov: Overview }) {
  const m = ov.metrics
  const proofRate = m?.proof_success_rate_pct
  const registered = ov.chain?.prover_registered
  const active = ov.chain?.prover_active

  const statusLabel = ov.chain
    ? registered
      ? active
        ? 'Registered · active'
        : 'Registered · inactive'
      : 'Not registered'
    : 'Chain unconfigured'
  const statusTone: 'ok' | 'warn' | 'bad' | 'default' =
    ov.chain && registered && active
      ? 'ok'
      : ov.chain && registered
        ? 'warn'
        : ov.chain
          ? 'bad'
          : 'default'

  const proofsSubmitted = m?.proofs_submitted_total ?? 0
  const proofsFailed = m?.proofs_failed_total ?? 0
  const hasAnyProofs = proofsSubmitted + proofsFailed > 0
  const proofText = !m
    ? 'n/a'
    : !hasAnyProofs
      ? 'no data'
      : `${proofRate!.toFixed(proofRate === 100 ? 0 : 1)}%`
  const proofSub = !m
    ? 'metrics not enabled'
    : !hasAnyProofs
      ? 'first proof lands when a deal becomes active'
      : `${proofsSubmitted} ok, ${proofsFailed} failed`
  const proofTone: 'ok' | 'warn' | 'bad' | 'default' = !hasAnyProofs
    ? 'default'
    : proofRate! >= 99
      ? 'ok'
      : proofRate! >= 90
        ? 'warn'
        : 'bad'

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
      <HeroTile
        label="Prover status"
        value={statusLabel}
        sub={ov.chain ? shortAddr(ov.chain.prover_address) : 'set chain config'}
        tone={statusTone}
        emphasize
      />
      <HeroTile
        label="Active deals"
        value={m?.deals_active ?? ov.deal_counts?.active ?? 0}
        sub={`${ov.deals_total ?? 0} total`}
        emphasize
      />
      <HeroTile
        label="Proof success rate"
        value={proofText}
        sub={proofSub}
        tone={proofTone}
        emphasize
      />
    </div>
  )
}

// SecondaryStats is the "small numbers you care about" strip below the
// hero. Each tile is standard-sized; the hero up top commands attention.
function SecondaryStats({ ov }: { ov: Overview }) {
  const m = ov.metrics
  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
      <Stat
        label="Uptime"
        value={m?.uptime_seconds ? formatDuration(m.uptime_seconds) : ov.uptime}
        sub={`started ${relativeTime(ov.started_at)}`}
      />
      <Stat
        label="Bytes stored"
        value={formatBytes(m?.piece_store_bytes)}
        sub="on local disk"
      />
      <Stat
        label="Bytes served"
        value={formatBytes(m?.retrieval_bytes_total)}
        sub="HTTPS retrieval, cumulative"
      />
      <Stat
        label="RPC errors"
        value={m?.chain_rpc_errors_total ?? 0}
        sub="cumulative, since boot"
        tone={
          m && m.chain_rpc_errors_total > 10
            ? 'bad'
            : m && m.chain_rpc_errors_total > 0
              ? 'warn'
              : 'default'
        }
      />
    </div>
  )
}

type HeroProps = {
  label: string
  value: string | number
  sub?: string
  tone?: 'default' | 'ok' | 'warn' | 'bad'
  emphasize?: boolean
}

const heroToneValue: Record<NonNullable<HeroProps['tone']>, string> = {
  default: 'text-ink',
  ok: 'text-emerald-700',
  warn: 'text-amber-700',
  bad: 'text-red-700',
}

const heroToneDot: Record<NonNullable<HeroProps['tone']>, string> = {
  default: 'bg-ink/40',
  ok: 'bg-emerald-500',
  warn: 'bg-amber-500',
  bad: 'bg-red-500',
}

// HeroTile is a larger, gold-accented stat tile used for top-of-page KPIs.
// All three hero tiles share the gold left bar for consistent brand
// presence; status is communicated via the value color and a small tone
// dot next to the label, not by hijacking the brand accent.
function HeroTile({ label, value, sub, tone = 'default' }: HeroProps) {
  return (
    <div className="surface-card p-5 pl-6 relative overflow-hidden flex flex-col gap-2">
      <span
        aria-hidden
        className="absolute left-0 top-0 bottom-0 w-1.5 bg-gold"
      />
      <div className="flex items-center gap-2 pl-1">
        {tone !== 'default' && (
          <span
            aria-hidden
            className={`h-1.5 w-1.5 rounded-full ${heroToneDot[tone]}`}
          />
        )}
        <span className="text-xs uppercase tracking-wider text-ink/60">
          {label}
        </span>
      </div>
      <span
        className={`text-4xl font-semibold leading-none pl-1 ${heroToneValue[tone]}`}
      >
        {value}
      </span>
      {sub && <span className="text-xs text-ink/50 pl-1">{sub}</span>}
    </div>
  )
}

function DealsFilter({
  value,
  onChange,
}: {
  value: string
  onChange: (v: string) => void
}) {
  const options = [
    'all',
    'proposed',
    'downloading',
    'verifying',
    'accepting',
    'active',
    'completed',
    'cancelled',
    'failed',
    'slashed',
  ]
  return (
    <label className="flex items-center gap-2 text-xs text-ink/60">
      filter:
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="text-sm border border-ink/20 rounded bg-white px-2 py-1 focus:outline-none focus:ring-2 focus:ring-gold/40"
      >
        {options.map((o) => (
          <option key={o} value={o}>
            {o}
          </option>
        ))}
      </select>
    </label>
  )
}

function DealsTable({ deals }: { deals: Deal[] }) {
  if (!deals.length) {
    return (
      <div className="surface-card p-10 text-center text-ink/50">
        <div className="text-3xl mb-2">∅</div>
        <div className="text-sm">No deals match this filter yet.</div>
      </div>
    )
  }
  return (
    <div className="surface-card overflow-x-auto">
      <table className="w-full text-sm">
        <thead className="text-xs uppercase tracking-wider text-ink/60 border-b border-ink/10 bg-cream/50">
          <tr>
            <th className="text-left px-3 py-2.5">ID</th>
            <th className="text-left px-3 py-2.5">Status</th>
            <th className="text-left px-3 py-2.5">Client</th>
            <th className="text-left px-3 py-2.5">CommP</th>
            <th className="text-right px-3 py-2.5">Size</th>
            <th className="text-left px-3 py-2.5">Updated</th>
          </tr>
        </thead>
        <tbody>
          {deals.map((d) => (
            <tr
              key={d.id}
              className="border-b border-ink/5 last:border-0 hover:bg-gold/5 transition-colors"
            >
              <td className="px-3 py-2 font-mono text-ink/80">#{d.id}</td>
              <td className="px-3 py-2">
                <StatusBadge status={d.status} />
              </td>
              <td className="px-3 py-2 mono">{shortAddr(d.client)}</td>
              <td className="px-3 py-2 mono">{shortHash(d.commp_hex, 16)}</td>
              <td className="px-3 py-2 text-right">{formatBytes(d.piece_size)}</td>
              <td className="px-3 py-2 text-ink/60 text-xs">
                <span title={d.updated_at}>{relativeTime(d.updated_at)}</span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

function ChainPanel({ ov }: { ov: Overview }) {
  if (ov.chain_error) {
    return (
      <div className="surface-card p-4 border-amber-300 bg-amber-50 text-sm text-amber-900">
        Chain RPC unreachable: {ov.chain_error}
      </div>
    )
  }
  if (!ov.chain) {
    return (
      <div className="surface-card p-4 text-sm text-ink/50">
        Chain reader not configured. On-chain stake, block height, and
        registration status will appear here once the daemon has access to a
        Base RPC endpoint.
      </div>
    )
  }
  const c = ov.chain
  return (
    <div className="space-y-3">
      {/* Primary: the three stats operators actually care about. */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        <Stat
          label="Staked"
          value={formatPROVA(c.staked_wei)}
          sub="slashable collateral"
        />
        <Stat
          label="Committed"
          value={formatBytes(c.committed_bytes)}
          sub="active deal byte sum"
        />
        <Stat
          label="Prover address"
          value={shortAddr(c.prover_address)}
          sub={c.prover_registered ? 'registered on-chain' : 'not yet registered'}
          tone={c.prover_registered ? 'ok' : 'warn'}
        />
      </div>

      {/* Secondary chain state: the debug-level stuff. Thinner strip. */}
      <div className="surface-card border-l-2 border-l-gold/30 px-5 py-3 flex flex-wrap gap-x-8 gap-y-2 text-xs text-ink/70">
        <ChainChip
          label="Block"
          value={c.block_number.toLocaleString()}
          sub={`${c.block_number - c.last_seen_block} behind head`}
        />
        <ChainChip
          label="Unbonding"
          value={formatPROVA(c.unbonding_wei)}
          sub={
            c.unbonding_ends_at_unix
              ? `ready ${new Date(c.unbonding_ends_at_unix * 1000).toLocaleDateString()}`
              : 'none'
          }
        />
        <ChainChip label="Reputation" value={`${c.reputation} / 10000`} />
        <ChainChip
          label="Chain"
          value={chainName(c.chain_id)}
          sub={`ID ${c.chain_id}`}
        />
      </div>
    </div>
  )
}

function ChainChip({
  label,
  value,
  sub,
}: {
  label: string
  value: string
  sub?: string
}) {
  return (
    <div className="flex flex-col">
      <span className="text-[10px] uppercase tracking-wider text-ink/40">
        {label}
      </span>
      <span className="text-sm font-medium text-ink">{value}</span>
      {sub && <span className="text-[10px] text-ink/40">{sub}</span>}
    </div>
  )
}

function chainName(id: number): string {
  switch (id) {
    case 8453:
      return 'Base mainnet'
    case 84532:
      return 'Base Sepolia'
    case 31337:
      return 'anvil (local)'
    default:
      return `chain ${id}`
  }
}

function formatPROVA(wei: string | undefined): string {
  if (!wei || wei === '0') return '0 PROVA'
  try {
    // Rough formatting, avoiding BN libs. Split on 1e18.
    const s = wei.padStart(19, '0')
    const whole = s.slice(0, -18).replace(/^0+/, '') || '0'
    const frac = s.slice(-18).slice(0, 4).replace(/0+$/, '')
    return `${whole}${frac ? '.' + frac : ''} PROVA`
  } catch {
    return wei
  }
}
