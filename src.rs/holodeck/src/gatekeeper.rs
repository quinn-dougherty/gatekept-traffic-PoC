//! The gatekeeper on the traffic toy problem.
//!
//! We're going to /evaluate/ trajectories from the simulation with the LTL compiler.
//! TODO we have this new idea where "world" and "sim" do not have a granularity difference. think about this more.
//!     - instead, the atomic propositions will be `Trajectory`.
//!     - or maybe the terms should just be some hashable thing that can map to trajectory, or trajectories.
use crate::cfg::cfg;
use crate::logic::interpreter::interpret;
use crate::logic::syntax::Prop;
use crate::logic::types::{Atomic, Valuation};
use crate::traffic::simulation::{Controller, Simulation, World};
use crate::traffic::trajectory::{Trajectory, TrajectoryEntry};

pub struct Gatekeeper<C, T>
where
    C: Controller,
    T: Atomic + 'static,
{
    controller: C,
    simulation: Simulation<C>,
    world: World<C>,
    spec: Box<dyn Fn(Vec<T>) -> Prop<T>>,
}

impl<C, T> Gatekeeper<C, T>
where
    C: Controller,
    T: Atomic + 'static,
{
    // pub(crate) fn spec<F>(&self) -> Box<F>
    // where
    //     F: Fn(Vec<T>) -> Prop<T>,
    // {
    //     self.spec
    // }
    // missing getters cuz borrow checker and lifetimes.
    pub(crate) fn spec_at(&self, atom: Vec<T>) -> Prop<T> {
        (self.spec)(atom)
    }
}

pub struct GatekeeperBuilder<C, T>
where
    C: Controller,
    T: Atomic + 'static,
{
    gatekeeper: Gatekeeper<C, T>,
}

impl<C, T> GatekeeperBuilder<C, T>
where
    C: Controller,
    T: Atomic,
{
    pub fn new(simulation: Simulation<C>, world: World<C>) -> Self {
        fn spec<'a, T>(_: Vec<T>) -> Prop<T>
        where
            T: Atomic + 'static,
        {
            Prop::True
        }
        GatekeeperBuilder {
            gatekeeper: Gatekeeper {
                controller: C::default(),
                simulation,
                world,
                spec: Box::new(spec),
            },
        }
    }
    pub fn with_controller(mut self, controller: C) -> Self {
        self.gatekeeper.controller = controller;
        self
    }
    pub fn with_simulation(mut self, simulation: Simulation<C>) -> Self {
        self.gatekeeper.simulation = simulation;
        self
    }
    pub fn with_world(mut self, world: World<C>) -> Self {
        self.gatekeeper.world = world;
        self
    }
    pub fn with_spec<F>(mut self, spec: F) -> Self
    where
        F: Fn(Vec<T>) -> Prop<T> + 'static,
    {
        self.gatekeeper.spec = Box::new(spec);
        self
    }
    pub fn build(self) -> Gatekeeper<C, T> {
        self.gatekeeper
    }
}

static EPSILON: f64 = 1e-5;

impl<C> Gatekeeper<C, TrajectoryEntry>
where
    C: Controller,
{
    pub(crate) fn evaluate(&self, trajectory: Trajectory) -> Valuation {
        let time_horizon = trajectory.clone().len();
        (0..time_horizon)
            .map(|time| interpret(self.spec_at(trajectory.clone()), time))
            .sum::<f64>()
            / time_horizon as f64
    }

    pub fn run(&mut self) {
        let mut num_rejections = 0;
        loop {
            // Need in this loop to keep track of how many rejections there are.
            let action = self.controller.select_action();
            let trajectory_ofsim = self.simulation.run_recording_trajectory(action.clone());
            let proba_safe_ofsim = self.evaluate(trajectory_ofsim);
            if proba_safe_ofsim > 1.0 - EPSILON {
                let trajectory_ofworld = self.world.run_recording_trajectory(action);
                let proba_safe_ofworld = self.evaluate(trajectory_ofworld);
                if cfg().get("debug").unwrap() {
                    println!("Simulated trajectory was safe at {}", proba_safe_ofsim);
                    println!(
                        "We ran the action in the world and it was also safe at {}",
                        proba_safe_ofworld
                    );
                }
                if proba_safe_ofworld > 1.0 - EPSILON {
                    break;
                }
            } else {
                num_rejections += 1;
                if cfg().get("debug").unwrap() {
                    println!("Trajectory of is not safe at {}", proba_safe_ofsim);
                }
            }
        }
        if cfg().get("debug").unwrap() {
            println!("Number of rejections: {}", num_rejections);
        }
    }
}
