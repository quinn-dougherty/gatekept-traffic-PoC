use crate::logic::syntax::Prop;
use crate::logic::types::{Atomic, BoundType, Interval, Time, TimeWindow, Valuation};
use std::cmp::Ordering;

static EPSILON: f64 = 1e-6;
static MAX_ITERATIONS: usize = 1e4 as usize;

pub(crate) trait Interpreter<T: Atomic>: Clone + Fn(Prop<T>, Time) -> Valuation {}
impl<T: Atomic, F: Clone + Fn(Prop<T>, Time) -> Valuation> Interpreter<T> for F {}

fn approximate_bound<T, F>(
    interpreter: F,
    proposition: Prop<T>,
    window: TimeWindow,
    initial: Interval,
    bound_type: BoundType,
) -> Valuation
where
    T: Atomic,
    F: Interpreter<T>,
{
    let mut intervals = vec![initial];
    let mut global_bound = match bound_type {
        BoundType::Supremum => Interval::new(f64::NEG_INFINITY, f64::INFINITY),
        BoundType::Infimum => Interval::new(f64::INFINITY, f64::INFINITY),
    };
    let mut window = window.clone();

    for _ in 0..MAX_ITERATIONS {
        intervals.sort_by(|a, b| {
            match bound_type {
                BoundType::Supremum => b.upper().partial_cmp(&a.upper()),
                BoundType::Infimum => a.lower().partial_cmp(&b.lower()),
            }
            .unwrap_or(Ordering::Equal)
        });

        if let Some(interval) = intervals.pop() {
            let value_start = interpreter(proposition.clone(), window.start());
            let value_end = interpreter(proposition.clone(), window.end());
            let value = if value_start <= value_end {
                Interval::new(value_start, value_end)
            } else {
                Interval::new(value_end, value_start)
            };
            global_bound = Interval::new(
                global_bound.lower().max(value.lower()),
                global_bound.upper().min(value.upper()),
            );

            if global_bound.upper() - global_bound.lower() < EPSILON {
                return global_bound.upper();
            }

            let mid = (interval.lower() + interval.upper()) / 2.0;
            intervals.push(Interval::new(interval.lower(), mid));
            intervals.push(Interval::new(mid, interval.upper()));
        } else {
            break;
        }
    }
    match bound_type {
        BoundType::Supremum => global_bound.lower(),
        BoundType::Infimum => global_bound.upper(),
    }
}

pub fn approximate_supremum<T, F>(
    interpreter: F,
    proposition: Prop<T>,
    window: TimeWindow,
) -> Valuation
where
    T: Atomic,
    F: Interpreter<T>,
{
    approximate_bound(
        interpreter,
        proposition,
        window,
        Interval::new(f64::MIN, f64::MAX),
        BoundType::Supremum,
    )
}

pub fn approximate_infimum<T, F>(
    interpreter: F,
    proposition: Prop<T>,
    window: TimeWindow,
) -> Valuation
where
    T: Atomic,
    F: Interpreter<T>,
{
    approximate_bound(
        interpreter,
        proposition,
        window,
        Interval::new(f64::MAX, f64::MAX),
        BoundType::Infimum,
    )
}
