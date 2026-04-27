// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.
//
// IPC bridge for the Prova Desktop renderer. Unlike the standalone
// prover/webui/ SPA which polls a local HTTP JSON API, this one talks
// to the Electron main process via `window.electron` (exposed by the
// preload script).

// ─── Types ────────────────────────────────────────────────────────────

export type Activity = {
  id: string
  type: 'info' | 'error' | 'started'
  source: string
  message: string
  timestamp: Date | string
}

export type UpdaterStatus =
  | 'idle'
  | 'checking'
  | 'update-available'
  | 'downloading'
  | 'ready'
  | 'error'

export type WalletInfo = {
  address: string
}

export type StorageDirInfo = {
  /// Currently configured directory (custom user choice or default).
  current: string
  /// The default directory for this platform.
  default: string
  /// True if the user has chosen something other than the default.
  isCustom: boolean
}

export type NetworkKey = 'anvil' | 'base-sepolia' | 'base-mainnet'

export type NetworkPresetInfo = {
  key: NetworkKey
  label: string
  rpcUrl: string
  chainId: number
  /// True if every contract address in the preset is set; false means
  /// the desktop is shipping with placeholders and the user (or a future
  /// app update) needs to drop in real addresses.
  isConfigured: boolean
}

export type NetworkConfig = NetworkPresetInfo & {
  contracts: {
    provaToken: string
    proofVerifier: string
    proverRegistry: string
    proverStaking: string
    contentRegistry: string
    storageMarketplace: string
  }
}

export type DaemonStatus = {
  state: 'idle' | 'starting' | 'running' | 'failing'
  lastError: string
  lastExitCode: number | null
  consecutiveFailures: number
}

export type StakeSnapshot = {
  address: string
  /// All amounts as decimal strings of base units (wei). Render-side
  /// formats them via formatUnits to avoid BigInt churn over IPC.
  ethWei: string
  provaWei: string
  provaDecimals: number
  stakedWei: string
  unbondingWei: string
  /// Unix-seconds timestamp when the unbonding queue becomes
  /// withdrawable. 0 if no unbonding in progress.
  unbondingEndsAt: number
  committedBytes: string
  minStakePerTiBWei: string
  unbondingPeriodSeconds: number
  isRegistered: boolean
  registeredEndpoint: string
}

export type ProverStats = {
  /// Bytes currently stored on disk in the configured storage dir.
  bytesStored: number
  /// Number of distinct piece files in the local store.
  piecesStored: number
  /// Active (in-progress) deals as tracked by the supervisor.
  dealsActive: number
  /// Total proofs successfully submitted on-chain since boot.
  proofsSubmitted: number
  /// USDC earned (lifetime). Null = not yet wired (needs marketplace read).
  earnedUsdc: number | null
  /// PROVA staked. Null = not yet wired (needs ProverStaking read).
  stakedProva: number | null
  /// Seconds since the daemon last emitted 'provad start'. Null when offline.
  daemonUptimeSeconds: number | null
}

// ─── Preload surface ──────────────────────────────────────────────────

