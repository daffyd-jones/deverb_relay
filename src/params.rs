// src/params.rs

use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum RoutingMode {
    #[id = "d_then_r"]
    #[name = "Delay → Reverb"]
    DelayThenReverb,

    #[id = "r_then_d"]
    #[name = "Reverb → Delay"]
    ReverbThenDelay,

    #[id = "parallel"]
    #[name = "Parallel"]
    Parallel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum SyncDivision {
    #[id = "1_1"]
    #[name = "1/1"]
    Whole,
    #[id = "1_2"]
    #[name = "1/2"]
    Half,
    #[id = "1_2d"]
    #[name = "1/2 Dotted"]
    HalfDotted,
    #[id = "1_4"]
    #[name = "1/4"]
    Quarter,
    #[id = "1_4d"]
    #[name = "1/4 Dotted"]
    QuarterDotted,
    #[id = "1_4t"]
    #[name = "1/4 Triplet"]
    QuarterTriplet,
    #[id = "1_8"]
    #[name = "1/8"]
    Eighth,
    #[id = "1_8d"]
    #[name = "1/8 Dotted"]
    EighthDotted,
    #[id = "1_8t"]
    #[name = "1/8 Triplet"]
    EighthTriplet,
    #[id = "1_16"]
    #[name = "1/16"]
    Sixteenth,
    #[id = "1_16d"]
    #[name = "1/16 Dotted"]
    SixteenthDotted,
    #[id = "1_16t"]
    #[name = "1/16 Triplet"]
    SixteenthTriplet,
}

pub fn sync_division_to_beats(div: SyncDivision) -> f32 {
    match div {
        SyncDivision::Whole => 4.0,
        SyncDivision::Half => 2.0,
        SyncDivision::HalfDotted => 3.0,
        SyncDivision::Quarter => 1.0,
        SyncDivision::QuarterDotted => 1.5,
        SyncDivision::QuarterTriplet => 2.0 / 3.0,
        SyncDivision::Eighth => 0.5,
        SyncDivision::EighthDotted => 0.75,
        SyncDivision::EighthTriplet => 1.0 / 3.0,
        SyncDivision::Sixteenth => 0.25,
        SyncDivision::SixteenthDotted => 0.375,
        SyncDivision::SixteenthTriplet => 1.0 / 6.0,
    }
}

// ─── Top-level params ───────────────────────────────────────────────

#[derive(Params)]
pub struct DelayReverbParams {
    /// Editor state persisted with the plugin
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "main_mix"]
    pub main_mix: FloatParam,

    #[id = "routing"]
    pub routing_mode: EnumParam<RoutingMode>,

    #[id = "out_gain"]
    pub output_gain: FloatParam,

    // ── Primary Effects ──
    #[nested(group = "Delay")]
    pub delay: DelayGroupParams,

    #[nested(group = "Reverb")]
    pub reverb: ReverbGroupParams,

    // ── Delay sub-effects ──
    #[nested(id_prefix = "dc", group = "Delay Crush")]
    pub delay_crush: CrushGroupParams,

    #[nested(id_prefix = "dco", group = "Delay Comp")]
    pub delay_comp: CompGroupParams,

    #[nested(id_prefix = "dch", group = "Delay Chorus")]
    pub delay_chorus: ChorusGroupParams,

    // ── Reverb sub-effects ──
    #[nested(id_prefix = "rch", group = "Reverb Chorus")]
    pub reverb_chorus: ChorusGroupParams,

    #[nested(id_prefix = "rc", group = "Reverb Crush")]
    pub reverb_crush: CrushGroupParams,

    #[nested(id_prefix = "rco", group = "Reverb Comp")]
    pub reverb_comp: CompGroupParams,
}

impl Default for DelayReverbParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(900, 700),

            main_mix: FloatParam::new("Main Mix", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_unit("%")
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),

            routing_mode: EnumParam::new("Routing", RoutingMode::DelayThenReverb),

            output_gain: FloatParam::new(
                "Output Gain",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 6.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB")
            .with_step_size(0.1),

            delay: DelayGroupParams::default(),
            reverb: ReverbGroupParams::default(),

            delay_crush: CrushGroupParams::default(),
            delay_comp: CompGroupParams::default(),
            delay_chorus: ChorusGroupParams::default(),

            reverb_chorus: ChorusGroupParams::default(),
            reverb_crush: CrushGroupParams::default(),
            reverb_comp: CompGroupParams::default(),
        }
    }
}

// ─── Delay ──────────────────────────────────────────────────────────

#[derive(Params)]
pub struct DelayGroupParams {
    #[id = "dly_on"]
    pub enabled: BoolParam,
    #[id = "dly_time"]
    pub time_ms: FloatParam,
    #[id = "dly_fb"]
    pub feedback: FloatParam,
    #[id = "dly_xfb"]
    pub cross_feedback: FloatParam,
    #[id = "dly_mix"]
    pub mix: FloatParam,
    #[id = "dly_sync"]
    pub tempo_sync: BoolParam,
    #[id = "dly_div"]
    pub sync_division: EnumParam<SyncDivision>,
    #[id = "dly_fbhp"]
    pub feedback_hp: FloatParam,
}

