pub mod config;
pub mod engine;
pub mod infrastructure;
pub mod junctions;
pub mod los;
pub mod osm;
pub mod spatial;
pub mod speed;
pub mod traffic;

pub use config::{DesignStandard, LosConfig, TravelMode};
pub use engine::{
    CategoryMetric, ClassifiedSegment, LosAnalysisResult, LosEngine, ModeSuitability, SummaryStats,
};
pub use infrastructure::{
    classify_cycle_infrastructure, extract_incline_pct, extract_separation,
    extract_surface_quality, extract_width_m, InfrastructureType, SurfaceQuality,
};
pub use junctions::{analyze_junctions, ClassifiedJunction, JunctionSummaryStats};
pub use los::{
    lookup_level_of_service, score_cycling_grade, score_multimodal_grade, score_walking_grade,
    score_wheeling_grade, LevelOfServiceScore, MmlosGrade,
};
pub use osm::{
    extract_resolved_ways, extract_tagged_nodes, parse_overpass_json, OsmElement, OverpassResponse,
    ResolvedOsmNode, ResolvedOsmWay,
};
pub use spatial::{
    associate_nearest_driving_road, build_driving_roads_index,
    build_driving_roads_index_with_config, compute_distance_to_nearest_driving_road,
    haversine_distance, polyline_length_m, polyline_midpoint, sample_polyline_points,
    RoadAssociation,
};
pub use speed::{
    classify_speed_category, is_active_travel_path, is_urban_context, parse_maxspeed,
    parse_maxspeed_with_context,
};
pub use traffic::{
    classify_aadt_category, classify_aadt_ltn120, default_aadt_estimates, estimate_traffic_aadt,
};

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
    fn test_speed_classification_and_rural_context() {
        assert_eq!(classify_speed_category(15.0), "<20 mph");
        assert_eq!(classify_speed_category(20.0), "20 mph");
        assert_eq!(classify_speed_category(30.0), "30 mph");
        assert_eq!(classify_speed_category(60.0), "60+ mph");

        let defaults = LosConfig::default().speed_defaults;
        // Rural untagged unclassified road defaults to 60 mph per osmactive
        assert_eq!(
            parse_maxspeed_with_context(None, "unclassified", false, &defaults),
            Some(60.0)
        );
        // Urban untagged unclassified road defaults to 30 mph
        assert_eq!(
            parse_maxspeed_with_context(None, "unclassified", true, &defaults),
            Some(30.0)
        );
    }

    #[test]
    fn test_aadt_classification() {
        assert_eq!(classify_aadt_category(None), "0 to 999");
        assert_eq!(classify_aadt_category(Some(500)), "0 to 999");
        assert_eq!(classify_aadt_ltn120(Some(1800)), "0 to 1999");
        assert_eq!(classify_aadt_ltn120(Some(4500)), "4000 to 5999");
        assert_eq!(classify_aadt_ltn120(Some(7000)), "6000+");
    }

    #[test]
    fn test_level_of_service_matrix_and_ltn120() {
        assert_eq!(
            lookup_level_of_service("60+ mph", "4000+", InfrastructureType::OffRoadPath),
            LevelOfServiceScore::High
        );
        assert_eq!(
            lookup_level_of_service("40 mph", "4000+", InfrastructureType::MixedTrafficStreet),
            LevelOfServiceScore::UnsuitableMixed
        );

        // LTN 1/20: 30 mph painted cycle lane with 6000+ AADT is non-compliant Grade E
        assert_eq!(
            score_cycling_grade(
                DesignStandard::Ltn120,
                30.0,
                Some(6500),
                InfrastructureType::PaintedCycleLane,
                SurfaceQuality::Smooth,
                true
            ),
            MmlosGrade::E
        );

        // LTN 1/20: Light segregation on 20 mph street is Grade A
        assert_eq!(
            score_cycling_grade(
                DesignStandard::Ltn120,
                20.0,
                Some(3500),
                InfrastructureType::LightSegregation,
                SurfaceQuality::Smooth,
                true
            ),
            MmlosGrade::A
        );
    }

    #[test]
    fn test_multimodal_walking_vs_wheeling_steps_and_gradient() {
        // Steps are walkable (Grade C) but impassable for Wheeling (Grade E)
        let walk_steps = score_walking_grade(
            "steps",
            InfrastructureType::Steps,
            10.0,
            None,
            None,
            Some(2.0),
            SurfaceQuality::Smooth,
            true,
            true,
        );
        let wheel_steps = score_wheeling_grade(
            "steps",
            InfrastructureType::Steps,
            walk_steps,
            Some(2.0),
            SurfaceQuality::Smooth,
            None,
            false,
        );
        assert_eq!(walk_steps, MmlosGrade::C);
        assert_eq!(wheel_steps, MmlosGrade::E);

        // Steep 9% gradient caps Wheeling at Grade E even on an A-grade pedestrian path
        let wheel_steep = score_wheeling_grade(
            "footway",
            InfrastructureType::PedestrianFootway,
            MmlosGrade::A,
            Some(3.0),
            SurfaceQuality::Smooth,
            Some(9.0),
            false,
        );
        assert_eq!(wheel_steep, MmlosGrade::E);
    }

    #[test]
    fn test_sidepath_inherits_adjacent_road_speed_and_junctions_generated() {
        let sample_osm = r#"{
          "elements": [
            {"type": "node", "id": 1, "lat": 53.8000, "lon": -1.5500},
            {"type": "node", "id": 2, "lat": 53.8000, "lon": -1.5480, "tags": {"highway": "traffic_signals"}},
            {"type": "node", "id": 3, "lat": 53.8000, "lon": -1.5460},
            {"type": "node", "id": 4, "lat": 53.7990, "lon": -1.5480},
            {"type": "node", "id": 5, "lat": 53.8010, "lon": -1.5480},
            {"type": "node", "id": 6, "lat": 53.80004, "lon": -1.5500},
            {"type": "node", "id": 7, "lat": 53.80004, "lon": -1.5480},
            {"type": "way", "id": 100, "nodes": [1, 2, 3], "tags": {"highway": "primary", "name": "Woodhouse Lane", "maxspeed": "40 mph"}},
            {"type": "way", "id": 101, "nodes": [4, 2, 5], "tags": {"highway": "secondary", "name": "Clarendon Road", "maxspeed": "30 mph"}},
            {"type": "way", "id": 102, "nodes": [6, 7], "tags": {"highway": "cycleway", "width": "1.5"}}
          ]
        }"#;

        let engine = LosEngine::from_overpass_json(sample_osm).unwrap();
        let res = engine.run_analysis(&LosConfig::default()).unwrap();

        // Parallel cycleway ~4.4m from Woodhouse Lane should inherit 40 mph & 6000 AADT!
        let track = res.segments.iter().find(|s| s.osm_id == 102).unwrap();
        assert_eq!(track.speed_limit_mph, Some(40.0));
        assert_eq!(track.estimated_aadt, Some(6000));
        assert!(track.distance_to_road_m.unwrap() < 10.0);

        // Crossroads at node 2 should be detected as a Signal-controlled junction polygon!
        assert_eq!(res.junctions_count, 1);
        assert_eq!(res.junctions[0].control_type, "Signal-controlled");
        assert!(res.junctions[0].arm_count >= 4);
    }
}
