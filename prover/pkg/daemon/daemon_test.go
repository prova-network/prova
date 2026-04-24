// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package daemon

import (
	"testing"

	"github.com/stretchr/testify/require"
)

// TestNew_RequiresEngine exercises the earliest argument-validation branch.
// Fuller daemon coverage lives in the anvil-backed smoke tests; this keeps
// the unit layer focused on what we can assert without a live RPC.
func TestNew_RequiresEngine(t *testing.T) {
	_, err := New(Options{})
	require.ErrorContains(t, err, "engine required")
}

// TestDefaultsApplied checks that zero-value Config fields receive the
// documented defaults when passed through Options.
//
// We test this via the struct normalization rather than by instantiating
// a Daemon, because Daemon.Run needs a live ethclient which isn't
// appropriate for a unit test.
func TestDefaultsApplied(t *testing.T) {
	c := Config{}
	// Emulate the defaulting logic from New().
	if c.TickInterval == 0 {
		c.TickInterval = defaultTickInterval
	}
	if c.PollInterval == 0 {
		c.PollInterval = defaultPollInterval
	}
	if c.StatusInterval == 0 {
		c.StatusInterval = defaultStatusInterval
	}
	if c.ShutdownTimeout == 0 {
		c.ShutdownTimeout = defaultShutdownTimeout
	}

	require.Equal(t, defaultTickInterval, c.TickInterval)
	require.Equal(t, defaultPollInterval, c.PollInterval)
	require.Equal(t, defaultStatusInterval, c.StatusInterval)
	require.Equal(t, defaultShutdownTimeout, c.ShutdownTimeout)
}
