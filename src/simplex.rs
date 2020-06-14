use noise::OpenSimplex;
use noise::Seedable;
use noise::NoiseFn;

pub struct Simplex {
    n: noise::OpenSimplex
}

#[allow(dead_code)]
impl Simplex {
    pub fn with_seed(seed: u32) -> Self {
        let n = OpenSimplex::new();
        n.set_seed(seed);

        Self { n }
    }

    pub fn get2d(&self, x: f64, y: f64) -> f64 {
        self.n.get([x, y])
    }

    pub fn get3d(&self, x: f64, y: f64, z: f64) -> f64 {
        self.n.get([x, y, z])
    }
}
