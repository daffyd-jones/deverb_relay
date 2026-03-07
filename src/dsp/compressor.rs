// src/dsp/compressor.rs

pub struct StereoCompressor {
    env_l: f32,
    env_r: f32,
    sample_rate: f32,
}

impl StereoCompressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            env_l: 0.0,
            env_r: 0.0,
            sample_rate,
        }
    }

    #[inline]
    pub fn process(
        &mut self,
        input_l: f32,
        input_r: f32,
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        makeup_db: f32,
        mix: f32,
    ) -> (f32, f32) {
        let attack_s = (attack_ms * 0.001).max(1.0 / self.sample_rate);
        let release_s = (release_ms * 0.001).max(1.0 / self.sample_rate);
        let attack_coeff = (-1.0 / (attack_s * self.sample_rate)).exp();
        let release_coeff = (-1.0 / (release_s * self.sample_rate)).exp();
        let makeup = nih_plug::util::db_to_gain(makeup_db);

        // Envelope follower — left
        let level_l = input_l.abs();
        let coeff_l = if level_l > self.env_l {
            attack_coeff
        } else {
            release_coeff
        };
        self.env_l = coeff_l * self.env_l + (1.0 - coeff_l) * level_l;
        if self.env_l < 1e-15 {
            self.env_l = 0.0;
        }

        // Envelope follower — right
        let level_r = input_r.abs();
        let coeff_r = if level_r > self.env_r {
            attack_coeff
        } else {
            release_coeff
        };
        self.env_r = coeff_r * self.env_r + (1.0 - coeff_r) * level_r;
        if self.env_r < 1e-15 {
            self.env_r = 0.0;
        }

        // Gain reduction
        let gain_l = Self::compute_gain(self.env_l, threshold_db, ratio, makeup);
        let gain_r = Self::compute_gain(self.env_r, threshold_db, ratio, makeup);

        let compressed_l = input_l * gain_l;
        let compressed_r = input_r * gain_r;

        // Dry/wet
        (
            input_l * (1.0 - mix) + compressed_l * mix,
            input_r * (1.0 - mix) + compressed_r * mix,
        )
    }

    #[inline(always)]
    fn compute_gain(envelope: f32, threshold_db: f32, ratio: f32, makeup: f32) -> f32 {
        if envelope < 1e-9 {
            return makeup;
        }
        let env_db = 20.0 * envelope.log10();
        let over = (env_db - threshold_db).max(0.0);
        let gain_reduction_db = -(over - over / ratio.max(1.0));
        10.0_f32.powf(gain_reduction_db / 20.0) * makeup
    }

    pub fn reset(&mut self) {
        self.env_l = 0.0;
        self.env_r = 0.0;
    }
}

