// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Prova Network contributors.

package config

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestLoad_Valid(t *testing.T) {
	tmpDir := t.TempDir()
	cfgPath := filepath.Join(tmpDir, "prover.toml")

	err := os.WriteFile(cfgPath, []byte(`
[identity]
private_key_hex = "0x1111111111111111111111111111111111111111111111111111111111111111"

[chain]
rpc_url = "https://sepolia.base.org"
chain_id = 84532

[chain.contracts]
prova_token = "0x0000000000000000000000000000000000000001"
proof_verifier = "0x0000000000000000000000000000000000000002"

[storage]
data_dir = "/var/lib/prova/data"

[http]
enabled = true
public_url = "https://prover.example"
`), 0o600)
	require.NoError(t, err)

	c, err := Load(cfgPath)
	require.NoError(t, err)

	require.Equal(t, uint64(84532), c.Chain.ChainID)
	require.Equal(t, "0x0000000000000000000000000000000000000001", c.Chain.Contracts.ProvaToken)
	require.Equal(t, "/var/lib/prova/data/index.sqlite", c.Storage.IndexPath) // default applied
	require.Equal(t, 12, c.Chain.PollIntervalSeconds)                         // default applied
	require.Equal(t, ":8443", c.HTTP.ListenAddr)                              // default applied
	require.True(t, c.HTTP.Enabled)
}

func TestLoad_MissingRPC(t *testing.T) {
	tmpDir := t.TempDir()
	cfgPath := filepath.Join(tmpDir, "prover.toml")

	err := os.WriteFile(cfgPath, []byte(`
[identity]
private_key_hex = "0x1"

[chain]
chain_id = 84532

[storage]
data_dir = "/var/lib/prova/data"
`), 0o600)
	require.NoError(t, err)

	_, err = Load(cfgPath)
	require.ErrorContains(t, err, "chain.rpc_url")
}

func TestLoad_BothIdentitySources(t *testing.T) {
	tmpDir := t.TempDir()
	cfgPath := filepath.Join(tmpDir, "prover.toml")

	err := os.WriteFile(cfgPath, []byte(`
[identity]
keystore_path = "/some/path"
private_key_hex = "0x1"

[chain]
rpc_url = "https://sepolia.base.org"
chain_id = 84532

[storage]
data_dir = "/var/lib/prova/data"
`), 0o600)
	require.NoError(t, err)

	_, err = Load(cfgPath)
	require.ErrorContains(t, err, "cannot set both")
}