// This mirrors the `contextBridge.exposeInMainWorld('electron', ...)`
// shape in desktop/main/preload.js. Keep them in sync.
declare global {
  interface Window {
    electron: {
      buildVersion: string

      // Wallet
      getWalletAddress: () => Promise<string>
      signMessage: (msg: string) => Promise<string>
      exportSeedPhrase: () => Promise<string>
      importSeedPhrase: (phrase: string) => Promise<string>

      // Prover stats
      getTotalProofsSubmitted: () => Promise<number>
      getTotalDealsActive: () => Promise<number>
      getActivities: () => Promise<Activity[]>

      // Lifecycle
      restartToUpdate: () => Promise<void>
      openReleaseNotes: () => Promise<void>
      getUpdaterStatus: () => Promise<UpdaterStatus>
      checkForUpdates: () => Promise<void>
      toggleOpenAtLogin: () => Promise<boolean>
      isOpenAtLogin: () => Promise<boolean>

      // Logs & external URLs
      saveLogsAs: () => Promise<void>
      openExternalURL: (url: string) => Promise<void>

      // Storage location
      getStorageDir: () => Promise<StorageDirInfo>
      /** Returns the chosen path, or null if the user canceled the dialog. */
      selectStorageDir: () => Promise<string | null>
      /** Resets to the platform default; returns the now-effective path. */
      resetStorageDir: () => Promise<string>

      // First-run onboarding
      getOnboardingState: () => Promise<{ completed: boolean; firstRunWalletAddress: string }>
      completeOnboarding: () => Promise<boolean>

      // Daemon status
      getDaemonStatus: () => Promise<DaemonStatus>

      // Prover stats (storage, earnings, staked, deals, proofs)
      getProverStats: () => Promise<ProverStats>

      // On-chain reads + actions
      getStakeSnapshot: () => Promise<StakeSnapshot | null>
      stake: (amountWei: string) => Promise<{ txHash: string; approveTxHash?: string }>
      requestUnstake: (amountWei: string) => Promise<{ txHash: string }>
      withdrawUnbonded: () => Promise<{ txHash: string }>
      registerProver: (opts?: { endpoint?: string; maxBytes?: string }) => Promise<{ txHash: string }>

      // Network preset selection
      listNetworks: () => Promise<NetworkPresetInfo[]>
      getNetwork: () => Promise<NetworkConfig>
      setNetwork: (key: NetworkKey) => Promise<NetworkConfig>

      // Subscriptions (event-driven updates from main -> renderer)
      onActivityLogged: (cb: (a: Activity) => void) => () => void
      onProofStatsUpdated: (cb: (total: number) => void) => () => void
      onDealsActiveUpdated: (cb: (total: number) => void) => () => void
      onWalletAddressUpdated: (cb: (addr: string) => void) => () => void
      onStorageDirChanged: (cb: (dir: string) => void) => () => void
      onNetworkChanged: (cb: (cfg: NetworkConfig) => void) => () => void
      onDaemonStatusChanged: (cb: (status: DaemonStatus) => void) => () => void
      onUpdaterStatusChanged: (cb: (status: UpdaterStatus) => void) => () => void
    }
  }
}

// ─── Bridge-available guard ─────────────────────────────────────────

// The preload script exposes `window.electron` before the renderer runs,
// so in a normal Electron boot this is always defined. But there are two
// situations where it's not:
//   1. Dev-loop: someone serves the built dist/ statically (e.g. via vite
//      preview or `python -m http.server`) to QA the UI chrome.
//   2. A future regression where the preload script fails to load.
//
// In either case we want the app to render a minimal "disconnected" shell
// instead of throwing an uncaught TypeError and showing a blank page.
export const bridgeAvailable = (): boolean =>
  typeof window !== 'undefined' && !!(window as unknown as { electron?: unknown }).electron

