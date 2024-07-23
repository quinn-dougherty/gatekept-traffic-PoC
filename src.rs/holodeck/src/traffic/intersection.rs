use crate::traffic::car::{Car, CarId, CarPos};
use crate::traffic::light::{CurrentlyGreen, Light};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

#[derive(Clone)]
pub(crate) struct IntersectionConfig {
    pub(crate) road_length: u32,
    pub(crate) light_coord: u32,
    pub(crate) debug: bool,
}

struct IntersectionConfigBuilder {
    config: IntersectionConfig,
}

impl IntersectionConfigBuilder {
    fn new() -> Self {
        IntersectionConfigBuilder {
            config: IntersectionConfig {
                road_length: 10,
                light_coord: 4,
                debug: true,
            },
        }
    }

    fn with_road_length(&mut self, road_length: u32) -> &mut Self {
        self.config.road_length = road_length;
        self
    }

    fn with_light_coord(&mut self, light_coord: u32) -> &mut Self {
        self.config.light_coord = light_coord;
        self
    }

    fn build(self) -> IntersectionConfig {
        self.config
    }
}
pub(crate) static CONFIG: OnceLock<IntersectionConfig> = OnceLock::new();

#[derive(Clone)]
pub struct Intersection {
    pub(crate) cars: Vec<Car>,
    pub(crate) green_lights: CurrentlyGreen,
    pub(crate) num_crashes: u32,
    pub(crate) total_throughput: u32,
}

pub struct IntersectionBuilder {
    intersection: Intersection,
}

impl IntersectionBuilder {
    pub fn new() -> Self {
        IntersectionBuilder {
            intersection: Intersection {
                cars: Vec::new(),
                green_lights: CurrentlyGreen::default(),
                num_crashes: 0,
                total_throughput: 0,
            },
        }
    }

    pub(crate) fn add_car(&mut self, car: Car) -> &mut Self {
        self.intersection.cars.push(car);
        self
    }

