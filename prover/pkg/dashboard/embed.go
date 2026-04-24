// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package dashboard

import "embed"

// distFS embeds the pre-built SPA. The directory is intentionally empty
// in the repo; running `cd prover/webui && npm ci && npm run build`
// populates it with hashed assets, then `go build` picks them up.
//
// We include a .gitkeep sentinel so go:embed doesn't fail on a missing
// dir in fresh checkouts. When the SPA isn't built, the Server falls
// back to an HTML placeholder that still exposes the JSON API.
//
//go:embed all:dist
var distFS embed.FS
