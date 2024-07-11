use crate::traffic::light::{CurrentlyGreen, Light};

pub(crate) type CarId = u32;
pub(crate) type CarPos = u32;

#[derive(Clone, Debug)]
pub(crate) struct Car {
    pub id: CarId,
    pub light: Light,
    pub position: CarPos,
}

impl Car {
    pub(crate) fn new(id: CarId, light: Light) -> Self {
        Car {
            id,
            light,
            position: 0,
        }
    }

    pub(crate) fn advance(&mut self, lights: &CurrentlyGreen) {
        if lights.contains(&self.light) {
            self.position += 1;
        }
    }
}
