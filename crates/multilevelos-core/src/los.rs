use crate::config::DesignStandard;
use crate::infrastructure::{InfrastructureType, SurfaceQuality};
use crate::traffic::classify_aadt_ltn120;
use serde::{Deserialize, Serialize};

/// Unified 5-tier Multi-Modal Level of Service (MMLoS) Grade (A–E)
/// Calibrated so Grade A represents equivalent all-ages-and-abilities quality across Walking, Wheeling & Cycling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MmlosGrade {
    E = 0,
    D = 1,
    C = 2,
    B = 3,
    A = 4,
}

impl MmlosGrade {
    pub fn letter(&self) -> &'static str {
        match self {
            MmlosGrade::A => "A",
            MmlosGrade::B => "B",
            MmlosGrade::C => "C",
            MmlosGrade::D => "D",
            MmlosGrade::E => "E",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MmlosGrade::A => "Grade A (Excellent)",
            MmlosGrade::B => "Grade B (Good)",
            MmlosGrade::C => "Grade C (Moderate)",
            MmlosGrade::D => "Grade D (Poor)",
            MmlosGrade::E => "Grade E (Hostile / Barrier)",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            MmlosGrade::A => "#059669", // Emerald
            MmlosGrade::B => "#65a30d", // Lime Green
            MmlosGrade::C => "#eab308", // Amber Yellow
            MmlosGrade::D => "#f97316", // Orange
            MmlosGrade::E => "#dc2626", // Crimson
        }
    }

    pub fn numeric_score(&self) -> i8 {
        match self {
            MmlosGrade::A => 4,
            MmlosGrade::B => 3,
            MmlosGrade::C => 2,
            MmlosGrade::D => 1,
            MmlosGrade::E => 0,
        }
    }

    pub fn from_numeric(v: i8) -> Self {
        match v {
            v if v >= 4 => MmlosGrade::A,
            3 => MmlosGrade::B,
            2 => MmlosGrade::C,
            1 => MmlosGrade::D,
            _ => MmlosGrade::E,
        }
    }
}

/// Scottish Cycling by Design (CbD) native 5-tier score (preserved for backwards compatibility & Scotland mode)
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
            LevelOfServiceScore::High => "#16a34a",
            LevelOfServiceScore::Medium => "#eab308",
            LevelOfServiceScore::Low => "#f97316",
            LevelOfServiceScore::UnsuitableMixed => "#dc2626",
            LevelOfServiceScore::NonCompliant => "#9333ea",
        }
    }

    pub fn raw_value(&self) -> i8 {
        *self as i8
    }

    pub fn to_mmlos_grade(&self, infra: InfrastructureType) -> MmlosGrade {
        match self {
            LevelOfServiceScore::High => {
                if matches!(
                    infra,
                    InfrastructureType::OffRoadPath | InfrastructureType::SegregatedTrackWide
                ) {
                    MmlosGrade::A
                } else {
                    MmlosGrade::B
                }
            }
            LevelOfServiceScore::Medium => MmlosGrade::C,
            LevelOfServiceScore::Low => MmlosGrade::D,
            LevelOfServiceScore::UnsuitableMixed | LevelOfServiceScore::NonCompliant => {
                MmlosGrade::E
            }
        }
    }
}

