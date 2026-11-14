use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CelestialBody {
    Moon,
    Mars,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MissionType {
    RoverMobility,
    ThermalControl,
    SolarPower,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMasks {
    pub not_water: bool,
    pub not_ice: bool,
    pub not_dense_vegetation: bool,
    pub not_urban_light: bool,
    pub valid_data_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainFeatures {
    pub elevation_m: f64,
    pub rms_slope_deg: f64,
    pub roughness_m: f64,
    pub spectrum_low: f64,
    pub spectrum_mid: f64,
    pub spectrum_high: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalFeatures {
    pub temp_mean_k: f64,
    pub diurnal_amp_k: f64,
    pub peak_lag_hours: f64,
    pub heating_rate_k_per_h: f64,
    pub cooling_rate_k_per_h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    pub region_name: String,
    pub is_known_analog: bool,
    pub is_negative_control: bool,
    pub masks: QualityMasks,
    pub terrain: TerrainFeatures,
    pub thermal: ThermalFeatures,
    pub uncertainty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetProfile {
    pub id: String,
    pub name: String,
    pub body: CelestialBody,
    pub description: String,
    pub terrain: TerrainFeatures,
    pub thermal: ThermalFeatures,
}
