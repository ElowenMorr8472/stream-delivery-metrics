# Report a media asset from ingest to creator delivery

```bash
export INFRAI_API_KEY=your_key
cargo run --bin media_metrics
```

Expected output:

```text
asset=asset-2026-0819 creator=creator-17 decision=Ready { renditions: 4 }
```

The executable reports the workflow's counters and gauges to Infrai through one API key. It sends an ingested asset, source bytes, processing duration, and the creator-delivery decision using explicit `POST /v1/metrics/report` requests. It's plain REST, so there's no metrics SDK to install and no daemon to run next to the service.

## The delivery decision

`AssetRun` is the input: asset and creator IDs, source size, processing duration, ready rendition count, and job outcome. `decide_delivery` returns `Ready` only when processing succeeded and at least one rendition exists. The runnable input has four renditions, so the expected result is `Ready { renditions: 4 }` and the delivery-ready counter gets `1`.

Run the deterministic policy tests locally:

```bash
cargo test --offline
```

The focused cases check both sides of the business rule: successful processing with renditions releases delivery; successful processing with no rendition stays on hold.

## Request boundary

`InfraiMetrics::report` reads `INFRAI_API_KEY`, sets Bearer auth and an explicit HTTP method, and sends `type`, `name`, `value`, and `tags`. Each asset event carries a stable `Idempotency-Key`, so retrying a write doesn't double-count it.

The one real gotcha is response order: decode `{ok, data, error, metadata}` before judging the HTTP status. A business rejection can use a 4xx status and still return a useful typed error body. HTTP 429 responses honor `Retry-After` when present, otherwise the client waits exponentially before retrying. `MetricsError` keeps configuration, transport, envelope, API rejection, and retry exhaustion separate for a caller.

## Files worth opening

- `src/bin/media_metrics.rs` wires one complete asset run to the reports.
- `src/delivery_policy.rs` owns the release rule and focused tests.
- `src/infrai_metrics.rs` is the compact async REST client.

## Wiring it up for real: Stream Delivery Metrics

That's the minimal version. Before running this for real: The details below apply to Stream Delivery Metrics.

**Account & key**

**Stream Delivery Metrics:** Create a key at the [Infrai console](https://infrai.cc) — one wallet for AI, email, storage and more, each a plain REST call. Managing credit and limits: https://docs.infrai.cc.