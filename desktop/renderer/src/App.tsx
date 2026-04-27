import { useEffect, useState } from 'react'
import {
  electron,
  bridgeAvailable,
  type Activity,
  type DaemonStatus,
  type NetworkConfig,
  type NetworkKey,
  type NetworkPresetInfo,
  type ProverStats,
  type StorageDirInfo,
  type UpdaterStatus,
} from './api'
import { Logo } from './components/Logo'
import { Stat } from './components/Stat'
import { formatBytes, formatDuration, relativeTime, shortAddr } from './util'

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
  const [storage, setStorage] = useState<StorageDirInfo | null>(null)
  const [storageBusy, setStorageBusy] = useState(false)
  const [storageError, setStorageError] = useState<string | null>(null)
  const [network, setNetwork] = useState<NetworkConfig | null>(null)
  const [networkPresets, setNetworkPresets] = useState<NetworkPresetInfo[]>([])
  const [networkBusy, setNetworkBusy] = useState(false)
  const [onboardingNeeded, setOnboardingNeeded] = useState(false)
  const [firstRunAddress, setFirstRunAddress] = useState<string>('')
  const [seedModalOpen, setSeedModalOpen] = useState(false)
  const [daemonStatus, setDaemonStatus] = useState<DaemonStatus | null>(null)
  const [proverStats, setProverStats] = useState<ProverStats | null>(null)

  // ─── Initial state fetch ──────────────────────────────────────────
  useEffect(() => {
    let cancelled = false

    async function loadInitial() {
      try {
        const [addr, deals, proofs, acts, upd, storageInfo, netCfg, presets, onboard, dStatus, stats] = await Promise.all([
          electron.getWalletAddress().catch(() => ''),
          electron.getTotalDealsActive().catch(() => 0),
          electron.getTotalProofsSubmitted().catch(() => 0),
          electron.getActivities().catch(() => []),
          electron.getUpdaterStatus().catch(() => 'idle' as UpdaterStatus),
          electron.getStorageDir().catch(() => null),
          electron.getNetwork().catch(() => null),
          electron.listNetworks().catch(() => [] as NetworkPresetInfo[]),
          electron.getOnboardingState().catch(() => ({ completed: true, firstRunWalletAddress: '' })),
          electron.getDaemonStatus().catch(() => null as DaemonStatus | null),
          electron.getProverStats().catch(() => null as ProverStats | null),
        ])
        if (cancelled) return
        setWalletAddress(addr)
        setDealsActive(deals)
        setProofsSubmitted(proofs)
        setActivities(acts)
        setUpdaterStatus(upd)
        setStorage(storageInfo)
        setNetwork(netCfg)
        setNetworkPresets(presets)
        setDaemonStatus(dStatus)
        setProverStats(stats)
        // Show the multi-step first-run setup modal when (a) we just
        // generated a new wallet on this launch AND (b) the user hasn't
        // already completed onboarding on a previous launch.
        if (onboard && !onboard.completed && onboard.firstRunWalletAddress) {
          setOnboardingNeeded(true)
          setFirstRunAddress(onboard.firstRunWalletAddress)
        }
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
      electron.onStorageDirChanged(() => {
        electron.getStorageDir().then(setStorage).catch(() => {})
      }),
      electron.onNetworkChanged(cfg => {
        setNetwork(cfg)
        electron.listNetworks().then(setNetworkPresets).catch(() => {})
      }),
      electron.onDaemonStatusChanged(s => setDaemonStatus(s)),
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

  // ─── Stats poller: prover stats refresh every POLL_MS ─────────────
  useEffect(() => {
    const h = setInterval(() => {
      electron.getProverStats().then(setProverStats).catch(() => {})
    }, POLL_MS)
    return () => clearInterval(h)
  }, [])

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
    <div className="min-h-screen flex flex-col">
      <Header walletAddress={walletAddress} />
      {!bridgeAvailable() && <DisconnectedBanner />}
      {updaterStatus === 'ready' && <UpdateBanner />}
      {daemonStatus && daemonStatus.state === 'failing' && daemonStatus.consecutiveFailures >= 2 && (
        <DaemonFailingBanner
          status={daemonStatus}
          onSaveLogs={() => void electron.saveLogsAs()}
        />
      )}
      {onboardingNeeded && (
        <FirstRunModal
          address={firstRunAddress}
          presets={networkPresets}
          activeNetworkKey={network?.key ?? 'anvil'}
          onPickNetwork={key => electron.setNetwork(key).then(setNetwork)}
          onShowSeed={() => setSeedModalOpen(true)}
          onDone={() => {
            void electron.completeOnboarding()
            setOnboardingNeeded(false)
          }}
        />
      )}
      {seedModalOpen && (
        <SeedExportModal onClose={() => setSeedModalOpen(false)} />
      )}

      <main className="max-w-4xl w-full mx-auto px-4 py-6 space-y-8 flex-1">
        <section>
          <SectionHeading
            title="Status"
            sub={
              daemonStatus?.state === 'running'
                ? 'Your prover is running in the background. These numbers update live.'
                : daemonStatus?.state === 'failing'
                  ? `Daemon is failing to start. ${daemonStatus.consecutiveFailures} retries; supervisor will keep retrying.`
                  : daemonStatus?.state === 'starting'
                    ? 'Daemon is starting up...'
                    : 'Waiting on the prover daemon to come up.'
            }
          />
          <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
            <Stat
              label="Storage used"
              value={formatBytes(proverStats?.bytesStored ?? 0)}
              sub={
                proverStats && proverStats.piecesStored > 0
                  ? `${proverStats.piecesStored.toLocaleString()} piece${proverStats.piecesStored === 1 ? '' : 's'}`
                  : 'no pieces stored yet'
              }
              tone={proverStats && proverStats.bytesStored > 0 ? 'ok' : 'default'}
            />
            <Stat
              label="Earnings"
              value={
                proverStats?.earnedUsdc == null
                  ? '—'
                  : `$${proverStats.earnedUsdc.toFixed(2)}`
              }
              sub={
                proverStats?.earnedUsdc == null
                  ? 'available once on-chain reads are wired'
                  : 'lifetime USDC paid'
              }
            />
            <Stat
              label="Staked"
              value={
                proverStats?.stakedProva == null
                  ? '—'
                  : `${proverStats.stakedProva.toLocaleString()} PROVA`
              }
              sub={
                proverStats?.stakedProva == null
                  ? 'available once on-chain reads are wired'
                  : 'committed for this prover'
              }
            />
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
              label="Session uptime"
              value={formatDuration(uptime)}
              sub={`build ${electron.buildVersion()}`}
            />
          </div>
        </section>

        <section>
          <SectionHeading
            title="Wallet"
            sub="A local BIP-39 wallet was created on first launch. The seed is stored in your OS keychain."
            right={
              <button
                className="pill-button"
                onClick={() => setSeedModalOpen(true)}
              >
                export seed
              </button>
            }
          />
          <div className="surface-card p-4 flex items-center justify-between gap-4">
            <div className="flex flex-col gap-1 min-w-0">
              <span className="text-[11px] uppercase tracking-wider text-ink/55 dark:text-cream/55">
                Wallet address
              </span>
              <span className="mono text-sm truncate" title={walletAddress}>
                {walletAddress || '…'}
              </span>
            </div>
            {walletAddress && (
              <button
                className="pill-button flex-shrink-0"
                onClick={() => navigator.clipboard?.writeText(walletAddress)}
              >
                copy
              </button>
            )}
          </div>
        </section>

        <section>
          <SectionHeading
            title="Network"
            sub="Pick which chain the prover talks to. Pieces and stake are chain-specific."
          />
          <div className="surface-card p-4 flex items-center justify-between gap-4 flex-wrap">
            <div className="flex flex-col gap-1 min-w-0">
              <span className="font-display text-sm" title={network?.rpcUrl}>
                {network?.label ?? '…'}
                <span className="ml-2 text-ink/40 font-mono">
                  chain {network?.chainId ?? '?'}
                </span>
              </span>
              {network && !network.isConfigured && (
                <span className="text-[11px] text-amber-700">
                  No deployed contracts on this chain yet — the prover will start, but on-chain
                  events won’t flow until contract addresses are configured.
                </span>
              )}
            </div>
            <div className="flex items-center gap-2 flex-wrap">
              {networkPresets.map(p => (
                <button
                  key={p.key}
                  className={
                    'pill-button ' +
                    (network?.key === p.key
                      ? 'border-teal-cyan/70 text-teal-deep dark:text-teal-cyan bg-white/40 dark:bg-ink/30'
                      : '')
                  }
                  disabled={networkBusy || network?.key === p.key}
                  onClick={() => {
                    setNetworkBusy(true)
                    electron
                      .setNetwork(p.key)
                      .then(setNetwork)
                      .finally(() => setNetworkBusy(false))
                  }}
                  title={`${p.label} — ${p.rpcUrl}`}
                >
                  {p.label.replace(/ \(.+\)$/, '')}
                </button>
              ))}
            </div>
          </div>
          {network && (
            <p className="text-xs text-ink/50 mt-2">
              Restart the app for the network change to take effect.
            </p>
          )}
        </section>

        <section>
          <SectionHeading
            title="Storage"
            sub="Pieces are written here. Pick a folder on a fast internal drive, or an external drive you'll keep plugged in."
            right={
              storage?.isCustom ? (
                <button
                  className="pill-button"
                  disabled={storageBusy}
                  onClick={() => {
                    setStorageBusy(true)
                    setStorageError(null)
                    electron
                      .resetStorageDir()
                      .then(() => electron.getStorageDir())
                      .then(setStorage)
                      .catch(err => setStorageError(String(err?.message ?? err)))
                      .finally(() => setStorageBusy(false))
                  }}
                >
                  reset to default
                </button>
              ) : null
            }
          />
          <div className="surface-card p-4 flex items-center justify-between gap-4">
            <div className="flex flex-col gap-1 min-w-0">
              <span className="text-[11px] uppercase tracking-wider text-ink/55 dark:text-cream/55">
                {storage?.isCustom ? 'Custom path' : 'Default path'}
              </span>
              <span className="mono text-sm truncate" title={storage?.current ?? ''}>
                {storage?.current ?? '…'}
              </span>
              {storageError && (
                <span className="text-xs text-red-600">{storageError}</span>
              )}
            </div>
            <button
              className="pill-button flex-shrink-0"
              disabled={storageBusy}
              onClick={() => {
                setStorageBusy(true)
                setStorageError(null)
                electron
                  .selectStorageDir()
                  .then(chosen => {
                    if (chosen) return electron.getStorageDir().then(setStorage)
                  })
                  .catch(err => setStorageError(String(err?.message ?? err)))
                  .finally(() => setStorageBusy(false))
              }}
            >
              {storageBusy ? 'choosing…' : 'change…'}
            </button>
          </div>
          {storage?.isCustom && (
            <p className="text-xs text-ink/50 mt-2">
              Restart the app for the new location to take effect.
            </p>
          )}
        </section>

        <section>
          <SectionHeading
            title="Activity"
            sub="Events from the prover daemon as they happen."
            right={
              <button
                className="pill-button"
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
    <header className="draggable">
      <div className="max-w-4xl mx-auto px-4 pt-3 pb-2 tahoe-titlebar-pad flex items-center gap-3">
        <Logo size={28} />
        <div className="flex-1">
          <h1 className="font-display font-semibold text-[15px] leading-none tracking-tighter">
            Prova
          </h1>
          <div className="text-[11px] text-ink/55 mt-0.5 dark:text-cream/55">
            verifiable storage on Base
          </div>
        </div>
        {walletAddress && (
          <div
            className="hidden sm:flex items-center gap-2 pill-button bg-white/40 dark:bg-ink/30"
            title={walletAddress}
          >
            <span className="h-2 w-2 rounded-full bg-emerald-500 animate-pulse" aria-hidden />
            <span className="font-mono">{shortAddr(walletAddress)}</span>
          </div>
        )}
      </div>
    </header>
  )
}

function Footer() {
  return (
    <footer className="border-t border-ink/10 bg-white/60 mt-8">
      <div className="max-w-4xl mx-auto px-4 py-4 text-xs text-ink/50 flex items-center justify-between gap-4 flex-wrap">
        <div>
          <button
            className="hover:text-teal-deep dark:hover:text-teal-cyan transition-colors"
            onClick={() => void electron.openExternalURL('https://prova.network')}
          >
            prova.network
          </button>
          {' · '}
          <button
            className="hover:text-teal-deep dark:hover:text-teal-cyan transition-colors"
            onClick={() => void electron.openExternalURL('https://github.com/prova-network/prova')}
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
        <span aria-hidden className="w-[3px] rounded-full bg-gradient-to-b from-teal-cyan to-teal-deep" />
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
        ? 'text-emerald-700 border-l-emerald-300'
        : 'text-ink border-l-teal-cyan/40 dark:text-cream dark:border-l-teal-cyan/40'

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
    <div className="px-4 py-3 flex items-center justify-center gap-4 text-sm border-b border-teal-cyan/30 bg-teal-cyan/15 text-teal-deep dark:text-teal-cyan">
      <span>A new version of Prova Desktop is ready to install.</span>
      <button
        className="font-mono text-xs px-3 py-1 rounded-full bg-teal-deep text-cream hover:bg-teal-deep/90 transition-colors"
        onClick={() => void electron.restartToUpdate()}
      >
        restart &amp; update
      </button>
    </div>
  )
}

// ── First-run setup: pick chain, back up seed, finish ───────────────────────────────
function FirstRunModal({
  address,
  presets,
  activeNetworkKey,
  onPickNetwork,
  onShowSeed,
  onDone,
}: {
  address: string
  presets: NetworkPresetInfo[]
  activeNetworkKey: NetworkKey
  onPickNetwork: (key: NetworkKey) => Promise<unknown>
  onShowSeed: () => void
  onDone: () => void
}) {
  const [step, setStep] = useState<1 | 2 | 3>(1)
  const [pickedNetwork, setPickedNetwork] = useState<NetworkKey>(activeNetworkKey)
  const [seedShown, setSeedShown] = useState(false)

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center px-4 py-8 bg-black/40 backdrop-blur-sm">
      <div className="surface-card max-w-xl w-full p-6 space-y-5">
        <div className="flex items-center justify-between">
          <h2 className="font-display text-lg font-semibold tracking-tight">
            Welcome to Prova
          </h2>
          <span className="text-xs font-mono text-ink/40">
            step {step} of 3
          </span>
        </div>

        {step === 1 && (
          <div className="space-y-4">
            <p className="text-sm text-ink/70 dark:text-cream/70">
              We just created a fresh wallet for this prover. The mnemonic
              is stored in your OS keychain so this app can sign on its
              behalf. <span className="font-semibold">Back it up before storing anything important.</span>
            </p>
            <div className="surface-card p-3 font-mono text-xs break-all">
              {address}
            </div>
            <div className="flex justify-end gap-2">
              <button
                className="pill-button"
                onClick={() => {
                  setSeedShown(true)
                  onShowSeed()
                }}
              >
                show seed phrase
              </button>
              <button
                className="pill-button border-teal-cyan/60 text-teal-deep dark:text-teal-cyan"
                disabled={!seedShown}
                title={seedShown ? '' : 'Reveal the seed first so you can back it up'}
                onClick={() => setStep(2)}
              >
                next
              </button>
            </div>
          </div>
        )}

        {step === 2 && (
          <div className="space-y-4">
            <p className="text-sm text-ink/70 dark:text-cream/70">
              Pick the chain this prover will run on. You can change this
              later in the Network section.
            </p>
            <div className="flex flex-col gap-2">
              {presets.map(p => (
                <button
                  key={p.key}
                  className={
                    'text-left surface-card p-3 transition-colors ' +
                    (pickedNetwork === p.key
                      ? 'border-teal-cyan/70 ring-1 ring-teal-cyan/40'
                      : '')
                  }
                  onClick={() => setPickedNetwork(p.key)}
                >
                  <div className="flex items-baseline justify-between gap-3">
                    <span className="font-display text-sm font-semibold">
                      {p.label}
                    </span>
                    <span className="font-mono text-[11px] text-ink/40">
                      chain {p.chainId}
                    </span>
                  </div>
                  <div className="font-mono text-[11px] text-ink/50 mt-1 truncate">
                    {p.rpcUrl}
                  </div>
                  {!p.isConfigured && (
                    <div className="text-[11px] text-amber-700 mt-1">
                      Contracts not deployed yet on this chain.
                    </div>
                  )}
                </button>
              ))}
            </div>
            <div className="flex justify-between gap-2">
              <button className="pill-button" onClick={() => setStep(1)}>back</button>
              <button
                className="pill-button border-teal-cyan/60 text-teal-deep dark:text-teal-cyan"
                onClick={() => {
                  void onPickNetwork(pickedNetwork)
                  setStep(3)
                }}
              >
                use {presets.find(p => p.key === pickedNetwork)?.label.replace(/ \(.+\)$/, '')}
              </button>
            </div>
          </div>
        )}

        {step === 3 && (
          <div className="space-y-4">
            <p className="text-sm text-ink/70 dark:text-cream/70">
              You're set up. Storage location, network, and updates can be
              changed any time from the dashboard.
            </p>
            <div className="surface-card p-3 text-xs space-y-1">
              <div><span className="text-ink/50">Wallet:</span> <span className="font-mono">{address}</span></div>
              <div><span className="text-ink/50">Network:</span> {presets.find(p => p.key === pickedNetwork)?.label}</div>
            </div>
            <div className="flex justify-end">
              <button
                className="pill-button border-teal-cyan/60 text-teal-deep dark:text-teal-cyan"
                onClick={onDone}
              >
                let's go
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

// ── Daemon-failing banner: shown after >=2 consecutive supervisor failures ─────
function DaemonFailingBanner({
  status,
  onSaveLogs,
}: {
  status: DaemonStatus
  onSaveLogs: () => void
}) {
  const exitCodeNote =
    status.lastExitCode === null
      ? '(no exit code)'
      : `(exit code ${status.lastExitCode})`
  return (
    <div className="px-4 py-3 flex items-start justify-center gap-4 text-sm border-b border-red-300/70 bg-red-100/80 text-red-900">
      <div className="max-w-2xl">
        <div className="font-semibold">
          Prover daemon is failing to start {exitCodeNote}.
        </div>
        <div className="text-xs mt-0.5 break-words">
          {status.lastError || 'No diagnostic message captured.'}
        </div>
        <div className="text-xs text-red-900/60 mt-0.5">
          Retried {status.consecutiveFailures} time{status.consecutiveFailures === 1 ? '' : 's'}; supervisor will keep retrying with backoff.
        </div>
      </div>
      <button className="pill-button-primary shrink-0" onClick={onSaveLogs}>
        save logs
      </button>
    </div>
  )
}

// ── Seed export modal: masked by default, reveal toggle, copy button ───────────────
function SeedExportModal({ onClose }: { onClose: () => void }) {
  const [confirmed, setConfirmed] = useState(false)
  const [phrase, setPhrase] = useState<string | null>(null)
  const [revealed, setRevealed] = useState(false)
  const [copied, setCopied] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Once the user accepts the warning, fetch the mnemonic from main.
  useEffect(() => {
    if (!confirmed) return
    let cancelled = false
    electron
      .exportSeedPhrase()
      .then(p => { if (!cancelled) setPhrase(p) })
      .catch(err => { if (!cancelled) setError(String(err?.message ?? err)) })
    return () => { cancelled = true }
  }, [confirmed])

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center px-4 py-8 bg-black/40 backdrop-blur-sm">
      <div className="surface-card max-w-xl w-full p-6 space-y-5">
        <div className="flex items-center justify-between">
          <h2 className="font-display text-lg font-semibold tracking-tight">
            Seed phrase
          </h2>
          <button className="pill-button" onClick={onClose}>close</button>
        </div>

        {!confirmed && (
          <div className="space-y-4">
            <p className="text-sm text-ink/70 dark:text-cream/70">
              Anyone who sees this 12-word phrase can take any funds in this
              wallet. Make sure no screen recording, screenshare, or other
              process is observing this window before you continue.
            </p>
            <div className="flex justify-end gap-2">
              <button className="pill-button" onClick={onClose}>
                cancel
              </button>
              <button
                className="pill-button border-amber-400/70 text-amber-700"
                onClick={() => setConfirmed(true)}
              >
                I understand, show it
              </button>
            </div>
          </div>
        )}

        {confirmed && error && (
          <div className="text-sm text-red-700">
            Could not export seed: {error}
          </div>
        )}

        {confirmed && !error && phrase === null && (
          <div className="text-sm text-ink/60">decrypting…</div>
        )}

        {confirmed && phrase !== null && (
          <div className="space-y-4">
            <div className="surface-card p-4 font-mono text-sm leading-relaxed select-all">
              {revealed
                ? phrase
                : phrase
                    .split(' ')
                    .map(w => '•'.repeat(Math.max(4, w.length)))
                    .join(' ')}
            </div>
            <div className="flex items-center gap-2 justify-end">
              <button
                className="pill-button"
                onClick={() => setRevealed(r => !r)}
              >
                {revealed ? 'hide' : 'reveal'}
              </button>
              <button
                className="pill-button"
                onClick={() => {
                  void navigator.clipboard?.writeText(phrase)
                  setCopied(true)
                  setTimeout(() => setCopied(false), 1500)
                }}
              >
                {copied ? 'copied' : 'copy'}
              </button>
              <button className="pill-button border-teal-cyan/60 text-teal-deep dark:text-teal-cyan" onClick={onClose}>
                done
              </button>
            </div>
            <p className="text-xs text-ink/50">
              Tip: write it on paper. Clipboard contents are accessible to
              every app on your machine — don't leave it sitting there.
            </p>
          </div>
        )}
      </div>
    </div>
  )
}
