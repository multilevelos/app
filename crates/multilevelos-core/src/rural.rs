//! Active Travel England *Rural Design Guide* (July 2026) walking & cycling categories,
//! surface/lighting inclusivity rules, PRoW designations, and Table 4-3 / Table 10-1 scoring.

use crate::infrastructure::extract_separation;
use crate::los::MmlosGrade;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DESIRABLE_SHARED_WIDTH_M: f64 = 3.0;
pub const EQUESTRIAN_SHARED_WIDTH_M: f64 = 4.5;
pub const MIN_SHARED_WIDTH_M: f64 = 2.0;
pub const DESIRABLE_FOOTWAY_WIDTH_M: f64 = 2.0;
pub const MIN_FOOTWAY_WIDTH_M: f64 = 1.5;

/// Surfacing inclusivity tiers from Table 4-1 (p. 32) and Table 14-1 (pp. 210–212).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuralSurfaceGroup {
    /// Machine-laid or smooth bound surfacing — Suitable for most (`Grade A` ceiling).
    Bound,
    /// Self-binding aggregate, compacted stone, fine gravel, gravel or boardwalk — Not suitable for all (`Grade C` ceiling, or `Grade B` for smooth compacted greenways).
    Unbound,
    /// Unsurfaced earth, grass, mud, sand or degraded smoothness — Suitable for few (`Grade E` for cycling/wheeling, `Grade D` for walking).
    Unsurfaced,
}

impl RuralSurfaceGroup {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bound => "Bound paved",
            Self::Unbound => "Unbound gravel / aggregate",
            Self::Unsurfaced => "Unsurfaced earth / grass",
        }
    }
}

/// Public Rights of Way (PRoW) and Quiet Lane designations from Table 8.1 (p. 78) and Chapter 7.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProwDesignation {
    PublicFootpath,
    PublicBridleway,
    RestrictedByway,
    BywayOpenToAllTraffic,
    QuietLane,
}

impl ProwDesignation {
    pub fn label(self) -> &'static str {
        match self {
            Self::PublicFootpath => "Public Footpath",
            Self::PublicBridleway => "Public Bridleway",
            Self::RestrictedByway => "Restricted Byway",
            Self::BywayOpenToAllTraffic => "Byway Open to All Traffic (BOAT)",
            Self::QuietLane => "Designated Quiet Lane",
        }
    }

    pub fn allows_cycling_by_default(self) -> bool {
        matches!(
            self,
            Self::PublicBridleway
                | Self::RestrictedByway
                | Self::BywayOpenToAllTraffic
                | Self::QuietLane
        )
    }

    pub fn allows_motor_traffic(self) -> bool {
        matches!(self, Self::BywayOpenToAllTraffic | Self::QuietLane)
    }
}

/// Walking & wheeling infrastructure categories from the ATE Rural Design Guide (July 2026).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WalkingCategory {
    MotorFreePathProw,
    ProtectedFootway,
    SharedUsePath,
    VirtualFootway,
    QuietLaneMixed,
    CarriagewayNoFootway,
    Steps,
    NoWalkingAccess,
}

impl WalkingCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::MotorFreePathProw => "Motor-Free Path / PRoW",
            Self::ProtectedFootway => "Protected Footway",
            Self::SharedUsePath => "Shared-Use Path",
            Self::VirtualFootway => "Virtual / Implied Footway",
            Self::QuietLaneMixed => "Quiet Lane (Mixed)",
            Self::CarriagewayNoFootway => "Carriageway (No Footway)",
            Self::Steps => "Steps (Stepped Access)",
            Self::NoWalkingAccess => "No Pedestrian Access",
        }
    }

    pub fn color(self) -> &'static str {
        match self {
            Self::MotorFreePathProw => "#15803d",
            Self::ProtectedFootway => "#059669",
            Self::SharedUsePath => "#0d9488",
            Self::VirtualFootway => "#0284c7",
            Self::QuietLaneMixed => "#65a30d",
            Self::CarriagewayNoFootway => "#dc2626",
            Self::Steps => "#9333ea",
            Self::NoWalkingAccess => "#94a3b8",
        }
    }
}

/// Cycling infrastructure categories from the ATE Rural Design Guide (July 2026).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CyclingCategory {
    MotorFreeGreenwayBridleway,
    FullyProtectedCycleTrack,
    SteppedCycleTrack,
    SharedUsePathTrack,
    LightSegregation,
    CycleLane,
    QuietLaneCountryLane,
    MixedTrafficRoad,
    NoCyclingAccess,
}

