#![windows_subsystem = "windows"] // to turn off console.

use core::time;
use std::thread;

use egui::{Color32, DragValue, Frame, Margin, Shadow, Widget};
use egui_overlay::EguiOverlay;
#[cfg(feature = "three_d")]
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
#[cfg(feature = "wgpu")]
use egui_render_wgpu::WgpuBackend as DefaultGfxBackend;
use tracing_subscriber::filter::DynFilterFn;

use crate::monitoring;

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
        let (date_time, stats, messages) = monitoring::get_messages();

        egui::Window::new("")
            .title_bar(false)
            .interactable(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, [-20.0, 40.0])
            .frame(Frame {
                inner_margin: Margin::ZERO,
                outer_margin: Margin::ZERO,
                rounding: egui::Rounding::ZERO,
                shadow: Shadow::NONE,
                fill: Color32::TRANSPARENT,
                stroke: egui::Stroke::NONE,
            })
            .show(egui_context, |ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!( "DPS: total: {} last 5 mins: {}", stats.total_dps, stats.dps_5))
                        .color(Color32::GREEN)
                        .size(24.0)
                        .background_color(Color32::TRANSPARENT),
                    )
                    .extend(),
                );
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!( "EXP: {} total {} last 5 mins", stats.total_exp, stats.exp_5))
                        .color(Color32::GREEN)
                        .size(24.0)
                        .background_color(Color32::TRANSPARENT),
                    )
                    .extend(),
                );
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(format!( "Inf: {} total {} last 5 mins", stats.total_inf, stats.inf_5))
                        .color(Color32::GREEN)
                        .size(24.0)
                        .background_color(Color32::TRANSPARENT),
                    )
                    .extend(),
                );
            });

        egui::Window::new("")
            .title_bar(false)
            .interactable(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(Frame {
                inner_margin: Margin::ZERO,
                outer_margin: Margin::ZERO,
                rounding: egui::Rounding::ZERO,
                shadow: Shadow::NONE,
                fill: Color32::TRANSPARENT,
                stroke: egui::Stroke::NONE,
            })
            .show(egui_context, |ui| {
                for m in messages {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(m.output_text)
                                .color(Color32::GREEN)
                                .size(36.0)
                                .background_color(Color32::TRANSPARENT),
                        )
                        .extend(),
                    );
                }
            });
        // just some controls to show how you can use glfw_backend
        // egui::Window::new("controls").show(egui_context, |ui| {
        //     ui.set_width(300.0);
        //     self.frame += 1;
        //     ui.label(format!("current frame number: {}", self.frame));
        //     // sometimes, you want to see the borders to understand where the overlay is.
        //     let mut borders = glfw_backend.window.is_decorated();
        //     if ui.checkbox(&mut borders, "window borders").changed() {
        //         glfw_backend.window.set_decorated(borders);
        //     }

        //     ui.label(format!(
        //         "pixels_per_virtual_unit: {}",
        //         glfw_backend.physical_pixels_per_virtual_unit
        //     ));
        //     ui.label(format!("window scale: {}", glfw_backend.scale));
        //     ui.label(format!("cursor pos x: {}", glfw_backend.cursor_pos[0]));
        //     ui.label(format!("cursor pos y: {}", glfw_backend.cursor_pos[1]));

        //     ui.label(format!(
        //         "passthrough: {}",
        //         glfw_backend.window.is_mouse_passthrough()
        //     ));
        //     // how to change size.
        //     // WARNING: don't use drag value, because window size changing while dragging ui messes things up.
        //     let mut size = glfw_backend.window_size_logical;
        //     let mut changed = false;
        //     ui.horizontal(|ui| {
        //         ui.label("width: ");
        //         ui.add_enabled(false, DragValue::new(&mut size[0]));
        //         if ui.button("inc").clicked() {
        //             size[0] += 10.0;
        //             changed = true;
        //         }
        //         if ui.button("dec").clicked() {
        //             size[0] -= 10.0;
        //             changed = true;
        //         }
        //     });
        //     ui.horizontal(|ui| {
        //         ui.label("height: ");
        //         ui.add_enabled(false, DragValue::new(&mut size[1]));
        //         if ui.button("inc").clicked() {
        //             size[1] += 10.0;
        //             changed = true;
        //         }
        //         if ui.button("dec").clicked() {
        //             size[1] -= 10.0;
        //             changed = true;
        //         }
        //     });
        //     if changed {
        //         glfw_backend.set_window_size(size);
        //     }
        //     // how to change size.
        //     // WARNING: don't use drag value, because window size changing while dragging ui messes things up.
        //     let mut pos = glfw_backend.window_position;
        //     let mut changed = false;
        //     ui.horizontal(|ui| {
        //         ui.label("x: ");
        //         ui.add_enabled(false, DragValue::new(&mut pos[0]));
        //         if ui.button("inc").clicked() {
        //             pos[0] += 10;
        //             changed = true;
        //         }
        //         if ui.button("dec").clicked() {
        //             pos[0] -= 10;
        //             changed = true;
        //         }
        //     });
        //     ui.horizontal(|ui| {
        //         ui.label("y: ");
        //         ui.add_enabled(false, DragValue::new(&mut pos[1]));
        //         if ui.button("inc").clicked() {
        //             pos[1] += 10;
        //             changed = true;
        //         }
        //         if ui.button("dec").clicked() {
        //             pos[1] -= 10;
        //             changed = true;
        //         }
        //     });
        //     if changed {
        //         glfw_backend.window.set_pos(pos[0], pos[1]);
        //     }
        // });

        // // here you decide if you want to be passthrough or not.
        // if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
        //     // we need input, so we need the window to be NOT passthrough
        //     glfw_backend.set_passthrough(false);
        // } else {
        //     // we don't care about input, so the window can be passthrough now
        //     glfw_backend.set_passthrough(true)
        // }
        egui_context.request_repaint();
    }
}
