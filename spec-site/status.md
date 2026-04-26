---
description: Per-section state and audit status for the Prova spec.
---

# Status overview

This page tracks the state of every section of the Prova specification. The
[Filecoin spec](https://spec.filecoin.io/) inspired the structure: each
section gets a state label and a theory-audit label, both shown right next
to the section title.

## State legend

| Label | Meaning |
| --- | --- |
| <span class="badge state-stable">Stable</span> | Unlikely to change in the foreseeable future. Implementations may rely on the section being correct and complete. |
| <span class="badge state-reliable">Reliable</span> | All content is correct. Important details are covered. May be tightened over time but no breaking change expected. |
| <span class="badge state-wip">Draft / WIP</span> | All content is correct but details are still being worked on. Subject to change without breaking-change notice until promoted. |
| <span class="badge state-incorrect">Incorrect</span> | Do not follow. Important things have changed since this section was written. Used as a tombstone before rewrites. |
| <span class="badge state-missing">Missing</span> | No work has been done yet. Section is reserved in the table of contents only. |

## Audit legend

| Label | Meaning |
| --- | --- |
| <span class="badge audit-yes">Audited</span> | An external audit has reviewed the spec section and found it accurate. |
| <span class="badge audit-wip">In progress</span> | An audit is underway but the report has not landed. |
| <span class="badge audit-na">N/A</span> | No audit has been commissioned for this section yet, or the section is purely structural. |

## Spec status overview

<table class="spec-status-table">
  <thead>
    <tr>
      <th>Section</th>
      <th>Title</th>
      <th>State</th>
      <th>Theory audit</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td class="section-num">1.</td>
      <td class="section-title"><a href="/">Introduction</a></td>
      <td class="state"><span class="badge state-stable">Stable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">1.1</td>
      <td class="section-title"><a href="/">Spec home</a></td>
      <td class="state"><span class="badge state-stable">Stable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">1.2</td>
      <td class="section-title"><a href="/status">Status overview</a></td>
      <td class="state"><span class="badge state-stable">Stable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">1.3</td>
      <td class="section-title"><a href="/conventions">Conventions</a></td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">2.</td>
      <td class="section-title">Storage proofs</td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-wip">In progress</span></td>
    </tr>
    <tr>
      <td class="section-num">2.1</td>
      <td class="section-title"><a href="/pdp-integration">PDP integration</a></td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-wip">In progress</span></td>
    </tr>
    <tr>
      <td class="section-num">2.2</td>
      <td class="section-title"><a href="/checkpoint-anchoring">Checkpoint anchoring</a></td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">2.3</td>
      <td class="section-title"><a href="/data-availability">Data availability</a></td>
      <td class="state"><span class="badge state-wip">Draft / WIP</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">3.</td>
      <td class="section-title">Deal lifecycle</td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">3.1</td>
      <td class="section-title"><a href="/marketplace">Marketplace</a></td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">3.2</td>
      <td class="section-title"><a href="/event-schema">Event schema</a></td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">4.</td>
      <td class="section-title">Network</td>
      <td class="state"><span class="badge state-wip">Draft / WIP</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">4.1</td>
      <td class="section-title"><a href="/network-protocol">Network protocol</a></td>
      <td class="state"><span class="badge state-wip">Draft / WIP</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">4.2</td>
      <td class="section-title"><a href="/api-gateway">API gateway</a></td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">5.</td>
      <td class="section-title">Token economics</td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">5.1</td>
      <td class="section-title"><a href="/token-economics">Token economics</a></td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">5.2</td>
      <td class="section-title"><a href="/governance">Governance</a></td>
      <td class="state"><span class="badge state-wip">Draft / WIP</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
    <tr>
      <td class="section-num">6.</td>
      <td class="section-title">Security</td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-wip">In progress</span></td>
    </tr>
    <tr>
      <td class="section-num">6.1</td>
      <td class="section-title"><a href="/security-threat-model">Threat model</a></td>
      <td class="state"><span class="badge state-reliable">Reliable</span></td>
      <td class="audit"><span class="badge audit-wip">In progress</span></td>
    </tr>
    <tr>
      <td class="section-num">6.2</td>
      <td class="section-title"><a href="/security-audit-checklist">Audit checklist</a></td>
      <td class="state"><span class="badge state-stable">Stable</span></td>
      <td class="audit"><span class="badge audit-na">N/A</span></td>
    </tr>
  </tbody>
</table>

## How states are promoted

A section moves between states only via a pull request that:

1. Updates the section's metadata block.
2. Updates this status overview table.
3. Lists the rationale in the PR description.

A section moving from **Draft** → **Reliable** requires sign-off from at
least two protocol maintainers. **Reliable** → **Stable** requires that an
independent audit has covered the section, OR that the section has been
referenced unchanged in two consecutive published implementations.

## Source

The spec markdown lives at
[`prova-network/prova/tree/main/spec`](https://github.com/prova-network/prova/tree/main/spec).
This site renders those files. To propose changes, open a PR against
that directory.