impl CyclingCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::MotorFreeGreenwayBridleway => "Motor-Free Greenway / Bridleway",
            Self::FullyProtectedCycleTrack => "Fully Protected Cycle Track",
            Self::SteppedCycleTrack => "Stepped Cycle Track",
            Self::SharedUsePathTrack => "Shared-Use Path / Track",
            Self::LightSegregation => "Light Segregation",
            Self::CycleLane => "Cycle Lane (Mandatory / Advisory)",
            Self::QuietLaneCountryLane => "Quiet Lane / Country Lane",
            Self::MixedTrafficRoad => "Mixed Traffic Road",
            Self::NoCyclingAccess => "No Cycling (Footpath / Steps)",
        }
    }

    pub fn color(self) -> &'static str {
        match self {
            Self::MotorFreeGreenwayBridleway => "#15803d",
            Self::FullyProtectedCycleTrack => "#059669",
            Self::SteppedCycleTrack => "#65a30d",
            Self::SharedUsePathTrack => "#d97706",
            Self::LightSegregation => "#0d9488",
            Self::CycleLane => "#ea580c",
            Self::QuietLaneCountryLane => "#0284c7",
            Self::MixedTrafficRoad => "#64748b",
            Self::NoCyclingAccess => "#94a3b8",
        }
    }
}

/// Extract PRoW or Quiet Lane designation from OSM tags (Table 8.1 & Ch. 7.2).
pub fn extract_prow_designation(highway: &str, tags: &HashMap<String, String>) -> Option<ProwDesignation> {
    if tags.get("quiet_lane").is_some_and(|v| v == "yes")
        || tags.get("designation").is_some_and(|v| v == "quiet_lane")
    {
        return Some(ProwDesignation::QuietLane);
    }
    if let Some(des) = tags.get("designation").map(|s| s.trim().to_lowercase()) {
        match des.as_str() {
            "public_footpath" | "permissive_footpath" => {
                return Some(ProwDesignation::PublicFootpath);
            }
            "public_bridleway" | "permissive_bridleway" => {
                return Some(ProwDesignation::PublicBridleway);
            }
            "restricted_byway" => return Some(ProwDesignation::RestrictedByway),
            "byway_open_to_all_traffic" => {
                return Some(ProwDesignation::BywayOpenToAllTraffic);
            }
            _ => {}
        }
    }
    match highway {
        "bridleway" => Some(ProwDesignation::PublicBridleway),
        "byway" => Some(ProwDesignation::BywayOpenToAllTraffic),
        _ => None,
    }
}

/// Classify surface into Table 4-1 / Table 14-1 bound, unbound or unsurfaced group.
pub fn extract_rural_surface_group(
    highway: &str,
    tags: &HashMap<String, String>,
) -> RuralSurfaceGroup {
    if let Some(sm) = tags.get("smoothness").map(|s| s.trim().to_lowercase()) {
        if matches!(
            sm.as_str(),
            "very_bad" | "horrible" | "very_horrible" | "impassable"
        ) {
            return RuralSurfaceGroup::Unsurfaced;
        }
    }

    if let Some(surf) = tags.get("surface").map(|s| s.trim().to_lowercase()) {
        if matches!(
            surf.as_str(),
            "asphalt"
                | "paved"
                | "concrete"
                | "concrete:plates"
                | "tarmac"
                | "chipseal"
                | "resin_bound"
                | "rubber"
                | "tartan"
                | "paving_stones"
                | "metal"
        ) {
            return RuralSurfaceGroup::Bound;
        }
        if matches!(
            surf.as_str(),
            "compacted"
                | "fine_gravel"
                | "gravel"
                | "pebblestone"
                | "wood"
                | "brick"
                | "sett"
                | "cobblestone"
                | "concrete:lanes"
        ) {
            return RuralSurfaceGroup::Unbound;
        }
        if matches!(
            surf.as_str(),
            "ground"
                | "unpaved"
                | "grass"
                | "grass_paver"
                | "earth"
                | "mud"
                | "sand"
                | "dirt"
                | "woodchips"
                | "rock"
                | "unhewn_cobblestone"
        ) {
            return RuralSurfaceGroup::Unsurfaced;
        }
    }

    let prow = extract_prow_designation(highway, tags);
    if matches!(highway, "bridleway" | "track" | "byway")
        || matches!(
            prow,
            Some(
                ProwDesignation::PublicFootpath
                    | ProwDesignation::PublicBridleway
                    | ProwDesignation::RestrictedByway
                    | ProwDesignation::BywayOpenToAllTraffic
            )
        )
    {
        return RuralSurfaceGroup::Unbound;
    }
    if highway == "path" {
        return RuralSurfaceGroup::Unbound;
    }

    RuralSurfaceGroup::Bound
}

