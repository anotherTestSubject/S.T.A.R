use crate::envelope::AnalogGap;
use crate::pareto::{ParetoRole, RankedCandidate};
use crate::schema::{CelestialBody, Cell, MissionType, TargetProfile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapClosureRecommendation {
    pub condition: String,
    pub gap_value: f64,
    pub status: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogDna {
    pub cell_id: String,
    pub region_name: String,
    pub lat: f64,
    pub lon: f64,
    pub target_name: String,
    pub mission: MissionType,
    pub pareto_role: ParetoRole,
    
    pub overall_similarity_pct: f64,
    pub terrain_similarity_pct: f64,
    pub thermal_similarity_pct: f64,
    pub novelty_score: f64,
    pub geo_novelty_km: f64,
    pub confidence_pct: f64,

    pub novelty_status: String,

    pub naturally_matched: Vec<String>,
    pub gap_closures: Vec<GapClosureRecommendation>,
    pub irreproducible: Vec<String>,
}

pub fn build_analog_dna(
    candidate: &RankedCandidate,
    cell: &Cell,
    target: &TargetProfile,
    gap: &AnalogGap,
    mission: MissionType,
) -> AnalogDna {
    let score = &candidate.score;

    let slope_diff = (cell.terrain.rms_slope_deg - target.terrain.rms_slope_deg).abs();
    let terrain_sim = (1.0 / (1.0 + slope_diff / 10.0) * 100.0).clamp(0.0, 100.0);

    let amp_diff = (cell.thermal.diurnal_amp_k - target.thermal.diurnal_amp_k).abs();
    let thermal_sim = (1.0 / (1.0 + amp_diff / 30.0) * 100.0).clamp(0.0, 100.0);

    let novelty_status = if score.is_known_analog {
        "Known Analog (In Reference Catalog)".to_string()
    } else if score.geo_novelty_km > 500.0 && score.feature_novelty > 0.15 {
        "Novel Candidate (Requires Literature Verification)".to_string()
    } else {
        "Unconfirmed / Near Known Analog".to_string()
    };

    let mut naturally_matched = Vec::new();
    if slope_diff < 5.0 {
        naturally_matched.push(format!(
            "Terrain slope geometry ({:.1}° vs target {:.1}°)",
            cell.terrain.rms_slope_deg, target.terrain.rms_slope_deg
        ));
    }
    if (cell.terrain.roughness_m - target.terrain.roughness_m).abs() < 1.5 {
        naturally_matched.push("Surface macro-roughness and obstacle distribution".to_string());
    }
    if amp_diff < 15.0 {
        naturally_matched.push(format!(
            "Diurnal thermal swing amplitude ({:.1} K)",
            cell.thermal.diurnal_amp_k
        ));
    }
    if naturally_matched.is_empty() {
        naturally_matched.push("Partial mission-specific boundary conditions".to_string());
    }

    let mut gap_closures = Vec::new();

    if gap.temp_mean_gap > 0.05 {
        gap_closures.push(GapClosureRecommendation {
            condition: "Extreme cryogenic background temperature".to_string(),
            gap_value: gap.temp_mean_gap,
            status: "Equipment Required".to_string(),
            recommendation: "Use cryogenic thermal-vacuum chamber (TVAC) for hardware qualification.".to_string(),
        });
    }

    if gap.diurnal_amp_gap > 0.05 || gap.heating_rate_gap > 0.05 {
        gap_closures.push(GapClosureRecommendation {
            condition: "Rapid day-night thermal shock transitions".to_string(),
            gap_value: gap.diurnal_amp_gap.max(gap.heating_rate_gap),
            status: "Equipment Required".to_string(),
            recommendation: "Supplement field tests with programmable IR lamp arrays / thermal cycling.".to_string(),
        });
    }

    gap_closures.push(GapClosureRecommendation {
        condition: "Regolith geotechnical & electrostatic dust properties".to_string(),
        gap_value: 0.25,
        status: "Partial Natural Match".to_string(),
        recommendation: match target.body {
            CelestialBody::Moon => "Deploy lunar highland/mare regolith simulants (e.g., LHS-1, LMS-1) on testbed.".to_string(),
            CelestialBody::Mars => "Deploy Martian regolith simulant (e.g., MGS-1) for wheel-soil traction tests.".to_string(),
        },
    });

    let mut irreproducible = vec![
        match target.body {
            CelestialBody::Moon => "Reduced gravity (1/6 g) — requires parabolic flights or gravity-offload rig.".to_string(),
            CelestialBody::Mars => "Reduced gravity (3/8 g) — requires partial gravity suspension system.".to_string(),
        },
        "Deep-space ionizing radiation and solar particle events.".to_string(),
    ];

    match target.body {
        CelestialBody::Moon => {
            irreproducible.push("Ultra-high vacuum (absence of convective atmospheric heat transfer).".to_string());
            irreproducible.push("Multi-day continuous lunar night (354 hours) outside polar campaigns.".to_string());
        }
        CelestialBody::Mars => {
            irreproducible.push("Low-pressure CO2 atmosphere (~6 mbar) and global dust storm opacity.".to_string());
        }
    }

    AnalogDna {
        cell_id: cell.id.clone(),
        region_name: cell.region_name.clone(),
        lat: cell.lat,
        lon: cell.lon,
        target_name: target.name.clone(),
        mission,
        pareto_role: candidate.role.clone(),
        overall_similarity_pct: (score.similarity * 100.0).clamp(0.0, 100.0),
        terrain_similarity_pct: terrain_sim,
        thermal_similarity_pct: thermal_sim,
        novelty_score: score.combined_novelty,
        geo_novelty_km: score.geo_novelty_km,
        confidence_pct: (score.confidence * 100.0).clamp(0.0, 100.0),
        novelty_status,
        naturally_matched,
        gap_closures,
        irreproducible,
    }
}
