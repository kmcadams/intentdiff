# intentdiff

> Diff your environments by intent, not text.

`intentdiff` compares infrastructure and application environments based on **inferred semantic intent**, rather than raw configuration differences.

Instead of showing line-by-line YAML changes, it answers:

- Did authentication behavior change?
- Is TLS enforcement different?
- Did persistence semantics shift?
- Is this environment materially less secure?


## Motivation

Modern infrastructure is declarative — but behavior is emergent.

A deployment today may span:
- Kubernetes manifests
- Helm charts
- Environment overlays
- Secrets
- Feature flags
- Defaulted runtime behavior

Traditional diff tools compare text.

But operators care about behavior:
- Did the security boundary change?
- Did persistence guarantees weaken?
- Did exposure increase?
- Did authentication requirements shift?

Two environments may differ in hundreds of lines — yet be functionally equivalent.
Or differ in one line — and materially alter system guarantees.

`intentdiff` operates at the semantic layer.

It attempts to answer:

>What changed in system behavior?

>What guarantees does this environment provide now that it did not before?

## What intentdiff Does

`intentdiff` extracts structured IntentSignals from configuration snapshots and compares inferred behavioral intent across environments.

It highlights:
- Security posture differences
- Transport enforcement changes
- Authentication model shifts
- Persistence semantics changes
- Network exposure modifications
- Runtime behavior deviations

Only meaningful semantic differences are surfaced.


## Example
### Usage

```bash
intentdiff dev.yaml prod.yaml --profile k8s-web --fail-on critical --format markdown
```

**Output**
```bash
⚠️ Intent mismatch detected

CRITICAL:
- TLS enforcement differs (dev allows HTTP)
- Authentication mode differs (OIDC disabled in dev)

WARNING:
- Persistence semantics differ (prod uses PVC, dev uses emptyDir)
```

With exit codes designed for CI Enforcement

## MVP Scope

- CLI tool
- Snapshot-to-snapshot comparison
- Rule-based semantic inference engine
- Emits structured `IntentSignal`s
- Human-readable Markdown output
- Designed for CI usage

Not included (yet)
- Live Cluster interrogation
- Agent-based discovery
- SaaS components
- Policy packs



## Architecture
```
Snapshots
   ↓
Parser
   ↓
Semantic Engine
   ↓
IntentSignals
   ↓
Diff Engine
   ↓
Report
```

The semantic layer is intentionally isolated to allow:
- Pluggable rule sets
- Domain-specific profiles (e.g., k8s-web, api-gateway, auth-service)
- Future policy packs

## Design Principles
### Semantics Over Syntax

Infrastructure correctness is a behavioral contract problem — not a formatting problem.

### Signals, Not Noise

Large configuration sets produce noise. intentdiff extracts high-signal intent differences.

### Explicit Severity

Differences are classified by strength and impact to support automated enforcement.

### CI-First

Deterministic output, meaningful exit codes, and pipeline integration are first-class concerns.

## Installation
Binary releases and `cargo install` support will be provided once the first tagged release is published.

## Philosophy
Infrastructure differences should be evaluated by impact, not indentation.

## License
MIT License
Copyright (c) 2026 Kyle A. McAdams