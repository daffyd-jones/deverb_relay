// src/dsp/dc_blocker.rs

pub struct DcBlocker {
    prev_in: f32,
    prev_out: f32,
    r: f32,
}

impl DcBlocker {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            prev_in: 0.0,
            prev_out: 0.0,
            r: Self::calc_r(sample_rate),
        }
    }

    fn calc_r(sample_rate: f32) -> f32 {
        let fc = 5.0;
        1.0 - (2.0 * std::f32::consts::PI * fc / sample_rate)
    }

    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        let out = input - self.prev_in + self.r * self.prev_out;
        self.prev_in = input;
        self.prev_out = if out.abs() < 1e-15 { 0.0 } else { out };
        self.prev_out
    }

    pub fn reset(&mut self) {
        self.prev_in = 0.0;
        self.prev_out = 0.0;
    }
}
