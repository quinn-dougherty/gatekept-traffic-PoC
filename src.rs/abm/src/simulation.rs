use crate::intersection::{Intersection, IntersectionBuilder};
use crate::light::Light;

trait Controller: Default {
    fn control(&self, intersection: &mut Intersection);
}

pub struct Simulation<C: Controller> {
    intersection: Intersection,
    max_cars: u32,
    drive_steps_per_lightswitch: u32,
    max_steps: u32,
    controller: C,
    random_seed: usize,
}

pub struct SimulationBuilder<C: Controller> {
    simulation: Simulation<C>,
}

impl<C: Controller> SimulationBuilder<C> {
    pub fn new() -> Self {
        SimulationBuilder {
            simulation: Simulation {
                intersection: IntersectionBuilder::new().build(),
                max_cars: 0,
                drive_steps_per_lightswitch: 0,
                max_steps: 0,
                controller: C::default(),
                random_seed: 0,
            },
        }
    }

    pub fn with_max_cars(mut self, max_cars: u32) -> Self {
        self.simulation.max_cars = max_cars;
        self
    }

    pub fn with_intersection(mut self, intersection: Intersection) -> Self {
        self.simulation.intersection = intersection;
        self
    }

    pub fn with_drive_steps_per_lightswitch(mut self, drive_steps_per_lightswitch: u32) -> Self {
        self.simulation.drive_steps_per_lightswitch = drive_steps_per_lightswitch;
        self
    }

    pub fn with_max_steps(mut self, max_steps: u32) -> Self {
        self.simulation.max_steps = max_steps;
        self
    }

    pub fn with_controller(mut self, controller: C) -> Self {
        self.simulation.controller = controller;
        self
    }

    pub fn with_random_seed(mut self, random_seed: usize) -> Self {
        self.simulation.random_seed = random_seed;
        self
    }

    pub fn build(self) -> Simulation<C> {
        self.simulation
    }
}

impl<C: Controller> Simulation<C> {
    pub fn intersection(&self) -> &Intersection {
        &self.intersection
    }
    pub fn max_cars(&self) -> u32 {
        self.max_cars
    }
    pub fn drive_steps_per_lightswitch(&self) -> u32 {
        self.drive_steps_per_lightswitch
    }
    pub fn max_steps(&self) -> u32 {
        self.max_steps
    }
    pub fn controller(&self) -> &C {
        &self.controller
    }
    pub fn random_seed(&self) -> usize {
        self.random_seed
    }
}

#[derive(Default)]
pub struct Random;

impl Controller for Random {
    fn control(&self, intersection: &mut Intersection) {
        intersection.remove_all_lights();
        if rand::random() {
            intersection.add_light(Light::N);
        }
        if rand::random() {
            intersection.add_light(Light::S);
        }
        if rand::random() {
            intersection.add_light(Light::E);
        }
        if rand::random() {
            intersection.add_light(Light::W);
        }
    }
}

impl<C: Controller> Simulation<C> {
    pub(crate) fn spawn_random_car(&mut self) {
        if rand::random() && self.intersection.cars.len() < self.max_cars as usize {
            let light: Light = Light::random();
            self.intersection.spawn_car(light);
        }
    }

    /// Ask the controller to select an action and run it.
    pub(crate) fn ask_controller(&mut self) {
        self.controller.control(&mut self.intersection);
    }

    pub fn run(&mut self) {
        let mut count = 0;
        loop {
            for _ in 0..self.drive_steps_per_lightswitch {
                if rand::random() {
                    self.spawn_random_car();
                }
                self.intersection.advance();
            }
            self.ask_controller();
            count += 1;
            if count > self.max_steps {
                break;
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_simulation_() {
        let intersection = IntersectionBuilder::new().build();
        let simulation = SimulationBuilder::<Random>::new()
            .with_intersection(intersection)
            .with_max_cars(16)
            .with_drive_steps_per_lightswitch(8)
            .with_max_steps(512)
            .build();
    }
}
