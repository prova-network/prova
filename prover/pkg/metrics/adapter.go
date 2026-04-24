// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package metrics

// DealSink is the subset of Collector used by pkg/deal. Satisfies the
// deal.MetricsSink interface without importing deal (avoids cycle).
type DealSink struct {
	c *Collector
}

// NewDealSink wraps a Collector for use as a deal.MetricsSink.
func NewDealSink(c *Collector) *DealSink { return &DealSink{c: c} }

// DealIngested increments the ingested-deal counter.
func (d *DealSink) DealIngested() {
	if d == nil || d.c == nil {
		return
	}
	d.c.DealsIngestedTotal.Inc()
}

// DealFailed increments the failed-deal counter.
func (d *DealSink) DealFailed() {
	if d == nil || d.c == nil {
		return
	}
	d.c.DealsFailedTotal.Inc()
}

// DealCompleted increments the completed-deal counter.
func (d *DealSink) DealCompleted() {
	if d == nil || d.c == nil {
		return
	}
	d.c.DealsCompletedTotal.Inc()
}

// BytesStored adds to the total bytes counter and nudges the pieces gauge.
func (d *DealSink) BytesStored(n uint64) {
	if d == nil || d.c == nil {
		return
	}
	d.c.BytesStoredTotal.Add(float64(n))
	d.c.PiecesStored.Inc()
}

// HTTPSink adapts Collector to the httpserver.HTTPMetrics interface.
type HTTPSink struct {
	c *Collector
}

// NewHTTPSink wraps a Collector for HTTP middleware consumption.
func NewHTTPSink(c *Collector) *HTTPSink { return &HTTPSink{c: c} }

// Request increments the HTTP request counter for (method, path, status).
func (h *HTTPSink) Request(method, path, status string) {
	if h == nil || h.c == nil {
		return
	}
	h.c.HTTPRequestsTotal.WithLabelValues(method, path, status).Inc()
}

// Duration observes a request duration for the given path label.
func (h *HTTPSink) Duration(path string, seconds float64) {
	if h == nil || h.c == nil {
		return
	}
	h.c.HTTPDuration.WithLabelValues(path).Observe(seconds)
}

// BytesServed adds to the bytes-served counter.
func (h *HTTPSink) BytesServed(n int) {
	if h == nil || h.c == nil || n <= 0 {
		return
	}
	h.c.HTTPBytesServed.Add(float64(n))
}
