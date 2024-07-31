use holodeck::cfg::cfg;
use holodeck::gatekeeper::{Gatekeeper, GatekeeperBuilder};
use holodeck::logic::syntax::Prop;
use holodeck::traffic::intersection::IntersectionBuilder;
use holodeck::traffic::simulation::{Random as RandomController, SimulationBuilder};
use holodeck::traffic::trajectory::TrajectoryEntry;

fn main() {
    let n: u32 = cfg().get("max_timestamp").unwrap();
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
    fn traffic_safety(v: Vec<TrajectoryEntry>) -> Prop<TrajectoryEntry> {
        Prop::Var(v).always()
    }
    let _baseline = vec![TrajectoryEntry::new(0, 0); n as usize + 1];
    let mut gatekeeper: Gatekeeper<RandomController, TrajectoryEntry> =
        GatekeeperBuilder::new(simulation, world)
            .with_controller(controller)
            .with_spec(traffic_safety)
            .build();
    gatekeeper.run();
}