impl Default for DelayGroupParams {
    fn default() -> Self {
        Self {
            enabled: BoolParam::new("Delay On", true),
            time_ms: FloatParam::new(
                "Delay Time",
                300.0,
                FloatRange::Skewed {
                    min: 10.0,
                    max: 2000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(100.0))
            .with_unit(" ms")
            .with_step_size(0.1),

            feedback: FloatParam::new(
                "Delay Feedback",
                0.4,
                FloatRange::Linear {
                    min: 0.0,
                    max: 0.95,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            cross_feedback: FloatParam::new(
                "Delay Cross FB",
                0.0,
                FloatRange::Linear { min: 0.0, max: 0.5 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            mix: FloatParam::new("Delay Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0)),

            tempo_sync: BoolParam::new("Delay Sync", false),
            sync_division: EnumParam::new("Delay Division", SyncDivision::Quarter),

            feedback_hp: FloatParam::new(
                "Delay FB HP",
                80.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 500.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" Hz")
            .with_step_size(1.0),
        }
    }
}

// ─── Reverb ─────────────────────────────────────────────────────────

#[derive(Params)]
pub struct ReverbGroupParams {
    #[id = "rev_on"]
    pub enabled: BoolParam,
    #[id = "rev_room"]
    pub room_size: FloatParam,
    #[id = "rev_damp"]
    pub damping: FloatParam,
    #[id = "rev_width"]
    pub width: FloatParam,
    #[id = "rev_mix"]
    pub mix: FloatParam,
    #[id = "rev_pdly"]
    pub predelay_ms: FloatParam,
}

impl Default for ReverbGroupParams {
    fn default() -> Self {
        Self {
            enabled: BoolParam::new("Reverb On", true),
            room_size: FloatParam::new("Room Size", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(100.0)),

            damping: FloatParam::new("Damping", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(100.0)),

            width: FloatParam::new("Width", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0)),

            mix: FloatParam::new("Reverb Mix", 0.3, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0)),

            predelay_ms: FloatParam::new(
                "Pre-delay",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" ms")
            .with_step_size(0.1),
        }
    }
}

// ─── Chorus sub-effect ──────────────────────────────────────────────

#[derive(Params)]
pub struct ChorusGroupParams {
    #[id = "cho_on"]
    pub enabled: BoolParam,
    #[id = "cho_rate"]
    pub rate: FloatParam,
    #[id = "cho_depth"]
    pub depth: FloatParam,
    #[id = "cho_fb"]
    pub feedback: FloatParam,
    #[id = "cho_mix"]
    pub mix: FloatParam,
}

impl Default for ChorusGroupParams {
    fn default() -> Self {
        Self {
            enabled: BoolParam::new("Chorus On", false),
            rate: FloatParam::new(
                "Chorus Rate",
                1.5,
                FloatRange::Skewed {
                    min: 0.05,
                    max: 10.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" Hz")
            .with_step_size(0.01),

            depth: FloatParam::new(
                "Chorus Depth",
                0.3,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            feedback: FloatParam::new(
                "Chorus Feedback",
                0.1,
                FloatRange::Linear { min: 0.0, max: 0.5 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),

            mix: FloatParam::new("Chorus Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0)),
        }
    }
}

// ─── Crush sub-effect ───────────────────────────────────────────────

#[derive(Params)]
pub struct CrushGroupParams {
    #[id = "cru_on"]
    pub enabled: BoolParam,
    #[id = "cru_bits"]
    pub bits: FloatParam,
    #[id = "cru_ds"]
    pub downsample: FloatParam,
    #[id = "cru_mix"]
    pub mix: FloatParam,
}

impl Default for CrushGroupParams {
    fn default() -> Self {
        Self {
            enabled: BoolParam::new("Crush On", false),
            bits: FloatParam::new(
                "Crush Bits",
                12.0,
                FloatRange::Linear {
                    min: 2.0,
                    max: 16.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_step_size(1.0),

            downsample: FloatParam::new(
                "Crush Downsample",
                1.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 32.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_step_size(1.0),

            mix: FloatParam::new("Crush Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0)),
        }
    }
}

// ─── Compressor sub-effect ──────────────────────────────────────────

#[derive(Params)]
pub struct CompGroupParams {
    #[id = "cmp_on"]
    pub enabled: BoolParam,
    #[id = "cmp_thresh"]
    pub threshold: FloatParam,
    #[id = "cmp_ratio"]
    pub ratio: FloatParam,
    #[id = "cmp_atk"]
    pub attack_ms: FloatParam,
    #[id = "cmp_rel"]
    pub release_ms: FloatParam,
    #[id = "cmp_makeup"]
    pub makeup_db: FloatParam,
    #[id = "cmp_mix"]
    pub mix: FloatParam,
}

impl Default for CompGroupParams {
    fn default() -> Self {
        Self {
            enabled: BoolParam::new("Comp On", false),
            threshold: FloatParam::new(
                "Comp Threshold",
                -20.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB")
            .with_step_size(0.1),

            ratio: FloatParam::new(
                "Comp Ratio",
                4.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 20.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_step_size(0.1),

            attack_ms: FloatParam::new(
                "Comp Attack",
                5.0,
                FloatRange::Skewed {
                    min: 0.1,
                    max: 100.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" ms")
            .with_step_size(0.1),

            release_ms: FloatParam::new(
                "Comp Release",
                100.0,
                FloatRange::Skewed {
                    min: 10.0,
                    max: 1000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" ms")
            .with_step_size(0.1),

            makeup_db: FloatParam::new(
                "Comp Makeup",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 24.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB")
            .with_step_size(0.1),

            mix: FloatParam::new("Comp Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0)),
        }
    }
}