// noop/default-returning stubs for the case where the bridge is missing.
// These match the real signatures so TypeScript stays happy.
const stub = {
  async getWalletAddress() { return '' },
  async signMessage(_m: string) { return '' },
  async exportSeedPhrase() { throw new Error('bridge not available') },
  async importSeedPhrase(_p: string) { throw new Error('bridge not available') },
  async getTotalProofsSubmitted() { return 0 },
  async getTotalDealsActive() { return 0 },
  async getActivities(): Promise<Activity[]> { return [] },
  async restartToUpdate() {},
  async openReleaseNotes() {},
  async getUpdaterStatus() { return 'idle' as UpdaterStatus },
  async checkForUpdates() {},
  async toggleOpenAtLogin() { return false },
  async isOpenAtLogin() { return false },
  async saveLogsAs() {},
  async openExternalURL(_u: string) {},
  async getStorageDir(): Promise<StorageDirInfo> {
    return { current: '', default: '', isCustom: false }
  },
  async selectStorageDir(): Promise<string | null> { return null },
  async resetStorageDir(): Promise<string> { return '' },
  async getOnboardingState() { return { completed: true, firstRunWalletAddress: '' } },
  async completeOnboarding() { return true },
  async getDaemonStatus(): Promise<DaemonStatus> {
    return { state: 'idle', lastError: '', lastExitCode: null, consecutiveFailures: 0 }
  },
  async getProverStats(): Promise<ProverStats> {
    return {
      bytesStored: 0,
      piecesStored: 0,
      dealsActive: 0,
      proofsSubmitted: 0,
      earnedUsdc: null,
      stakedProva: null,
      daemonUptimeSeconds: null,
    }
  },
  async getStakeSnapshot(): Promise<StakeSnapshot | null> { return null },
  async stake(_amountWei: string) { return { txHash: '' } },
  async requestUnstake(_amountWei: string) { return { txHash: '' } },
  async withdrawUnbonded() { return { txHash: '' } },
  async registerProver(_opts?: { endpoint?: string; maxBytes?: string }) { return { txHash: '' } },
  async listNetworks(): Promise<NetworkPresetInfo[]> { return [] },
  async getNetwork(): Promise<NetworkConfig> {
    return {
      key: 'anvil',
      label: 'Local anvil (dev)',
      rpcUrl: '',
      chainId: 0,
      isConfigured: false,
      contracts: {
        provaToken: '',
        proofVerifier: '',
        proverRegistry: '',
        proverStaking: '',
        contentRegistry: '',
        storageMarketplace: ''
      }
    }
  },
  async setNetwork(_key: NetworkKey): Promise<NetworkConfig> { return this.getNetwork() },
  onActivityLogged(_cb: (a: Activity) => void) { return () => {} },
  onProofStatsUpdated(_cb: (n: number) => void) { return () => {} },
  onDealsActiveUpdated(_cb: (n: number) => void) { return () => {} },
  onWalletAddressUpdated(_cb: (addr: string) => void) { return () => {} },
  onStorageDirChanged(_cb: (dir: string) => void) { return () => {} },
  onNetworkChanged(_cb: (cfg: NetworkConfig) => void) { return () => {} },
  onDaemonStatusChanged(_cb: (s: DaemonStatus) => void) { return () => {} },
  onUpdaterStatusChanged(_cb: (s: UpdaterStatus) => void) { return () => {} }
}

// ─── Convenience wrapper ──────────────────────────────────────────────

