use holodeck::traffic::intersection::IntersectionBuilder;
use holodeck::traffic::simulation::{Random, SimulationBuilder};

fn main() {
    let intersection = IntersectionBuilder::new().build();
    let mut simulation = SimulationBuilder::<Random>::new()
        .with_intersection(intersection)
        .with_max_cars(16)
        .with_drive_steps_per_lightswitch(8)
        .with_max_steps(512)
        .build();
    simulation.run();
    println!(
        "Num crashes: {:?} (from {:?} controller actions)",
        simulation.intersection().num_crashes(),
        simulation.max_steps()
    );
}
