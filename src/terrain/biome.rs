use noise::*;

pub struct BiomeComponent {
    biome: Biome,
}

pub enum Biome {
    Rainforest,
}

pub struct BiomeGenerator {
    generator: Perlin,
    scale: f64,
    bias: f64,
    uscale: f64,
    vscale: f64,
}
impl BiomeGenerator {
    pub fn get_biome(&self, _q: i32, _r: i32) -> Biome {
        let sp =
            ScalePoint::new(&self.generator).set_all_scales(self.uscale, self.vscale, 0.0, 0.0);
        let _noise_gen: ScaleBias<Point3<f64>> = ScaleBias::new(&sp)
            .set_bias(self.bias)
            .set_scale(self.scale);
        Biome::Rainforest
    }
}

impl Default for BiomeGenerator {
    fn default() -> Self {
        BiomeGenerator {
            generator: Perlin::new().set_seed(20),
            scale: 1.0,
            bias: 0.0,
            uscale: 0.07,
            vscale: 0.07,
        }
    }
}