/// Detect virtual / implied footways per Ch. 7.5 (pp. 73–74) and Table 4-3 col 2.
pub fn has_virtual_footway(tags: &HashMap<String, String>) -> bool {
    tags.get("footway").is_some_and(|v| v == "lane")
        || tags.get("sidewalk").is_some_and(|v| v == "lane")
        || tags.get("sidewalk:left").is_some_and(|v| v == "lane")
        || tags.get("sidewalk:right").is_some_and(|v| v == "lane")
        || tags.get("sidewalk:both").is_some_and(|v| v == "lane")
        || tags.get("shoulder:foot").is_some_and(|v| v == "yes")
}

/// Detect intermediate-height stepped cycle tracks per Table 4-3 col 5.
pub fn is_stepped_cycle_track(highway: &str, tags: &HashMap<String, String>) -> bool {
    for (k, v) in tags {
        if k.contains("separation") {
            let val = v.to_lowercase();
            if val.contains("stepped") || val.contains("separation_kerb") {
                return true;
            }
        }
    }
    let is_road = !matches!(
        highway,
        "cycleway" | "footway" | "path" | "pedestrian" | "bridleway" | "steps" | "track"
    );
    if is_road {
        let on_road_track = ["cycleway", "cycleway:left", "cycleway:right", "cycleway:both"]
            .iter()
            .filter_map(|k| tags.get(*k))
            .any(|v| v == "track" || v == "opposite_track");
        if on_road_track && extract_separation(tags) == "kerb" {
            let has_wide_buffer = [
                "cycleway:buffer",
                "cycleway:left:buffer",
                "cycleway:right:buffer",
                "cycleway:both:buffer",
                "separation:buffer",
            ]
            .iter()
            .filter_map(|k| tags.get(*k))
            .filter_map(|v| v.trim().trim_end_matches('m').trim().parse::<f64>().ok())
            .any(|b| b >= 0.5);
            if !has_wide_buffer {
                return true;
            }
        }
    }
    false
}

/// Classify a way into its ATE Rural Design Guide Walking Category.
pub fn classify_walking_category(
    highway: &str,
    tags: &HashMap<String, String>,
    distance_to_road_m: Option<f64>,
    min_offroad_distance: f64,
    speed_mph: f64,
    aadt: Option<u32>,
    sidewalk: Option<bool>,
    is_urban: bool,
    walking_allowed: bool,
) -> WalkingCategory {
    if !walking_allowed {
        return WalkingCategory::NoWalkingAccess;
    }
    if highway == "steps" {
        return WalkingCategory::Steps;
    }

    let prow = extract_prow_designation(highway, tags);
    let is_segregated = tags.get("segregated").is_some_and(|s| s == "yes");
    let bicycle = tags.get("bicycle").map(|s| s.as_str()).unwrap_or("");
    let cycling_shared = !is_segregated
        && (matches!(highway, "cycleway" | "bridleway")
            || matches!(bicycle, "yes" | "designated" | "permissive")
            || prow.is_some_and(ProwDesignation::allows_cycling_by_default));

    let is_active_path = matches!(
        highway,
        "cycleway" | "footway" | "path" | "pedestrian" | "bridleway"
    ) || (highway == "track"
        && !prow.is_some_and(ProwDesignation::allows_motor_traffic));

    if is_active_path {
        let off_road = distance_to_road_m.is_none_or(|d| d > min_offroad_distance)
            || matches!(
                prow,
                Some(
                    ProwDesignation::PublicFootpath
                        | ProwDesignation::PublicBridleway
                        | ProwDesignation::RestrictedByway
                )
            );
        if is_segregated {
            return WalkingCategory::ProtectedFootway;
        }
        if off_road
            && (!cycling_shared
                || matches!(
                    prow,
                    Some(ProwDesignation::PublicBridleway | ProwDesignation::RestrictedByway)
                ))
        {
            return WalkingCategory::MotorFreePathProw;
        }
        if cycling_shared {
            return WalkingCategory::SharedUsePath;
        }
        return if off_road {
            WalkingCategory::MotorFreePathProw
        } else {
            WalkingCategory::ProtectedFootway
        };
    }

    // Carriageways, BOATs & Quiet Lanes
    if highway == "living_street"
        || matches!(
            prow,
            Some(ProwDesignation::QuietLane | ProwDesignation::BywayOpenToAllTraffic)
        )
    {
        return WalkingCategory::QuietLaneMixed;
    }
    if has_virtual_footway(tags) {
        return WalkingCategory::VirtualFootway;
    }
    if sidewalk == Some(true) {
        return WalkingCategory::ProtectedFootway;
    }
    if sidewalk == Some(false) || !is_urban {
        if speed_mph < 30.0 && aadt.unwrap_or(0) < 1000 {
            return WalkingCategory::QuietLaneMixed;
        }
        return WalkingCategory::CarriagewayNoFootway;
    }

    // Urban street with untagged sidewalk (assumed footway present on urban residential/classified streets)
    WalkingCategory::ProtectedFootway
}

