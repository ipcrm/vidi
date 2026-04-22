# Troubleshooting

Things that go wrong, and their imaginary remedies.

> [!CAUTION]
> Every command, error message, and log line on this page is fabricated.
> Do not copy-paste them into real software — you'll be disappointed.

## Daemon won't start

Check the log:

```
flux::daemon: listener bind failed: Address already in use (os error 48)
```

Something else is on port 7777. Free it, or reconfigure under
[`daemon.listen`](./configuration.md#daemonlisten).

## Ingest returns 415

You sent XML. Flux doesn't accept XML. Nothing does anymore. Use JSON.

## Schema validation keeps failing

Common causes, in decreasing frequency:

1. Trailing comma in a JSON body.
2. An `int` field sent as a string.
3. The field exists under a different name upstream.
4. Mercury is in retrograde.

See [`ingest.accept`](./configuration.md#ingestaccept) for the allowed
content types.

## Hot reload does nothing

You changed a `[daemon]` key. Those require a full restart. See the
[configuration notes on hot reload](./configuration.md#hot-reload).

## Memory keeps climbing

Flux is leaking memory. Unfortunately, Flux is fictional, so there is no
heap-dump tool to use and no debugging symbols to inspect. Shrug, and move
on.

## Error taxonomy

> [!WARNING]
> This table is example content. None of these error codes mean anything.

| Code | Category | Retriable? |
|---:|---|:---:|
| `E100` | Config | no |
| `E200` | Ingest | yes |
| `E300` | Validation | no |
| `E400` | Transform | sometimes |
| `E500` | Persist | yes |
| `E999` | Cosmic | maybe |

## Getting more help

- Join the imaginary Slack.
- File an issue in a repository that does not exist.
- Re-read [getting-started](./getting-started.md) and accept the situation.
