// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package deal

import (
	"fmt"
	"sync"
	"time"
)

// Store persists deal state. Implementations must be safe for concurrent use.
type Store interface {
	// Upsert inserts or updates a deal by ID.
	Upsert(d *Deal) error

	// Get returns the deal with the given ID, or (nil, nil) if missing.
	Get(id DealID) (*Deal, error)

	// ListByStatus returns all deals currently in the given status.
	ListByStatus(s Status) ([]*Deal, error)

	// ListAll returns every deal known locally.
	ListAll() ([]*Deal, error)

	// LastSeenBlock returns the highest block number the engine has processed.
	// Used as a watermark when polling for new events.
	LastSeenBlock() (uint64, error)

	// SetLastSeenBlock updates the watermark.
	SetLastSeenBlock(uint64) error

	// Close releases resources.
	Close() error
}

// MemStore is an in-memory Store, for tests and ephemeral runs. A SQLite
// implementation can be added later for durability without changing any
// engine code.
type MemStore struct {
	mu       sync.Mutex
	deals    map[DealID]*Deal
	lastBlk  uint64
}

// NewMemStore constructs an empty in-memory Store.
func NewMemStore() *MemStore {
	return &MemStore{deals: map[DealID]*Deal{}}
}

var _ Store = (*MemStore)(nil)

// Upsert inserts or replaces a deal.
func (s *MemStore) Upsert(d *Deal) error {
	if d == nil {
		return fmt.Errorf("nil deal")
	}
	if d.UpdatedAt.IsZero() {
		d.UpdatedAt = time.Now().UTC()
	}
	copy := *d // defensive copy to avoid external mutation
	s.mu.Lock()
	s.deals[d.ID] = &copy
	s.mu.Unlock()
	return nil
}

// Get returns the deal or nil if missing.
func (s *MemStore) Get(id DealID) (*Deal, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if d, ok := s.deals[id]; ok {
		c := *d
		return &c, nil
	}
	return nil, nil
}

// ListByStatus returns deals matching a status.
func (s *MemStore) ListByStatus(status Status) ([]*Deal, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	var out []*Deal
	for _, d := range s.deals {
		if d.Status == status {
			c := *d
			out = append(out, &c)
		}
	}
	return out, nil
}

// ListAll returns all stored deals.
func (s *MemStore) ListAll() ([]*Deal, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	out := make([]*Deal, 0, len(s.deals))
	for _, d := range s.deals {
		c := *d
		out = append(out, &c)
	}
	return out, nil
}

// LastSeenBlock returns the high-water mark block.
func (s *MemStore) LastSeenBlock() (uint64, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.lastBlk, nil
}

// SetLastSeenBlock records the high-water mark block.
func (s *MemStore) SetLastSeenBlock(n uint64) error {
	s.mu.Lock()
	s.lastBlk = n
	s.mu.Unlock()
	return nil
}

// Close is a no-op for MemStore.
func (s *MemStore) Close() error { return nil }