/// Classify a way into its ATE Rural Design Guide Cycling Category.
pub fn classify_cycling_category(
    highway: &str,
    tags: &HashMap<String, String>,
    distance_to_road_m: Option<f64>,
    min_offroad_distance: f64,
    speed_mph: f64,
    aadt: Option<u32>,
    is_urban: bool,
    cycling_allowed: bool,
) -> CyclingCategory {
    if !cycling_allowed {
        return CyclingCategory::NoCyclingAccess;
    }

    let prow = extract_prow_designation(highway, tags);
    let is_segregated = tags.get("segregated").is_some_and(|s| s == "yes");
    let foot = tags.get("foot").map(|s| s.as_str()).unwrap_or("");
    let footway_tag = tags.get("footway").map(|s| s.as_str()).unwrap_or("");
    let name = tags.get("name").map(|s| s.as_str()).unwrap_or("");

    let is_active_path = matches!(
        highway,
        "cycleway" | "footway" | "path" | "pedestrian" | "bridleway"
    ) || (highway == "track"
        && !prow.is_some_and(ProwDesignation::allows_motor_traffic));

    if is_active_path {
        let named_trail = ["Path", "Towpath", "Railway", "Trail", "Greenway"]
            .iter()
            .any(|w| name.contains(w));
        let off_road = distance_to_road_m.is_none_or(|d| d > min_offroad_distance)
            || named_trail
            || matches!(
                prow,
                Some(ProwDesignation::PublicBridleway | ProwDesignation::RestrictedByway)
            );
        let shared = !is_segregated
            && (matches!(highway, "footway" | "path" | "pedestrian" | "bridleway" | "track")
                || matches!(foot, "yes" | "designated")
                || tags.get("segregated").is_some_and(|s| s == "no")
                || footway_tag.eq_ignore_ascii_case("sidewalk"));

        if off_road
            && (!shared
                || matches!(
                    prow,
                    Some(ProwDesignation::PublicBridleway | ProwDesignation::RestrictedByway)
                ))
        {
            return CyclingCategory::MotorFreeGreenwayBridleway;
        }
        if shared {
            return CyclingCategory::SharedUsePathTrack;
        }
        return if off_road {
            CyclingCategory::MotorFreeGreenwayBridleway
        } else {
            CyclingCategory::FullyProtectedCycleTrack
        };
    }

    if is_stepped_cycle_track(highway, tags) {
        return CyclingCategory::SteppedCycleTrack;
    }

    let sep = extract_separation(tags);
    if sep == "light" {
        return CyclingCategory::LightSegregation;
    }

    let mut cycleway_vals = String::new();
    for (k, v) in tags {
        if k.starts_with("cycleway") || k.starts_with("busway") || k == "lanes:bus" {
            cycleway_vals.push_str(v);
            cycleway_vals.push(' ');
        }
    }
    let cw = cycleway_vals.to_lowercase();

    if cw.contains("track")
        || tags.get("cycleway:left:segregated").is_some_and(|v| v == "yes")
        || tags.get("cycleway:right:segregated").is_some_and(|v| v == "yes")
    {
        return if matches!(sep, "verge" | "barrier") {
            CyclingCategory::FullyProtectedCycleTrack
        } else {
            CyclingCategory::LightSegregation
        };
    }

    if cw.contains("lane")
        || cw.contains("share_busway")
        || cw.contains("opposite")
        || highway == "busway"
    {
        return CyclingCategory::CycleLane;
    }

    if highway == "living_street"
        || matches!(
            prow,
            Some(ProwDesignation::QuietLane | ProwDesignation::BywayOpenToAllTraffic)
        )
    {
        return CyclingCategory::QuietLaneCountryLane;
    }

    let flow = aadt.unwrap_or(500);
    let low_speed_quiet = speed_mph < 30.0 && flow < 2000;
    let rural_footnote_5 = !is_urban && speed_mph < 40.0 && flow < 1000;
    if low_speed_quiet || rural_footnote_5 {
        CyclingCategory::QuietLaneCountryLane
    } else {
        CyclingCategory::MixedTrafficRoad
    }
}

