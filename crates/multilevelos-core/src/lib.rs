pub mod config;
pub mod engine;
pub mod infrastructure;
pub mod los;
pub mod osm;
pub mod spatial;
pub mod speed;
pub mod traffic;

pub use config::LosConfig;
pub use engine::{CategoryMetric, ClassifiedSegment, LosAnalysisResult, LosEngine, SummaryStats};
pub use infrastructure::{classify_cycle_infrastructure, extract_width_m, InfrastructureType};
pub use los::{lookup_level_of_service, LevelOfServiceScore};
pub use osm::{extract_resolved_ways, parse_overpass_json, OsmElement, OverpassResponse, ResolvedOsmWay};
pub use spatial::{
    build_driving_roads_index, compute_distance_to_nearest_driving_road, haversine_distance,
    polyline_length_m, polyline_midpoint,
};
pub use speed::{classify_speed_category, is_active_travel_path, parse_maxspeed};
pub use traffic::{classify_aadt_category, default_aadt_estimates, estimate_traffic_aadt};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        let p1 = (-3.1883, 55.9533); // Edinburgh
        let p2 = (-4.2518, 55.8642); // Glasgow
        let dist = haversine_distance(p1, p2);
        assert!((dist - 66500.0).abs() < 2000.0);
    }

    #[test]
    fn test_speed_classification() {
        assert_eq!(classify_speed_category(15.0), "<20 mph");
        assert_eq!(classify_speed_category(20.0), "20 mph");
        assert_eq!(classify_speed_category(30.0), "30 mph");
        assert_eq!(classify_speed_category(40.0), "40 mph");
        assert_eq!(classify_speed_category(50.0), "50 mph");
        assert_eq!(classify_speed_category(60.0), "60+ mph");
        assert_eq!(classify_speed_category(70.0), "60+ mph");
    }

    #[test]
    fn test_aadt_classification() {
        assert_eq!(classify_aadt_category(None), "0 to 999");
        assert_eq!(classify_aadt_category(Some(500)), "0 to 999");
        assert_eq!(classify_aadt_category(Some(1500)), "1000 to 1999");
        assert_eq!(classify_aadt_category(Some(3000)), "2000 to 3999");
        assert_eq!(classify_aadt_category(Some(6000)), "4000+");
    }

    #[test]
    fn test_level_of_service_matrix() {
        // Off road path is always High
        assert_eq!(
            lookup_level_of_service("60+ mph", "4000+", InfrastructureType::OffRoadPath),
            LevelOfServiceScore::High
        );

        // Mixed traffic on 20 mph quiet residential (500 AADT) is High
        assert_eq!(
            lookup_level_of_service("20 mph", "0 to 999", InfrastructureType::MixedTrafficStreet),
            LevelOfServiceScore::High
        );

        // Mixed traffic on 40 mph busy road (4000+ AADT) is Unsuitable
        assert_eq!(
            lookup_level_of_service("40 mph", "4000+", InfrastructureType::MixedTrafficStreet),
            LevelOfServiceScore::UnsuitableMixed
        );

        // Painted cycle lane on 60 mph road is Non-compliant
        assert_eq!(
            lookup_level_of_service("60+ mph", "4000+", InfrastructureType::PaintedCycleLane),
            LevelOfServiceScore::NonCompliant
        );
    }
}
