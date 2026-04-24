// Thin client for the Prova prover dashboard API.
// Backend lives at /api/... on the same origin in prod (embedded SPA),
// or proxied via vite dev at :5173 → :8081 in dev.

export type ChainSnapshot = {
  chain_id: number
  block_number: number
  last_seen_block: number
  prover_registered: boolean
  prover_active: boolean
  prover_address: string
  staked_wei: string
  unbonding_wei: string
  unbonding_ends_at_unix: number
  committed_bytes: number
  reputation: number
}

export type MetricsSummary = {
  deals_active: number
  deals_total: number
  proofs_submitted_total: number
  proofs_failed_total: number
  retrieval_bytes_total: number
  piece_store_bytes: number
  chain_rpc_errors_total: number
  proof_success_rate_pct: number
  uptime_seconds: number
}

export type Overview = {
  version: string
  started_at: string
  uptime: string
  deals_total?: number
  deal_counts?: Record<string, number>
  chain?: ChainSnapshot
  chain_error?: string
  metrics?: MetricsSummary
}

export type Deal = {
  id: number
  status: string
  status_message?: string
  client: string
  piece_size: number
  commp_hex: string
  source_url?: string
  total_payment_wei?: string
  duration?: string
  data_set_id?: number
  bytes_stored?: number
  accepted_at?: string
  updated_at: string
}

export type DealListResp = {
  deals: Deal[]
  count: number
}

async function fetchJSON<T>(path: string): Promise<T> {
  const r = await fetch(path, { headers: { Accept: 'application/json' } })
  if (!r.ok) throw new Error(`${r.status} ${r.statusText}`)
  return r.json() as Promise<T>
}

export const api = {
  overview: () => fetchJSON<Overview>('/api/overview'),
  deals: (status?: string) =>
    fetchJSON<DealListResp>(
      '/api/deals' + (status ? `?status=${encodeURIComponent(status)}` : ''),
    ),
  deal: (id: number) => fetchJSON<Deal>(`/api/deals/${id}`),
  chain: () => fetchJSON<ChainSnapshot>('/api/chain'),
  metrics: () => fetchJSON<MetricsSummary>('/api/metrics'),
  info: () => fetchJSON<Record<string, unknown>>('/api/info'),
}
