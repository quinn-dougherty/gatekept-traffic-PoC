use rand::SeedableRng;

const N: usize = 64;
#[derive(Clone, Debug)]
pub struct RngSeed(pub [u8; N]);
#[derive(Clone, Debug)]
pub struct Rng(RngSeed);

impl RngSeed {
    pub fn new() -> RngSeed {
        RngSeed([0; N])
    }
    pub fn from_u8(seed: u8) -> RngSeed {
        RngSeed([seed; N])
    }
}
impl Default for RngSeed {
    fn default() -> RngSeed {
        RngSeed::new()
    }
}

impl AsMut<[u8]> for RngSeed {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl Default for Rng {
    fn default() -> Rng {
        Rng(RngSeed::new())
    }
}

impl SeedableRng for Rng {
    type Seed = RngSeed;

    fn from_seed(seed: RngSeed) -> Rng {
        Rng(seed)
    }
}
