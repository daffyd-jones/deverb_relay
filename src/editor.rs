// src/editor.rs

use crate::params::DelayReverbParams;
use nih_plug::prelude::*;
use nih_plug_egui::egui::{self, Align, Color32, Layout, RichText, Rounding, Stroke, Vec2};
use nih_plug_egui::{create_egui_editor, EguiState};
use std::sync::Arc;

const BG: Color32 = Color32::from_rgb(25, 25, 30);
const PANEL_BG: Color32 = Color32::from_rgb(35, 35, 42);
const HEADER_BG: Color32 = Color32::from_rgb(45, 45, 55);
const ACCENT_DELAY: Color32 = Color32::from_rgb(80, 160, 220);
const ACCENT_REVERB: Color32 = Color32::from_rgb(180, 100, 220);
const ACCENT_GLOBAL: Color32 = Color32::from_rgb(220, 180, 80);
const TEXT_DIM: Color32 = Color32::from_rgb(160, 160, 170);
const TEXT_BRIGHT: Color32 = Color32::from_rgb(230, 230, 240);
const KNOB_TRACK: Color32 = Color32::from_rgb(55, 55, 65);
const ENABLED_GREEN: Color32 = Color32::from_rgb(80, 200, 120);
const DISABLED_RED: Color32 = Color32::from_rgb(200, 80, 80);

pub fn create(
    params: Arc<DelayReverbParams>,
    editor_state: Arc<EguiState>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        editor_state,
        (),
        |_, _| {},
        move |egui_ctx, setter, _state| {
            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(BG).inner_margin(8.0))
                .show(egui_ctx, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);

                    // ── Header ──
                    draw_header(ui, &params, setter);

                    ui.add_space(4.0);

                    // ── Main columns: Delay | Reverb ──
                    ui.columns(2, |cols| {
                        // Left column: Delay
                        draw_delay_column(&mut cols[0], &params, setter);

                        // Right column: Reverb
                        draw_reverb_column(&mut cols[1], &params, setter);
                    });
                });
        },
    )
}

fn draw_header(ui: &mut egui::Ui, params: &DelayReverbParams, setter: &ParamSetter) {
    egui::Frame::none()
        .fill(HEADER_BG)
        .rounding(Rounding::same(6))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("DELAY ◆ REVERB FX")
                        .color(ACCENT_GLOBAL)
                        .size(16.0)
                        .strong(),
                );
                ui.add_space(20.0);

                knob(
                    ui,
                    setter,
                    &params.main_mix,
                    "Main Mix",
                    ACCENT_GLOBAL,
                    40.0,
                );
                knob(
                    ui,
                    setter,
                    &params.output_gain,
                    "Output",
                    ACCENT_GLOBAL,
                    40.0,
                );

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.label(RichText::new("Routing").color(TEXT_DIM).size(10.0));
                    enum_selector(ui, setter, &params.routing_mode);
                });
            });
        });
}

fn draw_delay_column(ui: &mut egui::Ui, params: &DelayReverbParams, setter: &ParamSetter) {
    // Primary delay section
    section_frame(ui, "DELAY", ACCENT_DELAY, |ui| {
        ui.horizontal(|ui| {
            toggle_button(ui, setter, &params.delay.enabled, "ON");
            knob(
                ui,
                setter,
                &params.delay.time_ms,
                "Time",
                ACCENT_DELAY,
                36.0,
            );
            knob(
                ui,
                setter,
                &params.delay.feedback,
                "Feedback",
                ACCENT_DELAY,
                36.0,
            );
            knob(ui, setter, &params.delay.mix, "Mix", ACCENT_DELAY, 36.0);
        });
        ui.horizontal(|ui| {
            knob(
                ui,
                setter,
                &params.delay.cross_feedback,
                "Cross FB",
                ACCENT_DELAY,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.delay.feedback_hp,
                "FB HP",
                ACCENT_DELAY,
                32.0,
            );
            ui.add_space(4.0);
            ui.vertical(|ui| {
                toggle_button(ui, setter, &params.delay.tempo_sync, "Sync");
                if params.delay.tempo_sync.value() {
                    enum_selector(ui, setter, &params.delay.sync_division);
                }
            });
        });
    });

    ui.add_space(2.0);

    // Delay sub-effects
    sub_section(ui, "DELAY → CRUSH", ACCENT_DELAY, |ui| {
        ui.horizontal(|ui| {
            toggle_button(ui, setter, &params.delay_crush.enabled, "ON");
            knob(
                ui,
                setter,
                &params.delay_crush.bits,
                "Bits",
                ACCENT_DELAY,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.delay_crush.downsample,
                "Downsmp",
                ACCENT_DELAY,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.delay_crush.mix,
                "Mix",
                ACCENT_DELAY,
                32.0,
            );
        });
    });

    sub_section(ui, "DELAY → COMP", ACCENT_DELAY, |ui| {
        ui.horizontal(|ui| {
            toggle_button(ui, setter, &params.delay_comp.enabled, "ON");
            knob(
                ui,
                setter,
                &params.delay_comp.threshold,
                "Thresh",
                ACCENT_DELAY,
                30.0,
            );
            knob(
                ui,
                setter,
                &params.delay_comp.ratio,
                "Ratio",
                ACCENT_DELAY,
                30.0,
            );
            knob(
                ui,
                setter,
                &params.delay_comp.attack_ms,
                "Attack",
                ACCENT_DELAY,
                30.0,
            );
        });
        ui.horizontal(|ui| {
            ui.add_space(36.0);
            knob(
                ui,
                setter,
                &params.delay_comp.release_ms,
                "Release",
                ACCENT_DELAY,
                30.0,
            );
            knob(
                ui,
                setter,
                &params.delay_comp.makeup_db,
                "Makeup",
                ACCENT_DELAY,
                30.0,
            );
            knob(
                ui,
                setter,
                &params.delay_comp.mix,
                "Mix",
                ACCENT_DELAY,
                30.0,
            );
        });
    });

    sub_section(ui, "DELAY → CHORUS", ACCENT_DELAY, |ui| {
        ui.horizontal(|ui| {
            toggle_button(ui, setter, &params.delay_chorus.enabled, "ON");
            knob(
                ui,
                setter,
                &params.delay_chorus.rate,
                "Rate",
                ACCENT_DELAY,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.delay_chorus.depth,
                "Depth",
                ACCENT_DELAY,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.delay_chorus.feedback,
                "FB",
                ACCENT_DELAY,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.delay_chorus.mix,
                "Mix",
                ACCENT_DELAY,
                32.0,
            );
        });
    });
}

