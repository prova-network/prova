// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2024-2026 Filecoin Project contributors (upstream: filecoin-project/lotus).
// Copyright (c) 2026 Prova Network contributors.
//
// This file is adapted from filecoin-project/lotus storage/sealer/fr32/fr32.go
// (https://github.com/filecoin-project/lotus). Originally under the Permissive
// License Stack (Apache-2.0 OR MIT). Attribution preserved per license.
//
// Adaptations for Prova:
//   - Stripped abi.PaddedPieceSize / abi.UnpaddedPieceSize (use raw byte lens)
//   - Dropped multi-threaded fan-out; single-threaded handles our sizes fine
//   - Dropped unused Unpad helpers
//
// The pad() function below is byte-identical to upstream lotus' pad().
// Any change here risks bit-level divergence from Filecoin's canonical fr32
// padding, which would produce CommP roots that don't match the reference
// implementation. See pdptree_test.go for cross-validation against published
// CommP fixtures.

package pdptree

// Fr32 padding inserts two zero bits every 254 bits of input. The operation
// is performed in 127-byte-input → 128-byte-output chunks.
const (
	UnpaddedFr32Chunk = 127
	PaddedFr32Chunk   = 128
)

// fr32Pad expands len(in) bytes of raw data into len(out) bytes of fr32-padded
// data. Requires len(in) % 127 == 0 and len(out) == len(in) / 127 * 128.
//
// Output is bit-identical to Lotus' fr32.Pad and Filecoin-FFI's reference.
func fr32Pad(in, out []byte) {
	chunks := len(out) / PaddedFr32Chunk
	for chunk := 0; chunk < chunks; chunk++ {
		inOff := chunk * UnpaddedFr32Chunk
		outOff := chunk * PaddedFr32Chunk

		copy(out[outOff:outOff+31], in[inOff:inOff+31])

		t := in[inOff+31] >> 6
		out[outOff+31] = in[inOff+31] & 0x3f
		var v byte

		for i := 32; i < 64; i++ {
			v = in[inOff+i]
			out[outOff+i] = (v << 2) | t
			t = v >> 6
		}

		t = v >> 4
		out[outOff+63] &= 0x3f

		for i := 64; i < 96; i++ {
			v = in[inOff+i]
			out[outOff+i] = (v << 4) | t
			t = v >> 4
		}

		t = v >> 2
		out[outOff+95] &= 0x3f

		for i := 96; i < 127; i++ {
			v = in[inOff+i]
			out[outOff+i] = (v << 6) | t
			t = v >> 2
		}

		out[outOff+127] = t & 0x3f
	}
}
