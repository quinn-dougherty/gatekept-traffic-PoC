use crate::cfg::cfg;
use crate::data::nodup_stack::NoDupStack;
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
    return lower / 2.0 + upper / 2.0;
}

fn by(a: &f64, b: &f64, bound_type: BoundType) -> Ordering {
    match bound_type {
        BoundType::Supremum => a.partial_cmp(&b),
        BoundType::Infimum => b.partial_cmp(&a),
    }
    .unwrap_or(Ordering::Equal)
}

fn compute_interval_value<T, F>(
    interpreter: &F,
    proposition: &Prop<T>,
    window: &TimeWindow,
    bound_type: &BoundType,
) -> Interval
where
    T: Atomic,
    F: Interpreter<T>,
{
    let value = (window.start()..window.end()).map(|t| interpreter(proposition.clone(), t));
    let value_start = match value.clone().min_by(|a, b| by(a, b, bound_type.clone())) {
        Some(v) => v,
        None => f64::MIN,
    };
    let value_end = match value.clone().max_by(|a, b| by(a, b, bound_type.clone())) {
        Some(v) => v,
        None => f64::MAX,
    };

    Interval::new(value_start, value_end)
}

fn update_global_bound(global_bound: &Interval, value: Interval) -> Interval {
    Interval::new(
        global_bound.lower().max(value.lower()),
        global_bound.upper().min(value.upper()),
    )
}

/// True if converged within a default epsilon of 1e-6.
///
/// # Assumptions
/// - `global_bound` is well-formed, meaning lower < upper.
fn is_converged(global_bound: &Interval) -> bool {
    let absolute_error = global_bound.upper() - global_bound.lower();
    if absolute_error < EPSILON {
        if cfg().get("debug").unwrap() {
            println!("approximate_bound converged with error {}", absolute_error);
        }
        true
    } else {
        false
    }
}

fn split_interval(interval: Interval) -> Vec<Interval> {
    let mid = safe_midpoint(interval.lower(), interval.upper());
    vec![
        Interval::new(interval.lower(), mid),
        Interval::new(mid, interval.upper()),
    ]
}

fn extract_result(global_bound: Interval, bound_type: BoundType) -> Valuation {
    match bound_type {
        BoundType::Supremum => global_bound.lower(),
        BoundType::Infimum => global_bound.upper(),
    }
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
    let mut intervals = NoDupStack::new_singleton(initial);
    let mut global_bound = Interval::new(f64::MIN, f64::MAX);
    loop {
        intervals.sort_by(|a, b| {
            match bound_type {
                BoundType::Supremum => b.upper().partial_cmp(&a.upper()),
                BoundType::Infimum => a.lower().partial_cmp(&b.lower()),
            }
            .unwrap_or(Ordering::Equal)
        });
        if let Some(interval) = intervals.pop() {
            let value = compute_interval_value(&interpreter, &proposition, &window, &bound_type);
            global_bound = update_global_bound(&global_bound, value);

            if is_converged(&global_bound) {
                break;
            }
            intervals.extend(split_interval(interval).into_iter());
        } else {
            if cfg().get("debug").unwrap() {
                println!("Warning: empty interval stack in approximate_bound");
            }
            break;
        }
    }
    println!(
        "Global bound: {:?} for bound type {:?}",
        global_bound, bound_type
    );
    extract_result(global_bound, bound_type)
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
        let mid = safe_midpoint(MIN, MAX);
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

    #[test]
    fn test_update_global_bound() {
        let global_bound = Interval::new(0.0, 10.0);
        let value = Interval::new(2.0, 8.0);
        let result = update_global_bound(&global_bound, value);
        assert_eq!(result, Interval::new(2.0, 8.0));
    }

    #[test]
    fn test_is_converged() {
        let converged_bound = Interval::new(5.0, 5.0 + EPSILON / 2.0);
        assert!(is_converged(&converged_bound));

        let not_converged_bound = Interval::new(5.0, 6.0);
        assert!(!is_converged(&not_converged_bound));
    }

    #[test]
    fn test_split_interval() {
        let interval = Interval::new(0.0, 10.0);
        let result = split_interval(interval);
        assert_eq!(result[0], Interval::new(0.0, 5.0));
        assert_eq!(result[1], Interval::new(5.0, 10.0));
    }

    #[test]
    fn test_extract_result() {
        let global_bound = Interval::new(1.0, 2.0);

        let supremum = extract_result(global_bound.clone(), BoundType::Supremum);
        assert_eq!(supremum, 1.0);

        let infimum = extract_result(global_bound, BoundType::Infimum);
        assert_eq!(infimum, 2.0);
    }
}
