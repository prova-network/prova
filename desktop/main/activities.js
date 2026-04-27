// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) Protocol Labs (original), Prova Network contributors (modifications).
// Forked from CheckerNetwork/desktop main/activities.js.
//
// In-memory ring buffer of recent user-facing activity log lines, plus
// a tiny derived-state machine that tracks whether the local prover
// daemon (`provad`) is "online" from the user's perspective.
//
// The upstream Checker version watched for first-word `spark` / `voyager`
// markers because the Station daemon split its work between two named
// modules. Prova's daemon is a single process, so we instead key the
// online flag off the supervisor + provad subsystems directly.

'use strict'

/** @typedef {import('./typings').Activity} Activity */
/** @typedef {import('./typings').Context} Context */

/// Subsystems whose successful messages indicate the prover daemon is alive.
/// `supervisor` is emitted by main/provad.js itself when it spawns/restarts
/// the daemon. `provad`, `engine`, `httpserver`, `daemon` are subsystem
/// labels emitted by the Go daemon's structured logger.
const ONLINE_SOURCES = new Set([
  'supervisor',
  'provad',
  'engine',
  'httpserver',
  'daemon'
])

class Activities {
  /** @type {Activity[]} */
  #activities = []
  /// True if we have seen a recent successful event from one of the
  /// online sources and no terminal error has flipped us off since.
  #online = false

  /**
   * Append an activity to the ring buffer (cap 100), update the online
   * flag, and notify the consumer via `ctx.recordActivity`.
   * @param {Context} ctx
   * @param {Activity} activity
   */
  push (ctx, activity) {
    this.#activities.push(activity)
    if (this.#activities.length > 100) {
      this.#activities.splice(0, this.#activities.length - 100)
    }
    this.#updateOnline(activity)
    ctx.recordActivity(activity)
  }

  /**
   * @param {Activity} activity
   */
  #updateOnline (activity) {
    const source = (activity.source || '').toLowerCase()
    if (!ONLINE_SOURCES.has(source)) return

    if (activity.type === 'started' || activity.type === 'info') {
      this.#online = true
    } else if (activity.type === 'error') {
      this.#online = false
    }
  }

  /// Snapshot copy so callers can't mutate our internal buffer.
  get () {
    return [...this.#activities]
  }

  /// True if the prover daemon has reported any healthy lifecycle/info
  /// event from a known subsystem and has not subsequently errored.
  isOnline () {
    return this.#online
  }
}

module.exports = {
  Activities
}
