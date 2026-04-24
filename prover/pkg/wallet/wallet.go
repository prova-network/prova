// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

// Package wallet loads a prover's signing key from either a go-ethereum
// keystore file or a raw hex-encoded private key.
//
// Precedence:
//   1. $PROVA_PRIVATE_KEY (hex) if set — development only
//   2. config.Identity.KeystorePath + passphrase (passphrase from env or config)
//   3. config.Identity.PrivateKeyHex — development only
//
// Production should always use the keystore path with a passphrase provided
// via environment variable, never committed to disk.
package wallet

import (
	"crypto/ecdsa"
	"fmt"
	"os"
	"strings"

	"github.com/ethereum/go-ethereum/accounts/keystore"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
)

// Wallet holds a loaded signing key and its derived address.
type Wallet struct {
	PrivateKey *ecdsa.PrivateKey
	Address    common.Address
}

// LoadHex parses a hex-encoded ECDSA private key (with or without 0x prefix).
func LoadHex(hexKey string) (*Wallet, error) {
	hexKey = strings.TrimPrefix(strings.TrimSpace(hexKey), "0x")
	if len(hexKey) != 64 {
		return nil, fmt.Errorf("private key hex must be 64 chars, got %d", len(hexKey))
	}

	priv, err := crypto.HexToECDSA(hexKey)
	if err != nil {
		return nil, fmt.Errorf("parse private key: %w", err)
	}

	return fromKey(priv), nil
}

// LoadKeystore decrypts an ethereum keystore v3 file.
//
// If passphrase is empty, the function tries $PROVA_KEYSTORE_PASSPHRASE.
func LoadKeystore(path string, passphrase string) (*Wallet, error) {
	if passphrase == "" {
		passphrase = os.Getenv("PROVA_KEYSTORE_PASSPHRASE")
	}

	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("read keystore %q: %w", path, err)
	}

	key, err := keystore.DecryptKey(data, passphrase)
	if err != nil {
		return nil, fmt.Errorf("decrypt keystore: %w", err)
	}

	return fromKey(key.PrivateKey), nil
}

// LoadFromEnv checks for a hex key in $PROVA_PRIVATE_KEY.
// Returns (nil, false) if the var is not set. Returns (w, true) on success.
func LoadFromEnv() (*Wallet, bool, error) {
	hexKey := os.Getenv("PROVA_PRIVATE_KEY")
	if hexKey == "" {
		return nil, false, nil
	}
	w, err := LoadHex(hexKey)
	return w, true, err
}

// fromKey builds a Wallet by deriving the address from the public key.
func fromKey(priv *ecdsa.PrivateKey) *Wallet {
	pub, _ := priv.Public().(*ecdsa.PublicKey)
	return &Wallet{
		PrivateKey: priv,
		Address:    crypto.PubkeyToAddress(*pub),
	}
}
