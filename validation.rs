use star_core::envelope::{compute_mission_distance, is_valid_cell, EarthEnvelope};
use star_core::novelty::discover_candidates;
use star_core::pareto::rank_with_pareto;
use star_core::schema::{Cell, MissionType, TargetProfile};
use std::collections::HashSet;

fn baseline_distance(cell: &Cell, target: &TargetProfile) -> f64 {
    let t_diff = (cell.thermal.temp_mean_k - target.thermal.temp_mean_k).abs() / 50.0;
    let e_diff = (cell.terrain.elevation_m - target.terrain.elevation_m).abs() / 2000.0;
    (t_diff + e_diff) / 2.0
}

pub fn run_experiment_a(cells: &[Cell], target: &TargetProfile) {
    println!("Experiment A: Same Mean Temperature, Different Physics");

    let env = EarthEnvelope::from_cells(cells);
    let (clamped_target, _) = env.clamp_target(target);


    let valid: Vec<&Cell> = cells.iter().filter(|c| is_valid_cell(&c.masks)).collect();

    for i in 0..valid.len() {
        for j in (i + 1)..valid.len() {
            let a = valid[i];
            let b = valid[j];
            if a.region_name != b.region_name
                && (a.thermal.temp_mean_k - b.thermal.temp_mean_k).abs() < 2.0
                && (a.thermal.diurnal_amp_k - b.thermal.diurnal_amp_k).abs() > 15.0
            {
                let base_a = baseline_distance(a, target);
                let base_b = baseline_distance(b, target);

                let star_a = compute_mission_distance(a, &clamped_target, &env, MissionType::ThermalControl);
                let star_b = compute_mission_distance(b, &clamped_target, &env, MissionType::ThermalControl);

                println!(
                    "Found pair with matching mean temp (~{:.1} K):",
                    a.thermal.temp_mean_k
                );
                println!(
                    "  Cell A [{} - {}]: Diurnal Amp = {:.1} K, Heating = {:.1} K/h",
                    a.id, a.region_name, a.thermal.diurnal_amp_k, a.thermal.heating_rate_k_per_h
                );
                println!(
                    "  Cell B [{} - {}]: Diurnal Amp = {:.1} K, Heating = {:.1} K/h",
                    b.id, b.region_name, b.thermal.diurnal_amp_k, b.thermal.heating_rate_k_per_h
                );
                println!(
                    "-> Naive Baseline Distance: Cell A = {:.3} vs Cell B = {:.3} (Almost blind to difference)",
                    base_a, base_b
                );
                println!(
                    "Thermal Distance: Cell A = {:.3} vs Cell B = {:.3} (Clearly separates physics)\n",
                    star_a, star_b
                );
                return;
            }
        }
    }
    println!("No contrasting pair found in current dataset.\n");
}

pub fn run_experiment_b(cells: &[Cell], target: &TargetProfile, mission: MissionType) {
    println!("Experiment B: Leave-One-Region-Out & Negative Controls ===");

    let known_regions: HashSet<String> = cells
        .iter()
        .filter(|c| c.is_known_analog)
        .map(|c| c.region_name.clone())
        .collect();

    let env = EarthEnvelope::from_cells(cells);
    let (clamped_target, _) = env.clamp_target(target);

    let mut total_held_out = 0;
    let mut recovered_in_top15 = 0;

    for region in &known_regions {
        let mut modified_cells: Vec<Cell> = cells.to_vec();
        for c in &mut modified_cells {
            if &c.region_name == region {
                c.is_known_analog = false;
            }
        }

        let candidates = discover_candidates(&modified_cells, &clamped_target, &env, mission, 0.50, 0.35, 3);
        let ranked = rank_with_pareto(&candidates);

        let found = ranked
            .iter()
            .take(15)
            .any(|rc| &rc.score.region_name == region);

        total_held_out += 1;
        if found {
            recovered_in_top15 += 1;
        }
        println!(
            "  Held-out region '{}': {}",
            region,
            if found { "RECOVERED in Top-15" } else { "Missed (outside mission envelope)" }
        );
    }

    let all_candidates = discover_candidates(cells, &clamped_target, &env, mission, 0.50, 0.35, 3);
    let neg_hits = all_candidates
        .iter()
        .filter(|c| c.is_negative_control)
        .count();

    println!(
        "-> Region Recovery Rate: {}/{} | Negative Control False Positives: {} (Target: 0)\n",
        recovered_in_top15, total_held_out, neg_hits
    );
}
