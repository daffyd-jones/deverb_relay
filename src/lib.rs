// src/lib.rs

use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

mod dsp;
mod editor;
mod params;
mod routing;

use dsp::bitcrush::StereoBitCrush;
use dsp::chorus::StereoChorus;
use dsp::compressor::StereoCompressor;
use dsp::dc_blocker::DcBlocker;
use dsp::delay::StereoDelay;
use dsp::reverb::FreeverbReverb;
use params::{DelayReverbParams, RoutingMode};

pub struct DelayReverbPlugin {
    params: Arc<DelayReverbParams>,

    delay: StereoDelay,
    reverb: FreeverbReverb,

    delay_crush: StereoBitCrush,
    delay_comp: StereoCompressor,
    delay_chorus: StereoChorus,

    reverb_chorus: StereoChorus,
    reverb_crush: StereoBitCrush,
    reverb_comp: StereoCompressor,

    dc_blocker_l: DcBlocker,
    dc_blocker_r: DcBlocker,

    sample_rate: f32,
}

impl Default for DelayReverbPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(DelayReverbParams::default()),

            delay: StereoDelay::new(44100.0),
            reverb: FreeverbReverb::new(44100.0),

            delay_crush: StereoBitCrush::new(),
            delay_comp: StereoCompressor::new(44100.0),
            delay_chorus: StereoChorus::new(44100.0),

            reverb_chorus: StereoChorus::new(44100.0),
            reverb_crush: StereoBitCrush::new(),
            reverb_comp: StereoCompressor::new(44100.0),

            dc_blocker_l: DcBlocker::new(44100.0),
            dc_blocker_r: DcBlocker::new(44100.0),

            sample_rate: 44100.0,
        }
    }
}

impl Plugin for DelayReverbPlugin {
    const NAME: &'static str = "Delay Reverb FX";
    const VENDOR: &'static str = "Custom";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sr = buffer_config.sample_rate;
        self.sample_rate = sr;

        self.delay = StereoDelay::new(sr);
        self.reverb = FreeverbReverb::new(sr);

        self.delay_crush = StereoBitCrush::new();
        self.delay_comp = StereoCompressor::new(sr);
        self.delay_chorus = StereoChorus::new(sr);

        self.reverb_chorus = StereoChorus::new(sr);
        self.reverb_crush = StereoBitCrush::new();
        self.reverb_comp = StereoCompressor::new(sr);

        self.dc_blocker_l = DcBlocker::new(sr);
        self.dc_blocker_r = DcBlocker::new(sr);

