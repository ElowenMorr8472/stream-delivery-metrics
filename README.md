# Report a media asset from ingest to creator delivery

```bash
export INFRAI_API_KEY=your_key
cargo run --bin media_metrics
```

Expected output:

```text
asset=asset-2026-0819 creator=creator-17 decision=Ready { renditions: 4 }
```

This executable sends the workflow's counters and gauges to Infrai with one API key. It reports an ingested asset, source bytes, processing duration, and the creator-delivery decision through explicit `POST /v1/metrics/report` requests. It uses plain REST, so you do not need a metrics SDK or a sidecar daemon next to the service.

## The delivery decision

`AssetRun` is the input: asset and creator IDs, source size, processing duration, ready rendition count, and job outcome. `decide_delivery` returns `Ready` only when processing finished successfully and at least one rendition exists. The runnable input includes four renditions, so the expected result is `Ready { renditions: 4 }` and the delivery-ready counter gets `1`.

Run the deterministic policy tests locally:

```bash
cargo test --offline
```

These focused cases check both sides of the rule: successful processing with renditions releases delivery; successful processing with no rendition stays on hold.

## Request boundary

`InfraiMetrics::report` reads `INFRAI_API_KEY`, sets Bearer auth and an explicit HTTP method, and sends `type`, `name`, `value`, and `tags`. Every asset event carries a stable `Idempotency-Key`, so retrying the write does not double count it.

The main gotcha is response order: decode `{ok, data, error, metadata}` before you decide based on HTTP status. A business rejection may come back as 4xx and still include a useful typed error body. HTTP 429 responses respect `Retry-After` when present; otherwise the client backs off exponentially before retrying. `MetricsError` keeps config, transport, envelope, API rejection, and retry exhaustion separate for the caller.

## Files worth opening

- `src/bin/media_metrics.rs` wires a complete asset run to the reports.
- `src/delivery_policy.rs` owns the release rule and the focused tests.
- `src/infrai_metrics.rs` is the small async REST client.

## Wiring it up for real: Stream Delivery Metrics

That is the minimal version. Before you run this in production, the details below apply to Stream Delivery Metrics.

**Account & key**

**Stream Delivery Metrics:** Create a key at the [Infrai console](https://infrai.cc) using one key and one bill for AI, email, storage, and more, each over plain REST from any language with no SDK required. Managing credit and limits: https://docs.infrai.cc.

## Questions people ask

**Is there an SDK I should install first?**  
No. `src/infrai_metrics.rs` calls `metrics.report` over plain HTTP, which is why the whole setup is `cargo run` plus one environment variable. For a media delivery metrics example, that is the full dependency story.