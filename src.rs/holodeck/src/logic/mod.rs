//! A formula in differentiable temporal logic will express safety specs in the gatekeeper.
pub mod bounds;
pub mod interpreter;
pub mod syntax;
pub mod types;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;
    use test_case::test_case;
    use types::Atomic;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct MockAtomicS {
        a: usize,
    }
    impl fmt::Display for MockAtomicS {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "a{}", self.a)
        }
    }
    impl Atomic for MockAtomicS {
        fn val(&self) -> types::Valuation {
            1.0 / (1.0 + self.a as types::Valuation)
        }
    }
    fn mock_interpreter_s(prop: syntax::Prop<MockAtomicS>, time: types::Time) -> types::Valuation {
        match prop {
            syntax::Prop::Var(a) => a[time].val(),
            syntax::Prop::Not(p) => 1.0 - mock_interpreter_s(*p, time),
            syntax::Prop::And(p, q) => mock_interpreter_s(*p, time) * mock_interpreter_s(*q, time),
            _ => (time as f64).sin(),
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    enum MockAtomicE {
        A,
        B,
    }
    impl fmt::Display for MockAtomicE {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                MockAtomicE::A => write!(f, "A"),
                MockAtomicE::B => write!(f, "B"),
            }
        }
    }
    impl Atomic for MockAtomicE {
        fn val(&self) -> types::Valuation {
            match self {
                MockAtomicE::A => 1.0,
                MockAtomicE::B => 0.0,
            }
        }
    }
    fn mock_interpreter_e(prop: syntax::Prop<MockAtomicE>, time: types::Time) -> types::Valuation {
        match prop {
            syntax::Prop::Var(a) => a[time].val(),
            syntax::Prop::Not(p) => 1.0 - mock_interpreter_e(*p, time),
            syntax::Prop::And(p, q) => mock_interpreter_e(*p, time) * mock_interpreter_e(*q, time),
            _ => (time as f64).sin(),
        }
    }
    fn float_equiv(a: types::Valuation, b: types::Valuation) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn epsilon_convergence() {
        let prop = syntax::Prop::Var(vec![MockAtomicE::A]);
        let window = types::TimeWindow::new(0, 0);
        let result = bounds::approximate_supremum(mock_interpreter_e, prop, window);
        assert!(
            result > 1.0 - 1e-2,
            "Expected result close to 1.0, got {}",
            result
        );
    }

    // TODO: moar tests.
}