        true
    }

    fn reset(&mut self) {
        self.delay.reset();
        self.reverb.reset();
        self.delay_crush.reset();
        self.delay_comp.reset();
        self.delay_chorus.reset();
        self.reverb_chorus.reset();
        self.reverb_crush.reset();
        self.reverb_comp.reset();
        self.dc_blocker_l.reset();
        self.dc_blocker_r.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let sr = self.sample_rate;
        let p = &self.params;

        let transport = context.transport();
        let bpm = transport.tempo.unwrap_or(120.0);

        for mut frame in buffer.iter_samples() {
            let in_l = *frame.get_mut(0).unwrap();
            let in_r = *frame.get_mut(1).unwrap();

            // Global smoothed params
            let main_mix = p.main_mix.smoothed.next();
            let output_gain_db = p.output_gain.smoothed.next();
            let output_gain = nih_plug::util::db_to_gain(output_gain_db);

            let routing = p.routing_mode.value();
            let delay_enabled = p.delay.enabled.value();
            let reverb_enabled = p.reverb.enabled.value();

            // Delay params - always advance smoothers
            let delay_time_s = if p.delay.tempo_sync.value() {
                let beat_s = 60.0 / bpm as f32;
                // Still advance the time smoother to keep it in sync
                let _ = p.delay.time_ms.smoothed.next();
                params::sync_division_to_beats(p.delay.sync_division.value()) * beat_s
            } else {
                p.delay.time_ms.smoothed.next() * 0.001
            };
            let delay_feedback = p.delay.feedback.smoothed.next();
            let delay_cross_fb = p.delay.cross_feedback.smoothed.next();
            let delay_mix = p.delay.mix.smoothed.next();
            let delay_fb_hp = p.delay.feedback_hp.smoothed.next();

            // Reverb params - always advance smoothers
            let reverb_room = p.reverb.room_size.smoothed.next();
            let reverb_damp = p.reverb.damping.smoothed.next();
            let reverb_width = p.reverb.width.smoothed.next();
            let reverb_mix = p.reverb.mix.smoothed.next();
            let reverb_predelay_ms = p.reverb.predelay_ms.smoothed.next();

            // Always advance sub-effect smoothers even when disabled
            let dc_bits = p.delay_crush.bits.smoothed.next();
            let dc_ds = p.delay_crush.downsample.smoothed.next();
            let dc_mix = p.delay_crush.mix.smoothed.next();
            let dc_on = p.delay_crush.enabled.value();

            let dco_thresh = p.delay_comp.threshold.smoothed.next();
            let dco_ratio = p.delay_comp.ratio.smoothed.next();
            let dco_atk = p.delay_comp.attack_ms.smoothed.next();
            let dco_rel = p.delay_comp.release_ms.smoothed.next();
            let dco_makeup = p.delay_comp.makeup_db.smoothed.next();
            let dco_mix = p.delay_comp.mix.smoothed.next();
            let dco_on = p.delay_comp.enabled.value();

            let dch_rate = p.delay_chorus.rate.smoothed.next();
            let dch_depth = p.delay_chorus.depth.smoothed.next();
            let dch_fb = p.delay_chorus.feedback.smoothed.next();
            let dch_mix = p.delay_chorus.mix.smoothed.next();
            let dch_on = p.delay_chorus.enabled.value();

            let rch_rate = p.reverb_chorus.rate.smoothed.next();
            let rch_depth = p.reverb_chorus.depth.smoothed.next();
            let rch_fb = p.reverb_chorus.feedback.smoothed.next();
            let rch_mix = p.reverb_chorus.mix.smoothed.next();
            let rch_on = p.reverb_chorus.enabled.value();

            let rc_bits = p.reverb_crush.bits.smoothed.next();
            let rc_ds = p.reverb_crush.downsample.smoothed.next();
            let rc_mix = p.reverb_crush.mix.smoothed.next();
            let rc_on = p.reverb_crush.enabled.value();

            let rco_thresh = p.reverb_comp.threshold.smoothed.next();
            let rco_ratio = p.reverb_comp.ratio.smoothed.next();
            let rco_atk = p.reverb_comp.attack_ms.smoothed.next();
            let rco_rel = p.reverb_comp.release_ms.smoothed.next();
            let rco_makeup = p.reverb_comp.makeup_db.smoothed.next();
            let rco_mix = p.reverb_comp.mix.smoothed.next();
            let rco_on = p.reverb_comp.enabled.value();

            // Pack sub-effect params
            let delay_sub = routing::SubEffectParams {
                crush_on: dc_on,
                crush_bits: dc_bits,
                crush_ds: dc_ds,
                crush_mix: dc_mix,
                comp_on: dco_on,
                comp_thresh: dco_thresh,
                comp_ratio: dco_ratio,
                comp_atk: dco_atk,
                comp_rel: dco_rel,
                comp_makeup: dco_makeup,
                comp_mix: dco_mix,
                chorus_on: dch_on,
                chorus_rate: dch_rate,
                chorus_depth: dch_depth,
                chorus_fb: dch_fb,
                chorus_mix: dch_mix,
            };

            let reverb_sub = routing::SubEffectParams {
                crush_on: rc_on,
                crush_bits: rc_bits,
                crush_ds: rc_ds,
                crush_mix: rc_mix,
                comp_on: rco_on,
                comp_thresh: rco_thresh,
                comp_ratio: rco_ratio,
                comp_atk: rco_atk,
                comp_rel: rco_rel,
                comp_makeup: rco_makeup,
                comp_mix: rco_mix,
                chorus_on: rch_on,
                chorus_rate: rch_rate,
                chorus_depth: rch_depth,
                chorus_fb: rch_fb,
                chorus_mix: rch_mix,
            };

            let (mut out_l, mut out_r) = match routing {
                RoutingMode::DelayThenReverb => routing::process_delay_then_reverb(
                    in_l,
                    in_r,
                    delay_enabled,
                    reverb_enabled,
                    delay_time_s,
                    delay_feedback,
                    delay_cross_fb,
                    delay_mix,
                    delay_fb_hp,
                    reverb_room,
                    reverb_damp,
                    reverb_width,
                    reverb_mix,
                    reverb_predelay_ms,
                    &delay_sub,
                    &reverb_sub,
                    &mut self.delay,
                    &mut self.reverb,
                    &mut self.delay_crush,
                    &mut self.delay_comp,
                    &mut self.delay_chorus,
                    &mut self.reverb_chorus,
                    &mut self.reverb_crush,
                    &mut self.reverb_comp,
                    sr,
                ),
                RoutingMode::ReverbThenDelay => routing::process_reverb_then_delay(
                    in_l,
                    in_r,
                    delay_enabled,
                    reverb_enabled,
                    delay_time_s,
                    delay_feedback,
                    delay_cross_fb,
                    delay_mix,
                    delay_fb_hp,
                    reverb_room,
                    reverb_damp,
                    reverb_width,
                    reverb_mix,
                    reverb_predelay_ms,
                    &delay_sub,
                    &reverb_sub,
                    &mut self.delay,
                    &mut self.reverb,
                    &mut self.delay_crush,
                    &mut self.delay_comp,
                    &mut self.delay_chorus,
                    &mut self.reverb_chorus,
                    &mut self.reverb_crush,
                    &mut self.reverb_comp,
                    sr,
                ),
                RoutingMode::Parallel => routing::process_parallel(
                    in_l,
                    in_r,
                    delay_enabled,
                    reverb_enabled,
                    delay_time_s,
                    delay_feedback,
                    delay_cross_fb,
                    delay_mix,
                    delay_fb_hp,
                    reverb_room,
                    reverb_damp,
                    reverb_width,
                    reverb_mix,
                    reverb_predelay_ms,
                    &delay_sub,
                    &reverb_sub,
                    &mut self.delay,
                    &mut self.reverb,
                    &mut self.delay_crush,
                    &mut self.delay_comp,
                    &mut self.delay_chorus,
                    &mut self.reverb_chorus,
                    &mut self.reverb_crush,
                    &mut self.reverb_comp,
                    sr,
                ),
            };

            // Main dry/wet
            out_l = in_l * (1.0 - main_mix) + out_l * main_mix;
            out_r = in_r * (1.0 - main_mix) + out_r * main_mix;

            // Output gain
            out_l *= output_gain;
            out_r *= output_gain;

            // DC blocker
            out_l = self.dc_blocker_l.process(out_l);
            out_r = self.dc_blocker_r.process(out_r);

            // Soft clip
            out_l = out_l.tanh();
            out_r = out_r.tanh();

            *frame.get_mut(0).unwrap() = out_l;
            *frame.get_mut(1).unwrap() = out_r;
        }

        ProcessStatus::Tail(self.compute_tail())
    }
}

