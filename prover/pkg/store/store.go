// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Prova Network contributors.

// Package store provides the local blob storage backend for a Prova prover.
//
// Stored pieces are indexed by CommP hash. The on-disk layout is
// content-addressed: pieces live under paths derived from their hash so
// that deduplication is automatic and lookups are O(1).
package store

import (
	"fmt"
	"io"

	"github.com/ipfs/go-cid"
)

// Store is the minimal interface a blob storage backend must provide.
type Store interface {
	// Put writes a piece and returns its length in bytes.
	Put(pieceCid cid.Cid, r io.Reader) (uint64, error)

	// Get returns a reader for a piece.
	Get(pieceCid cid.Cid) (io.ReadCloser, error)

	// Has reports whether the piece is present in storage.
	Has(pieceCid cid.Cid) (bool, error)

	// Delete removes a piece.
	Delete(pieceCid cid.Cid) error

	// Size returns the on-disk size of a piece, or an error if not present.
	Size(pieceCid cid.Cid) (uint64, error)

	// Close releases any underlying resources.
	Close() error
}

// ErrNotFound is returned when a piece is not in the store.
var ErrNotFound = fmt.Errorf("piece not found")