fn draw_reverb_column(ui: &mut egui::Ui, params: &DelayReverbParams, setter: &ParamSetter) {
    section_frame(ui, "REVERB", ACCENT_REVERB, |ui| {
        ui.horizontal(|ui| {
            toggle_button(ui, setter, &params.reverb.enabled, "ON");
            knob(
                ui,
                setter,
                &params.reverb.room_size,
                "Room",
                ACCENT_REVERB,
                36.0,
            );
            knob(
                ui,
                setter,
                &params.reverb.damping,
                "Damp",
                ACCENT_REVERB,
                36.0,
            );
            knob(ui, setter, &params.reverb.mix, "Mix", ACCENT_REVERB, 36.0);
        });
        ui.horizontal(|ui| {
            knob(
                ui,
                setter,
                &params.reverb.width,
                "Width",
                ACCENT_REVERB,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.reverb.predelay_ms,
                "Pre-dly",
                ACCENT_REVERB,
                32.0,
            );
        });
    });

    ui.add_space(2.0);

    sub_section(ui, "REVERB → CHORUS", ACCENT_REVERB, |ui| {
        ui.horizontal(|ui| {
            toggle_button(ui, setter, &params.reverb_chorus.enabled, "ON");
            knob(
                ui,
                setter,
                &params.reverb_chorus.rate,
                "Rate",
                ACCENT_REVERB,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.reverb_chorus.depth,
                "Depth",
                ACCENT_REVERB,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.reverb_chorus.feedback,
                "FB",
                ACCENT_REVERB,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.reverb_chorus.mix,
                "Mix",
                ACCENT_REVERB,
                32.0,
            );
        });
    });

    sub_section(ui, "REVERB → CRUSH", ACCENT_REVERB, |ui| {
        ui.horizontal(|ui| {
            toggle_button(ui, setter, &params.reverb_crush.enabled, "ON");
            knob(
                ui,
                setter,
                &params.reverb_crush.bits,
                "Bits",
                ACCENT_REVERB,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.reverb_crush.downsample,
                "Downsmp",
                ACCENT_REVERB,
                32.0,
            );
            knob(
                ui,
                setter,
                &params.reverb_crush.mix,
                "Mix",
                ACCENT_REVERB,
                32.0,
            );
        });
    });

    sub_section(ui, "REVERB → COMP", ACCENT_REVERB, |ui| {
        ui.horizontal(|ui| {
            toggle_button(ui, setter, &params.reverb_comp.enabled, "ON");
            knob(
                ui,
                setter,
                &params.reverb_comp.threshold,
                "Thresh",
                ACCENT_REVERB,
                30.0,
            );
            knob(
                ui,
                setter,
                &params.reverb_comp.ratio,
                "Ratio",
                ACCENT_REVERB,
                30.0,
            );
            knob(
                ui,
                setter,
                &params.reverb_comp.attack_ms,
                "Attack",
                ACCENT_REVERB,
                30.0,
            );
        });
        ui.horizontal(|ui| {
            ui.add_space(36.0);
            knob(
                ui,
                setter,
                &params.reverb_comp.release_ms,
                "Release",
                ACCENT_REVERB,
                30.0,
            );
            knob(
                ui,
                setter,
                &params.reverb_comp.makeup_db,
                "Makeup",
                ACCENT_REVERB,
                30.0,
            );
            knob(
                ui,
                setter,
                &params.reverb_comp.mix,
                "Mix",
                ACCENT_REVERB,
                30.0,
            );
        });
    });
}

// ─── Widgets ────────────────────────────────────────────────────────