// If you squint hard enough this is the same shape as the standalone
// dashboard's `api` export, so components that consumed it previously
// only need trivial changes.
export const electron = {
  getWalletAddress: () => bridgeAvailable() ? window.electron.getWalletAddress() : stub.getWalletAddress(),
  signMessage: (msg: string) => bridgeAvailable() ? window.electron.signMessage(msg) : stub.signMessage(msg),
  exportSeedPhrase: () => bridgeAvailable() ? window.electron.exportSeedPhrase() : stub.exportSeedPhrase(),
  importSeedPhrase: (p: string) => bridgeAvailable() ? window.electron.importSeedPhrase(p) : stub.importSeedPhrase(p),

  getTotalProofsSubmitted: () => bridgeAvailable() ? window.electron.getTotalProofsSubmitted() : stub.getTotalProofsSubmitted(),
  getTotalDealsActive: () => bridgeAvailable() ? window.electron.getTotalDealsActive() : stub.getTotalDealsActive(),
  getActivities: () => bridgeAvailable() ? window.electron.getActivities() : stub.getActivities(),

  restartToUpdate: () => bridgeAvailable() ? window.electron.restartToUpdate() : stub.restartToUpdate(),
  openReleaseNotes: () => bridgeAvailable() ? window.electron.openReleaseNotes() : stub.openReleaseNotes(),
  getUpdaterStatus: () => bridgeAvailable() ? window.electron.getUpdaterStatus() : stub.getUpdaterStatus(),
  checkForUpdates: () => bridgeAvailable() ? window.electron.checkForUpdates() : stub.checkForUpdates(),
  toggleOpenAtLogin: () => bridgeAvailable() ? window.electron.toggleOpenAtLogin() : stub.toggleOpenAtLogin(),
  isOpenAtLogin: () => bridgeAvailable() ? window.electron.isOpenAtLogin() : stub.isOpenAtLogin(),

  saveLogsAs: () => bridgeAvailable() ? window.electron.saveLogsAs() : stub.saveLogsAs(),
  openExternalURL: (url: string) => bridgeAvailable() ? window.electron.openExternalURL(url) : stub.openExternalURL(url),

  getStorageDir: () => bridgeAvailable() ? window.electron.getStorageDir() : stub.getStorageDir(),
  selectStorageDir: () => bridgeAvailable() ? window.electron.selectStorageDir() : stub.selectStorageDir(),
  resetStorageDir: () => bridgeAvailable() ? window.electron.resetStorageDir() : stub.resetStorageDir(),

  getOnboardingState: () => bridgeAvailable() ? window.electron.getOnboardingState() : stub.getOnboardingState(),
  completeOnboarding: () => bridgeAvailable() ? window.electron.completeOnboarding() : stub.completeOnboarding(),

  getDaemonStatus: () => bridgeAvailable() ? window.electron.getDaemonStatus() : stub.getDaemonStatus(),
  getProverStats: () => bridgeAvailable() ? window.electron.getProverStats() : stub.getProverStats(),

  getStakeSnapshot: () => bridgeAvailable() ? window.electron.getStakeSnapshot() : stub.getStakeSnapshot(),
  stake: (amountWei: string) => bridgeAvailable() ? window.electron.stake(amountWei) : stub.stake(amountWei),
  requestUnstake: (amountWei: string) => bridgeAvailable() ? window.electron.requestUnstake(amountWei) : stub.requestUnstake(amountWei),
  withdrawUnbonded: () => bridgeAvailable() ? window.electron.withdrawUnbonded() : stub.withdrawUnbonded(),
  registerProver: (opts?: { endpoint?: string; maxBytes?: string }) =>
    bridgeAvailable() ? window.electron.registerProver(opts) : stub.registerProver(opts),

  listNetworks: () => bridgeAvailable() ? window.electron.listNetworks() : stub.listNetworks(),
  getNetwork: () => bridgeAvailable() ? window.electron.getNetwork() : stub.getNetwork(),
  setNetwork: (key: NetworkKey) => bridgeAvailable() ? window.electron.setNetwork(key) : stub.setNetwork(key),

  onActivityLogged: (cb: (a: Activity) => void) =>
    bridgeAvailable() ? window.electron.onActivityLogged(cb) : stub.onActivityLogged(cb),
  onProofStatsUpdated: (cb: (total: number) => void) =>
    bridgeAvailable() ? window.electron.onProofStatsUpdated(cb) : stub.onProofStatsUpdated(cb),
  onDealsActiveUpdated: (cb: (total: number) => void) =>
    bridgeAvailable() ? window.electron.onDealsActiveUpdated(cb) : stub.onDealsActiveUpdated(cb),
  onWalletAddressUpdated: (cb: (addr: string) => void) =>
    bridgeAvailable() ? window.electron.onWalletAddressUpdated(cb) : stub.onWalletAddressUpdated(cb),
  onStorageDirChanged: (cb: (dir: string) => void) =>
    bridgeAvailable() ? window.electron.onStorageDirChanged(cb) : stub.onStorageDirChanged(cb),
  onNetworkChanged: (cb: (cfg: NetworkConfig) => void) =>
    bridgeAvailable() ? window.electron.onNetworkChanged(cb) : stub.onNetworkChanged(cb),
  onDaemonStatusChanged: (cb: (s: DaemonStatus) => void) =>
    bridgeAvailable() ? window.electron.onDaemonStatusChanged(cb) : stub.onDaemonStatusChanged(cb),
  onUpdaterStatusChanged: (cb: (s: UpdaterStatus) => void) =>
    bridgeAvailable() ? window.electron.onUpdaterStatusChanged(cb) : stub.onUpdaterStatusChanged(cb),

  buildVersion: () =>
    bridgeAvailable() ? window.electron.buildVersion : 'dev (no bridge)'
}
