# Architecture

Flux is designed around a three-stage pipeline. Records enter at the left,
exit at the right, and nobody has ever seen this system in real life.

![Ingest pipeline](./images/pipeline.svg)

## High-level flow

```mermaid
flowchart LR
  A[Client] -- HTTP POST --> B(Ingest)
  B -- bytes --> C{Schema OK?}
  C -- yes --> D[Transform]
  C -- no  --> E[(Error log)]
  D -- row --> F[(Postgres)]
  D -- metric --> G[(Prometheus)]
  F --> H[Warehouse mirror]
```

## Sequence — a successful ingest

```mermaid
sequenceDiagram
  autonumber
  participant C as Client
  participant I as Ingest
  participant V as Validator
  participant T as Transform
  participant DB as Postgres

  C->>I: POST /v1/records (JSON)
  I->>V: bytes
  V-->>I: ok (typed record)
  I->>T: record
  T->>T: enrich with lookup tables
  T->>DB: INSERT
  DB-->>T: ok
  T-->>I: 202 Accepted + batchId
  I-->>C: 202 Accepted
```

## Capacity math

For $n$ worker threads with pool size $p$ and average statement latency
$\bar{t}$ seconds, the sustainable throughput ceiling is approximately

$$
Q = \frac{n \cdot p}{\bar{t}}
$$

For $n = 8$, $p = 16$, $\bar{t} = 0.012$ s, that's

$$
Q \approx \frac{8 \cdot 16}{0.012} \approx 10{,}666 \text{ rec/s}
$$

i.e. the classic "plenty, until something isn't". Prod reality is always
bounded by something else (the network, the TLS handshake, the database's
checkpoint behavior, Mercury).

## Failure domains

```mermaid
stateDiagram-v2
  [*] --> Healthy
  Healthy --> Degraded: pool > 80% utilized
  Degraded --> Healthy: pool < 40% for 5m
  Degraded --> Shedding: error rate > 5%
  Shedding --> Degraded: error rate < 1% for 10m
  Shedding --> [*]: operator stops daemon
```

## Trust boundaries

Everything outside the daemon process is untrusted. That includes:

- the HTTP clients
- the database (bad migrations happen)
- the metrics collector (malicious labels → cardinality explosion)
- the humans — especially the humans

Inside the daemon, the only thing we trust is the config file, and even then
we schema-validate it on startup.

## See also

- [Configuration](./guides/configuration.md)
- [API](./reference/api.md)
- [Changelog](./changelog.md)
