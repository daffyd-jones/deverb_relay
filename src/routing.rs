// src/routing.rs

use crate::dsp::bitcrush::StereoBitCrush;
use crate::dsp::chorus::StereoChorus;
use crate::dsp::compressor::StereoCompressor;
use crate::dsp::delay::StereoDelay;
use crate::dsp::reverb::FreeverbReverb;

/// Pre-read parameter values passed from the process loop.
/// This avoids advancing smoothers inside routing functions.
pub struct SubEffectParams {
    pub crush_on: bool,
    pub crush_bits: f32,
    pub crush_ds: f32,
    pub crush_mix: f32,

    pub comp_on: bool,
    pub comp_thresh: f32,
    pub comp_ratio: f32,
    pub comp_atk: f32,
    pub comp_rel: f32,
    pub comp_makeup: f32,
    pub comp_mix: f32,

    pub chorus_on: bool,
    pub chorus_rate: f32,
    pub chorus_depth: f32,
    pub chorus_fb: f32,
    pub chorus_mix: f32,
}

/// Delay wet chain: Crush → Compression → Chorus
#[inline]
fn process_delay_wet_chain(
    wet_l: f32,
    wet_r: f32,
    p: &SubEffectParams,
    crush: &mut StereoBitCrush,
    comp: &mut StereoCompressor,
    chorus: &mut StereoChorus,
) -> (f32, f32) {
    let (mut l, mut r) = (wet_l, wet_r);

    if p.crush_on {
        let (cl, cr) = crush.process(l, r, p.crush_bits, p.crush_ds, p.crush_mix);
        l = cl;
        r = cr;
    }

    if p.comp_on {
        let (cl, cr) = comp.process(
            l,
            r,
            p.comp_thresh,
            p.comp_ratio,
            p.comp_atk,
            p.comp_rel,
            p.comp_makeup,
            p.comp_mix,
        );
        l = cl;
        r = cr;
    }

    if p.chorus_on {
        let (cl, cr) = chorus.process(
            l,
            r,
            p.chorus_rate,
            p.chorus_depth,
            p.chorus_fb,
            p.chorus_mix,
        );
        l = cl;
        r = cr;
    }

    (l, r)
}

/// Reverb wet chain: Chorus → Crush → Compression
#[inline]
fn process_reverb_wet_chain(
    wet_l: f32,
    wet_r: f32,
    p: &SubEffectParams,
    chorus: &mut StereoChorus,
    crush: &mut StereoBitCrush,
    comp: &mut StereoCompressor,
) -> (f32, f32) {
    let (mut l, mut r) = (wet_l, wet_r);

    if p.chorus_on {
        let (cl, cr) = chorus.process(
            l,
            r,
            p.chorus_rate,
            p.chorus_depth,
            p.chorus_fb,
            p.chorus_mix,
        );
        l = cl;
        r = cr;
    }

    if p.crush_on {
        let (cl, cr) = crush.process(l, r, p.crush_bits, p.crush_ds, p.crush_mix);
        l = cl;
        r = cr;
    }

    if p.comp_on {
        let (cl, cr) = comp.process(
            l,
            r,
            p.comp_thresh,
            p.comp_ratio,
            p.comp_atk,
            p.comp_rel,
            p.comp_makeup,
            p.comp_mix,
        );
        l = cl;
        r = cr;
    }

    (l, r)
}

#[allow(clippy::too_many_arguments)]
#[inline]
pub fn process_delay_then_reverb(
    in_l: f32,
    in_r: f32,
    delay_enabled: bool,
    reverb_enabled: bool,
    delay_time_s: f32,
    delay_feedback: f32,
    delay_cross_fb: f32,
    delay_mix: f32,
    delay_fb_hp: f32,
    reverb_room: f32,
    reverb_damp: f32,
    reverb_width: f32,
    reverb_mix: f32,
    reverb_predelay_ms: f32,
    delay_sub: &SubEffectParams,
    reverb_sub: &SubEffectParams,
    delay: &mut StereoDelay,
    reverb: &mut FreeverbReverb,
    delay_crush: &mut StereoBitCrush,
    delay_comp: &mut StereoCompressor,
    delay_chorus: &mut StereoChorus,
    reverb_chorus: &mut StereoChorus,
    reverb_crush: &mut StereoBitCrush,
    reverb_comp: &mut StereoCompressor,
    _sr: f32,
) -> (f32, f32) {
    let (mut sig_l, mut sig_r) = (in_l, in_r);

    if delay_enabled {
        let (wet_l, wet_r) = delay.process(
            sig_l,
            sig_r,
            delay_time_s,
            delay_feedback,
            delay_cross_fb,
            delay_fb_hp,
        );
        let (proc_l, proc_r) = process_delay_wet_chain(
            wet_l,
            wet_r,
            delay_sub,
            delay_crush,
            delay_comp,
            delay_chorus,
        );
        sig_l += proc_l * delay_mix;
        sig_r += proc_r * delay_mix;
    }

    if reverb_enabled {
        let (wet_l, wet_r) = reverb.process(
            sig_l,
            sig_r,
            reverb_room,
            reverb_damp,
            reverb_width,
            reverb_predelay_ms,
        );
        let (proc_l, proc_r) = process_reverb_wet_chain(
            wet_l,
            wet_r,
            reverb_sub,
            reverb_chorus,
            reverb_crush,
            reverb_comp,
        );
        sig_l += proc_l * reverb_mix;
        sig_r += proc_r * reverb_mix;
    }

    (sig_l, sig_r)
}

