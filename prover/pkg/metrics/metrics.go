// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

// Package metrics exposes Prometheus metrics for the prover.
//
// Metrics are collected in three buckets:
//
//   - Deal lifecycle: counters per terminal state, gauge for live count
//   - HTTP retrieval: counter of requests, histogram of durations/bytes
//   - Chain interaction: counter of polls, counter of tx submissions
//
// All metrics are registered against a single Collector so the daemon can
// pass a single handle around rather than a registry pointer everywhere.
package metrics

import (
	"net/http"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

// Collector bundles all prover metrics. Constructed once at startup; passed
// into engine/daemon/httpserver via options.
type Collector struct {
	reg *prometheus.Registry

	// Deal lifecycle
	DealsIngestedTotal  prometheus.Counter
	DealsCompletedTotal prometheus.Counter
	DealsFailedTotal    prometheus.Counter
	DealsActiveGauge    prometheus.Gauge
	DealStepDuration    *prometheus.HistogramVec

	// Pieces
	BytesStoredTotal prometheus.Counter
	PiecesStored     prometheus.Gauge

	// HTTP
	HTTPRequestsTotal  *prometheus.CounterVec
	HTTPDuration       *prometheus.HistogramVec
	HTTPBytesServed    prometheus.Counter

	// Chain
	ChainPollsTotal       prometheus.Counter
	ChainPollFailures     prometheus.Counter
	ProofsSubmittedTotal  prometheus.Counter
	ProofsSubmitFailures  prometheus.Counter
	ChainHeadBlockGauge   prometheus.Gauge
}

// New constructs a Collector with all metrics registered against a fresh
// registry. Returns both the collector and the registry so the caller can
// expose it via promhttp.HandlerFor.
func New() *Collector {
	reg := prometheus.NewRegistry()

	c := &Collector{
		reg: reg,

		DealsIngestedTotal: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "prova_deals_ingested_total",
			Help: "Total deals ingested from on-chain events targeting this prover.",
		}),
		DealsCompletedTotal: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "prova_deals_completed_total",
			Help: "Total deals that reached the Completed terminal state.",
		}),
		DealsFailedTotal: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "prova_deals_failed_total",
			Help: "Total deals that reached the Failed terminal state (local error before acceptance).",
		}),
		DealsActiveGauge: prometheus.NewGauge(prometheus.GaugeOpts{
			Name: "prova_deals_active",
			Help: "Number of deals currently in Active state.",
		}),
		DealStepDuration: prometheus.NewHistogramVec(prometheus.HistogramOpts{
			Name:    "prova_deal_step_duration_seconds",
			Help:    "Duration of each engine-driven deal step by transition name.",
			Buckets: prometheus.DefBuckets,
		}, []string{"step"}),

		BytesStoredTotal: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "prova_bytes_stored_total",
			Help: "Total raw bytes written to the local piece store.",
		}),
		PiecesStored: prometheus.NewGauge(prometheus.GaugeOpts{
			Name: "prova_pieces_stored",
			Help: "Current number of pieces in the local store.",
		}),

		HTTPRequestsTotal: prometheus.NewCounterVec(prometheus.CounterOpts{
			Name: "prova_http_requests_total",
			Help: "HTTP requests handled, partitioned by method, path kind, and status.",
		}, []string{"method", "path", "status"}),
		HTTPDuration: prometheus.NewHistogramVec(prometheus.HistogramOpts{
			Name:    "prova_http_duration_seconds",
			Help:    "HTTP request latency by path kind.",
			Buckets: []float64{0.001, 0.01, 0.1, 0.5, 1, 5, 30, 120},
		}, []string{"path"}),
		HTTPBytesServed: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "prova_http_bytes_served_total",
			Help: "Total bytes streamed out via HTTP retrieval.",
		}),

		ChainPollsTotal: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "prova_chain_polls_total",
			Help: "Total chain event polls performed.",
		}),
		ChainPollFailures: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "prova_chain_poll_failures_total",
			Help: "Total chain polls that returned an error.",
		}),
		ProofsSubmittedTotal: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "prova_proofs_submitted_total",
			Help: "Total PDP proofs successfully submitted on-chain.",
		}),
		ProofsSubmitFailures: prometheus.NewCounter(prometheus.CounterOpts{
			Name: "prova_proofs_submit_failures_total",
			Help: "Total PDP proof submissions that returned an error.",
		}),
		ChainHeadBlockGauge: prometheus.NewGauge(prometheus.GaugeOpts{
			Name: "prova_chain_head_block",
			Help: "Most recently observed chain head block number.",
		}),
	}

	reg.MustRegister(
		c.DealsIngestedTotal,
		c.DealsCompletedTotal,
		c.DealsFailedTotal,
		c.DealsActiveGauge,
		c.DealStepDuration,
		c.BytesStoredTotal,
		c.PiecesStored,
		c.HTTPRequestsTotal,
		c.HTTPDuration,
		c.HTTPBytesServed,
		c.ChainPollsTotal,
		c.ChainPollFailures,
		c.ProofsSubmittedTotal,
		c.ProofsSubmitFailures,
		c.ChainHeadBlockGauge,
	)

	// Go runtime + process metrics (memory, goroutines, GC pauses, etc.)
	reg.MustRegister(
		prometheus.NewGoCollector(),
		prometheus.NewProcessCollector(prometheus.ProcessCollectorOpts{}),
	)

	return c
}

// Handler returns an HTTP handler that serves the Prometheus metrics.
func (c *Collector) Handler() http.Handler {
	return promhttp.HandlerFor(c.reg, promhttp.HandlerOpts{
		Timeout:             30 * time.Second,
		EnableOpenMetrics:   true,
		ErrorHandling:       promhttp.ContinueOnError,
	})
}

// Registry returns the underlying Prometheus registry, for advanced use
// (additional custom collectors, tests, etc.).
func (c *Collector) Registry() *prometheus.Registry { return c.reg }