/// Score Walking Level of Service (Grades A–E) under ATE Rural Design Guide (July 2026)
/// Table 4-3 (cols 1–3), Table 4-1 (surfacing), Table 4-2 (lighting) & Chapter 5.3 (footway widths).
pub fn score_rural_walking_grade(
    walk_cat: WalkingCategory,
    speed_mph: f64,
    aadt: Option<u32>,
    width_m: Option<f64>,
    surface_group: RuralSurfaceGroup,
    is_lit: bool,
    is_urban: bool,
) -> MmlosGrade {
    if walk_cat == WalkingCategory::NoWalkingAccess {
        return MmlosGrade::E;
    }
    if walk_cat == WalkingCategory::Steps {
        return MmlosGrade::C;
    }

    let flow = aadt.unwrap_or(0);

    let mut grade = match walk_cat {
        WalkingCategory::MotorFreePathProw | WalkingCategory::ProtectedFootway => {
            // Table 4-3 col 1: Green across all speeds & flows; Ch. 5.3 width refinement
            match width_m {
                Some(w) if w < MIN_FOOTWAY_WIDTH_M => MmlosGrade::C,
                Some(w) if w < DESIRABLE_FOOTWAY_WIDTH_M => MmlosGrade::B,
                _ => MmlosGrade::A,
            }
        }
        WalkingCategory::SharedUsePath => {
            // Table 4-3 Footnote 4 + Table 4-4 + Ch. 5.3 (>=3.0m desirable shared width)
            match width_m {
                Some(w) if w >= DESIRABLE_SHARED_WIDTH_M && !is_urban => MmlosGrade::A,
                Some(w) if w < MIN_SHARED_WIDTH_M => MmlosGrade::C,
                _ => MmlosGrade::B,
            }
        }
        WalkingCategory::VirtualFootway => {
            // Table 4-3 col 2: Virtual / implied footway
            if speed_mph < 30.0 {
                if flow < 2000 {
                    MmlosGrade::A
                } else if flow < 4000 {
                    MmlosGrade::B
                } else {
                    MmlosGrade::C
                }
            } else if speed_mph < 40.0 {
                MmlosGrade::C
            } else {
                MmlosGrade::E
            }
        }
        WalkingCategory::QuietLaneMixed | WalkingCategory::CarriagewayNoFootway => {
            // Table 4-3 col 3: Walking/wheeling in mixed traffic
            if speed_mph < 30.0 {
                if flow < 1000 {
                    MmlosGrade::A
                } else if flow < 2000 {
                    MmlosGrade::B
                } else if flow < 4000 {
                    MmlosGrade::D
                } else {
                    MmlosGrade::E
                }
            } else {
                MmlosGrade::E
            }
        }
        WalkingCategory::Steps | WalkingCategory::NoWalkingAccess => unreachable!(),
    };

    // Table 4-1 Surfacing inclusivity cap
    match surface_group {
        RuralSurfaceGroup::Unbound => {
            grade = grade.min(MmlosGrade::C);
        }
        RuralSurfaceGroup::Unsurfaced => {
            grade = grade.min(MmlosGrade::D);
        }
        RuralSurfaceGroup::Bound => {}
    }

    // Table 4-2 & Section 8.4 Lighting inclusivity: in urban areas unlit isolated paths cap at B;
    // in rural dark-sky settings Section 8.4 notes unlit bound motor-free routes remain acceptable (capped at B only when narrow).
    if !is_lit && is_urban && walk_cat == WalkingCategory::MotorFreePathProw {
        grade = grade.min(MmlosGrade::B);
    }

    grade
}