fn section_frame(
    ui: &mut egui::Ui,
    title: &str,
    accent: Color32,
    content: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::none()
        .fill(PANEL_BG)
        .rounding(Rounding::same(6))
        .stroke(Stroke::new(1.0, accent.linear_multiply(0.4)))
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.label(RichText::new(title).color(accent).size(12.0).strong());
            ui.add_space(2.0);
            content(ui);
        });
}

fn sub_section(
    ui: &mut egui::Ui,
    title: &str,
    accent: Color32,
    content: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::none()
        .fill(PANEL_BG.linear_multiply(0.8))
        .rounding(Rounding::same(4))
        .stroke(Stroke::new(0.5, accent.linear_multiply(0.2)))
        .inner_margin(4.0)
        .show(ui, |ui| {
            ui.label(
                RichText::new(title)
                    .color(accent.linear_multiply(0.7))
                    .size(10.0),
            );
            ui.add_space(1.0);
            content(ui);
        });
}

fn knob(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &FloatParam,
    label: &str,
    accent: Color32,
    size: f32,
) {
    ui.vertical(|ui| {
        ui.set_width(size + 16.0);
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.label(RichText::new(label).color(TEXT_DIM).size(9.0));

            let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::drag());
            let painter = ui.painter_at(rect);

            let normalized = param.unmodulated_normalized_value();
            let center = rect.center();
            let radius = size * 0.4;

            // Track arc
            let start_angle = std::f32::consts::PI * 0.75;
            let end_angle = std::f32::consts::PI * 2.25;

            // Background arc
            draw_arc(
                &painter,
                center,
                radius,
                start_angle,
                end_angle,
                KNOB_TRACK,
                2.5,
            );

            // Value arc
            let value_angle = start_angle + (end_angle - start_angle) * normalized;
            if normalized > 0.001 {
                draw_arc(
                    &painter,
                    center,
                    radius,
                    start_angle,
                    value_angle,
                    accent,
                    2.5,
                );
            }

            // Center dot
            painter.circle_filled(center, 3.0, accent.linear_multiply(0.6));

            // Pointer line
            let pointer_angle = value_angle - std::f32::consts::FRAC_PI_2;
            let pointer_end =
                center + Vec2::new(pointer_angle.cos(), pointer_angle.sin()) * (radius - 2.0);
            painter.line_segment([center, pointer_end], Stroke::new(1.5, TEXT_BRIGHT));

            // Drag interaction
            if response.dragged() {
                let delta = -response.drag_delta().y;
                let speed = if ui.input(|i| i.modifiers.shift) {
                    0.001
                } else {
                    0.005
                };
                let new_val = (normalized + delta * speed).clamp(0.0, 1.0);
                setter.set_parameter_normalized(param, new_val);
            }

            // Double-click reset
            if response.double_clicked() {
                setter.set_parameter_normalized(param, param.default_normalized_value());
            }

            // Tooltip
            response.on_hover_text(format!("{}: {}", param.name(), param.to_string()));

            // Value text
            let display = param.to_string();
            ui.label(RichText::new(display).color(TEXT_BRIGHT).size(9.0));
        });
    });
}

fn draw_arc(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    start: f32,
    end: f32,
    color: Color32,
    stroke_width: f32,
) {
    let segments = 32;
    let step = (end - start) / segments as f32;
    let points: Vec<egui::Pos2> = (0..=segments)
        .map(|i| {
            let angle = start + step * i as f32 - std::f32::consts::FRAC_PI_2;
            center + Vec2::new(angle.cos(), angle.sin()) * radius
        })
        .collect();

    for window in points.windows(2) {
        painter.line_segment([window[0], window[1]], Stroke::new(stroke_width, color));
    }
}

fn toggle_button(ui: &mut egui::Ui, setter: &ParamSetter, param: &BoolParam, label: &str) {
    let on = param.value();
    let color = if on { ENABLED_GREEN } else { DISABLED_RED };
    let text_color = if on { Color32::BLACK } else { TEXT_BRIGHT };

    let btn = egui::Button::new(RichText::new(label).color(text_color).size(10.0).strong())
        .fill(color.linear_multiply(if on { 0.8 } else { 0.3 }))
        .rounding(Rounding::same(3))
        .min_size(Vec2::new(28.0, 18.0));

    if ui.add(btn).clicked() {
        setter.set_parameter(param, !on);
    }
}

fn enum_selector<T: Enum + PartialEq + 'static>(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &EnumParam<T>,
) {
    let current = param.value();
    let current_idx = param.unmodulated_normalized_value();

    egui::ComboBox::from_id_salt(param.name())
        .selected_text(
            RichText::new(param.to_string())
                .size(10.0)
                .color(TEXT_BRIGHT),
        )
        .width(100.0)
        .show_ui(ui, |ui| {
            let variants = T::variants();
            for (i, name) in variants.iter().enumerate() {
                let norm = i as f32 / (variants.len() - 1).max(1) as f32;
                let selected = (norm - current_idx).abs() < 0.01;
                if ui
                    .selectable_label(selected, RichText::new(*name).size(10.0))
                    .clicked()
                {
                    setter.set_parameter_normalized(param, norm);
                }
            }
        });
}
