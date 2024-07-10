use crate::intersection::{Intersection, IntersectionBuilder};
use crate::light::Light;
use rand::distributions::Standard;

struct Simulation {
    intersection: Intersection,
    max_cars: u32,
    drive_steps_per_lightswitch: u32,
}

struct SimulationBuilder {
    simulation: Simulation,
}

impl SimulationBuilder {
    fn new() -> Self {
        SimulationBuilder {
            simulation: Simulation {
                intersection: IntersectionBuilder::new().build(),
                max_cars: 0,
                drive_steps_per_lightswitch: 0,
            },
        }
    }

    fn with_max_cars(mut self, max_cars: u32) -> Self {
        self.simulation.max_cars = max_cars;
        self
    }

    fn with_intersection(mut self, intersection: Intersection) -> Self {
        self.simulation.intersection = intersection;
        self
    }

    fn with_drive_steps_per_lightswitch(mut self, drive_steps_per_lightswitch: u32) -> Self {
        self.simulation.drive_steps_per_lightswitch = drive_steps_per_lightswitch;
        self
    }

    fn build(self) -> Simulation {
        self.simulation
    }
}

impl Simulation {
    pub(crate) fn spawn_random_car(&mut self) {
        if rand::random() && self.intersection.cars.len() < self.max_cars as usize {
            let light: Light = Light::random();
            self.intersection.spawn_car(light);
        }
    }
    /// TODO This gets factored into a Controller trait later.
    pub(crate) fn random_controller(&mut self) {
        if rand::random() {
            self.intersection.add_light(Light::N);
        }
        if rand::random() {
            self.intersection.add_light(Light::S);
        }
        if rand::random() {
            self.intersection.add_light(Light::E);
        }
        if rand::random() {
            self.intersection.add_light(Light::W);
        }
    }
    fn run(&mut self) {
        loop {
            self.spawn_random_car();
            for _ in 0..self.drive_steps_per_lightswitch {
                self.intersection.advance();
            }
            self.random_controller();
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_simulation_() {
        let intersection = IntersectionBuilder::new().build();
        let simulation = SimulationBuilder::new()
            .with_intersection(intersection)
            .with_max_cars(16)
            .with_drive_steps_per_lightswitch(8)
            .build();
    }
}
