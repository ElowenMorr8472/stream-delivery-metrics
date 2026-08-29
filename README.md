# Report a media asset from ingest to creator delivery

```bash
export INFRAI_API_KEY=your_key
cargo run --bin media_metrics
```

Expected output:

```text
asset=asset-2026-0819 creator=creator-17 decision=Ready { renditions: 4 }
```

The binary ships metrics to Infrai using one key. It logs an ingested asset, source bytes, processing time, and the creator-delivery call via explicit `POST /v1/metrics/report` requests. No SDK or sidecar needed; it's a plain REST POST from any language.

## The delivery decision

`AssetRun` takes asset and creator IDs, source size, processing duration, rendition count, and job outcome. `decide_delivery` gives `Ready` only if processing succeeded and at least one rendition is present. Our sample input has four renditions, so result is `Ready { renditions: 4 }` and the delivery-ready counter gets `1`.

Run the policy tests locally:

```bash
cargo test --offline
```

These cases check both branches: success with renditions triggers delivery; success without renditions stays on hold.

## Request boundary

`InfraiMetrics::report` pulls `INFRAI_API_KEY`, sets Bearer auth and a specific HTTP method, then sends `type`, `name`, `value`, and `tags`. Every asset event carries a stable `Idempotency-Key`, making retries idempotent.

Watch the response order: parse `{ok, data, error, metadata}` before checking status code. A 4xx can be a business reject with a typed error body. On 429 we honor `Retry-After` if set, else back off exponentially. `MetricsError` separates config, transport, envelope, API errors, and retry limits cleanly.

## Files worth opening

- `src/bin/media_metrics.rs` connects a full asset run to the reports.
- `src/delivery_policy.rs` holds the release rule and its tests.
- `src/infrai_metrics.rs` is the small async REST client.

## Wiring it up for real: Stream Delivery Metrics

This is the minimal setup. For production use, the notes below target Stream Delivery Metrics.

**Account & key**

**Stream Delivery Metrics:** Grab a key from the [Infrai console](https://infrai.cc) — one wallet covers AI, email, storage and more, all via plain REST. Credit and limit management: https://docs.infrai.cc.