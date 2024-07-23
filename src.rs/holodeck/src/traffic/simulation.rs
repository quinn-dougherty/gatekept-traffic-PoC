use std::collections::HashSet;

use crate::traffic::intersection::{Intersection, IntersectionBuilder};
use crate::traffic::light::Light;
use crate::traffic::trajectory::{Trajectory, TrajectoryEntry};

static DEBUG: bool = true;
static N: u32 = 512;

pub trait Controller: Default + Clone {
    fn select_action(&self) -> HashSet<Light>;
    fn control(&self, intersection: &mut Intersection);
}

#[derive(Clone)]
pub struct Simulation<C: Controller> {
    intersection: Intersection,
    max_cars: u32,
    drive_steps_per_lightswitch: u32,
    max_steps: u32,
    controller: C,
    random_seed: usize,
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

#[derive(Default, Clone)]
pub struct Random;

impl Controller for Random {
    // TODO: make this accord with gymnasium. signature should take current state as input.
    fn select_action(&self) -> HashSet<Light> {
        let mut result = HashSet::new();
        if rand::random() {
            result.insert(Light::N);
        }
        if rand::random() {
            result.insert(Light::S);
        }
        if rand::random() {
            result.insert(Light::E);
        }
        if rand::random() {
            result.insert(Light::W);
        }
        result
    }
    fn control(&self, intersection: &mut Intersection) {
        intersection.remove_all_lights();
        for light in self.select_action() {
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
    pub(crate) fn ask_controller(&mut self) {
        self.controller.control(&mut self.intersection);
    }

    /// Advance the simulation forward, adding new cars sometimes.
    pub(crate) fn drive_between_lightswitch(&mut self) {
        for _ in 0..self.drive_steps_per_lightswitch {
            if rand::random() {
                // TODO: pass through seed (to make it obvious to everyone that there's randomness here)
                self.spawn_random_car();
            }
            self.intersection.advance();
        }
    }

    pub fn run(&mut self) {
        for _ in 0..self.max_steps {
            self.drive_between_lightswitch();
            self.ask_controller();
        }
    }

    pub fn run_recording_trajectory(&mut self, action: HashSet<Light>) -> Trajectory {
        let mut trajectory: Trajectory = Vec::new();
        let mut previous_crashes = 0;
        let mut previous_throughput = 0;

        for _ in 0..self.max_steps {
            // Run a single step
            self.drive_between_lightswitch();
            self.ask_controller(); // refactor this line to put randomness outside of the function

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
pub mod tests {
    use super::*;

    #[test]
    fn test_simulation_sum_numcrashes_local_equals_numcraches() {
        let intersection = IntersectionBuilder::new().build();
        let mut simulation = SimulationBuilder::<Random>::new()
            .with_intersection(intersection)
            .with_max_cars(16)
            .with_drive_steps_per_lightswitch(8)
            .with_max_steps(N)
            .build();
        let action = simulation.controller.select_action();
        let trajectory = simulation.run_recording_trajectory(action);
        assert_eq!(
            simulation.intersection.num_crashes(),
            trajectory
                .iter()
                .map(|entry| entry.num_crashes_local())
                .sum()
        )
    }

    #[test]
    fn test_simulation_run_nonzerocrashes() {
        let intersection = IntersectionBuilder::new().build();
        let mut simulation = SimulationBuilder::<Random>::new()
            .with_intersection(intersection)
            .with_max_cars(16)
            .with_drive_steps_per_lightswitch(8)
            .with_max_steps(N)
            .build();
        let action = simulation.controller.select_action();
        let trajectory = simulation.run_recording_trajectory(action);
        assert!(
            trajectory
                .iter()
                .map(|entry| entry.num_crashes_local())
                .sum::<u32>()
                > 0
        )
    }
    // A test here ought to be that run_recording_trajectory should have nonzero crashes.
}
