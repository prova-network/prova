import { useEffect, useMemo, useState } from 'react'
import {
  electron,
  bridgeAvailable,
  type Activity,
  type DaemonStatus,
  type NetworkConfig,
  type NetworkKey,
  type NetworkPresetInfo,
  type ProverStats,
  type StakeSnapshot,
  type StorageDirInfo,
  type UpdaterStatus,
} from './api'
import { Logo } from './components/Logo'
import { Stat } from './components/Stat'
import { formatBytes, formatDuration, relativeTime, shortAddr } from './util'

// Push-based values (activity feed, deals active, proofs submitted) update
// instantly via IPC subscriptions; polling is a safety net for the rest.
const POLL_MS = 10_000
const STAKE_POLL_MS = 6_000

type View = 'dashboard' | 'stake' | 'deals' | 'settings'

export default function App() {
  // ── Shared state used across views ────────────────────────────────
  const [view, setView] = useState<View>('dashboard')
  const [walletAddress, setWalletAddress] = useState('')
  const [dealsActive, setDealsActive] = useState(0)
  const [proofsSubmitted, setProofsSubmitted] = useState(0)
  const [activities, setActivities] = useState<Activity[]>([])
  const [updaterStatus, setUpdaterStatus] = useState<UpdaterStatus>('idle')
  const [uptime, setUptime] = useState(0)
  const [bootedAt] = useState(() => Date.now())
  const [storage, setStorage] = useState<StorageDirInfo | null>(null)
  const [network, setNetwork] = useState<NetworkConfig | null>(null)
  const [networkPresets, setNetworkPresets] = useState<NetworkPresetInfo[]>([])
  const [daemonStatus, setDaemonStatus] = useState<DaemonStatus | null>(null)
  const [proverStats, setProverStats] = useState<ProverStats | null>(null)
  const [stake, setStake] = useState<StakeSnapshot | null>(null)
  const [seedModalOpen, setSeedModalOpen] = useState(false)
  const [onboardingNeeded, setOnboardingNeeded] = useState(false)
  const [firstRunAddress, setFirstRunAddress] = useState('')

  // ── Initial load ──────────────────────────────────────────────────
  useEffect(() => {
    let cancelled = false
    ;(async () => {
      try {
        const [
          addr, deals, proofs, acts, upd, storageInfo, netCfg, presets, onboard, dStatus, pStats, stk,
        ] = await Promise.all([
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
          electron.getStakeSnapshot().catch(() => null as StakeSnapshot | null),
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
        setProverStats(pStats)
        setStake(stk)
        if (onboard && !onboard.completed && onboard.firstRunWalletAddress) {
          setOnboardingNeeded(true)
          setFirstRunAddress(onboard.firstRunWalletAddress)
        }
      } catch {
        // main process not ready yet; subscriptions will fill in
      }
    })()
    return () => { cancelled = true }
  }, [])

  // ── Push subscriptions ───────────────────────────────────────────
  useEffect(() => {
    const unsubs = [
      electron.onWalletAddressUpdated(addr => setWalletAddress(addr)),
      electron.onDealsActiveUpdated(n => setDealsActive(n)),
      electron.onProofStatsUpdated(n => setProofsSubmitted(n)),
      electron.onUpdaterStatusChanged(s => setUpdaterStatus(s)),
      electron.onActivityLogged(a => {
        setActivities(prev => [a, ...prev].slice(0, 200))
      }),
      electron.onStorageDirChanged(() => {
        electron.getStorageDir().then(setStorage).catch(() => {})
      }),
      electron.onNetworkChanged(cfg => {
        setNetwork(cfg)
        electron.listNetworks().then(setNetworkPresets).catch(() => {})
      }),
      electron.onDaemonStatusChanged(s => setDaemonStatus(s)),
      electron.onSetView(v => {
        if (v === 'dashboard' || v === 'stake' || v === 'deals' || v === 'settings') {
          setView(v)
        }
      }),
    ]
    return () => unsubs.forEach(u => u())
  }, [])

  // ── Pollers ──────────────────────────────────────────────────────
  useEffect(() => {
    const h = setInterval(() => setUptime(Math.floor((Date.now() - bootedAt) / 1000)), 1000)
    return () => clearInterval(h)
  }, [bootedAt])
  useEffect(() => {
    const h = setInterval(() => {
      electron.getProverStats().then(setProverStats).catch(() => {})
    }, POLL_MS)
    return () => clearInterval(h)
  }, [])
  useEffect(() => {
    const h = setInterval(() => {
      electron.getStakeSnapshot().then(setStake).catch(() => {})
    }, STAKE_POLL_MS)
    return () => clearInterval(h)
  }, [])

  return (
    <div className="min-h-screen flex flex-col">
      {!bridgeAvailable() && <DisconnectedBanner />}
      {updaterStatus === 'ready' && <UpdateBanner />}
      {daemonStatus && daemonStatus.state === 'failing' && daemonStatus.consecutiveFailures >= 2 && (
        <DaemonFailingBanner status={daemonStatus} onSaveLogs={() => void electron.saveLogsAs()} />
      )}

      {/* Drag-handle bar with the macOS traffic-light reservation */}
      <div className="draggable h-11 tahoe-titlebar-pad flex items-center px-4 text-[11px] text-ink/65 dark:text-cream/65 border-b border-ink/10 dark:border-cream/10 bg-white/50 dark:bg-ink/50">
        <span
          className={
            'inline-block h-2 w-2 rounded-full mr-2 ' +
            (daemonStatus?.state === 'running'
              ? 'bg-emerald-500'
              : daemonStatus?.state === 'failing'
                ? 'bg-red-500'
                : daemonStatus?.state === 'starting'
                  ? 'bg-amber-500 animate-pulse'
                  : 'bg-ink/30 dark:bg-cream/30')
          }
          aria-hidden
        />
        <span className="font-mono text-ink dark:text-cream">{network?.label ?? '…'}</span>
        <span className="mx-2 opacity-40">·</span>
        <span className={
          'font-mono ' +
          (daemonStatus?.state === 'running'
            ? 'text-emerald-700 dark:text-emerald-400'
            : daemonStatus?.state === 'failing'
              ? 'text-red-700 dark:text-red-400'
              : 'text-ink/65 dark:text-cream/65')
        }>
          daemon {daemonStatus?.state ?? 'idle'}
        </span>
      </div>

      <div className="flex flex-1 min-h-0">
        <Sidebar
          view={view}
          onView={setView}
          walletAddress={walletAddress}
          stake={stake}
          provaDecimals={stake?.provaDecimals ?? 18}
        />

        <main className="flex-1 overflow-y-auto px-6 py-6 max-w-5xl mx-auto w-full space-y-8">
          {view === 'dashboard' && (
            <DashboardView
              proverStats={proverStats}
              dealsActive={dealsActive}
              proofsSubmitted={proofsSubmitted}
              uptime={uptime}
              activities={activities}
              daemonStatus={daemonStatus}
            />
          )}
          {view === 'stake' && (
            <StakeView
              snapshot={stake}
              network={network}
              onAfterAction={() => electron.getStakeSnapshot().then(setStake).catch(() => {})}
            />
          )}
          {view === 'deals' && <DealsView dealsActive={dealsActive} />}
          {view === 'settings' && (
            <SettingsView
              walletAddress={walletAddress}
              storage={storage}
              network={network}
              networkPresets={networkPresets}
              onShowSeed={() => setSeedModalOpen(true)}
            />
          )}
        </main>
      </div>

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
      {seedModalOpen && <SeedExportModal onClose={() => setSeedModalOpen(false)} />}
    </div>
  )
}

// ─── Sidebar ──────────────────────────────────────────────────────────
function Sidebar({
  view, onView, walletAddress, stake, provaDecimals,
}: {
  view: View
  onView: (v: View) => void
  walletAddress: string
  stake: StakeSnapshot | null
  provaDecimals: number
}) {
  const items: Array<{ key: View; label: string; icon: React.ReactNode }> = [
    { key: 'dashboard', label: 'Dashboard', icon: <NavIconDashboard /> },
    { key: 'stake',     label: 'Stake',     icon: <NavIconStake /> },
    { key: 'deals',     label: 'Deals',     icon: <NavIconDeals /> },
    { key: 'settings',  label: 'Settings',  icon: <NavIconSettings /> },
  ]
  const provaBalance = useMemo(
    () => stake ? formatTokenAmount(stake.provaWei, provaDecimals, 2) : '—',
    [stake, provaDecimals]
  )
  const stakedAmount = useMemo(
    () => stake ? formatTokenAmount(stake.stakedWei, provaDecimals, 2) : '—',
    [stake, provaDecimals]
  )

  return (
    <aside className="w-60 shrink-0 border-r border-ink/10 dark:border-cream/10 px-3 py-4 flex flex-col gap-0.5 bg-white/60 dark:bg-ink/60">
      <div className="px-3 pb-4 flex items-center gap-2.5">
        <span className="text-teal-deep dark:text-teal-cyan">
          <Logo size={26} />
        </span>
        <span className="font-display font-semibold text-[15px] tracking-tighter text-ink dark:text-cream">Prova</span>
      </div>
      {items.map(it => (
        <button
          key={it.key}
          onClick={() => onView(it.key)}
          className={
            'w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ' +
            (view === it.key
              ? 'bg-teal-cyan/20 text-teal-deep dark:text-teal-cyan font-semibold'
              : 'text-ink/85 dark:text-cream/85 hover:bg-ink/5 dark:hover:bg-cream/5')
          }
        >
          <span
            className={
              'w-4 h-4 shrink-0 ' +
              (view === it.key
                ? 'text-teal-deep dark:text-teal-cyan'
                : 'text-ink/65 dark:text-cream/70')
            }
            aria-hidden
          >
            {it.icon}
          </span>
          <span>{it.label}</span>
        </button>
      ))}

      <div className="mt-auto px-3 pt-4 border-t border-ink/10 dark:border-cream/10 text-[11px]">
        <div className="text-ink/65 dark:text-cream/65">Prover wallet</div>
        <div className="mono text-[12px] truncate text-ink dark:text-cream" title={walletAddress}>
          {shortAddr(walletAddress) || '…'}
        </div>
        <div className="mt-2 flex items-center justify-between">
          <span className="text-ink/65 dark:text-cream/65">PROVA</span>
          <span className="mono text-ink dark:text-cream">{provaBalance}</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-ink/65 dark:text-cream/65">Staked</span>
          <span className="mono text-ink dark:text-cream">{stakedAmount}</span>
        </div>
      </div>
    </aside>
  )
}

// ─── Sidebar nav icons (inline SVG, picks up currentColor) ──────────────────────────────
function NavIconDashboard() {
  return (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth={1.6} strokeLinecap="round" strokeLinejoin="round" className="w-full h-full">
      <rect x="2" y="2" width="5" height="5" rx="1" />
      <rect x="9" y="2" width="5" height="3" rx="1" />
      <rect x="2" y="9" width="5" height="5" rx="1" />
      <rect x="9" y="7" width="5" height="7" rx="1" />
    </svg>
  )
}
function NavIconStake() {
  return (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth={1.6} strokeLinecap="round" strokeLinejoin="round" className="w-full h-full">
      <ellipse cx="8" cy="4" rx="5" ry="2" />
      <path d="M3 4v8c0 1.1 2.24 2 5 2s5-.9 5-2V4" />
      <path d="M3 8c0 1.1 2.24 2 5 2s5-.9 5-2" />
    </svg>
  )
}
function NavIconDeals() {
  return (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth={1.6} strokeLinecap="round" strokeLinejoin="round" className="w-full h-full">
      <rect x="2" y="3" width="12" height="10" rx="1.5" />
      <path d="M5 7h6M5 10h4" />
    </svg>
  )
}
function NavIconSettings() {
  return (
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth={1.6} strokeLinecap="round" strokeLinejoin="round" className="w-full h-full">
      <circle cx="8" cy="8" r="2.4" />
      <path d="M8 1.5v1.6M8 12.9v1.6M14.5 8h-1.6M3.1 8H1.5M12.6 3.4l-1.1 1.1M4.5 11.5l-1.1 1.1M12.6 12.6l-1.1-1.1M4.5 4.5L3.4 3.4" />
    </svg>
  )
}

// ─── Dashboard view ───────────────────────────────────────────────────
function DashboardView({
  proverStats, dealsActive, proofsSubmitted, uptime, activities, daemonStatus,
}: {
  proverStats: ProverStats | null
  dealsActive: number
  proofsSubmitted: number
  uptime: number
  activities: Activity[]
  daemonStatus: DaemonStatus | null
}) {
  return (
    <>
      <section>
        <SectionHeading
          title="Status"
          sub={
            daemonStatus?.state === 'running'
              ? 'Daemon is online and polling the chain. These numbers update live.'
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
            value={proverStats?.earnedUsdc == null ? '—' : `$${proverStats.earnedUsdc.toFixed(2)}`}
            sub={proverStats?.earnedUsdc == null ? 'available once on-chain reads are wired' : 'lifetime USDC paid'}
          />
          <Stat
            label="Staked"
            value={proverStats?.stakedProva == null ? '—' : `${proverStats.stakedProva.toLocaleString()} PROVA`}
            sub={proverStats?.stakedProva == null ? 'see Stake tab for the on-chain figure' : 'committed for this prover'}
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
            sub="app open"
          />
        </div>
      </section>

      <section>
        <SectionHeading
          title="Activity"
          sub="Events from the prover daemon as they happen."
          right={
            <button className="pill-button" onClick={() => void electron.saveLogsAs()}>
              save full log
            </button>
          }
        />
        {activities.length === 0 ? (
          <div className="surface-card p-8 text-center text-ink/50 dark:text-cream/50 text-sm">
            No activity yet. The prover is starting up; events will appear here as they happen.
          </div>
        ) : (
          <div className="surface-card divide-y divide-ink/5 dark:divide-cream/5">
            {activities.slice(0, 30).map(a => <ActivityRow key={a.id} activity={a} />)}
          </div>
        )}
      </section>
    </>
  )
}

// ─── Stake view ───────────────────────────────────────────────────────
function StakeView({
  snapshot, network, onAfterAction,
}: {
  snapshot: StakeSnapshot | null
  network: NetworkConfig | null
  onAfterAction: () => void
}) {
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [lastTx, setLastTx] = useState<string | null>(null)
  const [stakeAmount, setStakeAmount] = useState('')
  const [unstakeAmount, setUnstakeAmount] = useState('')

  const decimals = snapshot?.provaDecimals ?? 18
  const provaBal = snapshot ? formatTokenAmount(snapshot.provaWei, decimals, 4) : '—'
  const ethBal = snapshot ? formatTokenAmount(snapshot.ethWei, 18, 4) : '—'
  const staked = snapshot ? formatTokenAmount(snapshot.stakedWei, decimals, 4) : '—'
  const unbonding = snapshot ? formatTokenAmount(snapshot.unbondingWei, decimals, 4) : '—'
  const unbondingReady =
    !!snapshot && BigInt(snapshot.unbondingWei) > 0n &&
    snapshot.unbondingEndsAt > 0 &&
    Date.now() / 1000 >= snapshot.unbondingEndsAt
  const unbondingCountdown =
    snapshot && BigInt(snapshot.unbondingWei) > 0n
      ? formatDuration(Math.max(0, snapshot.unbondingEndsAt - Math.floor(Date.now() / 1000)))
      : null

  const contractsConfigured =
    !!network && Object.values(network.contracts).every(a => a && a.length > 0)

  async function run<T>(fn: () => Promise<T>) {
    setBusy(true); setError(null); setLastTx(null)
    try {
      const r = await fn() as { txHash?: string }
      if (r && r.txHash) setLastTx(r.txHash)
      onAfterAction()
    } catch (e) {
      setError(extractErrorMessage(e))
    } finally {
      setBusy(false)
    }
  }

  return (
    <>
      <section>
        <SectionHeading
          title="Stake"
          sub="Stake PROVA to take deals and earn USDC. Unstaking moves stake into a 14-day cooling period before withdrawal."
        />

        {!contractsConfigured && (
          <div className="surface-card p-4 mb-4 text-sm border-l-4 border-l-amber-400">
            <div className="font-semibold text-amber-700">No deployed contracts on this chain.</div>
            <div className="text-ink/70 dark:text-cream/70 mt-1 text-xs">
              Switch to a network with contracts deployed in <span className="font-mono">Settings → Network</span>, or run the contract deploy script against your local anvil.
            </div>
          </div>
        )}

        {snapshot && !snapshot.isRegistered && contractsConfigured && (
          <div className="surface-card p-4 mb-4 text-sm border-l-4 border-l-teal-cyan">
            <div className="font-semibold">Register this prover</div>
            <div className="text-ink/70 dark:text-cream/70 mt-1 text-xs">
              Before you can take deals, the prover must be registered in <span className="font-mono">ProverRegistry</span>. This is a one-time on-chain transaction.
            </div>
            <button
              className="pill-button-primary mt-3"
              disabled={busy}
              onClick={() => run(() => electron.registerProver({ endpoint: 'https://localhost' }))}
            >
              {busy ? 'submitting…' : 'register prover'}
            </button>
          </div>
        )}

        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-6">
          <Stat label="Staked"          value={`${staked} PROVA`}   sub="locked, slashable on fault" tone={BigInt(snapshot?.stakedWei || '0') > 0n ? 'ok' : 'default'} />
          <Stat label="Unbonding"       value={`${unbonding} PROVA`} sub={unbondingCountdown ? `ready in ${unbondingCountdown}` : 'no pending unstakes'} />
          <Stat label="Wallet balance"  value={`${provaBal} PROVA`} sub="available to stake" />
          <Stat label="ETH for gas"     value={`${ethBal} ETH`}    sub="pays for tx fees on-chain" />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Stake form */}
          <div className="surface-card p-5">
            <h3 className="font-display text-sm font-semibold mb-1">Stake PROVA</h3>
            <p className="text-xs text-ink/55 dark:text-cream/55 mb-4">
              Locks PROVA in <span className="font-mono">ProverStaking</span>. Sets a per-TiB capacity ceiling.
            </p>
            <AmountInput
              value={stakeAmount}
              onChange={setStakeAmount}
              maxLabel={`max ${provaBal} PROVA`}
              onMax={() => snapshot && setStakeAmount(formatTokenAmount(snapshot.provaWei, decimals, 6))}
              suffix="PROVA"
            />
            <button
              className="pill-button-primary mt-4 w-full"
              disabled={busy || !contractsConfigured || !stakeAmount || !snapshot}
              onClick={() => run(async () => {
                const wei = parseTokenAmount(stakeAmount, decimals)
                return electron.stake(wei.toString())
              })}
            >
              {busy ? 'submitting…' : 'stake'}
            </button>
          </div>

          {/* Unstake / withdraw form */}
          <div className="surface-card p-5">
            <h3 className="font-display text-sm font-semibold mb-1">Unstake / withdraw</h3>
            <p className="text-xs text-ink/55 dark:text-cream/55 mb-4">
              Unstake moves PROVA into a 14-day cooling queue. After it ends, withdraw transfers it back to your wallet.
            </p>
            <AmountInput
              value={unstakeAmount}
              onChange={setUnstakeAmount}
              maxLabel={`max ${staked} PROVA`}
              onMax={() => snapshot && setUnstakeAmount(formatTokenAmount(snapshot.stakedWei, decimals, 6))}
              suffix="PROVA"
            />
            <div className="flex gap-2 mt-4">
              <button
                className="pill-button flex-1"
                disabled={busy || !contractsConfigured || !unstakeAmount || !snapshot}
                onClick={() => run(async () => {
                  const wei = parseTokenAmount(unstakeAmount, decimals)
                  return electron.requestUnstake(wei.toString())
                })}
              >
                request unstake
              </button>
              <button
                className="pill-button-primary flex-1"
                disabled={busy || !contractsConfigured || !unbondingReady}
                title={!unbondingReady ? 'Available after the 14-day unbonding period ends' : ''}
                onClick={() => run(() => electron.withdrawUnbonded())}
              >
                withdraw
              </button>
            </div>
          </div>
        </div>

        {(error || lastTx) && (
          <div className="mt-4 surface-card p-4 text-sm">
            {error && (
              <div className="text-red-700 dark:text-red-300 break-all">
                <span className="font-semibold">Error: </span>{error}
              </div>
            )}
            {lastTx && (
              <div className="text-emerald-700 dark:text-emerald-300 mono break-all">
                tx submitted: {lastTx}
              </div>
            )}
          </div>
        )}
      </section>
    </>
  )
}

function AmountInput({
  value, onChange, suffix, maxLabel, onMax,
}: {
  value: string
  onChange: (v: string) => void
  suffix: string
  maxLabel: string
  onMax: () => void
}) {
  return (
    <div>
      <div className="flex items-center gap-2 surface-card px-3 py-2">
        <input
          inputMode="decimal"
          placeholder="0.00"
          value={value}
          onChange={e => onChange(e.target.value.replace(/[^\d.]/g, ''))}
          className="flex-1 bg-transparent outline-none text-base font-mono text-ink dark:text-cream placeholder:text-ink/40 dark:placeholder:text-cream/40"
        />
        <span className="text-xs text-ink/55 dark:text-cream/55">{suffix}</span>
      </div>
      <button
        type="button"
        onClick={onMax}
        className="text-[11px] mt-1 text-ink/55 hover:text-teal-deep dark:text-cream/55 dark:hover:text-teal-cyan transition-colors"
      >
        {maxLabel}
      </button>
    </div>
  )
}

// ─── Deals view (placeholder) ─────────────────────────────────────────
function DealsView({ dealsActive }: { dealsActive: number }) {
  return (
    <section>
      <SectionHeading
        title="Deals"
        sub="Active and historical deals served by this prover."
      />
      <div className="surface-card p-8 text-center">
        {dealsActive === 0 ? (
          <>
            <div className="text-sm text-ink/70 dark:text-cream/70">No active deals.</div>
            <div className="text-xs text-ink/45 dark:text-cream/45 mt-1">
              Deals show up here as soon as the marketplace routes one to this prover.
            </div>
          </>
        ) : (
          <div className="text-sm">
            {dealsActive} active deal{dealsActive === 1 ? '' : 's'}. Per-deal detail view is on the roadmap.
          </div>
        )}
      </div>
    </section>
  )
}

// ─── Settings view ────────────────────────────────────────────────────
function SettingsView({
  walletAddress, storage, network, networkPresets, onShowSeed,
}: {
  walletAddress: string
  storage: StorageDirInfo | null
  network: NetworkConfig | null
  networkPresets: NetworkPresetInfo[]
  onShowSeed: () => void
}) {
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  return (
    <>
      <section>
        <SectionHeading
          title="Wallet"
          sub="A local BIP-39 wallet. The seed is stored in your OS keychain."
          right={<button className="pill-button" onClick={onShowSeed}>export seed</button>}
        />
        <div className="surface-card p-4 flex items-center justify-between gap-4">
          <div className="flex flex-col gap-1 min-w-0">
            <span className="text-[11px] uppercase tracking-wider text-ink/55 dark:text-cream/55">Wallet address</span>
            <span className="mono text-sm truncate" title={walletAddress}>{walletAddress || '…'}</span>
          </div>
          {walletAddress && (
            <button className="pill-button shrink-0" onClick={() => navigator.clipboard?.writeText(walletAddress)}>
              copy
            </button>
          )}
        </div>
      </section>

      <section>
        <SectionHeading
          title="Network"
          sub="Pick which chain the prover talks to. Restart the app for changes to take effect."
        />
        <div className="surface-card p-4 flex items-center justify-between gap-4 flex-wrap">
          <div className="flex flex-col gap-1 min-w-0">
            <span className="font-display text-sm" title={network?.rpcUrl}>
              {network?.label ?? '…'}
              <span className="ml-2 text-ink/40 dark:text-cream/40 font-mono">chain {network?.chainId ?? '?'}</span>
            </span>
            {network && !network.isConfigured && (
              <span className="text-[11px] text-amber-700">
                No deployed contracts on this chain yet — the prover will start, but on-chain events won't flow until contracts are configured.
              </span>
            )}
          </div>
          <div className="flex items-center gap-2 flex-wrap">
            {networkPresets.map(p => (
              <button
                key={p.key}
                className={
                  'pill-button ' +
                  (network?.key === p.key ? 'border-teal-cyan/70 text-teal-deep dark:text-teal-cyan' : '')
                }
                disabled={busy || network?.key === p.key}
                onClick={() => {
                  setBusy(true); setError(null)
                  electron.setNetwork(p.key)
                    .catch(err => setError(String(err?.message ?? err)))
                    .finally(() => setBusy(false))
                }}
                title={`${p.label} — ${p.rpcUrl}`}
              >
                {p.label.replace(/ \(.+\)$/, '')}
              </button>
            ))}
          </div>
        </div>
      </section>

      <section>
        <SectionHeading
          title="Storage"
          sub="Pieces are written here. Pick a folder on a fast internal drive, or an external drive you'll keep plugged in."
          right={
            storage?.isCustom ? (
              <button
                className="pill-button"
                disabled={busy}
                onClick={() => {
                  setBusy(true); setError(null)
                  electron.resetStorageDir()
                    .catch(err => setError(String(err?.message ?? err)))
                    .finally(() => setBusy(false))
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
            <span className="mono text-sm truncate" title={storage?.current ?? ''}>{storage?.current ?? '…'}</span>
          </div>
          <button
            className="pill-button flex-shrink-0"
            disabled={busy}
            onClick={() => {
              setBusy(true); setError(null)
              electron.selectStorageDir()
                .catch(err => setError(String(err?.message ?? err)))
                .finally(() => setBusy(false))
            }}
          >
            change…
          </button>
        </div>
      </section>

      {error && (
        <section>
          <div className="surface-card p-4 text-sm text-red-700 dark:text-red-300 break-all">{error}</div>
        </section>
      )}
    </>
  )
}

// ─── Banners + reusable bits ──────────────────────────────────────────
function SectionHeading({ title, sub, right }: { title: string; sub?: string; right?: React.ReactNode }) {
  return (
    <div className="flex items-end justify-between mb-3 gap-3">
      <div className="flex items-stretch gap-2.5">
        <span aria-hidden className="w-[3px] rounded-full bg-gradient-to-b from-teal-cyan to-teal-deep" />
        <div>
          <h2 className="text-sm font-semibold uppercase tracking-wider text-ink/70 dark:text-cream/70">{title}</h2>
          {sub && <p className="text-xs text-ink/45 dark:text-cream/45 mt-0.5 max-w-[60ch]">{sub}</p>}
        </div>
      </div>
      {right}
    </div>
  )
}

function ActivityRow({ activity }: { activity: Activity }) {
  const toneClass =
    activity.type === 'error' ? 'text-red-700 border-l-red-300'
    : activity.type === 'started' ? 'text-emerald-700 border-l-emerald-300'
    : 'text-ink border-l-teal-cyan/40 dark:text-cream dark:border-l-teal-cyan/40'
  return (
    <div className={`px-4 py-3 border-l-2 ${toneClass} flex items-start gap-4`}>
      <div className="flex-1 min-w-0">
        <div className="text-sm">{activity.message}</div>
        <div className="text-xs text-ink/50 dark:text-cream/50 mt-0.5 font-mono">{activity.source}</div>
      </div>
      <div className="text-xs text-ink/50 dark:text-cream/50 font-mono whitespace-nowrap" title={String(activity.timestamp)}>
        {relativeTime(activity.timestamp instanceof Date ? activity.timestamp.toISOString() : activity.timestamp)}
      </div>
    </div>
  )
}

function DisconnectedBanner() {
  return (
    <div className="bg-red-100 text-red-900 border-b border-red-200 px-4 py-2 text-center text-xs">
      Electron bridge unavailable · renderer running in standalone mode. Stats + wallet functions are stubbed.
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

function DaemonFailingBanner({ status, onSaveLogs }: { status: DaemonStatus; onSaveLogs: () => void }) {
  const exitCodeNote = status.lastExitCode === null ? '(no exit code)' : `(exit code ${status.lastExitCode})`
  return (
    <div className="px-4 py-3 flex items-start justify-center gap-4 text-sm border-b border-red-300/70 bg-red-100/80 text-red-900">
      <div className="max-w-2xl">
        <div className="font-semibold">Prover daemon is failing to start {exitCodeNote}.</div>
        <div className="text-xs mt-0.5 break-words">{status.lastError || 'No diagnostic message captured.'}</div>
        <div className="text-xs text-red-900/60 mt-0.5">
          Retried {status.consecutiveFailures} time{status.consecutiveFailures === 1 ? '' : 's'}; supervisor will keep retrying with backoff.
        </div>
      </div>
      <button className="pill-button-primary shrink-0" onClick={onSaveLogs}>save logs</button>
    </div>
  )
}

// ─── Modals ───────────────────────────────────────────────────────────
function FirstRunModal({
  address, presets, activeNetworkKey, onPickNetwork, onShowSeed, onDone,
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
          <h2 className="font-display text-lg font-semibold tracking-tight">Welcome to Prova</h2>
          <span className="text-xs font-mono text-ink/40 dark:text-cream/40">step {step} of 3</span>
        </div>
        {step === 1 && (
          <div className="space-y-4">
            <p className="text-sm text-ink/70 dark:text-cream/70">
              We just created a fresh wallet for this prover. The mnemonic is stored in your OS keychain.{' '}
              <span className="font-semibold">Back it up before storing anything important.</span>
            </p>
            <div className="surface-card p-3 font-mono text-xs break-all">{address}</div>
            <div className="flex justify-end gap-2">
              <button className="pill-button" onClick={() => { setSeedShown(true); onShowSeed() }}>show seed phrase</button>
              <button
                className="pill-button-primary"
                disabled={!seedShown}
                title={seedShown ? '' : 'Reveal the seed first so you can back it up'}
                onClick={() => setStep(2)}
              >next</button>
            </div>
          </div>
        )}
        {step === 2 && (
          <div className="space-y-4">
            <p className="text-sm text-ink/70 dark:text-cream/70">
              Pick the chain this prover will run on. You can change this later in Settings → Network.
            </p>
            <div className="flex flex-col gap-2">
              {presets.map(p => (
                <button
                  key={p.key}
                  className={
                    'text-left surface-card p-3 transition-colors ' +
                    (pickedNetwork === p.key ? 'border-teal-cyan/70 ring-1 ring-teal-cyan/40' : '')
                  }
                  onClick={() => setPickedNetwork(p.key)}
                >
                  <div className="flex items-baseline justify-between gap-3">
                    <span className="font-display text-sm font-semibold">{p.label}</span>
                    <span className="font-mono text-[11px] text-ink/40 dark:text-cream/40">chain {p.chainId}</span>
                  </div>
                  <div className="font-mono text-[11px] text-ink/50 dark:text-cream/50 mt-1 truncate">{p.rpcUrl}</div>
                  {!p.isConfigured && <div className="text-[11px] text-amber-700 mt-1">Contracts not deployed yet on this chain.</div>}
                </button>
              ))}
            </div>
            <div className="flex justify-between gap-2">
              <button className="pill-button" onClick={() => setStep(1)}>back</button>
              <button
                className="pill-button-primary"
                onClick={() => { void onPickNetwork(pickedNetwork); setStep(3) }}
              >
                use {presets.find(p => p.key === pickedNetwork)?.label.replace(/ \(.+\)$/, '')}
              </button>
            </div>
          </div>
        )}
        {step === 3 && (
          <div className="space-y-4">
            <p className="text-sm text-ink/70 dark:text-cream/70">
              You're set up. Storage location, network, and updates can be changed any time from Settings.
            </p>
            <div className="surface-card p-3 text-xs space-y-1">
              <div><span className="text-ink/50 dark:text-cream/50">Wallet:</span> <span className="font-mono">{address}</span></div>
              <div><span className="text-ink/50 dark:text-cream/50">Network:</span> {presets.find(p => p.key === pickedNetwork)?.label}</div>
            </div>
            <div className="flex justify-end">
              <button className="pill-button-primary" onClick={onDone}>let's go</button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

function SeedExportModal({ onClose }: { onClose: () => void }) {
  const [confirmed, setConfirmed] = useState(false)
  const [phrase, setPhrase] = useState<string | null>(null)
  const [revealed, setRevealed] = useState(false)
  const [copied, setCopied] = useState(false)
  const [error, setError] = useState<string | null>(null)
  useEffect(() => {
    if (!confirmed) return
    let cancelled = false
    electron.exportSeedPhrase()
      .then(p => { if (!cancelled) setPhrase(p) })
      .catch(err => { if (!cancelled) setError(String(err?.message ?? err)) })
    return () => { cancelled = true }
  }, [confirmed])
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center px-4 py-8 bg-black/40 backdrop-blur-sm">
      <div className="surface-card max-w-xl w-full p-6 space-y-5">
        <div className="flex items-center justify-between">
          <h2 className="font-display text-lg font-semibold tracking-tight">Seed phrase</h2>
          <button className="pill-button" onClick={onClose}>close</button>
        </div>
        {!confirmed && (
          <div className="space-y-4">
            <p className="text-sm text-ink/70 dark:text-cream/70">
              Anyone who sees this 12-word phrase can take any funds in this wallet. Make sure no screen recording, screenshare, or other process is observing this window before you continue.
            </p>
            <div className="flex justify-end gap-2">
              <button className="pill-button" onClick={onClose}>cancel</button>
              <button className="pill-button border-amber-400/70 text-amber-700" onClick={() => setConfirmed(true)}>
                I understand, show it
              </button>
            </div>
          </div>
        )}
        {confirmed && error && <div className="text-sm text-red-700">Could not export seed: {error}</div>}
        {confirmed && !error && phrase === null && <div className="text-sm text-ink/60 dark:text-cream/60">decrypting…</div>}
        {confirmed && phrase !== null && (
          <div className="space-y-4">
            <div className="surface-card p-4 font-mono text-sm leading-relaxed select-all">
              {revealed ? phrase : phrase.split(' ').map(w => '•'.repeat(Math.max(4, w.length))).join(' ')}
            </div>
            <div className="flex items-center gap-2 justify-end">
              <button className="pill-button" onClick={() => setRevealed(r => !r)}>
                {revealed ? 'hide' : 'reveal'}
              </button>
              <button
                className="pill-button"
                onClick={() => {
                  void navigator.clipboard?.writeText(phrase)
                  setCopied(true); setTimeout(() => setCopied(false), 1500)
                }}
              >
                {copied ? 'copied' : 'copy'}
              </button>
              <button className="pill-button-primary" onClick={onClose}>done</button>
            </div>
            <p className="text-xs text-ink/50 dark:text-cream/50">
              Tip: write it on paper. Clipboard contents are accessible to every app on your machine — don't leave it sitting there.
            </p>
          </div>
        )}
      </div>
    </div>
  )
}

// ─── Helpers ──────────────────────────────────────────────────────────

/// Format a base-unit BigInt amount (encoded as decimal string) using the
/// token's decimals. Returns up to `maxFractionalDigits` after the dot,
/// trimming trailing zeros for compact display.
function formatTokenAmount(wei: string, decimals: number, maxFractionalDigits = 4): string {
  if (!wei) return '0'
  const negative = wei.startsWith('-')
  const raw = negative ? wei.slice(1) : wei
  const padded = raw.padStart(decimals + 1, '0')
  const intPart = padded.slice(0, padded.length - decimals)
  const fracPart = padded.slice(padded.length - decimals).slice(0, maxFractionalDigits).replace(/0+$/, '')
  const intFormatted = Number(intPart).toLocaleString()
  return (negative ? '-' : '') + (fracPart ? `${intFormatted}.${fracPart}` : intFormatted)
}

/// Inverse of formatTokenAmount: parse a user-typed decimal amount into a
/// base-unit BigInt. Throws on overflow or invalid input.
function parseTokenAmount(input: string, decimals: number): bigint {
  const s = input.trim()
  if (!s) return 0n
  const m = /^(\d+)(?:\.(\d{0,30}))?$/.exec(s)
  if (!m) throw new Error(`invalid amount: ${input}`)
  const whole = m[1]
  const frac = (m[2] || '').slice(0, decimals).padEnd(decimals, '0')
  return BigInt(whole) * (10n ** BigInt(decimals)) + BigInt(frac || '0')
}

function extractErrorMessage(e: unknown): string {
  if (!e) return 'unknown error'
  // Electron IPC wraps thrown Errors in `Error: <msg>` strings; ethers v6
  // throws CallExceptions with a `.shortMessage` field. Try both.
  const anyE = e as { shortMessage?: string; reason?: string; message?: string }
  return (
    anyE.shortMessage ||
    anyE.reason ||
    anyE.message ||
    String(e)
  )
}
