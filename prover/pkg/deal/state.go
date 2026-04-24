// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

// Package deal implements the deal lifecycle state machine for a Prova prover.
//
// The state machine is intentionally simple and pull-based:
//
//   Proposed ─ download ─▶ Downloading ─ hash ─▶ Verifying ─ tx ─▶ Accepting ─ event ─▶ Active
//        │                     │                    │              │                    │
//        └── (fatal error) ────┴────────────────────┴──────────────┴────────────────────┴─▶ Failed
//        │
//        └── (cancelled on-chain) ─────────────────────────────────────────────────────────▶ Cancelled
//                                                                                          │
//                                    Active ─ (time elapsed, client called complete) ──────▶ Completed
//                                    Active ─ (missed proof, anyone called fault) ─────────▶ Slashed
//
// The prover owns Proposed → Active transitions; the chain owns Active →
// Completed/Slashed. Cancelled can come from either (client cancellation or
// our own bail-out on a fatal error before acceptance).
package deal

import (
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common"
)

// DealID is the on-chain numeric deal id.
type DealID uint64

// Status is a local view of the deal's state. It overlaps with but is not
// identical to the on-chain StorageMarketplace.DealStatus enum; we add a
// few "in progress on our side" intermediate states (Downloading, Verifying,
// Accepting) that are invisible on-chain.
type Status string

const (
	StatusProposed    Status = "proposed"
	StatusDownloading Status = "downloading"
	StatusVerifying   Status = "verifying"
	StatusAccepting   Status = "accepting"
	StatusActive      Status = "active"
	StatusCompleted   Status = "completed"
	StatusCancelled   Status = "cancelled"
	StatusSlashed     Status = "slashed"
	StatusFailed      Status = "failed" // local-only: we bailed before acceptance
)

// Deal is the local record of a deal's lifecycle.
type Deal struct {
	ID           DealID
	Client       common.Address
	Prover       common.Address
	CommPHash    [32]byte        // 32-byte commitment, not a full CID
	PieceSize    uint64          // padded piece size in bytes
	Duration     time.Duration   // from DealProposed event
	TotalPayment string          // stringified wei, avoids precision surprises
	SourceURL    string          // where to fetch the piece from (prover-supplied out-of-band or later: in DealProposed extension)

	Status    Status
	StatusMsg string    // human-readable note about last transition or error
	UpdatedAt time.Time // local timestamp

	// Set after successful CommP computation on downloaded content.
	ComputedCommP [32]byte
	BytesStored   uint64

	// Set after the Acceptance tx is confirmed.
	DataSetID  uint64
	AcceptedAt time.Time
}

// String renders the deal for logs.
func (d *Deal) String() string {
	return fmt.Sprintf("deal#%d [%s] client=%s commP=0x%x size=%d",
		d.ID, d.Status, d.Client.Hex()[:10], d.CommPHash[:4], d.PieceSize)
}

// IsTerminal reports whether the deal has reached a final state.
func (d *Deal) IsTerminal() bool {
	switch d.Status {
	case StatusCompleted, StatusCancelled, StatusSlashed, StatusFailed:
		return true
	}
	return false
}

// IsOurs reports whether this deal is targeted at the given prover address.
// Prover entries that don't match our address are ignored by the engine.
func (d *Deal) IsOurs(ourAddr common.Address) bool {
	return d.Prover == ourAddr
}

// NextStatus returns the step that should happen next for a deal in the
// given status. Returns the same status unchanged for terminal or
// chain-driven states (Active, Completed, etc.).
func NextStatus(s Status) Status {
	switch s {
	case StatusProposed:
		return StatusDownloading
	case StatusDownloading:
		return StatusVerifying
	case StatusVerifying:
		return StatusAccepting
	case StatusAccepting:
		return StatusActive
	default:
		return s
	}
}
