// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package dashboard

import (
	"time"

	dto "github.com/prometheus/client_model/go"
	"github.com/prometheus/client_golang/prometheus"

	"github.com/prova-network/prova/prover/pkg/metrics"
)

// MetricsCollectorAdapter is a MetricsReader that pulls a compact
// summary from a *metrics.Collector. It's kept in pkg/dashboard so
// pkg/metrics doesn't take a dependency on a UI concept.
type MetricsCollectorAdapter struct {
	C         *metrics.Collector
	StartedAt time.Time
}

// Summary implements MetricsReader.
func (a MetricsCollectorAdapter) Summary() MetricsSummary {
	if a.C == nil {
		return MetricsSummary{}
	}

	active := readGauge(a.C.DealsActiveGauge)
	ingested := readCounter(a.C.DealsIngestedTotal)
	completed := readCounter(a.C.DealsCompletedTotal)
	failed := readCounter(a.C.DealsFailedTotal)
	proofsOK := readCounter(a.C.ProofsSubmittedTotal)
	proofsFail := readCounter(a.C.ProofsSubmitFailures)
	rpcFail := readCounter(a.C.ChainPollFailures)
	bytesServed := readCounter(a.C.HTTPBytesServed)
	bytesStored := readCounter(a.C.BytesStoredTotal)

	var rate float64
	if total := proofsOK + proofsFail; total > 0 {
		rate = float64(proofsOK) / float64(total) * 100.0
	}

	uptime := int64(0)
	if !a.StartedAt.IsZero() {
		uptime = int64(time.Since(a.StartedAt).Seconds())
	}

	return MetricsSummary{
		DealsActive:          uint64(active),
		DealsTotal:           ingested + completed + failed, // best-effort aggregate
		ProofsSubmittedTotal: proofsOK,
		ProofsFailedTotal:    proofsFail,
		RetrievalBytesTotal:  bytesServed,
		PieceStoreBytes:      bytesStored,
		ChainRPCErrorsTotal:  rpcFail,
		ProofSuccessRatePct:  rate,
		UptimeSeconds:        uptime,
	}
}

// readCounter extracts the current value from a Prometheus Counter.
// Returns 0 on any error (dashboard is best-effort).
func readCounter(c prometheus.Counter) uint64 {
	if c == nil {
		return 0
	}
	m := &dto.Metric{}
	if err := c.Write(m); err != nil || m.Counter == nil {
		return 0
	}
	if m.Counter.Value == nil {
		return 0
	}
	v := *m.Counter.Value
	if v < 0 {
		return 0
	}
	return uint64(v)
}

// readGauge extracts the current value from a Prometheus Gauge.
func readGauge(g prometheus.Gauge) float64 {
	if g == nil {
		return 0
	}
	m := &dto.Metric{}
	if err := g.Write(m); err != nil || m.Gauge == nil {
		return 0
	}
	if m.Gauge.Value == nil {
		return 0
	}
	return *m.Gauge.Value
}
