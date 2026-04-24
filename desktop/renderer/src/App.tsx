import { useEffect, useState } from 'react'
import { electron, bridgeAvailable, type Activity, type UpdaterStatus } from './api'
import { Logo } from './components/Logo'
import { Stat } from './components/Stat'
import { formatDuration, relativeTime, shortAddr } from './util'

// Poll interval for refreshing pull-based values (wallet balance, etc).
// Push-based values (activity feed, deals active, proofs submitted) update
// instantly via IPC subscriptions; polling is only a safety net.
const POLL_MS = 10_000

export default function App() {
  const [walletAddress, setWalletAddress] = useState<string>('')
  const [dealsActive, setDealsActive] = useState<number>(0)
  const [proofsSubmitted, setProofsSubmitted] = useState<number>(0)
  const [activities, setActivities] = useState<Activity[]>([])
  const [updaterStatus, setUpdaterStatus] = useState<UpdaterStatus>('idle')
  const [uptime, setUptime] = useState<number>(0)
  const [bootedAt] = useState(() => Date.now())

  // ─── Initial state fetch ──────────────────────────────────────────
  useEffect(() => {
    let cancelled = false

    async function loadInitial() {
      try {
        const [addr, deals, proofs, acts, upd] = await Promise.all([
          electron.getWalletAddress().catch(() => ''),
          electron.getTotalDealsActive().catch(() => 0),
          electron.getTotalProofsSubmitted().catch(() => 0),
          electron.getActivities().catch(() => []),
          electron.getUpdaterStatus().catch(() => 'idle' as UpdaterStatus),
        ])
        if (cancelled) return
        setWalletAddress(addr)
        setDealsActive(deals)
        setProofsSubmitted(proofs)
        setActivities(acts)
        setUpdaterStatus(upd)
      } catch {
        // main process not ready yet; subscriptions will fill in
      }
    }

    loadInitial()
    return () => { cancelled = true }
  }, [])

  // ─── Push subscriptions ──────────────────────────────────────────
  useEffect(() => {
    const unsubs = [
      electron.onWalletAddressUpdated(addr => setWalletAddress(addr)),
      electron.onDealsActiveUpdated(n => setDealsActive(n)),
      electron.onProofStatsUpdated(n => setProofsSubmitted(n)),
      electron.onUpdaterStatusChanged(s => setUpdaterStatus(s)),
      electron.onActivityLogged(a => {
        setActivities(prev => {
          // Prepend, cap at 200 to prevent unbounded growth
          const next = [a, ...prev]
          return next.slice(0, 200)
        })
      }),
    ]
    return () => unsubs.forEach(u => u())
  }, [])

  // ─── Uptime tick ─────────────────────────────────────────────────
  useEffect(() => {
    const h = setInterval(() => {
      setUptime(Math.floor((Date.now() - bootedAt) / 1000))
    }, 1000)
    return () => clearInterval(h)
  }, [bootedAt])

  // ─── Safety-net polling for anything that might have desynced ────
  useEffect(() => {
    const h = setInterval(async () => {
      try {
        const [deals, proofs] = await Promise.all([
          electron.getTotalDealsActive(),
          electron.getTotalProofsSubmitted(),
        ])
        setDealsActive(deals)
        setProofsSubmitted(proofs)
      } catch {
        // ignore transient IPC failures
      }
    }, POLL_MS)
    return () => clearInterval(h)
  }, [])

  return (
    <div className="min-h-screen bg-cream flex flex-col">
      <Header walletAddress={walletAddress} />
      {!bridgeAvailable() && <DisconnectedBanner />}
      {updaterStatus === 'ready' && <UpdateBanner />}

      <main className="max-w-4xl w-full mx-auto px-4 py-6 space-y-8 flex-1">
        <section>
          <SectionHeading
            title="Status"
            sub="Your prover is running in the background. These numbers update live."
          />
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            <Stat
              label="Active deals"
              value={dealsActive}
              sub={dealsActive === 0 ? 'none yet' : 'being served'}
              tone={dealsActive > 0 ? 'ok' : 'default'}
            />
            <Stat
              label="Proofs submitted"
              value={proofsSubmitted.toLocaleString()}
              sub="cumulative, since boot"
            />
            <Stat
              label="Uptime"
              value={formatDuration(uptime)}
              sub="current session"
            />
            <Stat
              label="Build"
              value={electron.buildVersion()}
              sub="Prova Desktop"
            />
          </div>
        </section>

        <section>
          <SectionHeading
            title="Wallet"
            sub="A local BIP-39 wallet was created on first launch. The seed is stored in your OS keychain."
            right={
              <button
                className="text-xs font-mono px-3 py-1.5 border border-ink/20 rounded-full hover:border-gold hover:text-gold transition-colors"
                onClick={() => void handleExportSeed()}
              >
                export seed
              </button>
            }
          />
          <div className="surface-card p-4 flex items-center justify-between gap-4">
            <div className="flex flex-col gap-1 min-w-0">
              <span className="text-xs uppercase tracking-wider text-ink/60">
                Address
              </span>
              <span className="mono text-sm truncate" title={walletAddress}>
                {walletAddress || '…'}
              </span>
            </div>
            {walletAddress && (
              <button
                className="text-xs font-mono px-3 py-1.5 border border-ink/20 rounded-full hover:border-gold hover:text-gold transition-colors flex-shrink-0"
                onClick={() => navigator.clipboard?.writeText(walletAddress)}
              >
                copy
              </button>
            )}
          </div>
        </section>

        <section>
          <SectionHeading
            title="Activity"
            sub="Events from the prover daemon as they happen."
            right={
              <button
                className="text-xs font-mono px-3 py-1.5 border border-ink/20 rounded-full hover:border-gold hover:text-gold transition-colors"
                onClick={() => void electron.saveLogsAs()}
              >
                save full log
              </button>
            }
          />
          {activities.length === 0 ? (
            <div className="surface-card p-8 text-center text-ink/50 text-sm">
              No activity yet. The prover is starting up; events will appear here
              as they happen.
            </div>
          ) : (
            <div className="surface-card divide-y divide-ink/5">
              {activities.slice(0, 30).map(a => (
                <ActivityRow key={a.id} activity={a} />
              ))}
            </div>
          )}
        </section>
      </main>

      <Footer />
    </div>
  )
}

