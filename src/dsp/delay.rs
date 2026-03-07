// src/dsp/delay.rs

use super::circular_buffer::CircularBuffer;

/// One-pole high-pass for the delay feedback path.
struct OnePoleHP {
    prev_in: f32,
    prev_out: f32,
    coeff: f32,
}

impl OnePoleHP {
    fn new(sample_rate: f32, cutoff_hz: f32) -> Self {
        Self {
            prev_in: 0.0,
            prev_out: 0.0,
            coeff: Self::calc_coeff(sample_rate, cutoff_hz),
        }
    }

    #[inline(always)]
    fn calc_coeff(sample_rate: f32, cutoff_hz: f32) -> f32 {
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz.max(1.0));
        let dt = 1.0 / sample_rate;
        rc / (rc + dt)
    }

    #[inline(always)]
    fn set_cutoff(&mut self, sample_rate: f32, cutoff_hz: f32) {
        self.coeff = Self::calc_coeff(sample_rate, cutoff_hz);
    }

    #[inline(always)]
    fn process(&mut self, input: f32) -> f32 {
        let out = self.coeff * (self.prev_out + input - self.prev_in);
        self.prev_in = input;
        self.prev_out = out;
        out
    }

    fn reset(&mut self) {
        self.prev_in = 0.0;
        self.prev_out = 0.0;
    }
}

pub struct StereoDelay {
    buf_l: CircularBuffer,
    buf_r: CircularBuffer,
    hp_l: OnePoleHP,
    hp_r: OnePoleHP,
    sample_rate: f32,
}

impl StereoDelay {
    pub fn new(sample_rate: f32) -> Self {
        let max_samples = (5.0 * sample_rate) as usize + 1024;
        Self {
            buf_l: CircularBuffer::new(max_samples),
            buf_r: CircularBuffer::new(max_samples),
            hp_l: OnePoleHP::new(sample_rate, 80.0),
            hp_r: OnePoleHP::new(sample_rate, 80.0),
            sample_rate,
        }
    }

    /// Returns the *wet-only* echo signal (caller handles mix).
    #[inline]
    pub fn process(
        &mut self,
        input_l: f32,
        input_r: f32,
        delay_time_s: f32,
        feedback: f32,
        cross_feedback: f32,
        fb_hp_hz: f32,
    ) -> (f32, f32) {
        self.hp_l.set_cutoff(self.sample_rate, fb_hp_hz);
        self.hp_r.set_cutoff(self.sample_rate, fb_hp_hz);

        let delay_samples = (delay_time_s * self.sample_rate)
            .max(1.0)
            .min((self.buf_l.len() - 2) as f32);

        // Read echoes
        let echo_l = self.buf_l.read_interpolated(delay_samples);
        let echo_r = self.buf_r.read_interpolated(delay_samples);

        let fb = feedback.min(0.95);
        let xfb = cross_feedback.min(0.5);

        // Feedback with cross-feedback, high-pass filtered
        let fb_sig_l = echo_l * fb + echo_r * xfb;
        let fb_sig_r = echo_r * fb + echo_l * xfb;
        let fb_l = self.hp_l.process(fb_sig_l);
        let fb_r = self.hp_r.process(fb_sig_r);

        // Write input + filtered feedback
        self.buf_l.write_sample(input_l + fb_l);
        self.buf_r.write_sample(input_r + fb_r);

        (echo_l, echo_r)
    }

    pub fn reset(&mut self) {
        self.buf_l.clear();
        self.buf_r.clear();
        self.hp_l.reset();
        self.hp_r.reset();
    }
}
