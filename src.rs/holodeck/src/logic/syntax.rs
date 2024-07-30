//! The specification language
use crate::logic::types::Atomic;
use std::fmt;

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum Prop<T>
where
    T: Atomic,
{
    True,
    Var(Vec<T>),
    Le(Vec<T>, Vec<T>),
    Not(Box<Prop<T>>),
    And(Box<Prop<T>>, Box<Prop<T>>),
    Until(Box<Prop<T>>, Box<Prop<T>>),
}

impl<T> fmt::Display for Prop<T>
where
    T: Atomic,
{
    // TODO: get rid of brackets and escaped quote char to fix format tests in this file
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_inner<T>(v: Vec<T>) -> String
        where
            T: Atomic,
        {
            let mut s = String::new();
            s.push('[');
            for (i, x) in v.iter().enumerate() {
                s.push_str(&format!("{}", x));
                if i < v.len() - 1 {
                    s.push_str(", ");
                }
            }
            s.push(']');
            s
        }
        match self {
            Prop::True => write!(f, "⊤"),
            Prop::Var(x) => write!(f, "{}", fmt_inner(x.to_vec())),
            Prop::Le(x, y) => write!(f, "{} < {}", fmt_inner(x.to_vec()), fmt_inner(y.to_vec())),
            Prop::Not(p) => write!(f, "¬({})", p),
            Prop::And(p, q) => write!(f, "({}) ∧ ({})", p, q),
            Prop::Until(p, q) => write!(f, "({}) U ({})", p, q),
        }
    }
}

impl<T> Prop<T>
where
    T: Atomic,
{
    pub fn and(self, other: Self) -> Self {
        Prop::And(Box::new(self), Box::new(other))
    }

    pub fn tt() -> Self {
        Prop::True
    }

    pub fn var(x: Vec<T>) -> Self {
        Prop::Var(x)
    }

    pub fn le(x: Vec<T>, y: Vec<T>) -> Self {
        Prop::Le(x, y)
    }

    pub fn eq(x: Vec<T>, y: Vec<T>) -> Self {
        Prop::le(x.clone(), y.clone()).and(Prop::le(y, x))
    }

    pub fn not(self) -> Self {
        Prop::Not(Box::new(self))
    }

    pub fn ff() -> Self {
        Self::tt().not()
    }

    pub fn next(self) -> Self {
        Self::ff().until(self)
    }

    pub fn until(self, other: Self) -> Self {
        Prop::Until(Box::new(self), Box::new(other))
    }

    pub fn eventually(self) -> Self {
        Self::tt().until(self)
    }

    pub fn always(self) -> Self {
        self.not().eventually().not()
    }

    pub fn or(self, other: Self) -> Self {
        self.not().and(other.not()).not()
    }

    pub fn release(self, other: Self) -> Self {
        self.not().until(other.not()).not()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Atomic for String {
        fn val(&self) -> f64 {
            1.0
        }
    }

    #[test]
    fn prop_display() {
        let xstr = vec!["x".to_string()];
        let ystr = vec!["y".to_string()];
        let x = Prop::var(xstr.clone());
        let y = Prop::var(ystr.clone());
        let a = x.clone().not();
        let b = x.clone().and(y.clone());
        let c = x.clone().always();
        let d = x.clone().eventually();
        let e = x.clone().until(y.clone());
        assert_eq!(format!("{}", x), "[x]");
        assert_eq!(format!("{}", y), "[y]");
        assert_eq!(
            format!("{}", Prop::le(xstr.clone(), ystr.clone())),
            "[x] < [y]"
        );
        assert_eq!(format!("{}", a), "¬([x])");
        assert_eq!(format!("{}", b), "([x]) ∧ ([y])");
        assert_eq!(format!("{}", c), "¬((⊤) U (¬([x])))");
        assert_eq!(format!("{}", d), "(⊤) U ([x])");
        assert_eq!(format!("{}", e), "([x]) U ([y])");
    }
}
