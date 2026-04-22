# Configuration

Flux is configured through a single TOML file. All values are invented.

## Shape

```toml
# flux.toml
[daemon]
listen  = "0.0.0.0:7777"
workers = 8
level   = "info"

[ingest]
accept   = ["application/json", "application/cbor"]
max_body = "16 MiB"

[database]
url        = "postgres://flux@localhost/flux"
pool_size  = 16
statement_timeout_ms = 5_000

[metrics]
prometheus = true
otlp       = "http://collector.local:4317"
```

## Keys in detail

### `daemon.listen`

Address and port the daemon binds to. Defaults to `127.0.0.1:7777`. Set to
`0.0.0.0:7777` to listen on every interface; do not do this if your demo
pipeline isn't fictional (this one is).

### `daemon.workers`

Number of worker threads. Tune to match available cores. Going above
`num_cpus()` usually hurts more than it helps.

### `ingest.accept`

Comma-separated MIME types the ingest endpoint will accept. Any other
content type is rejected with a `415 Unsupported Media Type`.

### `database.url`

Standard [libpq connection URI][libpq]. All the usual parameters apply:

- `sslmode=require` to enforce TLS
- `connect_timeout=5` to fail fast on unreachable hosts
- `application_name=flux-{node}` to make `pg_stat_activity` readable

[libpq]: https://www.postgresql.org/docs/current/libpq-connect.html

### `database.pool_size`

Connection-pool size *per daemon worker*. So `workers × pool_size` is the
upper bound on open Postgres connections. Tune accordingly if your database
has a `max_connections` limit — which it does, and you have miscounted.

## Hot reload

Flux watches its config file and reloads on SIGHUP[^hup]:

```bash
kill -HUP $(pgrep flux)
```

Only a subset of keys hot-reload — everything under `[daemon]` requires a
full restart. The docs for which keys reload are, like the rest of Flux,
imaginary.

[^hup]: SIGHUP is the traditional Unix reload signal; it dates back to a
  world where terminals literally hung up. Some daemons still respect this
  convention despite the telephone metaphor being several decades stale.

## See also

- [Troubleshooting](./troubleshooting.md) for reload failure modes.
- [API reference](../reference/api.md) for the config endpoint.
- [Getting started](./getting-started.md) if you landed here first.