impl DelayReverbPlugin {
    fn compute_tail(&self) -> u32 {
        let p = &self.params;
        let sr = self.sample_rate;
        let mut tail: f32 = 0.0;

        if p.delay.enabled.value() {
            let time_s = p.delay.time_ms.value() * 0.001;
            let fb = p.delay.feedback.value();
            if fb < 0.999 {
                let delay_tail = time_s / (1.0 - fb);
                tail = tail.max(delay_tail);
            } else {
                tail = tail.max(10.0);
            }
        }

        if p.reverb.enabled.value() {
            let room = p.reverb.room_size.value();
            let reverb_tail = 1.0 + room * 5.0;
            tail = tail.max(reverb_tail);
        }

        (tail * sr) as u32
    }
}

impl ClapPlugin for DelayReverbPlugin {
    const CLAP_ID: &'static str = "com.custom.delay-reverb-fx";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Delay/Reverb with Chorus, Compression, and BitCrush sub-effects");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Delay,
        ClapFeature::Reverb,
    ];
}

impl Vst3Plugin for DelayReverbPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"DlyRevFX__Custom";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Delay,
        Vst3SubCategory::Reverb,
    ];
}

nih_export_clap!(DelayReverbPlugin);
nih_export_vst3!(DelayReverbPlugin);
