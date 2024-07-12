use std::fmt::{Debug, Display};

pub type Valuation = f64;

pub type Time = usize; // Time indexes a vec.

pub trait Atomic: Debug + Display + Clone + PartialEq + Eq + std::hash::Hash {
    fn val(&self) -> Valuation;
}

#[derive(Clone, Debug)]
pub(crate) struct Interval {
    lower: Valuation,
    upper: Valuation,
}

impl Interval {
    pub fn new(lower: Valuation, upper: Valuation) -> Self {
        if lower > upper {
            panic!(
                "Interval: lower bound {} is greater than upper bound {}",
                lower, upper
            );
        }
        Self { lower, upper }
    }
    pub fn lower(&self) -> Valuation {
        self.lower
    }
    pub fn upper(&self) -> Valuation {
        self.upper
    }
}

#[derive(Clone)]
pub(crate) struct TimeWindow {
    start: Time,
    end: Time,
}

impl TimeWindow {
    pub(crate) fn new(start: Time, end: Time) -> Self {
        if start > end {
            panic!(
                "TimeWindow: start time {} is greater than end time {}",
                start, end
            );
        }
        Self { start, end }
    }
    pub(crate) fn start(&self) -> Time {
        self.start
    }
    pub(crate) fn end(&self) -> Time {
        self.end
    }
}

pub(crate) enum BoundType {
    Supremum, // least upper bound
    Infimum,  // greatest lower bound
}
