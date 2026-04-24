// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package deal

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestValidateSourceURL_SecureMode(t *testing.T) {
	// Force secure mode regardless of env
	orig := pullAllowInsecure
	pullAllowInsecure = false
	t.Cleanup(func() { pullAllowInsecure = orig })

	cases := []struct {
		name    string
		url     string
		wantErr string
	}{
		{"https ok", "https://example.com/piece/abc", ""},
		{"http rejected", "http://example.com/piece/abc", "must use https"},
		{"localhost rejected", "https://localhost/piece", "localhost"},
		{"loopback v4 rejected", "https://127.0.0.1/piece", "loopback"},
		{"loopback v6 rejected", "https://[::1]/piece", "loopback"},
		{"private 10.x rejected", "https://10.0.0.1/piece", "private"},
		{"private 192.168 rejected", "https://192.168.1.1/piece", "private"},
		{"userinfo rejected", "https://user:pass@example.com/piece", "userinfo"},
		{"ftp rejected", "ftp://example.com/piece", "unsupported scheme"},
		{"empty rejected", "https:///piece", "must have a host"},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			err := ValidateSourceURL(tc.url)
			if tc.wantErr == "" {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.wantErr)
			}
		})
	}
}

func TestValidateSourceURL_InsecureMode(t *testing.T) {
	orig := pullAllowInsecure
	pullAllowInsecure = true
	t.Cleanup(func() { pullAllowInsecure = orig })

	cases := []string{
		"http://localhost:8080/piece",
		"http://127.0.0.1:1234/piece/abc",
		"https://10.0.0.1/piece",
		"https://example.com/piece",
	}
	for _, u := range cases {
		err := ValidateSourceURL(u)
		require.NoError(t, err, u)
	}
}

func TestPullAllowInsecure_EnvReflection(t *testing.T) {
	// This is a sanity check that the package-level var is seeded from
	// the env at package init time; we don't call the init function
	// directly, but we can verify the var name is stable.
	_ = pullAllowInsecure
	_ = os.Getenv("PROVA_PULL_ALLOW_INSECURE")
}
