use holodeck::gatekeeper::{Gatekeeper, GatekeeperBuilder};
use holodeck::logic::syntax::Prop;
use holodeck::traffic::intersection::IntersectionBuilder;
use holodeck::traffic::simulation::{Random as RandomController, SimulationBuilder};
use holodeck::traffic::trajectory::TrajectoryEntry;

fn main() {
    let n = 16;
    let intersection = IntersectionBuilder::new().build();
    let simulation = SimulationBuilder::<RandomController>::new()
        .with_intersection(intersection.clone())
        .with_max_cars(16)
        .with_drive_steps_per_lightswitch(8)
        .with_max_steps(n)
        .build();
    let world = SimulationBuilder::<RandomController>::new()
        .with_intersection(intersection)
        .with_max_cars(16)
        .with_drive_steps_per_lightswitch(8)
        .with_max_steps(n)
        .build();
    let controller = RandomController::default();
    let traffic_safety: Prop<TrajectoryEntry> = // TODO: ???
        Prop::Var(vec![TrajectoryEntry::new(0, 0); n as usize + 1]).always();
    let mut gatekeeper: Gatekeeper<RandomController, TrajectoryEntry> =
        GatekeeperBuilder::new(simulation, world)
            .with_controller(controller)
            .with_spec(traffic_safety)
            .build();
    gatekeeper.run();
}
