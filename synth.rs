use star_core::schema::{
    CelestialBody, Cell, QualityMasks, TargetProfile, TerrainFeatures, ThermalFeatures,
};

struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next_f64(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.0 >> 33) as f64) / (u32::MAX as f64)
    }
    fn range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_f64()
    }
}

pub fn generate_targets() -> Vec<TargetProfile> {
    vec![
        TargetProfile {
            id: "moon_shackleton".to_string(),
            name: "Shackleton Crater Rim (Moon South Pole)".to_string(),
            body: CelestialBody::Moon,
            description: "High-illumination polar rim adjacent to permanently shadowed regions. Extreme cold and low-sun shadows.".to_string(),
            terrain: TerrainFeatures {
                elevation_m: 1200.0,
                rms_slope_deg: 11.5,
                roughness_m: 1.8,
                spectrum_low: 0.65,
                spectrum_mid: 0.42,
                spectrum_high: 0.25,
            },
            thermal: ThermalFeatures {
                temp_mean_k: 110.0,
                diurnal_amp_k: 45.0,
                peak_lag_hours: 1.2,
                heating_rate_k_per_h: 4.5,
                cooling_rate_k_per_h: 4.2,
            },
        },
        TargetProfile {
            id: "mars_jezero".to_string(),
            name: "Jezero Crater Floor (Mars)".to_string(),
            body: CelestialBody::Mars,
            description: "Ancient lacustrine basin with basaltic/carbonate terrain and large diurnal temperature swings.".to_string(),
            terrain: TerrainFeatures {
                elevation_m: -2500.0,
                rms_slope_deg: 6.8,
                roughness_m: 1.1,
                spectrum_low: 0.40,
                spectrum_mid: 0.35,
                spectrum_high: 0.20,
            },
            thermal: ThermalFeatures {
                temp_mean_k: 215.0,
                diurnal_amp_k: 65.0,
                peak_lag_hours: 1.5,
                heating_rate_k_per_h: 8.5,
                cooling_rate_k_per_h: 7.8,
            },
        },
    ]
}

pub fn generate_synthetic_cells() -> Vec<Cell> {
    let mut rng = SimpleRng::new(42);
    let mut cells = Vec::new();
    let mut id_counter = 1;

    let mut add_region = |region: &str,
                          count: usize,
                          base_lat: f64,
                          base_lon: f64,
                          is_known: bool,
                          is_neg: bool,
                          masks: QualityMasks,
                          slope: (f64, f64),
                          rough: (f64, f64),
                          t_mean: (f64, f64),
                          d_amp: (f64, f64),
                          lag: (f64, f64),
                          heat: (f64, f64),
                          unc: (f64, f64)| {
        for _ in 0..count {
            let h_rate = rng.range(heat.0, heat.1);
            cells.push(Cell {
                id: format!("CELL_{:04}", id_counter),
                lat: base_lat + rng.range(-1.5, 1.5),
                lon: base_lon + rng.range(-1.5, 1.5),
                region_name: region.to_string(),
                is_known_analog: is_known,
                is_negative_control: is_neg,
                masks: masks.clone(),
                terrain: TerrainFeatures {
                    elevation_m: rng.range(500.0, 4000.0),
                    rms_slope_deg: rng.range(slope.0, slope.1),
                    roughness_m: rng.range(rough.0, rough.1),
                    spectrum_low: rng.range(0.35, 0.70),
                    spectrum_mid: rng.range(0.25, 0.50),
                    spectrum_high: rng.range(0.15, 0.35),
                },
                thermal: ThermalFeatures {
                    temp_mean_k: rng.range(t_mean.0, t_mean.1),
                    diurnal_amp_k: rng.range(d_amp.0, d_amp.1),
                    peak_lag_hours: rng.range(lag.0, lag.1),
                    heating_rate_k_per_h: h_rate,
                    cooling_rate_k_per_h: h_rate * rng.range(0.85, 1.05),
                },
                uncertainty: rng.range(unc.0, unc.1),
            });
            id_counter += 1;
        }
    };

    let clean_masks = QualityMasks {
        not_water: true,
        not_ice: true,
        not_dense_vegetation: true,
        not_urban_light: true,
        valid_data_ratio: 0.95,
    };

    add_region("Atacama_Desert", 40, -23.8, -69.1, true, false, clean_masks.clone(),
               (5.0, 9.0), (0.8, 1.5), (285.0, 295.0), (32.0, 42.0), (1.3, 1.8), (6.5, 8.5), (0.05, 0.15));

    add_region("Devon_Island_Haughton", 35, 75.4, -89.8, true, false, clean_masks.clone(),
               (9.0, 14.0), (1.4, 2.2), (250.0, 262.0), (15.0, 25.0), (1.0, 1.5), (3.5, 5.0), (0.10, 0.20));

    add_region("Mauna_Kea_Volcanic", 30, 19.8, -155.4, true, false, clean_masks.clone(),
               (10.0, 16.0), (1.5, 2.5), (275.0, 283.0), (22.0, 30.0), (1.2, 1.7), (4.5, 6.5), (0.08, 0.18));

    add_region("Puna_de_Atacama_Highlands", 25, -25.5, -67.5, false, false, clean_masks.clone(),
               (6.0, 11.0), (1.0, 1.7), (268.0, 278.0), (38.0, 48.0), (1.2, 1.6), (7.5, 9.2), (0.06, 0.14));

    add_region("Qaidam_Basin_Tibet", 30, 37.5, 94.0, false, false, clean_masks.clone(),
               (5.5, 8.5), (0.9, 1.4), (270.0, 280.0), (35.0, 46.0), (1.3, 1.8), (7.0, 8.8), (0.07, 0.16));

    let jungle_masks = QualityMasks {
        not_water: true,
        not_ice: true,
        not_dense_vegetation: false,
        not_urban_light: true,
        valid_data_ratio: 0.85,
    };
    add_region("Amazon_Basin", 50, -3.4, -62.2, false, true, jungle_masks,
               (0.5, 3.0), (0.2, 0.6), (298.0, 303.0), (6.0, 10.0), (3.5, 5.0), (1.5, 2.5), (0.15, 0.25));

    let urban_masks = QualityMasks {
        not_water: true,
        not_ice: true,
        not_dense_vegetation: true,
        not_urban_light: false,
        valid_data_ratio: 0.98,
    };
    add_region("Urban_Heat_Island_Trap", 20, 33.4, -112.0, false, false, urban_masks,
               (6.0, 8.0), (1.0, 1.2), (295.0, 305.0), (45.0, 60.0), (1.4, 1.6), (8.0, 9.0), (0.02, 0.05));

    add_region("Generic_Steppe_Background", 200, 48.0, 65.0, false, false, clean_masks,
               (1.0, 22.0), (0.3, 3.5), (265.0, 295.0), (10.0, 28.0), (2.0, 4.0), (2.0, 6.0), (0.10, 0.45));

    cells
}
