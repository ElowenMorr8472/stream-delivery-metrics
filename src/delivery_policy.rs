#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRun {
    pub asset_id: String,
    pub creator_id: String,
    pub source_bytes: u64,
    pub processing_ms: u64,
    pub renditions_ready: u32,
    pub job_succeeded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryDecision {
    Ready { renditions: u32 },
    Hold { reason: &'static str },
}

pub fn decide_delivery(run: &AssetRun) -> DeliveryDecision {
    if !run.job_succeeded {
        return DeliveryDecision::Hold {
            reason: "processing job did not succeed",
        };
    }
    if run.renditions_ready == 0 {
        return DeliveryDecision::Hold {
            reason: "no rendition is ready",
        };
    }
    DeliveryDecision::Ready {
        renditions: run.renditions_ready,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_processing_with_renditions_releases_creator_delivery() {
        let run = AssetRun {
            asset_id: "asset-042".into(),
            creator_id: "creator-7".into(),
            source_bytes: 48_000_000,
            processing_ms: 8_400,
            renditions_ready: 3,
            job_succeeded: true,
        };

        assert_eq!(
            decide_delivery(&run),
            DeliveryDecision::Ready { renditions: 3 }
        );
    }

    #[test]
    fn empty_processing_output_stays_on_hold() {
        let run = AssetRun {
            asset_id: "asset-043".into(),
            creator_id: "creator-7".into(),
            source_bytes: 20_000_000,
            processing_ms: 2_100,
            renditions_ready: 0,
            job_succeeded: true,
        };

        assert_eq!(
            decide_delivery(&run),
            DeliveryDecision::Hold {
                reason: "no rendition is ready"
            }
        );
    }
}

