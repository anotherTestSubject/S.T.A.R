use crate::envelope::{compute_mission_distance, is_valid_cell, EarthEnvelope};
use crate::schema::{Cell, MissionType, TargetProfile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateScore {
    pub cell_id: String,
    pub lat: f64,
    pub lon: f64,
    pub region_name: String,
    pub is_known_analog: bool,
    pub is_negative_control: bool,
    pub d_mission: f64,
    pub similarity: f64,
    pub feature_novelty: f64,
    pub geo_novelty_km: f64,
    pub combined_novelty: f64,
    pub confidence: f64,
}

fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0_f64;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

fn cell_feature_distance(a: &Cell, b: &Cell, env: &EarthEnvelope) -> f64 {
    let d_slope = (a.terrain.rms_slope_deg - b.terrain.rms_slope_deg).abs() / env.rms_slope.scale;
    let d_rough = (a.terrain.roughness_m - b.terrain.roughness_m).abs() / env.roughness.scale;
    let d_spec = ((a.terrain.spectrum_low - b.terrain.spectrum_low).abs() / env.spectrum_low.scale
        + (a.terrain.spectrum_mid - b.terrain.spectrum_mid).abs() / env.spectrum_mid.scale
        + (a.terrain.spectrum_high - b.terrain.spectrum_high).abs() / env.spectrum_high.scale)
        / 3.0;

    let d_amp = (a.thermal.diurnal_amp_k - b.thermal.diurnal_amp_k).abs() / env.diurnal_amp.scale;
    let d_lag = (a.thermal.peak_lag_hours - b.thermal.peak_lag_hours).abs() / env.peak_lag.scale;
    let d_heat =
        (a.thermal.heating_rate_k_per_h - b.thermal.heating_rate_k_per_h).abs() / env.heating_rate.scale;
    let d_cool =
        (a.thermal.cooling_rate_k_per_h - b.thermal.cooling_rate_k_per_h).abs() / env.cooling_rate.scale;

    (d_slope + d_rough + d_spec + d_amp + d_lag + d_heat + d_cool) / 7.0
}

pub fn discover_candidates(
    cells: &[Cell],
    clamped_target: &TargetProfile,
    env: &EarthEnvelope,
    mission: MissionType,
    epsilon: f64,
    u_max: f64,
    k_neighbors: usize,
) -> Vec<CandidateScore> {
    let known_analogs: Vec<&Cell> = cells
        .iter()
        .filter(|c| c.is_known_analog && is_valid_cell(&c.masks))
        .collect();

    let mut results = Vec::new();

    for cell in cells {
        if !is_valid_cell(&cell.masks) {
            continue;
        }

        if cell.uncertainty > u_max {
            continue;
        }

        let d_mission = compute_mission_distance(cell, clamped_target, env, mission);

        if d_mission > epsilon {
            continue;
        }

        let similarity = 1.0 / (1.0 + d_mission);

        let (feature_novelty, geo_novelty_km) = if known_analogs.is_empty() {
            (0.0, 0.0)
        } else {
            let mut feat_dists: Vec<f64> = known_analogs
                .iter()
                .map(|known| cell_feature_distance(cell, known, env))
                .collect();
            feat_dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let k = k_neighbors.min(feat_dists.len()).max(1);
            let avg_k_dist = feat_dists[..k].iter().sum::<f64>() / (k as f64);

            let min_geo_km = known_analogs
                .iter()
                .map(|known| haversine_km(cell.lat, cell.lon, known.lat, known.lon))
                .fold(f64::INFINITY, f64::min);

            (avg_k_dist, min_geo_km)
        };

        let geo_norm = (geo_novelty_km / 2000.0).min(1.5);
        let combined_novelty = 0.6 * feature_novelty + 0.4 * geo_norm;

        results.push(CandidateScore {
            cell_id: cell.id.clone(),
            lat: cell.lat,
            lon: cell.lon,
            region_name: cell.region_name.clone(),
            is_known_analog: cell.is_known_analog,
            is_negative_control: cell.is_negative_control,
            d_mission,
            similarity,
            feature_novelty,
            geo_novelty_km,
            combined_novelty,
            confidence: (1.0 - cell.uncertainty).clamp(0.0, 1.0),
        });
    }

    results
}
