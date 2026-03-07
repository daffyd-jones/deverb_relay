// src/dsp/chorus.rs

use super::circular_buffer::CircularBuffer;
use std::f32::consts::PI;

pub struct StereoChorus {
    buf_l: CircularBuffer,
    buf_r: CircularBuffer,
    lfo_phase: f32,
    sample_rate: f32,
}

impl StereoChorus {
    pub fn new(sample_rate: f32) -> Self {
        // Max depth 20ms + safety margin for interpolation
        let max_samples = (0.05 * sample_rate) as usize + 256;
        let mut s = Self {
            buf_l: CircularBuffer::new(max_samples),
            buf_r: CircularBuffer::new(max_samples),
            lfo_phase: 0.0,
            sample_rate,
        };
        // Pre-fill buffers with silence so initial reads are valid
        for _ in 0..s.buf_l.len() {
            s.buf_l.write_sample(0.0);
            s.buf_r.write_sample(0.0);
        }
        s
    }

    /// Returns (output_l, output_r) with dry/wet already blended.
    #[inline]
    pub fn process(
        &mut self,
        input_l: f32,
        input_r: f32,
        rate_hz: f32,
        depth: f32,
        feedback: f32,
        mix: f32,
    ) -> (f32, f32) {
        let max_depth_s: f32 = 0.02; // 20ms max modulation depth
        let depth_s = depth * max_depth_s;
        let centre = depth_s * self.sample_rate * 0.5;
        let min_delay: f32 = 5.0; // minimum delay to avoid comb filtering artifacts
        let max_buf = (self.buf_l.len() - 2) as f32;

        // Quadrature LFO
        let lfo_l = self.lfo_phase.sin();
        let lfo_r = (self.lfo_phase + PI * 0.5).sin();

        // Modulated delay times
        let delay_l = (centre + centre * lfo_l + min_delay).clamp(min_delay, max_buf);
        let delay_r = (centre + centre * lfo_r + min_delay).clamp(min_delay, max_buf);

        // Read modulated delays
        let delayed_l = self.buf_l.read_interpolated(delay_l);
        let delayed_r = self.buf_r.read_interpolated(delay_r);

        // Write input + gentle feedback into the delay line
        let fb = feedback.min(0.3);
        self.buf_l.write_sample(input_l + delayed_l * fb);
        self.buf_r.write_sample(input_r + delayed_r * fb);

        // Advance LFO
        self.lfo_phase += 2.0 * PI * rate_hz / self.sample_rate;
        if self.lfo_phase >= 2.0 * PI {
            self.lfo_phase -= 2.0 * PI;
        }

        // Blend
        let out_l = input_l * (1.0 - mix) + delayed_l * mix;
        let out_r = input_r * (1.0 - mix) + delayed_r * mix;

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        self.buf_l.clear();
        self.buf_r.clear();
        self.lfo_phase = 0.0;
        // Re-fill with silence
        for _ in 0..self.buf_l.len() {
            self.buf_l.write_sample(0.0);
            self.buf_r.write_sample(0.0);
        }
    }
}

