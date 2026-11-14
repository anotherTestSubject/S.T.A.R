mod validation;
mod synth;

use star_core::run_discovery;
use star_core::schema::MissionType;
use std::fs;
use std::path::Path;

fn main() {
    println!("Core Pipeline Check\n");

    let targets = synth::generate_targets();
    let cells = synth::generate_synthetic_cells();

    let data_dir = Path::new("data");
    if !data_dir.exists() {
        fs::create_dir(data_dir).expect("Failed to create data directory");
    }

    fs::write(
        data_dir.join("targets.json"),
        serde_json::to_string_pretty(&targets).unwrap(),
    )
        .unwrap();

    fs::write(
        data_dir.join("cells.json"),
        serde_json::to_string_pretty(&cells).unwrap(),
    )
        .unwrap();

    println!(
        "Loaded {} planetary targets and {} Earth grid cells into data/.\n",
        targets.len(),
        cells.len()
    );

    let moon_target = &targets[0];
    let moon_report = run_discovery(
        &cells,
        moon_target,
        MissionType::RoverMobility,
        0.45,
        0.30,
        5,
    );

    println!(
        "Target: {} | Mission: {:?}",
        moon_report.target.name, moon_report.mission
    );
    println!(
        "Earth-Envelope Temp Gap: {:.2} (0.00 = fully reachable on Earth)",
        moon_report.analog_gap.temp_mean_gap
    );
    println!(
        "Passed C.N.D. constraints: {} / {} cells",
        moon_report.candidates_passed_constraints, moon_report.total_cells_evaluated
    );
    println!("Top Pareto Candidates:");
    for (idx, dna) in moon_report.top_analogs.iter().enumerate() {
        println!(
            "  {}. [{}] {} — {} | Match: {:.1}% | Novelty: {:.2} ({:.0} km away) | Role: {:?}",
            idx + 1,
            dna.cell_id,
            dna.region_name,
            dna.novelty_status,
            dna.overall_similarity_pct,
            dna.novelty_score,
            dna.geo_novelty_km,
            dna.pareto_role
        );
    }

    println!("\n--------------------------------------------------\n");

    let mars_target = &targets[1];
    let mars_report = run_discovery(
        &cells,
        mars_target,
        MissionType::ThermalControl,
        0.45,
        0.30,
        5,
    );

    println!(
        "Target: {} | Mission: {:?}",
        mars_report.target.name, mars_report.mission
    );
    println!(
        "Earth-Envelope Temp Gap: {:.2}",
        mars_report.analog_gap.temp_mean_gap
    );
    println!(
        "Passed C.N.D. constraints: {} / {} cells",
        mars_report.candidates_passed_constraints, mars_report.total_cells_evaluated
    );
    println!("Top Pareto Candidates:");
    for (idx, dna) in mars_report.top_analogs.iter().enumerate() {
        println!(
            "  {}. [{}] {} — {} | Match: {:.1}% | Novelty: {:.2} ({:.0} km away) | Role: {:?}",
            idx + 1,
            dna.cell_id,
            dna.region_name,
            dna.novelty_status,
            dna.overall_similarity_pct,
            dna.novelty_score,
            dna.geo_novelty_km,
            dna.pareto_role
        );
    }
    println!("\n===============================================\n");
    validation::run_experiment_a(&cells, mars_target);
    validation::run_experiment_b(&cells, mars_target, MissionType::ThermalControl);
}

