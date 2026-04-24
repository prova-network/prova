// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package deal

import (
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/stretchr/testify/require"
)

func TestSourceURLResolver_Disabled(t *testing.T) {
	r := NewSourceURLResolver("")
	client := common.HexToAddress("0xCaFE00000000000000000000000000000000CAFE")
	var commP [32]byte
	commP[0] = 0xab
	got, err := r.Resolve(client, commP)
	require.NoError(t, err)
	require.Empty(t, got)
}

func TestSourceURLResolver_NilReceiver(t *testing.T) {
	var r *SourceURLResolver
	client := common.HexToAddress("0x1")
	got, err := r.Resolve(client, [32]byte{})
	require.NoError(t, err)
	require.Empty(t, got)
}

func TestSourceURLResolver_TemplateSubstitution(t *testing.T) {
	r := NewSourceURLResolver("https://pieces.example.com/{client}/{commpHex}")
	client := common.HexToAddress("0xCaFE00000000000000000000000000000000CAFE")
	var commP [32]byte
	copy(commP[:], []byte{0xab, 0xcd, 0xef, 0x01})
	got, err := r.Resolve(client, commP)
	require.NoError(t, err)
	require.Equal(t,
		"https://pieces.example.com/0xcafe00000000000000000000000000000000cafe/abcdef01"+
			"00000000000000000000000000000000000000000000000000000000",
		got)
}

func TestSourceURLResolver_ClientRawTemplate(t *testing.T) {
	r := NewSourceURLResolver("ipfs://ipns/{clientRaw}/piece")
	client := common.HexToAddress("0xDeadBeef00000000000000000000000000000000")
	got, err := r.Resolve(client, [32]byte{0x99})
	require.NoError(t, err)
	require.Equal(t, "ipfs://ipns/deadbeef00000000000000000000000000000000/piece", got)
}

func TestSourceURLResolver_CommpCidTemplate(t *testing.T) {
	r := NewSourceURLResolver("https://cdn.example/{commpCid}")
	var commP [32]byte
	for i := range commP {
		commP[i] = byte(i + 1)
	}
	got, err := r.Resolve(common.HexToAddress("0x0"), commP)
	require.NoError(t, err)
	// Canonical CommP CID starts with "baga6ea4seaq" for piece commitments
	require.Contains(t, got, "https://cdn.example/baga6ea4seaq")
}

func TestSourceURLResolver_UnresolvedPlaceholder(t *testing.T) {
	r := NewSourceURLResolver("https://x/{somethingUnknown}/{client}")
	_, err := r.Resolve(common.HexToAddress("0x1"), [32]byte{0x01})
	require.ErrorContains(t, err, "unresolved placeholders")
}
