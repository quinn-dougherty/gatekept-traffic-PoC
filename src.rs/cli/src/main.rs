use holodeck::gatekeeper::{Gatekeeper, GatekeeperBuilder};
use holodeck::logic::syntax::Prop;
use holodeck::traffic::intersection::IntersectionBuilder;
use holodeck::traffic::simulation::{Random as RandomController, SimulationBuilder};
use holodeck::traffic::trajectory::TrajectoryEntry;

fn main() {
    let intersection = IntersectionBuilder::new().build();
    let mut simulation = SimulationBuilder::<RandomController>::new()
        .with_intersection(intersection)
        .with_max_cars(16)
        .with_drive_steps_per_lightswitch(8)
        .with_max_steps(512)
        .build();
    let controller = RandomController::default();
    let traffic_safety: Prop<TrajectoryEntry> = // TODO: ???
        Prop::Var(vec![TrajectoryEntry::new(0, 0); 512 + 1]).always();
    let mut gatekeeper: Gatekeeper<RandomController, TrajectoryEntry> =
        GatekeeperBuilder::new(simulation.clone())
            .with_controller(controller)
            .with_spec(traffic_safety)
            .build();
    // let trajectory = gatekeeper.simulation().run_recording_trajectory();
    //    println!(
    //        "Num crashes: {:?} (from {:?} controller actions)",
    //        simulation.intersection().num_crashes(),
    //        simulation.max_steps()
    //    );
    //    println!(
    //        "total throughput: {:?}",
    //        trajectory
    //            .iter()
    //            .map(|entry| entry.num_cars_throughput())
    //            .sum::<u32>() as u32
    //    )
    gatekeeper.run();
}
