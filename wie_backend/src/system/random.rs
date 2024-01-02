use rand::{rngs::StdRng, RngCore, SeedableRng};

pub struct Random {
    rng: StdRng,
}

impl Random {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn next(&mut self) -> u64 {
        self.rng.next_u64()
    }
}
