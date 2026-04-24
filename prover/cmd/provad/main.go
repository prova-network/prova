// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Prova Network contributors.

// provad is the Prova prover daemon.
//
// Usage:
//
//	provad --config /etc/prova/prover.toml start
//	provad --config /etc/prova/prover.toml register
//	provad --config /etc/prova/prover.toml status
//	provad version
//
// The daemon watches the Prova contracts on Base, accepts deals targeting
// this prover, downloads pieces, computes + verifies CommP, stores pieces
// locally, responds to on-chain proof challenges, and optionally serves
// content over HTTPS.
package main

import (
	"flag"
	"fmt"
	"os"
	"runtime"

	"github.com/prova-network/prova/prover/pkg/config"
)

// These are set via -ldflags at build time:
//
//	go build -ldflags "-X main.version=0.1.0 -X main.commit=$(git rev-parse --short HEAD)"
var (
	version = "dev"
	commit  = "unknown"
)

func main() {
	if err := run(); err != nil {
		fmt.Fprintf(os.Stderr, "provad: %v\n", err)
		os.Exit(1)
	}
}

func run() error {
	var (
		configPath string
	)
	flag.StringVar(&configPath, "config", "", "path to prover TOML config (required for start/register/status)")
	flag.Usage = usage
	flag.Parse()

	args := flag.Args()
	if len(args) == 0 {
		usage()
		return fmt.Errorf("missing subcommand")
	}

	cmd := args[0]
	switch cmd {
	case "version":
		return cmdVersion()
	case "start":
		return cmdStart(configPath)
	case "register":
		return cmdRegister(configPath)
	case "status":
		return cmdStatus(configPath)
	case "help", "-h", "--help":
		usage()
		return nil
	default:
		usage()
		return fmt.Errorf("unknown subcommand: %q", cmd)
	}
}

func usage() {
	fmt.Fprintf(os.Stderr, `provad - Prova Network prover daemon

Usage:
  provad [flags] <subcommand>

Subcommands:
  start     Run the prover daemon (main mode)
  register  Register this prover in ProverRegistry on-chain
  status    Print current prover status and exit
  version   Print version info and exit
  help      Print this help

Flags:
  --config  Path to prover TOML config file

Examples:
  provad version
  provad --config /etc/prova/prover.toml start
`)
}

func cmdVersion() error {
	fmt.Printf("provad %s (commit %s, %s/%s, %s)\n",
		version, commit, runtime.GOOS, runtime.GOARCH, runtime.Version())
	return nil
}

func cmdStart(configPath string) error {
	cfg, err := loadConfig(configPath)
	if err != nil {
		return err
	}
	_ = cfg // TODO: wire up daemon loop
	return fmt.Errorf("start: not yet implemented")
}

func cmdRegister(configPath string) error {
	cfg, err := loadConfig(configPath)
	if err != nil {
		return err
	}
	_ = cfg // TODO: call ProverRegistry.register()
	return fmt.Errorf("register: not yet implemented")
}

func cmdStatus(configPath string) error {
	cfg, err := loadConfig(configPath)
	if err != nil {
		return err
	}
	_ = cfg // TODO: read on-chain registry state + local store state
	return fmt.Errorf("status: not yet implemented")
}

func loadConfig(path string) (*config.Config, error) {
	if path == "" {
		return nil, fmt.Errorf("--config is required")
	}
	return config.Load(path)
}
