use crate::schema::{Cell, MissionType, QualityMasks, TargetProfile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureBounds {
    pub min: f64,
    pub max: f64,
    pub scale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarthEnvelope {
    pub rms_slope: FeatureBounds,
    pub roughness: FeatureBounds,
    pub spectrum_low: FeatureBounds,
    pub spectrum_mid: FeatureBounds,
    pub spectrum_high: FeatureBounds,
    pub temp_mean: FeatureBounds,
    pub diurnal_amp: FeatureBounds,
    pub peak_lag: FeatureBounds,
    pub heating_rate: FeatureBounds,
    pub cooling_rate: FeatureBounds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogGap {
    pub rms_slope_gap: f64,
    pub roughness_gap: f64,
    pub temp_mean_gap: f64,
    pub diurnal_amp_gap: f64,
    pub heating_rate_gap: f64,
    pub cooling_rate_gap: f64,
    pub total_gap_score: f64,
}

pub fn is_valid_cell(masks: &QualityMasks) -> bool {
    masks.not_water
        && masks.not_ice
        && masks.not_dense_vegetation
        && masks.not_urban_light
        && masks.valid_data_ratio >= 0.7
}

fn compute_bounds(values: &mut [f64]) -> FeatureBounds {
    if values.is_empty() {
        return FeatureBounds { min: 0.0, max: 1.0, scale: 1.0 };
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = values.len();
    let idx_01 = ((n as f64) * 0.01).floor() as usize;
    let idx_99 = (((n as f64) * 0.99).ceil() as usize).min(n - 1);

    let min = values[idx_01];
    let max = values[idx_99];
    let scale = if (max - min).abs() < 1e-9 { 1.0 } else { max - min };

    FeatureBounds { min, max, scale }
}

impl EarthEnvelope {
    pub fn from_cells(cells: &[Cell]) -> Self {
        let valid: Vec<&Cell> = cells.iter().filter(|c| is_valid_cell(&c.masks)).collect();

        let mut slopes: Vec<f64> = valid.iter().map(|c| c.terrain.rms_slope_deg).collect();
        let mut roughs: Vec<f64> = valid.iter().map(|c| c.terrain.roughness_m).collect();
        let mut spec_l: Vec<f64> = valid.iter().map(|c| c.terrain.spectrum_low).collect();
        let mut spec_m: Vec<f64> = valid.iter().map(|c| c.terrain.spectrum_mid).collect();
        let mut spec_h: Vec<f64> = valid.iter().map(|c| c.terrain.spectrum_high).collect();

        let mut t_mean: Vec<f64> = valid.iter().map(|c| c.thermal.temp_mean_k).collect();
        let mut d_amp: Vec<f64> = valid.iter().map(|c| c.thermal.diurnal_amp_k).collect();
        let mut p_lag: Vec<f64> = valid.iter().map(|c| c.thermal.peak_lag_hours).collect();
        let mut h_rate: Vec<f64> = valid.iter().map(|c| c.thermal.heating_rate_k_per_h).collect();
        let mut c_rate: Vec<f64> = valid.iter().map(|c| c.thermal.cooling_rate_k_per_h).collect();

        Self {
            rms_slope: compute_bounds(&mut slopes),
            roughness: compute_bounds(&mut roughs),
            spectrum_low: compute_bounds(&mut spec_l),
            spectrum_mid: compute_bounds(&mut spec_m),
            spectrum_high: compute_bounds(&mut spec_h),
            temp_mean: compute_bounds(&mut t_mean),
            diurnal_amp: compute_bounds(&mut d_amp),
            peak_lag: compute_bounds(&mut p_lag),
            heating_rate: compute_bounds(&mut h_rate),
            cooling_rate: compute_bounds(&mut c_rate),
        }
    }

    pub fn clamp_target(&self, target: &TargetProfile) -> (TargetProfile, AnalogGap) {
        let mut clamped = target.clone();

        let clamp_val = |val: f64, b: &FeatureBounds| -> (f64, f64) {
            let c = val.clamp(b.min, b.max);
            let gap = (val - c).abs() / b.scale;
            (c, gap)
        };

        let (c_slope, g_slope) = clamp_val(target.terrain.rms_slope_deg, &self.rms_slope);
        let (c_rough, g_rough) = clamp_val(target.terrain.roughness_m, &self.roughness);
        clamped.terrain.rms_slope_deg = c_slope;
        clamped.terrain.roughness_m = c_rough;

        let (c_tmean, g_tmean) = clamp_val(target.thermal.temp_mean_k, &self.temp_mean);
        let (c_amp, g_amp) = clamp_val(target.thermal.diurnal_amp_k, &self.diurnal_amp);
        let (c_heat, g_heat) = clamp_val(target.thermal.heating_rate_k_per_h, &self.heating_rate);
        let (c_cool, g_cool) = clamp_val(target.thermal.cooling_rate_k_per_h, &self.cooling_rate);

        clamped.thermal.temp_mean_k = c_tmean;
        clamped.thermal.diurnal_amp_k = c_amp;
        clamped.thermal.heating_rate_k_per_h = c_heat;
        clamped.thermal.cooling_rate_k_per_h = c_cool;

        let total_gap = (g_slope + g_rough + g_tmean + g_amp + g_heat + g_cool) / 6.0;

        let gap = AnalogGap {
            rms_slope_gap: g_slope,
            roughness_gap: g_rough,
            temp_mean_gap: g_tmean,
            diurnal_amp_gap: g_amp,
            heating_rate_gap: g_heat,
            cooling_rate_gap: g_cool,
            total_gap_score: total_gap,
        };

        (clamped, gap)
    }
}

pub fn compute_mission_distance(
    cell: &Cell,
    clamped_target: &TargetProfile,
    env: &EarthEnvelope,
    mission: MissionType,
) -> f64 {
    let d_slope = (cell.terrain.rms_slope_deg - clamped_target.terrain.rms_slope_deg).abs() / env.rms_slope.scale;
    let d_rough = (cell.terrain.roughness_m - clamped_target.terrain.roughness_m).abs() / env.roughness.scale;
    let d_spec = (
        (cell.terrain.spectrum_low - clamped_target.terrain.spectrum_low).abs() / env.spectrum_low.scale
            + (cell.terrain.spectrum_mid - clamped_target.terrain.spectrum_mid).abs() / env.spectrum_mid.scale
            + (cell.terrain.spectrum_high - clamped_target.terrain.spectrum_high).abs() / env.spectrum_high.scale
    ) / 3.0;
    let terrain_dist = (d_slope + d_rough + d_spec) / 3.0;

    let d_amp = (cell.thermal.diurnal_amp_k - clamped_target.thermal.diurnal_amp_k).abs() / env.diurnal_amp.scale;
    let d_lag = (cell.thermal.peak_lag_hours - clamped_target.thermal.peak_lag_hours).abs() / env.peak_lag.scale;
    let d_heat = (cell.thermal.heating_rate_k_per_h - clamped_target.thermal.heating_rate_k_per_h).abs() / env.heating_rate.scale;
    let d_cool = (cell.thermal.cooling_rate_k_per_h - clamped_target.thermal.cooling_rate_k_per_h).abs() / env.cooling_rate.scale;
    let thermal_dist = (d_amp + d_lag + d_heat + d_cool) / 4.0;

    match mission {
        MissionType::RoverMobility => {
            let penalty = if cell.terrain.rms_slope_deg > 25.0 { 1.0 } else { 0.0 };
            0.75 * terrain_dist + 0.25 * thermal_dist + penalty
        }
        MissionType::ThermalControl => {
            0.20 * terrain_dist + 0.80 * thermal_dist
        }
        MissionType::SolarPower => {
            0.60 * d_slope + 0.40 * thermal_dist
        }
    }
}
