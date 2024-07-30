use crate::cfg::cfg;
use crate::logic::bounds::{approximate_infimum, approximate_supremum, Interpreter};
use crate::logic::syntax::Prop;
use crate::logic::types::{Atomic, Time, TimeWindow, Valuation};

/// pUq |=> sup { interpret(*q, t').min(inf{interpret(*p, t'') | t <= t'' < t'}) | t' >= time }
fn interpret_until<T: Atomic>(
    interpret_fn: impl Interpreter<T>,
    p: Prop<T>,
    q: Prop<T>,
    time: Time,
) -> Valuation {
    let inf_of = |_: Prop<T>, t_double_prime: Time| interpret_fn(p.clone(), t_double_prime);
    let sup_of = |prop: Prop<T>, t: Time| {
        if prop == q.clone() {
            interpret_fn(q.clone(), t)
        } else {
            let p_inf = approximate_infimum(inf_of, p.clone(), TimeWindow::new(time, t));
            interpret_fn(q.clone(), t).min(p_inf)
        }
    };
    approximate_supremum(
        sup_of,
        q.clone(),
        TimeWindow::new(time, cfg().get::<Time>("max_timestamp").unwrap() - 1),
    )
}
/// goedel's fuzzy logic (see LDL paper) with a custom `until` operator
pub(crate) fn interpret<T: Atomic>(formula: Prop<T>, time: Time) -> Valuation {
    if cfg().get("debug").unwrap() {
        // println!("Time {} interpreting {:?}", time, formula);
    }
    match formula {
        Prop::True => 1.0,
        Prop::Var(x) => x[time].val(),
        Prop::Le(x, y) => {
            let x = x[time].val();
            let y = y[time].val();
            1.0 - ((x - y) / (x + y)).max(0.0)
        }
        Prop::Not(p) => 1.0 - interpret(*p, time),
        Prop::And(p, q) => interpret(*p, time).min(interpret(*q, time)),
        Prop::Until(p, q) => interpret_until(interpret, *p, *q, time),
    }
}
