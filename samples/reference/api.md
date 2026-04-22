# API reference

Flux exposes a small HTTP surface. None of it works; all of it formats nicely.

## Ingest

### `POST /v1/records`

Accepts a single record or an array of records.

**Headers**

- `Content-Type: application/json` or `application/cbor`
- `Authorization: Bearer <token>` (required)
- `X-Flux-Batch-Id` (optional) — sets the batch identifier for downstream
  correlation. Generated automatically if omitted.

**Body** (single)

```json
{
  "id":        "abc-123",
  "captured":  "2025-08-14T12:34:56Z",
  "source":    "sensor-17",
  "payload":   { "temp": 21.4, "humidity": 0.58 }
}
```

**Response**

- `202 Accepted` — record queued. Contains a `Location` header pointing at
  the status endpoint for the batch.
- `400 Bad Request` — schema validation failed. Body is a JSON array of
  issues (see [schema issues](#schema-issues)).
- `415 Unsupported Media Type` — see [configuration `ingest.accept`](../guides/configuration.md#ingestaccept).

### `GET /v1/records/{id}`

Returns the stored record. Not available until Flux has persisted it, which
happens at most `statement_timeout_ms` after ingest.

## Status & health

### `GET /healthz`

Liveness probe. Always returns `200 OK` with body `ok` unless the daemon is
in the middle of a crash, in which case it returns nothing at all and the
question becomes philosophical.

### `GET /readyz`

Readiness probe. `200` when the database is reachable and the in-memory
queue has < 80% headroom; `503` otherwise.

### `GET /metrics`

Prometheus text format. Sample:

```
# HELP flux_records_ingested_total Records ingested by status
# TYPE flux_records_ingested_total counter
flux_records_ingested_total{status="accepted"} 1827423
flux_records_ingested_total{status="rejected"} 121

# HELP flux_pipeline_latency_seconds End-to-end latency
# TYPE flux_pipeline_latency_seconds histogram
flux_pipeline_latency_seconds_bucket{le="0.005"} 13
flux_pipeline_latency_seconds_bucket{le="0.01"}  98
flux_pipeline_latency_seconds_bucket{le="0.025"} 1204
flux_pipeline_latency_seconds_bucket{le="0.05"}  9987
flux_pipeline_latency_seconds_bucket{le="+Inf"}  10040
flux_pipeline_latency_seconds_sum 182.4
flux_pipeline_latency_seconds_count 10040
```

## Schema issues

A validation error payload looks like this:

```json
[
  { "pointer": "/captured",       "code": "E301", "message": "not an RFC3339 date-time" },
  { "pointer": "/payload/temp",   "code": "E302", "message": "expected number, got string" }
]
```

See the [error taxonomy](../guides/troubleshooting.md#error-taxonomy) for code
meanings. (There are no code meanings. Nothing is real.)

## Example in Rust

```rust
use reqwest::blocking::Client;
use serde_json::json;

fn main() -> anyhow::Result<()> {
    let client = Client::new();
    let body = json!({
        "id":       "abc-123",
        "captured": "2025-08-14T12:34:56Z",
        "source":   "sensor-17",
        "payload":  { "temp": 21.4, "humidity": 0.58 }
    });

    let res = client
        .post("https://flux.example/v1/records")
        .bearer_auth(std::env::var("FLUX_TOKEN")?)
        .json(&body)
        .send()?
        .error_for_status()?;

    println!("batch: {}", res.headers()["x-flux-batch-id"].to_str()?);
    Ok(())
}
```

## Example in TypeScript

```ts
const res = await fetch("https://flux.example/v1/records", {
  method: "POST",
  headers: {
    "content-type": "application/json",
    authorization: `Bearer ${process.env.FLUX_TOKEN}`,
  },
  body: JSON.stringify({
    id: "abc-123",
    captured: new Date().toISOString(),
    source: "sensor-17",
    payload: { temp: 21.4, humidity: 0.58 },
  }),
});

if (!res.ok) {
  throw new Error(`ingest failed: ${res.status}`);
}
```

## Python

```python
import httpx, os

resp = httpx.post(
    "https://flux.example/v1/records",
    headers={"authorization": f"Bearer {os.environ['FLUX_TOKEN']}"},
    json={
        "id": "abc-123",
        "captured": "2025-08-14T12:34:56Z",
        "source": "sensor-17",
        "payload": {"temp": 21.4, "humidity": 0.58},
    },
)
resp.raise_for_status()
print(resp.headers["x-flux-batch-id"])
```
