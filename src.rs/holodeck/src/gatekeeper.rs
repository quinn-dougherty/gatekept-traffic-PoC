//! The gatekeeper on the traffic toy problem.
//!
//! We're going to /evaluate/ trajectories from the simulation with the LTL compiler.
//! TODO we have this new idea where "world" and "sim" do not have a granularity difference. think about this more.
//!     - instead, the atomic propositions will be `Trajectory`.
//!     - or maybe the terms should just be some hashable thing that can map to trajectory, or trajectories.
use crate::logic::interpreter::interpret;
use crate::logic::syntax::Prop;
use crate::logic::types::{Atomic, Valuation};
use crate::traffic::simulation::{Controller, Simulation};
use crate::traffic::trajectory::Trajectory;

pub struct Gatekeeper<C, T>
where
    C: Controller,
    T: Atomic,
{
    controller: C,
    simulation: Simulation<C>,
    spec: Prop<T>,
}

impl<C, T> Gatekeeper<C, T>
where
    C: Controller,
    T: Atomic,
{
    pub(crate) fn spec(&self) -> Prop<T> {
        self.spec.clone()
    }
    // missing two getters cuz borrow checker.
}

pub struct GatekeeperBuilder<C, T>
where
    C: Controller,
    T: Atomic,
{
    gatekeeper: Gatekeeper<C, T>,
}

impl<C, T> GatekeeperBuilder<C, T>
where
    C: Controller,
    T: Atomic,
{
    pub fn new(simulation: Simulation<C>) -> Self {
        GatekeeperBuilder {
            gatekeeper: Gatekeeper {
                controller: C::default(),
                simulation,
                spec: Prop::True,
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
    pub fn with_spec(mut self, spec: Prop<T>) -> Self {
        self.gatekeeper.spec = spec;
        self
    }
    pub fn build(self) -> Gatekeeper<C, T> {
        self.gatekeeper
    }
}

static EPSILON: f64 = 1e-5;

impl<C, T> Gatekeeper<C, T>
where
    C: Controller,
    T: Atomic,
{
    pub(crate) fn evaluate(&self, trajectory: Trajectory) -> Valuation {
        let time_horizon = trajectory.clone().len();
        (0..time_horizon)
            .map(|time| interpret(self.spec(), time))
            .sum::<f64>()
            / time_horizon as f64
    }

    pub fn run(&mut self) {
        let action = self.controller.select_action();
        let trajectory = self.simulation.run_recording_trajectory(action);
        let proba_safe = self.evaluate(trajectory);
        // TODO: in loop, ask for actions and only run the safe ones.
        // TODO: Lightbulb: give gatekeeper a "world" field and a "simulation" field, but both will be instances of Simulation<C>.
        if proba_safe > 1.0 - EPSILON {
            println!("Trajectory is safe at {}.", proba_safe);
        } else {
            println!("Trajectory is not safe at {}.", proba_safe);
        }
    }
}
