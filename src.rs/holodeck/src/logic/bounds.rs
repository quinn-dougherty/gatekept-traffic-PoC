use crate::logic::syntax::Prop;
use crate::logic::types::{Atomic, BoundType, Interval, Time, TimeWindow, Valuation};
use std::cmp::Ordering;

static EPSILON: f64 = 1e-6;
static MAX_ITERATIONS: usize = 2.2e4 as usize;

pub(crate) trait Interpreter<T: Atomic>: Clone + Fn(Prop<T>, Time) -> Valuation {}
impl<T: Atomic, F: Clone + Fn(Prop<T>, Time) -> Valuation> Interpreter<T> for F {}

/// Calculates the midpoint of two f64 values safely, handling potential overflow.
///
/// # Arguments
///
/// * `lower` - The lower bound of the interval
/// * `upper` - The upper bound of the interval
///
/// # Returns
///
/// Returns a f64 value representing the midpoint.
///
/// # Assumptions
///
/// - `lower <= upper`. If this is not true, the behavior is unspecified.
/// - lower.is_finite() && upper.is_finite(). If this is not true, the behavior is unspecified
///
/// # Guarantees
///
/// - The function will always return a finite f64 value.
/// - The returned value will always be between `lower` and `upper`, inclusive.
///
/// # Examples
///
/// ```
/// let mid = safe_midpoint(0.0, 10.0);
/// assert_eq!(mid, 5.0);
///
/// let mid = safe_midpoint(f64::MAX, f64::MAX);
/// assert!(mid.is_finite() && mid > 0.0);
pub fn safe_midpoint(lower: f64, upper: f64) -> f64 {
    let difference = upper - lower;
    //if !difference.is_finite() {
    //    println!("upper - lower is infinite.");
    //    return 1.0 / EPSILON * safe_midpoint(EPSILON * lower, EPSILON * upper);
    //}
    let half_range = difference / 2.0;
    let sum = lower + half_range;
    if sum.is_sign_positive() {
        return sum.min(f64::MAX);
    }
    return sum.max(f64::MIN);
}

fn by(a: &f64, b: &f64, bound_type: BoundType) -> Ordering {
    match bound_type {
        BoundType::Supremum => a.partial_cmp(&b),
        BoundType::Infimum => b.partial_cmp(&a),
    }
    .unwrap_or(Ordering::Equal)
}

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
    let mut global_bound = Interval::new(f64::MIN, f64::MAX);
    println!("Computing bound of type {:?}", bound_type);
    // for _ in 0..MAX_ITERATIONS {
    loop {
        intervals.sort_by(|a, b| {
            match bound_type {
                BoundType::Supremum => b.upper().partial_cmp(&a.upper()),
                BoundType::Infimum => a.lower().partial_cmp(&b.lower()),
            }
            .unwrap_or(Ordering::Equal)
        });

        if let Some(interval) = intervals.pop() {
            let value = (window.start()..window.end()).map(|t| interpreter(proposition.clone(), t));
            let value_start = value
                .clone()
                .min_by(|a, b| by(a, b, bound_type.clone()))
                .unwrap();
            // let value_start = interpreter(proposition.clone(), window.start());
            let value_end = value
                .clone()
                .max_by(|a, b| by(a, b, bound_type.clone()))
                .unwrap();
            // let value_end = interpreter(proposition.clone(), window.end());
            let value = if value_start <= value_end {
                Interval::new(value_start, value_end)
            } else {
                Interval::new(value_end, value_start)
            };
            global_bound = Interval::new(
                global_bound.lower().max(value.lower()),
                global_bound.upper().min(value.upper()),
            );

            // let relative_error = (global_bound.upper() - global_bound.lower()).abs() / global_bound.upper().abs();
            let absolute_error = global_bound.upper() - global_bound.lower();
            if absolute_error < EPSILON {
                println!("approximate_bound converged. Error:{}", absolute_error);
                break;
            }
            let mid = safe_midpoint(interval.lower(), interval.upper());
            intervals.push(Interval::new(interval.lower(), mid));
            intervals.push(Interval::new(mid, interval.upper()));
        } else {
            println!("Warning: empty interval stack in approximate_bound");
            break;
        }
    }
    println!(
        "Global bound: {:?} for bound type {:?}",
        global_bound, bound_type
    );
    match bound_type {
        BoundType::Supremum => global_bound.lower(),
        BoundType::Infimum => global_bound.upper(),
    }
}

pub(crate) fn approximate_supremum<T, F>(
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
        Interval::new(EPSILON * f64::MIN, EPSILON * f64::MAX),
        BoundType::Supremum,
    )
}

pub(crate) fn approximate_infimum<T, F>(
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
        Interval::new(EPSILON * f64::MIN, EPSILON * f64::MAX),
        BoundType::Infimum,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    static MAX: f64 = f64::MAX;
    static MIN: f64 = f64::MIN;

    #[test]
    fn safe_midpoint_normal_cases() {
        assert_eq!(safe_midpoint(0.0, 10.0), 5.0);
        assert_eq!(safe_midpoint(-10.0, 10.0), 0.0);
        assert_eq!(safe_midpoint(1.0, 2.0), 1.5);
    }

    #[test]
    fn safe_midpoint_same_values() {
        assert_eq!(safe_midpoint(5.0, 5.0), 5.0);
        assert_eq!(safe_midpoint(0.0, 0.0), 0.0);
    }

    #[test]
    fn safe_midpoint_large_values() {
        let mid = safe_midpoint(MAX / 2.0, MAX);
        assert!(mid.is_finite());
        // println!("Mid: {}", mid);
        assert!(MAX / 2.0 < mid);
        assert!(mid < MAX);
    }

    #[test]
    fn safe_midpoint_small_values() {
        let mid = safe_midpoint(MIN, MIN / 2.0);
        assert!(mid.is_finite());
        // println!("Mid: {}", mid);
        assert!(MIN < mid);
        assert!(mid < MIN / 2.0);
    }

    #[test]
    fn safe_midpoint_extreme_values() {
        let mid = safe_midpoint(f64::MIN, f64::MAX);
        assert!(mid.is_finite());
        // println!("Mid: {}", mid);
        assert!(MIN < mid);
        assert!(mid < MAX);
        assert!(mid.abs() < 1.0);
    }

    #[test]
    fn safe_midpoint_very_close_values() {
        let x = 1.0;
        let y = x + f64::EPSILON;
        assert!(safe_midpoint(x, y).is_finite());
    }
}
