// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Prova Network contributors.

package deal

import (
	"context"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"os"
	"strings"
	"time"
)

// pullAllowInsecure relaxes security validations for dev/test environments.
// Set PROVA_PULL_ALLOW_INSECURE=1 to:
//   - Allow http:// in addition to https://
//   - Allow localhost and private IP addresses
//
// Never set this in production.
//
// Inspired by curio's CURIO_PULL_ALLOW_INSECURE flag.
var pullAllowInsecure = os.Getenv("PROVA_PULL_ALLOW_INSECURE") == "1"

// Fetcher downloads piece content from a source URL.
type Fetcher struct {
	client   *http.Client
	maxBytes int64 // hard limit on piece size downloads
}

// FetcherOptions configures a Fetcher.
type FetcherOptions struct {
	// Timeout is the overall request timeout. Default: 10 minutes.
	Timeout time.Duration

	// MaxBytes caps download size. Default: 32 GiB.
	MaxBytes int64
}

// NewFetcher constructs a fetcher with sensible defaults.
func NewFetcher(opts FetcherOptions) *Fetcher {
	if opts.Timeout == 0 {
		opts.Timeout = 10 * time.Minute
	}
	if opts.MaxBytes == 0 {
		opts.MaxBytes = 32 << 30
	}
	return &Fetcher{
		client: &http.Client{
			Timeout: opts.Timeout,
			// Don't follow redirects blindly; each hop could break our
			// URL validation. The client may re-supply a redirected URL
			// via the SourceURL field.
			CheckRedirect: func(_ *http.Request, via []*http.Request) error {
				if len(via) >= 3 {
					return fmt.Errorf("too many redirects")
				}
				return nil
			},
		},
		maxBytes: opts.MaxBytes,
	}
}

// ValidateSourceURL rejects unsafe URLs before download starts.
//
// Rules:
//   - Must be https:// (http:// only with PROVA_PULL_ALLOW_INSECURE=1)
//   - Hostname must not be empty
//   - Hostname must not be localhost / loopback / link-local / private
//     (unless PROVA_PULL_ALLOW_INSECURE=1)
//   - No userinfo (username:password in URL)
//
// Adapted from curio/pdp/pull_types.go ValidatePullSourceURL.
func ValidateSourceURL(raw string) error {
	u, err := url.Parse(raw)
	if err != nil {
		return fmt.Errorf("invalid URL: %w", err)
	}

	if u.User != nil {
		return fmt.Errorf("URL must not contain userinfo")
	}

	switch u.Scheme {
	case "https":
		// ok
	case "http":
		if !pullAllowInsecure {
			return fmt.Errorf("URL must use https; got %q (set PROVA_PULL_ALLOW_INSECURE=1 to permit http for dev)", u.Scheme)
		}
	default:
		return fmt.Errorf("unsupported scheme %q", u.Scheme)
	}

	host := u.Hostname()
	if host == "" {
		return fmt.Errorf("URL must have a host")
	}

	if !pullAllowInsecure {
		if err := rejectUnsafeHost(host); err != nil {
			return err
		}
	}

	return nil
}

// rejectUnsafeHost returns an error if host is a loopback, link-local,
// or private address / hostname.
func rejectUnsafeHost(host string) error {
	lower := strings.ToLower(host)
	if lower == "localhost" || strings.HasSuffix(lower, ".localhost") {
		return fmt.Errorf("localhost is not allowed as source")
	}

	ip := net.ParseIP(host)
	if ip == nil {
		// Hostname, not an IP. We can't block on name alone; DNS
		// resolution will happen inside the HTTP client. Defense in
		// depth would do a resolve + re-check here, but that's racy
		// against the actual connect. For now, hostname passes; dev
		// use of localhost is already blocked by the string check.
		return nil
	}
	if ip.IsLoopback() {
		return fmt.Errorf("loopback addresses are not allowed as source")
	}
	if ip.IsLinkLocalUnicast() || ip.IsLinkLocalMulticast() {
		return fmt.Errorf("link-local addresses are not allowed as source")
	}
	if ip.IsPrivate() {
		return fmt.Errorf("private addresses are not allowed as source")
	}
	return nil
}

// Fetch downloads a piece from sourceURL into the given writer, returning
// the number of bytes written.
//
// The download is bounded by the fetcher's maxBytes limit.
func (f *Fetcher) Fetch(ctx context.Context, sourceURL string, dst io.Writer) (int64, error) {
	if err := ValidateSourceURL(sourceURL); err != nil {
		return 0, err
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, sourceURL, nil)
	if err != nil {
		return 0, fmt.Errorf("build request: %w", err)
	}
	req.Header.Set("Accept", "application/octet-stream")
	req.Header.Set("User-Agent", "provad/0.1")

	resp, err := f.client.Do(req)
	if err != nil {
		return 0, fmt.Errorf("http get: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return 0, fmt.Errorf("http %d %s", resp.StatusCode, resp.Status)
	}

	// Apply a hard limit so a malicious server can't fill disk
	limited := io.LimitReader(resp.Body, f.maxBytes+1)
	n, err := io.Copy(dst, limited)
	if err != nil {
		return n, fmt.Errorf("copy body: %w", err)
	}
	if n > f.maxBytes {
		return n, fmt.Errorf("piece exceeds max bytes (%d > %d)", n, f.maxBytes)
	}
	return n, nil
}