/// Lookup Level of Service (LoS) score from Scottish Cycling by Design matrix
pub fn lookup_level_of_service(
    speed_cat: &str,
    aadt_cat: &str,
    infra: InfrastructureType,
) -> LevelOfServiceScore {
    if matches!(
        infra,
        InfrastructureType::Steps | InfrastructureType::PedestrianFootway
    ) {
        return LevelOfServiceScore::UnsuitableMixed;
    }

    if infra == InfrastructureType::OffRoadPath || infra == InfrastructureType::BridlewayGreenway {
        return LevelOfServiceScore::High;
    }

    if infra == InfrastructureType::SharedFootway {
        return LevelOfServiceScore::Medium;
    }

    let mapped_infra = match infra {
        InfrastructureType::LightSegregation | InfrastructureType::SteppedCycleTrack => {
            InfrastructureType::SegregatedTrackNarrow
        }
        InfrastructureType::BusLane => InfrastructureType::PaintedCycleLane,
        InfrastructureType::VirtualFootway | InfrastructureType::QuietLane => {
            InfrastructureType::MixedTrafficStreet
        }
        other => other,
    };

    let base_score: i8 = match (speed_cat, aadt_cat, mapped_infra) {
        ("<20 mph" | "20 mph", "0 to 999", _) => 3,
        ("<20 mph" | "20 mph", "1000 to 1999", _) => 3,
        ("<20 mph" | "20 mph", "2000 to 3999", InfrastructureType::MixedTrafficStreet) => 2,
        ("<20 mph" | "20 mph", "2000 to 3999", _) => 3,
        ("<20 mph" | "20 mph", "4000+", InfrastructureType::MixedTrafficStreet) => 1,
        ("<20 mph" | "20 mph", "4000+", InfrastructureType::PaintedCycleLane) => 2,
        ("<20 mph" | "20 mph", "4000+", _) => 3,

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

        ("40 mph", "0 to 999", _) => 2,
        ("40 mph", "1000 to 1999", InfrastructureType::MixedTrafficStreet) => 1,
        ("40 mph", "1000 to 1999", InfrastructureType::PaintedCycleLane) => 1,
        ("40 mph", "1000 to 1999", _) => 2,
        ("40 mph", "2000 to 3999", InfrastructureType::MixedTrafficStreet) => 0,
        ("40 mph", "2000 to 3999", InfrastructureType::PaintedCycleLane) => 1,
        ("40 mph", "2000 to 3999", InfrastructureType::SegregatedTrackNarrow) => 1,
        ("40 mph", "2000 to 3999", InfrastructureType::SegregatedTrackWide) => 2,
        ("40 mph", "4000+", InfrastructureType::MixedTrafficStreet) => 0,
        ("40 mph", "4000+", InfrastructureType::PaintedCycleLane) => 1,
        ("40 mph", "4000+", InfrastructureType::SegregatedTrackNarrow) => 1,
        ("40 mph", "4000+", InfrastructureType::SegregatedTrackWide) => 2,

        ("50 mph", "0 to 999", InfrastructureType::MixedTrafficStreet) => 1,
        ("50 mph", "0 to 999", InfrastructureType::PaintedCycleLane) => 1,
        ("50 mph", "0 to 999", _) => 2,
        ("50 mph", _, InfrastructureType::MixedTrafficStreet) => 0,
        ("50 mph", _, _) => 1,

        ("60+ mph", _, InfrastructureType::SegregatedTrackWide) => 1,
        ("60+ mph", _, _) => 0,

        _ => 2,
    };

    if base_score == 0 {
        if mapped_infra != InfrastructureType::MixedTrafficStreet {
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

/// Score Cycling Level of Service (A–E) under England DfT LTN 1/20 (`atf-lab/osmactive/data/los_england.csv`) or Scotland CbD
pub fn score_cycling_grade(
    standard: DesignStandard,
    speed_mph: f64,
    aadt: Option<u32>,
    infra: InfrastructureType,
    surface: SurfaceQuality,
    cycling_allowed: bool,
) -> MmlosGrade {
    if !cycling_allowed
        || matches!(
            infra,
            InfrastructureType::Steps | InfrastructureType::PedestrianFootway
        )
    {
        return MmlosGrade::E;
    }

    let base = if standard == DesignStandard::Cbd {
        let speed_cat = crate::speed::classify_speed_category(speed_mph);
        let aadt_cat = crate::traffic::classify_aadt_category(aadt);
        lookup_level_of_service(speed_cat, aadt_cat, infra).to_mmlos_grade(infra)
    } else {
        // England DfT LTN 1/20 Fig 4.1 (`atf-lab/osmactive/data/los_england.csv`)
        let aadt_band = classify_aadt_ltn120(aadt);
        match infra {
            InfrastructureType::OffRoadPath | InfrastructureType::BridlewayGreenway => MmlosGrade::A,
            InfrastructureType::SegregatedTrackWide => {
                if speed_mph >= 50.0 {
                    MmlosGrade::B
                } else {
                    MmlosGrade::A
                }
            }
            InfrastructureType::SegregatedTrackNarrow => {
                if speed_mph >= 50.0 {
                    MmlosGrade::C
                } else {
                    MmlosGrade::B
                }
            }
            InfrastructureType::SteppedCycleTrack | InfrastructureType::LightSegregation => {
                if speed_mph <= 20.0 {
                    MmlosGrade::A
                } else if speed_mph <= 30.0 {
                    MmlosGrade::B
                } else if speed_mph <= 40.0 {
                    MmlosGrade::C
                } else {
                    MmlosGrade::E
                }
            }
            InfrastructureType::SharedFootway => MmlosGrade::C,
            InfrastructureType::BusLane => {
                if speed_mph <= 20.0 {
                    MmlosGrade::B
                } else if speed_mph <= 30.0 {
                    MmlosGrade::C
                } else {
                    MmlosGrade::E
                }
            }
            InfrastructureType::PaintedCycleLane => {
                if speed_mph <= 20.0 {
                    if aadt_band == "6000+" {
                        MmlosGrade::C
                    } else {
                        MmlosGrade::B
                    }
                } else if speed_mph <= 30.0 {
                    match aadt_band {
                        "0 to 1999" | "2000 to 3999" => MmlosGrade::C,
                        "4000 to 5999" => MmlosGrade::D,
                        _ => MmlosGrade::E,
                    }
                } else {
                    MmlosGrade::E
                }
            }
            InfrastructureType::QuietLane
            | InfrastructureType::VirtualFootway
            | InfrastructureType::MixedTrafficStreet => {
                if speed_mph <= 20.0 {
                    match aadt_band {
                        "0 to 1999" => MmlosGrade::B,
                        "2000 to 3999" => MmlosGrade::C,
                        "4000 to 5999" => MmlosGrade::D,
                        _ => MmlosGrade::E,
                    }
                } else if speed_mph <= 30.0 {
                    match aadt_band {
                        "0 to 1999" => MmlosGrade::C,
                        "2000 to 3999" => MmlosGrade::D,
                        _ => MmlosGrade::E,
                    }
                } else {
                    MmlosGrade::E
                }
            }
            InfrastructureType::Steps | InfrastructureType::PedestrianFootway => MmlosGrade::E,
        }
    };

    // Surface roughness cap
    match surface {
        SurfaceQuality::Unpaved => base.min(MmlosGrade::D),
        SurfaceQuality::Rough => base.min(MmlosGrade::C),
        _ => base,
    }
}

/// Score Walking Level of Service (A–E) per Active Travel England MMLoS criteria
pub fn score_walking_grade(
    highway: &str,
    infra: InfrastructureType,
    speed_mph: f64,
    aadt: Option<u32>,
    sidewalk: Option<bool>,
    width_m: Option<f64>,
    surface: SurfaceQuality,
    is_lit: bool,
    walking_allowed: bool,
) -> MmlosGrade {
    if !walking_allowed || matches!(highway, "motorway" | "motorway_link") {
        return MmlosGrade::E;
    }

    if infra == InfrastructureType::Steps {
        // Walkable for able-bodied pedestrians, though effortful
        return MmlosGrade::C;
    }

    let aadt_val = aadt.unwrap_or(0);

    let mut grade = match infra {
        InfrastructureType::PedestrianFootway
        | InfrastructureType::OffRoadPath
        | InfrastructureType::BridlewayGreenway => {
            if width_m.map(|w| w < 1.5).unwrap_or(false) {
                MmlosGrade::B
            } else {
                MmlosGrade::A
            }
        }
        InfrastructureType::VirtualFootway => {
            if speed_mph < 30.0 && aadt_val < 2000 {
                MmlosGrade::A
            } else if speed_mph < 30.0 && aadt_val < 4000 {
                MmlosGrade::B
            } else if speed_mph < 40.0 {
                MmlosGrade::C
            } else {
                MmlosGrade::E
            }
        }
        InfrastructureType::QuietLane => {
            if speed_mph < 30.0 && aadt_val < 1000 {
                MmlosGrade::A
            } else if speed_mph < 30.0 && aadt_val < 2000 {
                MmlosGrade::B
            } else {
                MmlosGrade::D
            }
        }
        InfrastructureType::SharedFootway => {
            // Shared pedestrian-cyclist space reduces pedestrian comfort slightly unless wide
            if width_m.map(|w| w >= 3.0).unwrap_or(false) {
                MmlosGrade::A
            } else {
                MmlosGrade::B
            }
        }
        InfrastructureType::SegregatedTrackWide | InfrastructureType::SegregatedTrackNarrow => {
            if matches!(highway, "cycleway") && sidewalk == Some(false) {
                MmlosGrade::D
            } else {
                MmlosGrade::B
            }
        }
        _ => {
            // Road carriageway: evaluate footway presence, speed, and traffic volume
            if highway == "living_street" {
                MmlosGrade::A
            } else if sidewalk == Some(false) {
                if speed_mph <= 20.0 && aadt_val < 1000 {
                    MmlosGrade::B
                } else if speed_mph <= 30.0 && aadt_val < 2000 {
                    MmlosGrade::D
                } else {
                    MmlosGrade::E
                }
            } else if speed_mph <= 20.0 {
                if aadt_val < 2000 {
                    MmlosGrade::A
                } else {
                    MmlosGrade::B
                }
            } else if speed_mph <= 30.0 {
                if aadt_val < 3000 {
                    MmlosGrade::B
                } else {
                    MmlosGrade::C
                }
            } else if speed_mph <= 40.0 {
                if sidewalk == Some(true) {
                    MmlosGrade::C
                } else {
                    MmlosGrade::D
                }
            } else if sidewalk == Some(true) {
                MmlosGrade::D
            } else {
                MmlosGrade::E
            }
        }
    };

    if matches!(surface, SurfaceQuality::Unpaved) {
        grade = grade.min(MmlosGrade::C);
    }
    if !is_lit && matches!(infra, InfrastructureType::OffRoadPath) {
        // Unlit isolated paths capped at B for personal security / after-dark inclusivity
        grade = grade.min(MmlosGrade::B);
    }

    grade
}

/// Score Wheeling Level of Service (A–E) (wheelchairs, mobility aids, prams, adapted cycles)
pub fn score_wheeling_grade(
    highway: &str,
    infra: InfrastructureType,
    walking_grade: MmlosGrade,
    width_m: Option<f64>,
    surface: SurfaceQuality,
    incline_pct: Option<f64>,
    kerb_raised: bool,
) -> MmlosGrade {
    if infra == InfrastructureType::Steps
        || kerb_raised
        || matches!(highway, "motorway" | "motorway_link")
    {
        return MmlosGrade::E;
    }

    let mut grade = walking_grade;

    // Obstacle-free width thresholds (`multilevelos/bid`)
    if let Some(w) = width_m {
        if w < 1.2 {
            return MmlosGrade::E;
        } else if w < 1.5 {
            grade = grade.min(MmlosGrade::C);
        } else if w < 2.0 {
            grade = grade.min(MmlosGrade::B);
        }
    }

    // Surface smoothness penalties (high rolling resistance / vibration)
    match surface {
        SurfaceQuality::Unpaved => return MmlosGrade::E,
        SurfaceQuality::Rough => grade = grade.min(MmlosGrade::D),
        SurfaceQuality::Moderate => grade = grade.min(MmlosGrade::B),
        SurfaceQuality::Smooth => {}
    }

    // Longitudinal gradient caps (<=3% A, 3-5% B, 5-8% D, >8% E)
    if let Some(inc) = incline_pct {
        if inc > 8.0 {
            return MmlosGrade::E;
        } else if inc > 5.0 {
            grade = grade.min(MmlosGrade::D);
        } else if inc > 3.0 {
            grade = grade.min(MmlosGrade::B);
        }
    }

    grade
}

/// Compute unified Multi-Modal MMLoS Grade combining Walking, Wheeling, and Cycling
/// Applies weakest-link inclusivity sensitivity so severe modal barriers pull down corridor MMLoS
pub fn score_multimodal_grade(
    walking: MmlosGrade,
    wheeling: MmlosGrade,
    cycling: MmlosGrade,
    infra: InfrastructureType,
) -> MmlosGrade {
    // On dedicated pedestrian-only spaces, evaluate active pedestrian + wheeling quality
    if infra == InfrastructureType::PedestrianFootway {
        let min_g = walking.min(wheeling).numeric_score();
        let avg_g = ((walking.numeric_score() + wheeling.numeric_score()) as f64 / 2.0).round() as i8;
        return MmlosGrade::from_numeric(avg_g.min(min_g + 1));
    }

    let w = walking.numeric_score();
    let wh = wheeling.numeric_score();
    let c = cycling.numeric_score();

    let min_score = w.min(wh).min(c);
    let mean_score = ((w as f64 + wh as f64 + c as f64) / 3.0).round() as i8;

    // Weakest-link cap: corridor cannot score more than 1 grade above its worst active mode
    let combined = mean_score.min(min_score + 1);
    MmlosGrade::from_numeric(combined)
}
