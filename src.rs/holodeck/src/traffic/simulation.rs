use std::collections::HashSet;

use rand::rngs::ThreadRng;
use rand::Rng;

// use crate::data::rng::{Rng, RngSeed};
use crate::traffic::intersection::{Intersection, IntersectionBuilder};
use crate::traffic::light::Light;
use crate::traffic::trajectory::{Trajectory, TrajectoryEntry};

pub trait Controller: Default + Clone {
    fn select_action(&self, rng: &mut ThreadRng) -> HashSet<Light>;
    fn control(&self, intersection: &mut Intersection, rng: &mut ThreadRng);
}

#[derive(Clone)]
pub struct Simulation<C: Controller> {
    intersection: Intersection,
    max_cars: u32,
    drive_steps_per_lightswitch: u32,
    max_steps: u32,
    controller: C,
}

pub type World<C> = Simulation<C>;

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
}

#[derive(Default, Clone)]
pub struct Random;

impl Controller for Random {
    // TODO: make this accord with gymnasium. signature should take current state as input.
    fn select_action(&self, rng: &mut ThreadRng) -> HashSet<Light> {
        let mut result = HashSet::new();
        if rng.gen::<bool>() {
            result.insert(Light::N);
        }
        if rng.gen::<bool>() {
            result.insert(Light::S);
        }
        if rng.gen::<bool>() {
            result.insert(Light::E);
        }
        if rng.gen::<bool>() {
            result.insert(Light::W);
        }
        result
    }
    fn control(&self, intersection: &mut Intersection, rng: &mut ThreadRng) {
        intersection.remove_all_lights();
        for light in self.select_action(rng) {
            intersection.add_light(light);
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
    pub(crate) fn ask_controller(&mut self, rng: &mut ThreadRng) {
        self.controller.control(&mut self.intersection, rng);
    }

    /// Advance the simulation forward, adding new cars sometimes.
    pub(crate) fn drive_between_lightswitch(&mut self, rng: &mut ThreadRng) {
        for _ in 0..self.drive_steps_per_lightswitch {
            if rng.gen::<bool>() {
                // TODO: pass through seed (to make it obvious to everyone that there's randomness here)
                self.spawn_random_car();
            }
            self.intersection.advance();
        }
    }

    pub fn run(&mut self, rng: &mut ThreadRng) {
        for _ in 0..self.max_steps {
            self.drive_between_lightswitch(rng);
            self.ask_controller(rng);
        }
    }

    pub fn run_recording_trajectory(
        &mut self,
        _action: HashSet<Light>,
        rng: &mut ThreadRng,
    ) -> Trajectory {
        let mut trajectory: Trajectory = Vec::new();
        let mut previous_crashes = 0;
        let mut previous_throughput = 0;

        for _ in 0..self.max_steps {
            // Run a single step
            self.drive_between_lightswitch(rng);
            self.ask_controller(rng); // refactor this line to put randomness outside of the function

            // Calculate the changes in this step
            let crashes_after = self.intersection.num_crashes();
            let throughput_after = self.intersection.total_throughput();
            if false {
                println!("Crashes: {}", crashes_after - previous_crashes);
            }
            let entry = TrajectoryEntry::new(
                crashes_after - previous_crashes,
                throughput_after - previous_throughput,
            );
            trajectory.push(entry);

            // Update the previous values for the next iteration
            previous_crashes = crashes_after;
            previous_throughput = throughput_after;
        }
        trajectory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    use crate::cfg::cfg;

    #[test]
    fn simulation_sum_numcrashes_local_equals_numcraches() {
        fn run(k: u32) -> Result<(), TestCaseError> {
            let max_timestamp: u32 = cfg().get::<u32>("max_timestamp").unwrap();
            let intersection = IntersectionBuilder::new().build();
            let mut simulation = SimulationBuilder::<Random>::new()
                .with_intersection(intersection)
                .with_max_cars(16)
                .with_drive_steps_per_lightswitch(8)
                .with_max_steps(max_timestamp)
                .build();
            let mut prng = rand::thread_rng();
            let mut num_crashes = 0;
            for _ in 0..k {
                let action = simulation.controller.select_action(&mut prng);
                let trajectory = simulation.run_recording_trajectory(action, &mut prng);
                num_crashes += trajectory
                    .iter()
                    .map(|entry| entry.num_crashes_local())
                    .sum::<u32>();
            }
            prop_assert_eq!(simulation.intersection.num_crashes(), num_crashes);
            Ok(())
        }
        proptest!(|(k in 0u32..8u32)| {
            let _ = run(k);
        });
    }

    #[test]
    fn simulation_run_nonzerocrashes() {
        fn run(k: u32) -> Result<(), TestCaseError> {
            let max_timestamp: u32 = cfg().get("max_timestamp").unwrap();
            let intersection = IntersectionBuilder::new().build();
            let mut simulation = SimulationBuilder::<Random>::new()
                .with_intersection(intersection)
                .with_max_cars(16)
                .with_drive_steps_per_lightswitch(8)
                .with_max_steps(max_timestamp)
                .build();
            let mut prng = rand::thread_rng();
            for _ in 0..k {
                let action = simulation.controller.select_action(&mut prng);
                let trajectory = simulation.run_recording_trajectory(action, &mut prng);
                prop_assert!(
                    trajectory
                        .iter()
                        .map(|entry| entry.num_crashes_local())
                        .sum::<u32>()
                        > 0
                );
            }
            Ok(())
        }
        proptest!(|(k in 0u32..8u32)| {
            let _ = run(k);
        });
    }
}
