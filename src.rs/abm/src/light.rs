use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::collections::HashSet;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum Light {
    N,
    S,
    E,
    W,
}

impl Light {
    /// Returns the lights perpendicular to the current light.
    /// Importantly, the order of the returned lights are (nearer, farther).
    pub(crate) fn perpendiculars(&self) -> (Light, Light) {
        match self {
            Light::N => (Light::E, Light::W),
            Light::S => (Light::W, Light::E),
            Light::E => (Light::S, Light::N),
            Light::W => (Light::N, Light::S),
        }
    }
    pub(crate) fn random() -> Self {
        rand::random()
    }
}

pub(crate) type CurrentlyGreen = HashSet<Light>;

impl Distribution<Light> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Light {
        match rng.gen_range(0..=3) {
            0 => Light::N,
            1 => Light::S,
            2 => Light::E,
            3 => Light::W,
            _ => unreachable!(),
        }
    }
}