/// Score Cycling Level of Service (Grades A–E) under ATE Rural Design Guide (July 2026)
/// Table 4-3 (cols 4–8, Footnote 5), Table 4-1 (surfacing), and Table 4-4 / Ch. 5.3 (shared widths).
pub fn score_rural_cycling_grade(
    cycle_cat: CyclingCategory,
    speed_mph: f64,
    aadt: Option<u32>,
    width_m: Option<f64>,
    surface_group: RuralSurfaceGroup,
    is_urban: bool,
    equestrian: bool,
) -> MmlosGrade {
    if cycle_cat == CyclingCategory::NoCyclingAccess {
        return MmlosGrade::E;
    }

    let flow = aadt.unwrap_or(0);

    let mut grade = match cycle_cat {
        CyclingCategory::MotorFreeGreenwayBridleway | CyclingCategory::FullyProtectedCycleTrack => {
            // Table 4-3 col 4: Fully protected cycle track / motor-free greenway = Suitable for most at all speeds!
            if width_m.is_some_and(|w| w < 2.0) {
                MmlosGrade::B
            } else {
                MmlosGrade::A
            }
        }
        CyclingCategory::SharedUsePathTrack => {
            // Table 4-4 & Ch. 5.3: Outside built-up areas shared-use >=3.0m (>=4.5m equestrian) is Suitable for most (A),
            // 2.0-3.0m is Mostly suitable (B), <2.0m or urban shared is Not suitable for all (C).
            if is_urban {
                MmlosGrade::C
            } else {
                let desirable = if equestrian {
                    EQUESTRIAN_SHARED_WIDTH_M
                } else {
                    DESIRABLE_SHARED_WIDTH_M
                };
                match width_m {
                    Some(w) if w >= desirable => MmlosGrade::A,
                    Some(w) if w >= MIN_SHARED_WIDTH_M => MmlosGrade::B,
                    None => MmlosGrade::B,
                    Some(_) => MmlosGrade::C,
                }
            }
        }
        CyclingCategory::SteppedCycleTrack | CyclingCategory::LightSegregation => {
            // Table 4-3 cols 5 & 6: 20 & 30 mph = Suitable for most (A/B), 40 mph = Not suitable for all (C), 50+ mph = Suitable for few (E)
            if speed_mph < 30.0 {
                MmlosGrade::A
            } else if speed_mph < 40.0 {
                MmlosGrade::B
            } else if speed_mph < 50.0 {
                MmlosGrade::C
            } else {
                MmlosGrade::E
            }
        }
        CyclingCategory::CycleLane => {
            // Table 4-3 col 7: Cycle lane — mandatory/advisory
            if speed_mph < 30.0 {
                if flow < 4000 {
                    MmlosGrade::A
                } else if flow < 6000 {
                    MmlosGrade::B
                } else {
                    MmlosGrade::C
                }
            } else if speed_mph < 40.0 {
                if flow < 4000 {
                    MmlosGrade::C
                } else if flow < 6000 {
                    MmlosGrade::D
                } else {
                    MmlosGrade::E
                }
            } else {
                MmlosGrade::E
            }
        }
        CyclingCategory::QuietLaneCountryLane | CyclingCategory::MixedTrafficRoad => {
            // Table 4-3 col 8 + Footnote 5 (rural <=30 mph & <1,000 pcu/day)
            if speed_mph < 30.0 {
                if flow < 2000 {
                    MmlosGrade::A
                } else if flow < 4000 {
                    MmlosGrade::B
                } else if flow < 6000 {
                    MmlosGrade::C
                } else {
                    MmlosGrade::D
                }
            } else if speed_mph < 40.0 {
                if !is_urban && flow < 1000 {
                    // Footnote 5: in rural areas speeds up to 30 mph with <1,000 pcu/day are generally acceptable
                    MmlosGrade::B
                } else if flow < 2000 {
                    MmlosGrade::C
                } else if flow < 4000 {
                    MmlosGrade::D
                } else {
                    MmlosGrade::E
                }
            } else {
                MmlosGrade::E
            }
        }
        CyclingCategory::NoCyclingAccess => unreachable!(),
    };

    // Table 4-1 Surfacing inclusivity cap
    match surface_group {
        RuralSurfaceGroup::Unbound => {
            grade = grade.min(MmlosGrade::C);
        }
        RuralSurfaceGroup::Unsurfaced => {
            grade = MmlosGrade::E;
        }
        RuralSurfaceGroup::Bound => {}
    }

    grade
}

