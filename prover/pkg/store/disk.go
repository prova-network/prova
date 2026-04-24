// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Prova Network contributors.

package store

import (
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/ipfs/go-cid"
)

// DiskStore stores pieces as regular files under a base directory, addressed
// by CommP CID string.
//
// On-disk layout:
//
//	<baseDir>/<first 2 chars of CID>/<next 2 chars>/<full CID>
//
// Two-level fan-out keeps inode counts reasonable per directory at scale.
type DiskStore struct {
	baseDir string
}

// NewDiskStore creates a DiskStore rooted at baseDir. The directory is
// created if it does not already exist.
func NewDiskStore(baseDir string) (*DiskStore, error) {
	if baseDir == "" {
		return nil, fmt.Errorf("baseDir is required")
	}
	if err := os.MkdirAll(baseDir, 0o750); err != nil {
		return nil, fmt.Errorf("create base dir %q: %w", baseDir, err)
	}
	return &DiskStore{baseDir: baseDir}, nil
}

var _ Store = (*DiskStore)(nil)

func (s *DiskStore) pathFor(c cid.Cid) (dir, file string) {
	key := c.String()
	if len(key) < 4 {
		return s.baseDir, filepath.Join(s.baseDir, key)
	}
	dir = filepath.Join(s.baseDir, key[:2], key[2:4])
	return dir, filepath.Join(dir, key)
}

// Put writes the piece to disk atomically using a temp file + rename.
func (s *DiskStore) Put(pieceCid cid.Cid, r io.Reader) (uint64, error) {
	dir, final := s.pathFor(pieceCid)
	if err := os.MkdirAll(dir, 0o750); err != nil {
		return 0, fmt.Errorf("mkdir %q: %w", dir, err)
	}

	tmp, err := os.CreateTemp(dir, ".tmp-"+pieceCid.String()+"-*")
	if err != nil {
		return 0, fmt.Errorf("create tmp: %w", err)
	}
	tmpPath := tmp.Name()
	defer func() {
		// If rename failed, clean up the tmp file.
		_ = os.Remove(tmpPath)
	}()

	n, err := io.Copy(tmp, r)
	if err != nil {
		tmp.Close()
		return 0, fmt.Errorf("write tmp: %w", err)
	}
	if err := tmp.Sync(); err != nil {
		tmp.Close()
		return 0, fmt.Errorf("sync tmp: %w", err)
	}
	if err := tmp.Close(); err != nil {
		return 0, fmt.Errorf("close tmp: %w", err)
	}

	if err := os.Rename(tmpPath, final); err != nil {
		return 0, fmt.Errorf("rename %q -> %q: %w", tmpPath, final, err)
	}

	return uint64(n), nil
}

// Get returns a reader for the piece.
func (s *DiskStore) Get(pieceCid cid.Cid) (io.ReadCloser, error) {
	_, path := s.pathFor(pieceCid)
	f, err := os.Open(path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return nil, ErrNotFound
		}
		return nil, fmt.Errorf("open %q: %w", path, err)
	}
	return f, nil
}

// Has reports whether the piece is stored.
func (s *DiskStore) Has(pieceCid cid.Cid) (bool, error) {
	_, path := s.pathFor(pieceCid)
	_, err := os.Stat(path)
	if err == nil {
		return true, nil
	}
	if errors.Is(err, os.ErrNotExist) {
		return false, nil
	}
	return false, fmt.Errorf("stat %q: %w", path, err)
}

// Size returns the byte length of the stored piece.
func (s *DiskStore) Size(pieceCid cid.Cid) (uint64, error) {
	_, path := s.pathFor(pieceCid)
	info, err := os.Stat(path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return 0, ErrNotFound
		}
		return 0, fmt.Errorf("stat %q: %w", path, err)
	}
	return uint64(info.Size()), nil
}

// Delete removes the piece from disk.
func (s *DiskStore) Delete(pieceCid cid.Cid) error {
	_, path := s.pathFor(pieceCid)
	err := os.Remove(path)
	if err == nil {
		return nil
	}
	if errors.Is(err, os.ErrNotExist) {
		return ErrNotFound
	}
	return fmt.Errorf("remove %q: %w", path, err)
}

// Close is a no-op for a disk-backed store.
func (s *DiskStore) Close() error {
	return nil
}
