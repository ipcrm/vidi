# Flux

<p align="center">
  <img src="./images/logo.svg" alt="Flux logo" width="140">
</p>

> **Flux** /flʌks/ · noun. A continuous flow or change; an unstable movement.

Flux is a **fictional** data pipeline used as demo content for Visum. It has no
real code behind it — the point is to exercise every rendering feature of
Visum: GFM tables, task lists, alerts, math, diagrams, code highlighting,
footnotes, relative image paths, internal links, and external links.

## Table of contents

- [Overview](#overview)
- [Getting started](./guides/getting-started.md)
- [Configuration](./guides/configuration.md)
- [Troubleshooting](./guides/troubleshooting.md)
- [API reference](./reference/api.md)
- [Architecture](./architecture.md)
- [Changelog](./changelog.md)
- [Random thoughts](./notes/random-thoughts.md)

## Overview

Flux ingests records from anywhere HTTP, validates them against a schema,
enriches them with static lookups, and hands them off to Postgres. That's it.
You do not need to install Flux. You cannot install Flux. Flux is not real.

![Ingest pipeline](./images/pipeline.svg)

## Status

:sparkles: Actively **not** developed. :warning: Pull requests will be closed
without review because this is demo content inside another project's
`samples/` directory.

## Why this exists

To help Visum prove that it can:

- [x] Resolve relative `.md` links across folders
- [x] Render local SVGs through the `asset://` protocol
- [x] Prompt before loading external images
- [x] Render code, tables, math, alerts, and mermaid
- [x] Honor heading anchors and build a table of contents
- [ ] Do your laundry

See [getting-started](./guides/getting-started.md) for the walk-through.
