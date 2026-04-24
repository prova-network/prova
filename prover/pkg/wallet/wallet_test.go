// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package wallet

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/ethereum/go-ethereum/accounts/keystore"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/stretchr/testify/require"
)

// Test vector derived from an arbitrary secp256k1 private key. Not associated
// with any real funds; for unit tests only.
const (
	testPrivHex = "b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291"
	testAddr    = "0x71562b71999873DB5b286dF957af199Ec94617F7"
)

func TestLoadHex_StripsPrefix(t *testing.T) {
	cases := []string{
		testPrivHex,
		"0x" + testPrivHex,
		"  " + testPrivHex + "  ",
	}
	for _, c := range cases {
		w, err := LoadHex(c)
		require.NoError(t, err, c)
		require.Equal(t, testAddr, w.Address.Hex())
	}
}

func TestLoadHex_WrongLength(t *testing.T) {
	_, err := LoadHex("0xdeadbeef")
	require.ErrorContains(t, err, "must be 64 chars")
}

func TestLoadKeystore_Roundtrip(t *testing.T) {
	// Write a keystore file with a known key and decrypt it.
	priv, err := crypto.HexToECDSA(testPrivHex)
	require.NoError(t, err)

	tmpDir := t.TempDir()
	ks := keystore.NewKeyStore(tmpDir, keystore.LightScryptN, keystore.LightScryptP)
	acct, err := ks.ImportECDSA(priv, "testpass")
	require.NoError(t, err)

	w, err := LoadKeystore(acct.URL.Path, "testpass")
	require.NoError(t, err)
	require.Equal(t, testAddr, w.Address.Hex())
}

func TestLoadKeystore_BadPassphrase(t *testing.T) {
	priv, err := crypto.HexToECDSA(testPrivHex)
	require.NoError(t, err)

	tmpDir := t.TempDir()
	ks := keystore.NewKeyStore(tmpDir, keystore.LightScryptN, keystore.LightScryptP)
	acct, err := ks.ImportECDSA(priv, "goodpass")
	require.NoError(t, err)

	_, err = LoadKeystore(acct.URL.Path, "wrongpass")
	require.ErrorContains(t, err, "decrypt")
}

func TestLoadKeystore_PassphraseFromEnv(t *testing.T) {
	priv, err := crypto.HexToECDSA(testPrivHex)
	require.NoError(t, err)

	tmpDir := t.TempDir()
	ks := keystore.NewKeyStore(tmpDir, keystore.LightScryptN, keystore.LightScryptP)
	acct, err := ks.ImportECDSA(priv, "envpass")
	require.NoError(t, err)

	t.Setenv("PROVA_KEYSTORE_PASSPHRASE", "envpass")
	w, err := LoadKeystore(acct.URL.Path, "") // empty -> fall through to env
	require.NoError(t, err)
	require.Equal(t, testAddr, w.Address.Hex())
}

func TestLoadFromEnv_Unset(t *testing.T) {
	// Clear env
	t.Setenv("PROVA_PRIVATE_KEY", "")
	os.Unsetenv("PROVA_PRIVATE_KEY") // Setenv("") sets it to empty, Unsetenv removes
	w, ok, err := LoadFromEnv()
	require.NoError(t, err)
	require.False(t, ok)
	require.Nil(t, w)
}

func TestLoadFromEnv_Set(t *testing.T) {
	t.Setenv("PROVA_PRIVATE_KEY", testPrivHex)
	w, ok, err := LoadFromEnv()
	require.NoError(t, err)
	require.True(t, ok)
	require.Equal(t, testAddr, w.Address.Hex())
}

// Compile-time check: make sure we don't accidentally leak a tmp dir assumption
func TestLoadKeystore_PathExpansion(t *testing.T) {
	tmp := filepath.Join(t.TempDir(), "nope.json")
	_, err := LoadKeystore(tmp, "x")
	require.ErrorContains(t, err, "read keystore")
}
