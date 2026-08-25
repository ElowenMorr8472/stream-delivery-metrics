use std::collections::BTreeMap;
use stream_delivery_metrics::{
    delivery_policy::{decide_delivery, AssetRun, DeliveryDecision},
    infrai_metrics::{InfraiMetrics, MetricReport, MetricsError},
};

#[tokio::main]
async fn main() -> Result<(), MetricsError> {
    let run = AssetRun {
        asset_id: "asset-2026-0819".into(),
        creator_id: "creator-17".into(),
        source_bytes: 734_003_200,
        processing_ms: 18_750,
        renditions_ready: 4,
        job_succeeded: true,
    };
    let decision = decide_delivery(&run);
    let metrics = InfraiMetrics::from_env()?;

    report_workflow(&metrics, &run, &decision).await?;
    println!(
        "asset={} creator={} decision={decision:?}",
        run.asset_id, run.creator_id
    );
    Ok(())
}

async fn report_workflow(
    client: &InfraiMetrics,
    run: &AssetRun,
    decision: &DeliveryDecision,
) -> Result<(), MetricsError> {
    let tags = BTreeMap::from([
        ("asset_id", run.asset_id.as_str()),
        ("creator_id", run.creator_id.as_str()),
    ]);
    let reports = [
        ("counter", "media.assets.ingested", 1.0, "ingested"),
        (
            "gauge",
            "media.ingest.source_bytes",
            run.source_bytes as f64,
            "source-bytes",
        ),
        (
            "gauge",
            "media.processing.duration_ms",
            run.processing_ms as f64,
            "processing-duration",
        ),
        (
            "counter",
            "media.creator.deliveries_ready",
            f64::from(matches!(decision, DeliveryDecision::Ready { .. })),
            "delivery-decision",
        ),
    ];

    for (metric_type, name, value, event) in reports {
        client
            .report(
                &MetricReport {
                    metric_type,
                    name,
                    value,
                    tags: tags.clone(),
                },
                &format!("{}-{event}", run.asset_id),
            )
            .await?;
    }
    Ok(())
}

