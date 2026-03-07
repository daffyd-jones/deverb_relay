// src/dsp/circular_buffer.rs

/// Power-of-two circular buffer with single-sample and interpolated reads.
pub struct CircularBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
    mask: usize,
}

impl CircularBuffer {
    pub fn new(min_size: usize) -> Self {
        let size = min_size.max(4).next_power_of_two();
        Self {
            buffer: vec![0.0; size],
            write_pos: 0,
            mask: size - 1,
        }
    }

    /// Write one sample and advance write head.
    #[inline(always)]
    pub fn write_sample(&mut self, sample: f32) {
        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) & self.mask;
    }

    /// Read a sample `delay` steps behind the *current* write position.
    /// delay=1 returns the most recently written sample.
    /// delay=0 is clamped to 1.
    #[inline(always)]
    pub fn read_sample(&self, delay: usize) -> f32 {
        let d = delay.max(1).min(self.mask);
        let pos = self.write_pos.wrapping_sub(d) & self.mask;
        self.buffer[pos]
    }

    /// Read with sub-sample interpolation. `delay` is in samples (fractional).
    /// delay < 1.0 is clamped to 1.0.
    #[inline(always)]
    pub fn read_interpolated(&self, delay: f32) -> f32 {
        let delay = delay.max(1.0).min(self.mask as f32);
        let idx = delay as usize;
        let frac = delay - idx as f32;

        let s0 = self.read_sample(idx);
        let s1 = self.read_sample(idx + 1);

        // Linear interpolation
        s0 + (s1 - s0) * frac
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
