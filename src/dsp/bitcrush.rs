// src/dsp/bitcrush.rs

/// Bit depth reduction + sample rate reduction.
/// Each channel has its own sample-and-hold state.
pub struct StereoBitCrush {
    hold_l: f32,
    hold_r: f32,
    counter_l: u32,
    counter_r: u32,
}

impl StereoBitCrush {
    pub fn new() -> Self {
        Self {
            hold_l: 0.0,
            hold_r: 0.0,
            counter_l: 0,
            counter_r: 0,
        }
    }

    #[inline]
    pub fn process(
        &mut self,
        input_l: f32,
        input_r: f32,
        bits: f32,
        downsample: f32,
        mix: f32,
    ) -> (f32, f32) {
        let bits_clamped = bits.round().clamp(2.0, 16.0);
        let levels = 2.0_f32.powf(bits_clamped - 1.0);

        // Bit reduce
        let crushed_l = (input_l * levels).round() / levels;
        let crushed_r = (input_r * levels).round() / levels;

        // Sample-rate reduction (per-channel counters)
        let ds = (downsample.round() as u32).max(1);

        self.counter_l += 1;
        if self.counter_l >= ds {
            self.counter_l = 0;
            self.hold_l = crushed_l;
        }

        self.counter_r += 1;
        if self.counter_r >= ds {
            self.counter_r = 0;
            self.hold_r = crushed_r;
        }

        // Dry/wet blend
        (
            input_l * (1.0 - mix) + self.hold_l * mix,
            input_r * (1.0 - mix) + self.hold_r * mix,
        )
    }

    pub fn reset(&mut self) {
        self.hold_l = 0.0;
        self.hold_r = 0.0;
        self.counter_l = 0;
        self.counter_r = 0;
    }
}

