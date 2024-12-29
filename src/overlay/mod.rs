#![windows_subsystem = "windows"] // to turn off console.

use core::time;
use std::thread;

use egui::{Align2, Color32, DragValue, Frame, Margin, Shadow, Widget};
use egui_overlay::EguiOverlay;
#[cfg(feature = "three_d")]
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
#[cfg(feature = "wgpu")]
use egui_render_wgpu::WgpuBackend as DefaultGfxBackend;
use tracing_subscriber::filter::DynFilterFn;

use crate::monitoring::{self, monitor_structs::MonitorMessage};

#[cfg(not(any(feature = "three_d", feature = "wgpu")))]
compile_error!("you must enable either `three_d` or `wgpu` feature to run this example");
pub(crate) fn start() {
    // use tracing_subscriber::{fmt, prelude::*};
    // // if RUST_LOG is not set, we will use the following filters
    // tracing_subscriber::registry()
    //     .with(fmt::layer())
    //     .with(
    //         DynFilterFn::try_from_default_env()
    //             .unwrap_or(EnvFilter::new("debug,wgpu=warn,naga=warn")),
    //     )
    //     .init();

    egui_overlay::start(Overlay { frame: 0 });
}
pub struct Overlay {
    pub frame: u64,
}
impl EguiOverlay for Overlay {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut egui_overlay::egui_render_three_d::ThreeDBackend,
        glfw_backend: &mut egui_overlay::egui_window_glfw_passthrough::GlfwBackend,
    ) {
        glfw_backend.window.set_pos(0, 0);
        glfw_backend.window.set_size(1980, 1080);
        glfw_backend.window.set_mouse_passthrough(true);
        thread::sleep(time::Duration::from_millis(1000));
        let (_, stats, messages) = monitoring::get_messages();

        egui::Window::new("")
            .title_bar(false)
            .resizable(false)
            .min_height(360.0)
            .anchor(Align2::CENTER_TOP, [0.0, 100.0])
            .frame(Frame {
                inner_margin: Margin::ZERO,
                outer_margin: Margin::ZERO,
               rounding: egui::Rounding::ZERO,
                shadow: Shadow::NONE,
                fill: Color32::TRANSPARENT,
                stroke: egui::Stroke::NONE,
            })
            .show(egui_context, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(format!(
                                "DPS: total: {} 5m: {}",
                                stats.total_dps, stats.dps_5
                            ))
                            .color(Color32::WHITE)
                            .size(24.0)
                            .background_color(Color32::TRANSPARENT),
                        )
                        .extend(),
                    );
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(format!(
                                "EXP: total: {} 5m: {}",
                                stats.total_exp, stats.exp_5
                            ))
                            .color(Color32::WHITE)
                            .size(24.0)
                            .background_color(Color32::TRANSPARENT),
                        )
                        .extend(),
                    );
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(format!(
                                "Inf: total: {} 5m: {}",
                                stats.total_inf, stats.inf_5
                            ))
                            .color(Color32::WHITE)
                            .size(24.0)
                            .background_color(Color32::TRANSPARENT),
                        )
                        .extend(),
                    );
                    for m in messages {
                        let color = get_color(&m);
                        ui.vertical_centered_justified(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(m.output_text)
                                        .color(color)
                                        .size(m.font_size as f32)
                                        .background_color(Color32::TRANSPARENT),
                                )
                                .extend(),
                            );
                        });
                    }
                });
            });

        // egui::CentralPanel::default()
        //     .frame(Frame {
        //         inner_margin: Margin::ZERO,
        //         outer_margin: Margin::ZERO,
        //         rounding: egui::Rounding::ZERO,
        //         shadow: Shadow::NONE,
        //         fill: Color32::TRANSPARENT,
        //         stroke: egui::Stroke::NONE,
        //     })
        //     .show(egui_context, |ui| {
        //         for m in messages {
        //             let color = get_color(&m);
        //             ui.vertical_centered_justified(|ui| {
        //             ui.add(
        //                 egui::Label::new(
        //                     egui::RichText::new(m.output_text)
        //                         .color(color)
        //                         .size(m.font_size as f32)
        //                         .background_color(Color32::TRANSPARENT),
        //                 )
        //                 .extend(),
        //             );
        //         });
        //         }
        //     });

        egui_context.request_repaint();
    }
}

fn get_color(m: &MonitorMessage) -> Color32 {
    match m.color.as_str() {
        "green" => Color32::GREEN,
        "yellow" => Color32::YELLOW,
        "red" => Color32::RED,
        _ => Color32::GRAY,
    }
}
