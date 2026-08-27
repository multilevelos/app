use crate::infrastructure::InfrastructureType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelOfServiceScore {
    NonCompliant = -1,
    UnsuitableMixed = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}

impl LevelOfServiceScore {
    pub fn label(&self) -> &'static str {
        match self {
            LevelOfServiceScore::High => "High",
            LevelOfServiceScore::Medium => "Medium",
            LevelOfServiceScore::Low => "Low",
            LevelOfServiceScore::UnsuitableMixed => "Should not be used (mixed traffic)",
            LevelOfServiceScore::NonCompliant => "Should not be used (non-compliant intervention)",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            LevelOfServiceScore::High => "#16a34a",            // Green
            LevelOfServiceScore::Medium => "#eab308",          // Yellow
            LevelOfServiceScore::Low => "#f97316",             // Orange
            LevelOfServiceScore::UnsuitableMixed => "#dc2626", // Red
            LevelOfServiceScore::NonCompliant => "#9333ea",    // Purple
        }
    }

    pub fn raw_value(&self) -> i8 {
        *self as i8
    }
}

/// Lookup Level of Service (LoS) score from Scottish Cycling by Design matrix
pub fn lookup_level_of_service(
    speed_cat: &str,
    aadt_cat: &str,
    infra: InfrastructureType,
) -> LevelOfServiceScore {
    // Dedicated off-road paths are always High LoS (Score 3)
    if infra == InfrastructureType::OffRoadPath {
        return LevelOfServiceScore::High;
    }

    // Shared footways default to Medium LoS (Score 2)
    if infra == InfrastructureType::SharedFootway {
        return LevelOfServiceScore::Medium;
    }

    let base_score: i8 = match (speed_cat, aadt_cat, infra) {
        // <20 mph & 20 mph
        ("<20 mph" | "20 mph", "0 to 999", _) => 3,
        ("<20 mph" | "20 mph", "1000 to 1999", _) => 3,
        ("<20 mph" | "20 mph", "2000 to 3999", InfrastructureType::MixedTrafficStreet) => 2,
        ("<20 mph" | "20 mph", "2000 to 3999", _) => 3,
        ("<20 mph" | "20 mph", "4000+", InfrastructureType::MixedTrafficStreet) => 1,
        ("<20 mph" | "20 mph", "4000+", InfrastructureType::PaintedCycleLane) => 2,
        ("<20 mph" | "20 mph", "4000+", _) => 3,

        // 30 mph
        ("30 mph", "0 to 999", _) => 3,
        ("30 mph", "1000 to 1999", InfrastructureType::MixedTrafficStreet) => 2,
        ("30 mph", "1000 to 1999", InfrastructureType::PaintedCycleLane) => 2,
        ("30 mph", "1000 to 1999", _) => 3,
        ("30 mph", "2000 to 3999", InfrastructureType::MixedTrafficStreet) => 1,
        ("30 mph", "2000 to 3999", InfrastructureType::PaintedCycleLane) => 2,
        ("30 mph", "2000 to 3999", _) => 3,
        ("30 mph", "4000+", InfrastructureType::MixedTrafficStreet) => 1,
        ("30 mph", "4000+", InfrastructureType::PaintedCycleLane) => 1,
        ("30 mph", "4000+", InfrastructureType::SegregatedTrackNarrow) => 2,
        ("30 mph", "4000+", InfrastructureType::SegregatedTrackWide) => 3,

        // 40 mph
        ("40 mph", "0 to 999", InfrastructureType::MixedTrafficStreet) => 2,
        ("40 mph", "0 to 999", InfrastructureType::PaintedCycleLane) => 2,
        ("40 mph", "0 to 999", InfrastructureType::SegregatedTrackNarrow) => 2,
        ("40 mph", "0 to 999", InfrastructureType::SegregatedTrackWide) => 2,
        ("40 mph", "1000 to 1999", InfrastructureType::MixedTrafficStreet) => 1,
        ("40 mph", "1000 to 1999", InfrastructureType::PaintedCycleLane) => 1,
        ("40 mph", "1000 to 1999", InfrastructureType::SegregatedTrackNarrow) => 2,
        ("40 mph", "1000 to 1999", InfrastructureType::SegregatedTrackWide) => 2,
        ("40 mph", "2000 to 3999", InfrastructureType::MixedTrafficStreet) => 0,
        ("40 mph", "2000 to 3999", InfrastructureType::PaintedCycleLane) => 1,
        ("40 mph", "2000 to 3999", InfrastructureType::SegregatedTrackNarrow) => 1,
        ("40 mph", "2000 to 3999", InfrastructureType::SegregatedTrackWide) => 2,
        ("40 mph", "4000+", InfrastructureType::MixedTrafficStreet) => 0,
        ("40 mph", "4000+", InfrastructureType::PaintedCycleLane) => 1,
        ("40 mph", "4000+", InfrastructureType::SegregatedTrackNarrow) => 1,
        ("40 mph", "4000+", InfrastructureType::SegregatedTrackWide) => 2,

        // 50 mph
        ("50 mph", "0 to 999", InfrastructureType::MixedTrafficStreet) => 1,
        ("50 mph", "0 to 999", InfrastructureType::PaintedCycleLane) => 1,
        ("50 mph", "0 to 999", InfrastructureType::SegregatedTrackNarrow) => 2,
        ("50 mph", "0 to 999", InfrastructureType::SegregatedTrackWide) => 2,
        ("50 mph", "1000 to 1999", InfrastructureType::MixedTrafficStreet) => 0,
        ("50 mph", "1000 to 1999", InfrastructureType::PaintedCycleLane) => 1,
        ("50 mph", "1000 to 1999", InfrastructureType::SegregatedTrackNarrow) => 1,
        ("50 mph", "1000 to 1999", InfrastructureType::SegregatedTrackWide) => 1,
        ("50 mph", "2000 to 3999", InfrastructureType::MixedTrafficStreet) => 0,
        ("50 mph", "2000 to 3999", InfrastructureType::PaintedCycleLane) => 1,
        ("50 mph", "2000 to 3999", InfrastructureType::SegregatedTrackNarrow) => 1,
        ("50 mph", "2000 to 3999", InfrastructureType::SegregatedTrackWide) => 1,
        ("50 mph", "4000+", InfrastructureType::MixedTrafficStreet) => 0,
        ("50 mph", "4000+", InfrastructureType::PaintedCycleLane) => 1,
        ("50 mph", "4000+", InfrastructureType::SegregatedTrackNarrow) => 1,
        ("50 mph", "4000+", InfrastructureType::SegregatedTrackWide) => 1,

        // 60+ mph
        ("60+ mph", "0 to 999", InfrastructureType::MixedTrafficStreet) => 0,
        ("60+ mph", "0 to 999", InfrastructureType::PaintedCycleLane) => 0,
        ("60+ mph", "0 to 999", InfrastructureType::SegregatedTrackNarrow) => 0,
        ("60+ mph", "0 to 999", InfrastructureType::SegregatedTrackWide) => 1,
        ("60+ mph", _, InfrastructureType::MixedTrafficStreet) => 0,
        ("60+ mph", _, InfrastructureType::PaintedCycleLane) => 0,
        ("60+ mph", _, InfrastructureType::SegregatedTrackNarrow) => 0,
        ("60+ mph", _, InfrastructureType::SegregatedTrackWide) => 1,

        _ => 2,
    };

    // Differentiate between 0 (Unsuitable mixed traffic) and -1 (Substandard non-compliant intervention)
    if base_score == 0 {
        if infra != InfrastructureType::MixedTrafficStreet {
            LevelOfServiceScore::NonCompliant
        } else {
            LevelOfServiceScore::UnsuitableMixed
        }
    } else {
        match base_score {
            3 => LevelOfServiceScore::High,
            2 => LevelOfServiceScore::Medium,
            1 => LevelOfServiceScore::Low,
            _ => LevelOfServiceScore::Medium,
        }
    }
}
