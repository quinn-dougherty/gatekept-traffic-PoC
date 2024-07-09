use std::sync::OnceLock;

type CarId = u32;
type CarPos = u32;

#[derive(Clone)]
pub struct IntersectionConfig {
    pub road_length: u32,
    pub max_cars: u32,
    pub light_coord: u32,
}

struct IntersectionConfigBuilder {
    config: IntersectionConfig,
}

impl IntersectionConfigBuilder {
    fn new() -> Self {
        IntersectionConfigBuilder {
            config: IntersectionConfig {
                road_length: 10,
                max_cars: 25,
                light_coord: 4,
            },
        }
    }

    fn road_length(&mut self, road_length: u32) -> &mut Self {
        self.config.road_length = road_length;
        self
    }

    fn max_cars(&mut self, max_cars: u32) -> &mut Self {
        self.config.max_cars = max_cars;
        self
    }

    fn light_coord(&mut self, light_coord: u32) -> &mut Self {
        self.config.light_coord = light_coord;
        self
    }

    fn build(self) -> IntersectionConfig {
        self.config
    }
}
static CONFIG: OnceLock<IntersectionConfig> = OnceLock::new();

#[derive(PartialEq, Clone, Debug)]
enum Light {
    N,
    S,
    E,
    W,
}

type CurrentlyGreen = Vec<Light>;

#[derive(Clone, Debug)]
struct Car {
    id: CarId,
    light: Light,
    position: CarPos,
}

impl Car {
    fn new(id: CarId, light: Light) -> Self {
        Car {
            id,
            light,
            position: 0,
        }
    }

    fn advance(&mut self, lights: &CurrentlyGreen) {
        if lights.contains(&self.light) {
            self.position += 1;
        }
    }
}

#[derive(Clone)]
pub struct Intersection {
    pub cars: Vec<Car>,
    pub green_lights: CurrentlyGreen,
    pub num_crashes: u32,
}

struct IntersectionBuilder {
    intersection: Intersection,
}

impl IntersectionBuilder {
    pub fn new() -> Self {
        IntersectionBuilder {
            intersection: Intersection {
                cars: Vec::new(),
                green_lights: CurrentlyGreen::default(),
                num_crashes: 0,
            },
        }
    }

    pub fn add_car(&mut self, car: Car) -> &mut Self {
        self.intersection.cars.push(car);
        self
    }

    pub fn add_light(&mut self, light: Light) -> &mut Self {
        self.intersection.green_lights.push(light);
        self
    }

    pub fn build(&self) -> Intersection {
        self.intersection.clone()
    }
}

trait ConfiguredIntersection {
    type Config;
    fn config() -> Self::Config;
}

impl ConfiguredIntersection for Intersection {
    type Config = &'static IntersectionConfig;

    fn config() -> Self::Config {
        CONFIG.get_or_init(|| IntersectionConfigBuilder::new().build())
    }
}

impl Intersection {
    pub fn num_crashes(&self) -> u32 {
        self.num_crashes
    }
    pub fn incr_num_crashes(&mut self, x: u32) {
        self.num_crashes += x;
    }

    fn spawn_car(&mut self, light: Light) {
        let id = self.cars.len() as u32;
        if id < Self::config().max_cars {
            let car = Car::new(id, light);
            self.cars.push(car);
        }
    }
    fn advance(&mut self) {
        for car in self.cars.iter_mut() {
            car.advance(&self.green_lights);
        }
        self.update_crashes();
    }

    fn crash_opportunities(&self) -> Vec<(CarPos, CarPos)> {
        let config = Self::config();
        let mut result = Vec::new();
        for light_i in self.green_lights.iter() {
            for light_j in self.green_lights.iter() {
                match (light_i, light_j) {
                    (Light::N, Light::E) => {
                        result.push((config.light_coord + 1, config.light_coord + 2))
                    }
                    (Light::N, Light::W) => {
                        result.push((config.light_coord + 2, config.light_coord + 1))
                    }
                    (Light::N, _) => continue,
                    (Light::E, Light::N) => {
                        result.push((config.light_coord + 2, config.light_coord + 1))
                    }
                    (Light::E, Light::S) => {
                        result.push((config.light_coord + 1, config.light_coord + 2))
                    }
                    (Light::E, _) => continue,
                    (Light::S, Light::E) => {
                        result.push((config.light_coord + 2, config.light_coord + 1))
                    }
                    (Light::S, Light::W) => {
                        result.push((config.light_coord + 1, config.light_coord + 2))
                    }
                    (Light::S, _) => continue,
                    (Light::W, Light::N) => {
                        result.push((config.light_coord + 1, config.light_coord + 2))
                    }
                    (Light::W, Light::S) => {
                        result.push((config.light_coord + 2, config.light_coord + 1))
                    }
                    (Light::W, _) => continue,
                }
            }
        }
        result
    }

    pub fn update_crashes(&mut self) {
        let crash_opportunities = self.crash_opportunities();
        let mut crashes = 0;
        for car_i in &self.cars {
            for car_j in &self.cars {
                if crash_opportunities.contains(&(car_i.position, car_j.position)) {
                    crashes += 1;
                }
            }
        }
        self.incr_num_crashes(crashes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    //fn it_

    #[test]
    fn it_should_one_crash() {
        let mut intersection = IntersectionBuilder::new()
            .add_light(Light::N)
            .add_light(Light::E)
            .build();
        intersection.spawn_car(Light::N);
        intersection.spawn_car(Light::S);
        intersection.spawn_car(Light::E);
        intersection.spawn_car(Light::W);
        for _ in 0..6 {
            println!("{:?}", intersection.cars);
            intersection.advance();
            intersection.spawn_car(Light::N);
            intersection.spawn_car(Light::E);
        }
        assert_eq!(intersection.num_crashes(), 1);
    }
}