    pub(crate) fn add_light(&mut self, light: Light) -> &mut Self {
        self.intersection.green_lights.insert(light);
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

struct CrashCoordinates {
    light: Light,
    car_inlane: CarPos,
    car_inperp: CarPos,
}

impl CrashCoordinates {
    fn new(light: Light, car_inlane: CarPos, car_inperp: CarPos) -> Self {
        CrashCoordinates {
            light,
            car_inlane,
            car_inperp,
        }
    }
}

struct CrashOpportunity {
    closer: CrashCoordinates,
    farther: CrashCoordinates,
}

impl CrashOpportunity {
    fn new(closer: CrashCoordinates, farther: CrashCoordinates) -> Self {
        CrashOpportunity { closer, farther }
    }
}

impl Intersection {
    pub fn num_crashes(&self) -> u32 {
        self.num_crashes
    }
    pub fn total_throughput(&self) -> u32 {
        self.total_throughput
    }

    pub(crate) fn incr_num_crashes(&mut self, x: u32) {
        self.num_crashes += x;
    }

    pub(crate) fn spawn_car(&mut self, light: Light) {
        let id = self.cars.len() as u32;
        let car = Car::new(id, light);
        self.cars.push(car);
    }

    pub(crate) fn add_light(&mut self, light: Light) {
        self.green_lights.insert(light);
    }

    pub(crate) fn remove_light(&mut self, light: Light) {
        self.green_lights.retain(|l| l != &light);
    }

    /// Set all lights to red
    pub(crate) fn remove_all_lights(&mut self) {
        self.green_lights.clear();
    }
    fn remove_cars_that_drove_too_far(&mut self) {
        let cars_before = self.cars.len();
        self.cars
            .retain(|car| car.position < Self::config().road_length);
        let cars_removed = cars_before - self.cars.len();
        self.total_throughput += cars_removed as u32;
    }
    pub(crate) fn advance(&mut self) {
        for car in self.cars.iter_mut() {
            car.advance(&self.green_lights);
        }
        let before_crashes = self.num_crashes();
        self.update_crashes();
        let after_crashes = self.num_crashes();
        if Self::config().debug && before_crashes != after_crashes {
            println!("Crash! Num crashes: {}", after_crashes);
        }
        self.remove_cars_that_drove_too_far();
    }

    fn crash_opportunities(&self) -> HashMap<Light, CrashOpportunity> {
        let config = Self::config();
        let mut result = HashMap::new();
        for light_i in self.green_lights.iter() {
            let (light_closer, light_farther) = light_i.perpendiculars();
            result.insert(
                light_i.clone(),
                CrashOpportunity::new(
                    CrashCoordinates::new(
                        light_closer,
                        config.light_coord + 1,
                        config.light_coord + 2,
                    ),
                    CrashCoordinates::new(
                        light_farther,
                        config.light_coord + 2,
                        config.light_coord + 1,
                    ),
                ),
            );
        }
        result
    }

    pub(crate) fn update_crashes(&mut self) {
        let crash_opportunities = self.crash_opportunities();
        let mut crash_pairs = HashSet::new();
        for car_inlane in &self.cars {
            if let Some(crash_opportunity) = crash_opportunities.get(&car_inlane.light) {
                let cars_inperp: Vec<&Car> = self
                    .cars
                    .iter()
                    .filter(|car| {
                        car.light == crash_opportunity.closer.light
                            || car.light == crash_opportunity.farther.light
                    })
                    .collect();
                for car_inperp in cars_inperp {
                    if (car_inlane.position == crash_opportunity.closer.car_inlane
                        && car_inperp.position == crash_opportunity.closer.car_inperp)
                        || (car_inlane.position == crash_opportunity.farther.car_inlane
                            && car_inperp.position == crash_opportunity.farther.car_inperp)
                    {
                        // deduplicate crash pairs
                        crash_pairs.insert(if car_inlane.id < car_inperp.id {
                            (car_inlane.id, car_inperp.id)
                        } else {
                            (car_inperp.id, car_inlane.id)
                        });
                    }
                }
            }
        }
        self.incr_num_crashes(crash_pairs.len() as u32);
        let crashed_car_ids: HashSet<CarId> = crash_pairs
            .into_iter()
            .flat_map(|(id1, id2)| vec![id1, id2])
            .collect();
        self.cars.retain(|car| !crashed_car_ids.contains(&car.id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Light::N, Light::E)]
    #[test_case(Light::E, Light::N)]
    #[test_case(Light::S, Light::E)]
    #[test_case(Light::E, Light::S)]
    #[test_case(Light::N, Light::W)]
    #[test_case(Light::W, Light::N)]
    #[test_case(Light::S, Light::W)]
    #[test_case(Light::W, Light::S)]
    fn advance_with_one_crash_updates_numcrashes_to_1(light1: Light, light2: Light) {
        let mut intersection = IntersectionBuilder::new()
            .add_light(light1.clone())
            .add_light(light2.clone())
            .build();

        intersection.spawn_car(light1.clone());
        intersection.advance();
        intersection.spawn_car(light2.clone());

        for _ in 0..(Intersection::config().light_coord + 3) {
            intersection.advance();
        }

        assert_eq!(intersection.num_crashes(), 1);
    }

    #[test_case(Light::N, Light::E)]
    #[test_case(Light::E, Light::N)]
    #[test_case(Light::S, Light::E)]
    #[test_case(Light::E, Light::S)]
    #[test_case(Light::N, Light::W)]
    #[test_case(Light::W, Light::N)]
    #[test_case(Light::S, Light::W)]
    #[test_case(Light::W, Light::S)]
    fn advance_with_two_crashes_updates_numcrashes_to_2(light1: Light, light2: Light) {
        let mut intersection = IntersectionBuilder::new()
            .add_light(light1.clone())
            .add_light(light2.clone())
            .build();

        intersection.spawn_car(light1.clone());
        intersection.advance();
        intersection.spawn_car(light1.clone());
        intersection.spawn_car(light2.clone());
        intersection.advance();
        intersection.spawn_car(light2.clone());

        for _ in 2..(Intersection::config().road_length) {
            intersection.advance();
        }

        assert_eq!(intersection.num_crashes(), 2);
    }

    #[test_case(Light::N, Light::E)]
    #[test_case(Light::E, Light::N)]
    #[test_case(Light::S, Light::E)]
    #[test_case(Light::E, Light::S)]
    #[test_case(Light::N, Light::W)]
    #[test_case(Light::W, Light::N)]
    #[test_case(Light::S, Light::W)]
    #[test_case(Light::W, Light::S)]
    fn advance_single_crash_removes_two_cars(light1: Light, light2: Light) {
        let mut intersection = IntersectionBuilder::new()
            .add_light(light1.clone())
            .add_light(light2.clone())
            .build();
        // Add cars that will crash
        intersection.spawn_car(light1.clone());
        intersection.advance();
        intersection.spawn_car(light2.clone());

        // Move cars through crash position, but not off edge
        for _ in 1..(Intersection::config().road_length - 1) {
            intersection.advance();
        }

        // Check that crashed cars were removed
        assert_eq!(intersection.cars.len(), 0);
    }

    #[test_case(Light::N, Light::E)]
    #[test_case(Light::E, Light::N)]
    #[test_case(Light::S, Light::E)]
    #[test_case(Light::E, Light::S)]
    #[test_case(Light::N, Light::W)]
    #[test_case(Light::W, Light::N)]
    #[test_case(Light::S, Light::W)]
    #[test_case(Light::W, Light::S)]
    fn advance_two_crashes_removes_four_cars(light1: Light, light2: Light) {
        let mut intersection = IntersectionBuilder::new()
            .add_light(light1.clone())
            .add_light(light2.clone())
            .build();

        // Add cars that will crash
        intersection.spawn_car(light1.clone());
        intersection.advance();
        intersection.spawn_car(light2.clone());
        intersection.advance();
        intersection.spawn_car(light1.clone());
        intersection.advance();
        intersection.spawn_car(light2.clone());

        // Move cars through crash positions, but not off edge
        for _ in 3..(Intersection::config().road_length - 1) {
            intersection.advance();
        }

        // Check that crashed cars were removed
        assert_eq!(intersection.cars.len(), 0);
    }

    #[test_case(Light::N, Light::E)]
    #[test_case(Light::E, Light::N)]
    #[test_case(Light::S, Light::E)]
    #[test_case(Light::E, Light::S)]
    #[test_case(Light::N, Light::W)]
    #[test_case(Light::W, Light::N)]
    #[test_case(Light::S, Light::W)]
    #[test_case(Light::W, Light::S)]
    fn advance_single_crash_removes_two_cars_remaining_third(light1: Light, light2: Light) {
        let mut intersection = IntersectionBuilder::new()
            .add_light(light1.clone())
            .add_light(light2.clone())
            .build();

        // Add cars that will crash and one that won't
        intersection.spawn_car(light1.clone());
        intersection.advance();
        intersection.spawn_car(light2.clone());
        intersection.spawn_car(light1.clone()); // This car will be behind the crash

        // Move cars through crash position, but not off edge
        for _ in 1..(Intersection::config().road_length - 1) {
            intersection.advance();
        }

        // Check that only crashed cars were removed
        assert_eq!(intersection.cars.len(), 1);
        assert_eq!(intersection.cars[0].light, light1);
    }

    #[test_case(Light::N)]
    #[test_case(Light::E)]
    #[test_case(Light::S)]
    #[test_case(Light::W)]
    fn advance_off_edge_deletes_car(light: Light) {
        let mut intersection = IntersectionBuilder::new().add_light(light.clone()).build();
        intersection.spawn_car(light.clone());
        for _ in 0..Intersection::config().road_length {
            intersection.advance();
        }
        assert_eq!(intersection.cars.len(), 0);
    }
}
