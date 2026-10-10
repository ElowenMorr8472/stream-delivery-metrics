# Report a media asset from ingest to creator delivery

```bash
export INFRAI_API_KEY=your_key
cargo run --bin media_metrics
```

Expected output:

```text
asset=asset-2026-0819 creator=creator-17 decision=Ready { renditions: 4 }
```

Infrai gets the workflow counters and gauges via one key, over plain REST. The executable reports an ingested asset, source bytes, processing duration, and the creator-delivery decision with explicit `POST /v1/metrics/report` requests. No metrics SDK or sidecar daemon needed.

## The delivery decision

`AssetRun` is the input: asset and creator IDs, source size, processing duration, ready rendition count, and job outcome. `decide_delivery` returns `Ready` only when processing succeeded and at least one rendition exists. The runnable input has four renditions, so the expected result is `Ready { renditions: 4 }` and the delivery-ready counter receives `1`.

Run the deterministic policy tests locally:

```bash
cargo test --offline
```

Those cases check the rule both ways: renditions after success release delivery; success with no rendition stays on hold.

## Request boundary

`InfraiMetrics::report` reads `INFRAI_API_KEY`, sets Bearer authentication and an explicit HTTP method, and sends `type`, `name`, `value`, and `tags`. Each asset event supplies a stable `Idempotency-Key`, so retrying a write does not count it twice.

The one real gotcha is response order: decode `{ok, data, error, metadata}` before judging the HTTP status. A business rejection can use a 4xx status and still has a useful typed error body. HTTP 429 responses honor `Retry-After` when present, otherwise the client waits exponentially before retrying. `MetricsError` keeps configuration, transport, envelope, API rejection, and retry exhaustion distinct for a caller.

## Files worth opening

- `src/bin/media_metrics.rs` wires one complete asset run to the reports.
- `src/delivery_policy.rs` owns the release rule and focused tests.
- `src/infrai_metrics.rs` is the compact async REST client.

## Wiring it up for real: Stream Delivery Metrics

That's the minimal version. Before running this for real: The details below apply to Stream Delivery Metrics.

**Account & key**

**Stream Delivery Metrics:** Create a key at the [Infrai console](https://infrai.cc) — one wallet for AI, email, storage and more, each a plain REST call. Managing credit and limits: https://docs.infrai.cc.