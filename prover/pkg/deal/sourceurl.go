// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package deal

import (
	"encoding/hex"
	"errors"
	"fmt"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ipfs/go-cid"
	"github.com/multiformats/go-multihash"
)

// SourceURLResolver converts a client address + commP hash into a URL from
// which the piece content can be fetched.
//
// v1 DealProposed events do not carry a source URL field (adding it would
// have increased on-chain storage cost on every deal). Instead, clients
// publish pieces at a predictable location and the prover derives the URL
// from deal metadata. This resolver encodes that convention.
//
// Two strategies are supported:
//
//   - Template: substitutes {client}, {clientRaw}, {commpHex}, {commpCid}
//     in a configured template URL
//   - ENS lookup (future): resolves clientAddress.resolver("source") → URL
type SourceURLResolver struct {
	template string
}

// NewSourceURLResolver constructs a resolver. An empty template disables
// resolution; pieces ingested without an out-of-band SourceURL will fail
// at the download step as before.
func NewSourceURLResolver(template string) *SourceURLResolver {
	return &SourceURLResolver{template: template}
}

// Resolve returns the URL to fetch a piece from, given the deal's client
// and commP hash. Returns an empty string (no error) if the resolver is
// not configured.
func (r *SourceURLResolver) Resolve(client common.Address, commpHash [32]byte) (string, error) {
	if r == nil || r.template == "" {
		return "", nil
	}

	commpHex := hex.EncodeToString(commpHash[:])
	commpCidStr, err := commpCIDString(commpHash)
	if err != nil {
		return "", fmt.Errorf("build commp cid: %w", err)
	}

	clientLower := strings.ToLower(client.Hex())
	clientRaw := strings.TrimPrefix(clientLower, "0x")

	out := r.template
	out = strings.ReplaceAll(out, "{client}", clientLower)
	out = strings.ReplaceAll(out, "{clientRaw}", clientRaw)
	out = strings.ReplaceAll(out, "{commpHex}", commpHex)
	out = strings.ReplaceAll(out, "{commpCid}", commpCidStr)

	if strings.Contains(out, "{") {
		return "", fmt.Errorf("template has unresolved placeholders: %s", out)
	}
	return out, nil
}

// commpCIDString builds the canonical piece-commitment CID v1 string for a
// 32-byte CommP hash.
func commpCIDString(hash [32]byte) (string, error) {
	if hash == ([32]byte{}) {
		return "", errors.New("empty commP hash")
	}
	// sha2-256-trunc254-padded multihash code = 0x1012
	mh, err := multihash.Encode(hash[:], 0x1012)
	if err != nil {
		return "", err
	}
	// fil-commitment-unsealed codec = 0xf101
	return cid.NewCidV1(0xf101, mh).String(), nil
}
