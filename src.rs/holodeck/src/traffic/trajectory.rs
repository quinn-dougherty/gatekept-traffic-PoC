use crate::logic::types::Atomic;
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash, Copy)]
pub struct TrajectoryEntry {
    num_crashes_local: u32,
    num_cars_throughput: u32,
}

impl Display for TrajectoryEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "TrajectoryEntry(num_crashes_local={}, num_cars_throughput={})",
            self.num_crashes_local, self.num_cars_throughput
        )
    }
}

impl TrajectoryEntry {
    pub fn new(num_crashes_local: u32, num_cars_throughput: u32) -> Self {
        Self {
            num_crashes_local,
            num_cars_throughput,
        }
    }
    pub fn num_crashes_local(&self) -> u32 {
        self.num_crashes_local
    }
    pub fn num_cars_throughput(&self) -> u32 {
        self.num_cars_throughput
    }
}

pub type Trajectory = Vec<TrajectoryEntry>;

impl Atomic for TrajectoryEntry {
    fn val(&self) -> f64 {
        1.0 / (1.0 + self.num_crashes_local as f64)
    }
}