// ─── Subcomponents ──────────────────────────────────────────────────

function Header({ walletAddress }: { walletAddress: string }) {
  return (
    <header className="bg-ink text-cream">
      <div className="max-w-4xl mx-auto px-4 py-4 flex items-center gap-3 draggable">
        <div className="text-gold">
          <Logo size={36} />
        </div>
        <div className="flex-1">
          <h1 className="font-semibold text-lg leading-none tracking-tight">
            Prova <span className="text-gold">Desktop</span>
          </h1>
          <div className="text-xs text-cream/60 mt-1">
            verifiable storage on Base, running locally
          </div>
        </div>
        {walletAddress && (
          <div className="hidden sm:flex items-center gap-2 px-3 py-1.5 border border-cream/15 rounded-full bg-white/5 text-xs">
            <span className="h-2 w-2 rounded-full bg-emerald-400 animate-pulse" aria-hidden />
            <span className="text-cream/80 font-mono">{shortAddr(walletAddress)}</span>
          </div>
        )}
      </div>
      <div className="h-[2px] bg-gradient-to-r from-transparent via-gold to-transparent" />
    </header>
  )
}

function Footer() {
  return (
    <footer className="border-t border-ink/10 bg-white/60 mt-8">
      <div className="max-w-4xl mx-auto px-4 py-4 text-xs text-ink/50 flex items-center justify-between gap-4 flex-wrap">
        <div>
          <button
            className="hover:text-gold transition-colors"
            onClick={() => void electron.openExternalURL('https://prova.network')}
          >
            prova.network
          </button>
          {' · '}
          <button
            className="hover:text-gold transition-colors"
            onClick={() => void electron.openExternalURL('https://github.com/Reiers/prova')}
          >
            github
          </button>
        </div>
        <div>Read-only view. No data leaves this machine except on-chain txs.</div>
      </div>
    </footer>
  )
}

function SectionHeading({
  title, sub, right
}: {
  title: string
  sub?: string
  right?: React.ReactNode
}) {
  return (
    <div className="flex items-end justify-between mb-3 gap-3">
      <div className="flex items-stretch gap-2.5">
        <span aria-hidden className="w-[3px] bg-gold rounded-full" />
        <div>
          <h2 className="text-sm font-semibold uppercase tracking-wider text-ink/70">
            {title}
          </h2>
          {sub && <p className="text-xs text-ink/40 mt-0.5 max-w-[52ch]">{sub}</p>}
        </div>
      </div>
      {right}
    </div>
  )
}

function ActivityRow({ activity }: { activity: Activity }) {
  const toneClass =
    activity.type === 'error'
      ? 'text-red-700 border-l-red-300'
      : activity.type === 'started'
        ? 'text-amber-700 border-l-amber-300'
        : 'text-ink border-l-gold/30'

  return (
    <div className={`px-4 py-3 border-l-2 ${toneClass} flex items-start gap-4`}>
      <div className="flex-1 min-w-0">
        <div className="text-sm">{activity.message}</div>
        <div className="text-xs text-ink/50 mt-0.5 font-mono">
          {activity.source}
        </div>
      </div>
      <div
        className="text-xs text-ink/50 font-mono whitespace-nowrap"
        title={String(activity.timestamp)}
      >
        {relativeTime(
          activity.timestamp instanceof Date
            ? activity.timestamp.toISOString()
            : activity.timestamp
        )}
      </div>
    </div>
  )
}

function DisconnectedBanner() {
  return (
    <div className="bg-red-100 text-red-900 border-b border-red-200 px-4 py-2 text-center text-xs">
      Electron bridge unavailable · renderer running in standalone mode.
      Stats + wallet functions are stubbed.
    </div>
  )
}

function UpdateBanner() {
  return (
    <div className="bg-gold text-ink px-4 py-3 flex items-center justify-center gap-4 text-sm">
      <span>A new version of Prova Desktop is ready to install.</span>
      <button
        className="font-mono text-xs px-3 py-1 rounded-full bg-ink text-gold hover:bg-ink/90 transition-colors"
        onClick={() => void electron.restartToUpdate()}
      >
        restart &amp; update
      </button>
    </div>
  )
}

async function handleExportSeed() {
  const confirmed = window.confirm(
    'Exporting your seed phrase reveals the key that controls this wallet. ' +
    'Anyone who sees it can take funds. Are you sure?'
  )
  if (!confirmed) return
  try {
    const phrase = await electron.exportSeedPhrase()
    // Show it in a dialog. In a future iteration this should be a modal with
    // a masked/unmasked toggle and a copy button.
    window.prompt('Your 12-word seed phrase (copy, then clear clipboard):', phrase)
  } catch (err) {
    window.alert(`Could not export seed: ${(err as Error).message}`)
  }
}
