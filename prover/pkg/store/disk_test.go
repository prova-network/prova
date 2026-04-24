// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package store

import (
	"bytes"
	"errors"
	"io"
	"os"
	"strings"
	"testing"

	"github.com/ipfs/go-cid"
	"github.com/multiformats/go-multihash"
	"github.com/stretchr/testify/require"
)

func mkCid(t *testing.T, content string) cid.Cid {
	t.Helper()
	mh, err := multihash.Sum([]byte(content), multihash.SHA2_256, -1)
	require.NoError(t, err)
	return cid.NewCidV1(cid.Raw, mh)
}

func TestDiskStore_PutGetHasDelete(t *testing.T) {
	dir := t.TempDir()
	s, err := NewDiskStore(dir)
	require.NoError(t, err)
	defer s.Close()

	c := mkCid(t, "hello world")
	payload := []byte("hello world")

	// Initially absent
	has, err := s.Has(c)
	require.NoError(t, err)
	require.False(t, has)

	// Put
	n, err := s.Put(c, bytes.NewReader(payload))
	require.NoError(t, err)
	require.Equal(t, uint64(len(payload)), n)

	// Has + Size
	has, err = s.Has(c)
	require.NoError(t, err)
	require.True(t, has)
	sz, err := s.Size(c)
	require.NoError(t, err)
	require.Equal(t, uint64(len(payload)), sz)

	// Get
	r, err := s.Get(c)
	require.NoError(t, err)
	got, err := io.ReadAll(r)
	require.NoError(t, err)
	require.NoError(t, r.Close())
	require.Equal(t, payload, got)

	// Delete
	require.NoError(t, s.Delete(c))
	has, err = s.Has(c)
	require.NoError(t, err)
	require.False(t, has)
}

func TestDiskStore_NotFound(t *testing.T) {
	dir := t.TempDir()
	s, err := NewDiskStore(dir)
	require.NoError(t, err)

	c := mkCid(t, "never stored")

	_, err = s.Get(c)
	require.True(t, errors.Is(err, ErrNotFound))

	_, err = s.Size(c)
	require.True(t, errors.Is(err, ErrNotFound))

	err = s.Delete(c)
	require.True(t, errors.Is(err, ErrNotFound))
}

func TestDiskStore_AtomicWrite(t *testing.T) {
	// Verify the tmp file is not left behind after a successful Put.
	dir := t.TempDir()
	s, err := NewDiskStore(dir)
	require.NoError(t, err)

	c := mkCid(t, "atomic test")
	_, err = s.Put(c, bytes.NewReader([]byte("atomic test")))
	require.NoError(t, err)

	subdir, _ := s.pathFor(c)
	entries, err := os.ReadDir(subdir)
	require.NoError(t, err)

	var leaked []string
	for _, e := range entries {
		if strings.HasPrefix(e.Name(), ".tmp-") {
			leaked = append(leaked, e.Name())
		}
	}
	require.Empty(t, leaked, "found leaked tmp files: %v", leaked)
}