#[allow(clippy::too_many_arguments)]
#[inline]
pub fn process_reverb_then_delay(
    in_l: f32,
    in_r: f32,
    delay_enabled: bool,
    reverb_enabled: bool,
    delay_time_s: f32,
    delay_feedback: f32,
    delay_cross_fb: f32,
    delay_mix: f32,
    delay_fb_hp: f32,
    reverb_room: f32,
    reverb_damp: f32,
    reverb_width: f32,
    reverb_mix: f32,
    reverb_predelay_ms: f32,
    delay_sub: &SubEffectParams,
    reverb_sub: &SubEffectParams,
    delay: &mut StereoDelay,
    reverb: &mut FreeverbReverb,
    delay_crush: &mut StereoBitCrush,
    delay_comp: &mut StereoCompressor,
    delay_chorus: &mut StereoChorus,
    reverb_chorus: &mut StereoChorus,
    reverb_crush: &mut StereoBitCrush,
    reverb_comp: &mut StereoCompressor,
    _sr: f32,
) -> (f32, f32) {
    let (mut sig_l, mut sig_r) = (in_l, in_r);

    if reverb_enabled {
        let (wet_l, wet_r) = reverb.process(
            sig_l,
            sig_r,
            reverb_room,
            reverb_damp,
            reverb_width,
            reverb_predelay_ms,
        );
        let (proc_l, proc_r) = process_reverb_wet_chain(
            wet_l,
            wet_r,
            reverb_sub,
            reverb_chorus,
            reverb_crush,
            reverb_comp,
        );
        sig_l += proc_l * reverb_mix;
        sig_r += proc_r * reverb_mix;
    }

    if delay_enabled {
        let (wet_l, wet_r) = delay.process(
            sig_l,
            sig_r,
            delay_time_s,
            delay_feedback,
            delay_cross_fb,
            delay_fb_hp,
        );
        let (proc_l, proc_r) = process_delay_wet_chain(
            wet_l,
            wet_r,
            delay_sub,
            delay_crush,
            delay_comp,
            delay_chorus,
        );
        sig_l += proc_l * delay_mix;
        sig_r += proc_r * delay_mix;
    }

    (sig_l, sig_r)
}

#[allow(clippy::too_many_arguments)]
#[inline]
pub fn process_parallel(
    in_l: f32,
    in_r: f32,
    delay_enabled: bool,
    reverb_enabled: bool,
    delay_time_s: f32,
    delay_feedback: f32,
    delay_cross_fb: f32,
    delay_mix: f32,
    delay_fb_hp: f32,
    reverb_room: f32,
    reverb_damp: f32,
    reverb_width: f32,
    reverb_mix: f32,
    reverb_predelay_ms: f32,
    delay_sub: &SubEffectParams,
    reverb_sub: &SubEffectParams,
    delay: &mut StereoDelay,
    reverb: &mut FreeverbReverb,
    delay_crush: &mut StereoBitCrush,
    delay_comp: &mut StereoCompressor,
    delay_chorus: &mut StereoChorus,
    reverb_chorus: &mut StereoChorus,
    reverb_crush: &mut StereoBitCrush,
    reverb_comp: &mut StereoCompressor,
    _sr: f32,
) -> (f32, f32) {
    let (mut sig_l, mut sig_r) = (in_l, in_r);

    if delay_enabled {
        let (wet_l, wet_r) = delay.process(
            in_l,
            in_r,
            delay_time_s,
            delay_feedback,
            delay_cross_fb,
            delay_fb_hp,
        );
        let (proc_l, proc_r) = process_delay_wet_chain(
            wet_l,
            wet_r,
            delay_sub,
            delay_crush,
            delay_comp,
            delay_chorus,
        );
        sig_l += proc_l * delay_mix;
        sig_r += proc_r * delay_mix;
    }

    if reverb_enabled {
        let (wet_l, wet_r) = reverb.process(
            in_l,
            in_r,
            reverb_room,
            reverb_damp,
            reverb_width,
            reverb_predelay_ms,
        );
        let (proc_l, proc_r) = process_reverb_wet_chain(
            wet_l,
            wet_r,
            reverb_sub,
            reverb_chorus,
            reverb_crush,
            reverb_comp,
        );
        sig_l += proc_l * reverb_mix;
        sig_r += proc_r * reverb_mix;
    }

    (sig_l, sig_r)
}

