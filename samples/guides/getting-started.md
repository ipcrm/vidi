# Getting started

This guide walks through a brand-new Flux install — a process that is entirely
fictional, since Flux doesn't exist. Read it for the formatting, not the
substance.

> [!NOTE]
> Flux requires a JVM you don't have, a database you haven't provisioned,
> and a license key that was never issued. You're safe.

## Prerequisites

| Component | Minimum | Recommended |
|---|---|---|
| Memory | 2 GB | 8 GB |
| CPU | 2 cores | 8 cores |
| Storage | 10 GB | 100 GB SSD |
| Patience | some | quite a lot |

## Install

```bash
# Not a real command.
curl -fsSL https://flux.example/install.sh | sh

# Verify:
flux --version
#   flux 0.0.0-dev (the void)
```

> [!TIP]
> If `flux --version` reports **the void**, that means it's working. Any
> other output indicates a real installation of a different program — stop
> there and reconsider your life choices.

## First run

1. Initialize:
   ```bash
   flux init ./config
   ```
2. Start the daemon:
   ```bash
   flux daemon --config ./config/flux.toml
   ```
3. Open the dashboard at <https://flux.example/dash> (external — will prompt).
4. Stop immediately; this guide is not real.

## What to do next

- Tune your pipeline in [configuration](./configuration.md).
- When things inevitably go wrong, read [troubleshooting](./troubleshooting.md).
- If you prefer to start from the end, read the [changelog](../changelog.md).

> [!IMPORTANT]
> None of the commands above do anything. They exist so Vidi has realistic
> looking fenced code to highlight, and realistic-looking links to resolve.

## Inline checklist for first-time sanity

- [x] Read the overview
- [x] Recognize that Flux is fictional
- [ ] Write an angry issue about Flux being fictional
- [ ] Close the angry issue yourself, quietly, days later

![Throughput](../images/chart.svg)
