// src/dsp/reverb.rs

use super::circular_buffer::CircularBuffer;

const COMB_DELAYS_L: [usize; 4] = [1557, 1617, 1491, 1422];
const COMB_OFFSET_R: usize = 23;
const ALLPASS_DELAYS: [usize; 2] = [225, 556];
const ALLPASS_GAIN: f32 = 0.5;
const REFERENCE_SR: f32 = 44100.0;

struct CombFilter {
    buffer: Vec<f32>,
    write_pos: usize,
    filter_state: f32,
    delay: usize,
    buf_len: usize,
}

impl CombFilter {
    fn new(delay: usize) -> Self {
        let buf_len = (delay + 64).next_power_of_two();
        Self {
            buffer: vec![0.0; buf_len],
            write_pos: 0,
            filter_state: 0.0,
            delay,
            buf_len,
        }
    }

    #[inline(always)]
    fn process(&mut self, input: f32, feedback: f32, damp1: f32, damp2: f32) -> f32 {
        let read_pos = (self.write_pos + self.buf_len - self.delay) % self.buf_len;
        let delayed = self.buffer[read_pos];

        self.filter_state = delayed * damp2 + self.filter_state * damp1;

        // Denormal prevention
        if self.filter_state.abs() < 1e-15 {
            self.filter_state = 0.0;
        }

        self.buffer[self.write_pos] = input + self.filter_state * feedback;
        self.write_pos = (self.write_pos + 1) % self.buf_len;

        delayed
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
        self.filter_state = 0.0;
    }
}

struct AllpassFilter {
    buffer: Vec<f32>,
    write_pos: usize,
    delay: usize,
    buf_len: usize,
}

impl AllpassFilter {
    fn new(delay: usize) -> Self {
        let buf_len = (delay + 64).next_power_of_two();
        Self {
            buffer: vec![0.0; buf_len],
            write_pos: 0,
            delay,
            buf_len,
        }
    }

    #[inline(always)]
    fn process(&mut self, input: f32, gain: f32) -> f32 {
        let read_pos = (self.write_pos + self.buf_len - self.delay) % self.buf_len;
        let delayed = self.buffer[read_pos];

        self.buffer[self.write_pos] = input + delayed * gain;
        self.write_pos = (self.write_pos + 1) % self.buf_len;

        delayed - input * gain
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

struct PreDelay {
    buffer: CircularBuffer,
    sample_rate: f32,
}

impl PreDelay {
    fn new(sample_rate: f32) -> Self {
        let max_samples = (0.1 * sample_rate) as usize + 64;
        Self {
            buffer: CircularBuffer::new(max_samples),
            sample_rate,
        }
    }

    #[inline(always)]
    fn process(&mut self, input: f32, delay_ms: f32) -> f32 {
        self.buffer.write_sample(input);
        let delay_samples = (delay_ms * 0.001 * self.sample_rate).max(1.0) as usize;
        self.buffer.read_sample(delay_samples)
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }
}

pub struct FreeverbReverb {
    combs_l: Vec<CombFilter>,
    combs_r: Vec<CombFilter>,
    allpasses_l: Vec<AllpassFilter>,
    allpasses_r: Vec<AllpassFilter>,
    predelay_l: PreDelay,
    predelay_r: PreDelay,
}

impl FreeverbReverb {
    pub fn new(sample_rate: f32) -> Self {
        let sr_ratio = sample_rate / REFERENCE_SR;

        let combs_l = COMB_DELAYS_L
            .iter()
            .map(|&d| CombFilter::new((d as f32 * sr_ratio) as usize))
            .collect();

        let combs_r = COMB_DELAYS_L
            .iter()
            .map(|&d| CombFilter::new(((d + COMB_OFFSET_R) as f32 * sr_ratio) as usize))
            .collect();

        let allpasses_l = ALLPASS_DELAYS
            .iter()
            .map(|&d| AllpassFilter::new((d as f32 * sr_ratio) as usize))
            .collect();

        let allpasses_r = ALLPASS_DELAYS
            .iter()
            .map(|&d| AllpassFilter::new((d as f32 * sr_ratio) as usize))
            .collect();

        Self {
            combs_l,
            combs_r,
            allpasses_l,
            allpasses_r,
            predelay_l: PreDelay::new(sample_rate),
            predelay_r: PreDelay::new(sample_rate),
        }
    }

    #[inline]
    pub fn process(
        &mut self,
        input_l: f32,
        input_r: f32,
        room_size: f32,
        damping: f32,
        width: f32,
        predelay_ms: f32,
    ) -> (f32, f32) {
        let pd_l = self.predelay_l.process(input_l, predelay_ms);
        let pd_r = self.predelay_r.process(input_r, predelay_ms);

        let mono = (pd_l + pd_r) * 0.015;

        let feedback = (0.7 + room_size * 0.28).min(0.98);
        let damp1 = damping * 0.4;
        let damp2 = 1.0 - damp1;

        let mut sum_l: f32 = 0.0;
        let mut sum_r: f32 = 0.0;

        for comb in self.combs_l.iter_mut() {
            sum_l += comb.process(mono, feedback, damp1, damp2);
        }
        for comb in self.combs_r.iter_mut() {
            sum_r += comb.process(mono, feedback, damp1, damp2);
        }

        for ap in self.allpasses_l.iter_mut() {
            sum_l = ap.process(sum_l, ALLPASS_GAIN);
        }
        for ap in self.allpasses_r.iter_mut() {
            sum_r = ap.process(sum_r, ALLPASS_GAIN);
        }

        if width < 1.0 {
            let mono_wet = (sum_l + sum_r) * 0.5;
            sum_l = mono_wet * (1.0 - width) + sum_l * width;
            sum_r = mono_wet * (1.0 - width) + sum_r * width;
        }

        (sum_l, sum_r)
    }

    pub fn reset(&mut self) {
        for c in &mut self.combs_l {
            c.reset();
        }
        for c in &mut self.combs_r {
            c.reset();
        }
        for a in &mut self.allpasses_l {
            a.reset();
        }
        for a in &mut self.allpasses_r {
            a.reset();
        }
        self.predelay_l.reset();
        self.predelay_r.reset();
    }
}

