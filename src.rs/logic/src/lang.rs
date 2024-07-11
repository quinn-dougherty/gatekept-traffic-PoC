//! The specification language
use std::fmt::{Debug, Display};

pub trait Terms: Debug + Display + Clone + PartialEq + Eq + std::hash::Hash {}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum Prop<T>
where
    T: Terms,
{
    True,
    Var(T),
    Lt(T, T),
    Eq(T, T),
    Not(Box<Prop<T>>),
    And(Box<Prop<T>>, Box<Prop<T>>),
    Until(Box<Prop<T>>, Box<Prop<T>>),
}

impl<T> Display for Prop<T>
where
    T: Terms,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Prop::True => write!(f, "⊤"),
            Prop::Var(x) => write!(f, "{}", x),
            Prop::Lt(x, y) => write!(f, "{} < {}", x, y),
            Prop::Eq(x, y) => write!(f, "{} = {}", x, y),
            Prop::Not(p) => write!(f, "¬({})", p),
            Prop::And(p, q) => write!(f, "({}) ∧ ({})", p, q),
            Prop::Until(p, q) => write!(f, "({}) U ({})", p, q),
        }
    }
}

impl<T> Prop<T>
where
    T: Terms,
{
    pub fn and(self, other: Self) -> Self {
        Prop::And(Box::new(self), Box::new(other))
    }

    pub fn tt() -> Self {
        Prop::True
    }

    pub fn var(x: T) -> Self {
        Prop::Var(x)
    }

    pub fn lt(x: T, y: T) -> Self {
        Prop::Lt(x, y)
    }

    pub fn eq(x: T, y: T) -> Self {
        Prop::Eq(x, y)
    }

    pub fn not(self) -> Self {
        Prop::Not(Box::new(self))
    }

    pub fn ff() -> Self {
        Self::tt().not()
    }

    pub fn next(self) -> Self {
        self.ff().until(self)
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

/**
 * Tests
 * */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        impl Terms for String {}
        let x = Prop::Var("x".to_string());
        let y = Prop::Var("y".to_string());
        let a = x.clone().not();
        let b = x.clone().and(y.clone());
        let c = x.clone().next();
        let d = x.clone().always();
        let e = x.clone().eventually();
        let f = x.clone().until(y.clone());
        let g = x.clone().release(y.clone());
        assert_eq!(format!("{}", x), "x");
        assert_eq!(format!("{}", y), "y");
        assert_eq!(
            format!("{}", Prop::Cmp("x".to_string(), "y".to_string())),
            "x <= y"
        );
        assert_eq!(format!("{}", a), "¬(x)");
        assert_eq!(format!("{}", b), "(x) ∧ (y)");
        assert_eq!(format!("{}", d), "¬((⊤) U (¬(x)))");
        assert_eq!(format!("{}", e), "(⊤) U (x)");
        assert_eq!(format!("{}", f), "(x) U (y)");
        assert_eq!(format!("{}", g), "(x) R (y)");
    }
}