/// Score Highway Crossing / Junction suitability under Table 10-1 (p. 133) of the ATE Rural Design Guide.
pub fn score_rural_junction_grades(
    max_speed_mph: f64,
    max_aadt: u32,
    lanes: u8,
    is_roundabout: bool,
    has_signals: bool,
    has_marked_crossing: bool,
    has_tactile_or_kerb: bool,
) -> (MmlosGrade, MmlosGrade, MmlosGrade) {
    // Table 10-1: At >= 60 mph, ALL at-grade crossings (Signal, Parallel/Zebra, Cycle priority, Uncontrolled) are Pink (Grade E)!
    if max_speed_mph >= 60.0 {
        return (MmlosGrade::E, MmlosGrade::E, MmlosGrade::E);
    }

    if is_roundabout {
        let g_walk = if has_signals && max_speed_mph <= 40.0 {
            MmlosGrade::B
        } else if max_speed_mph < 30.0 && max_aadt < 2000 {
            MmlosGrade::B
        } else if max_speed_mph < 40.0 && max_aadt < 4000 {
            MmlosGrade::D
        } else {
            MmlosGrade::E
        };
        let g_cycle = if max_speed_mph < 30.0 && max_aadt < 2000 {
            MmlosGrade::B
        } else if has_signals && max_speed_mph < 40.0 {
            MmlosGrade::C
        } else {
            MmlosGrade::E
        };
        let g_wheel = if has_signals || has_tactile_or_kerb {
            g_walk
        } else {
            g_walk.min(MmlosGrade::D)
        };
        return (g_walk, g_wheel, g_cycle);
    }

    // Signal-controlled crossing (Table 10-1: Green up to 50 mph!)
    if has_signals {
        let g = if max_speed_mph < 40.0 {
            MmlosGrade::A
        } else {
            // 40 or 50 mph signal crossing
            MmlosGrade::B
        };
        let g_wheel = if has_tactile_or_kerb {
            g
        } else {
            g.min(MmlosGrade::B)
        };
        return (g, g_wheel, g);
    }

    // Parallel / Zebra marked crossing (Table 10-1: Green <=30 mph up to 8,000, Green-Amber >8,000 2-lane, Pink >=40 mph!)
    if has_marked_crossing {
        let g = if max_speed_mph < 40.0 {
            if lanes > 2 {
                MmlosGrade::E
            } else if max_aadt < 8000 {
                MmlosGrade::A
            } else {
                MmlosGrade::B
            }
        } else {
            MmlosGrade::E
        };
        let g_wheel = if has_tactile_or_kerb {
            g
        } else {
            g.min(MmlosGrade::B)
        };
        return (g, g_wheel, g);
    }

    // Uncontrolled crossing / priority junction (Table 10-1)
    let g = if max_speed_mph >= 50.0 {
        if lanes <= 1 && max_aadt < 2000 {
            MmlosGrade::B
        } else if lanes <= 1 && max_aadt < 8000 {
            MmlosGrade::C
        } else {
            MmlosGrade::E
        }
    } else if max_speed_mph >= 40.0 {
        if lanes <= 1 && max_aadt < 4000 {
            MmlosGrade::A
        } else if lanes <= 1 && max_aadt < 8000 {
            MmlosGrade::C
        } else {
            MmlosGrade::E
        }
    } else {
        // <= 30 mph
        if max_aadt < 2000 {
            MmlosGrade::A
        } else if max_aadt < 4000 && lanes <= 2 {
            MmlosGrade::B
        } else if max_aadt < 8000 && lanes <= 1 {
            MmlosGrade::C
        } else if max_aadt < 8000 && lanes <= 2 {
            MmlosGrade::D
        } else {
            MmlosGrade::E
        }
    };

    let g_wheel = if !has_tactile_or_kerb && g == MmlosGrade::A {
        MmlosGrade::B
    } else {
        g
    };

    (g, g_wheel, g)
}
