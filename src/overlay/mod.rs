#![windows_subsystem = "windows"] // to turn off console.

use core::time;
use std::{fs::File, io::BufReader, path::PathBuf, thread};
use num_format::{Locale, ToFormattedString};

use egui::{Align2, Color32, DragValue, Frame, Margin, Shadow, Vec2, Widget};
use egui_overlay::EguiOverlay;
#[cfg(feature = "three_d")]
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
#[cfg(feature = "wgpu")]
use egui_render_wgpu::WgpuBackend as DefaultGfxBackend;
use serde::{Deserialize, Serialize};
use tracing_subscriber::filter::DynFilterFn;

use crate::monitoring::{self, monitor_structs::MonitorMessage};

#[cfg(not(any(feature = "three_d", feature = "wgpu")))]
compile_error!("you must enable either `three_d` or `wgpu` feature to run this example");
pub(crate) fn start(working_dir: PathBuf) {
    // use tracing_subscriber::{fmt, prelude::*};
    // // if RUST_LOG is not set, we will use the following filters
    // tracing_subscriber::registry()
    //     .with(fmt::layer())
    //     .with(
    //         DynFilterFn::try_from_default_env()
    //             .unwrap_or(EnvFilter::new("debug,wgpu=warn,naga=warn")),
    //     )
    //     .init();
    let config = load_config(&working_dir);
    egui_overlay::start(Overlay {
        width_i32: config.overlay_size[0] as i32,
        width_f32: config.overlay_size[0] as f32,
        height_i32: config.overlay_size[0] as i32,
        height_f32: config.overlay_size[1] as f32,
        alignment: get_alignment(&config.alignment.as_str()),
        offset: [config.offset[0] as f32, config.offset[1] as f32],
    });
}

pub struct Overlay {
    width_i32: i32,
    width_f32: f32,
    height_i32: i32,
    height_f32: f32,
    alignment: Align2,
    offset: [f32; 2],
}
impl EguiOverlay for Overlay {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut egui_overlay::egui_render_three_d::ThreeDBackend,
        glfw_backend: &mut egui_overlay::egui_window_glfw_passthrough::GlfwBackend,
    ) {
        glfw_backend.window.set_pos(0, 0);
        glfw_backend
            .window
            .set_size(self.width_i32, self.height_i32);
        glfw_backend.window.set_mouse_passthrough(true);
        glfw_backend.window.set_title("");
        glfw_backend.window.set_decorated(false);

        thread::sleep(time::Duration::from_millis(1000));
        let (_, stats, messages) = monitoring::get_messages();

        egui::Window::new("")
            .title_bar(false)
            .resizable(false)
            .default_width(self.width_f32)
            .default_height(self.height_f32)
            .anchor(self.alignment, self.offset)
            .frame(Frame {
                inner_margin: Margin::ZERO,
                outer_margin: Margin::ZERO,
                rounding: egui::Rounding::ZERO,
                shadow: Shadow::NONE,
                fill: Color32::TRANSPARENT,
                stroke: egui::Stroke::NONE,
            })
            .show(egui_context, |ui| {
                egui::Grid::new("some_unique_id")
                    .min_col_width(640.0)
                    .min_row_height(360.0)
                    .show(ui, |ui| {
                        ui.vertical_centered_justified(|ui| {
                            ui.add_space(50.0);
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!(
                                    "DPS total: {} 5m: {}",
                                    stats.total_dps.to_formatted_string(&Locale::en), stats.dps_5.to_formatted_string(&Locale::en)
                                ))
                                .color(Color32::WHITE)
                                .size(24.0)
                                .strong()
                                .background_color(Color32::TRANSPARENT),
                            ));
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!(
                                    "EXP total: {} 5m: {}",
                                    stats.total_exp.to_formatted_string(&Locale::en), stats.exp_5.to_formatted_string(&Locale::en)
                                ))
                                .color(Color32::WHITE)
                                .size(24.0)
                                .background_color(Color32::TRANSPARENT),
                            ));
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!(
                                    "Inf total: {} 5m: {}",
                                    stats.total_inf.to_formatted_string(&Locale::en), stats.inf_5.to_formatted_string(&Locale::en)
                                ))
                                .color(Color32::WHITE)
                                .size(24.0)
                                .background_color(Color32::TRANSPARENT),
                            ));
                        });
                        ui.vertical_centered(|ui| {
                            ui.add_space(250.0);
                            for m in messages {
                                let color = get_color(&m);
                                ui.add(egui::Label::new(
                                    egui::RichText::new(m.output_text)
                                        .color(color)
                                        .size(m.font_size as f32)
                                        .strong()
                                        .background_color(Color32::TRANSPARENT),
                                ));
                            }
                        });
                        ui.label("");
                        ui.end_row();

                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.end_row();

                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                    });
            });

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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayConfig {
    #[serde(rename = "overlay_size")]
    pub overlay_size: Vec<u64>,
    pub alignment: String,
    #[serde(rename = "offset:")]
    pub offset: Vec<i32>,
}

fn get_alignment(align_str: &str) -> Align2 {
    match align_str {
        "LEFT_BOTTOM" => Align2::LEFT_BOTTOM,
        "LEFT_CENTER" => Align2::LEFT_CENTER,
        "LEFT_TOP" => Align2::LEFT_TOP,
        "CENTER_BOTTOM" => Align2::CENTER_BOTTOM,
        "CENTER_CENTER" => Align2::CENTER_CENTER,
        "CENTER_TOP" => Align2::CENTER_TOP,
        "RIGHT_BOTTOM" => Align2::RIGHT_BOTTOM,
        "RIGHT_CENTER" => Align2::RIGHT_CENTER,
        "RIGHT_TOP" => Align2::RIGHT_TOP,
        _ => panic!("Unknown alignment in overlay config"),
    }
}

fn load_config(working_dir: &PathBuf) -> OverlayConfig {
    let path = PathBuf::new()
        .join(working_dir)
        .join("configs")
        .join("overlay.config.json");
    if path.exists() {
        println!("Display configuration path: {:?}", path);
        let f = File::open(path).expect("Unable to find overlay file");
        let reader = BufReader::new(f);
        let overlay_config: OverlayConfig =
            serde_json::from_reader(reader).expect("Unable to read config file");
        println!(
            "Overlay config data: {}",
            serde_json::to_string_pretty(&overlay_config).expect("Unable to serialize config")
        );
        overlay_config
    } else {
        panic!("Overlay configuration file is not readable: {:?}", path);
    }
}
