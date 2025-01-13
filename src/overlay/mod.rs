#![windows_subsystem = "windows"] // to turn off console.

use core::time;
use num_format::{Locale, ToFormattedString};
use std::{fs::File, io::BufReader, path::PathBuf, thread};

use egui::{Align2, Color32, Frame, Label, Margin, RichText, Shadow, TextStyle};
use egui_overlay::EguiOverlay;
#[cfg(feature = "wgpu")]
use egui_render_wgpu::WgpuBackend as DefaultGfxBackend;
use serde::{Deserialize, Serialize};

use crate::{
    models::SessionStats,
    monitoring::{self, monitor_structs::MonitorMessage},
};

#[cfg(not(any(feature = "three_d", feature = "wgpu")))]
compile_error!("you must enable either `three_d` or `wgpu` feature to run this example");
pub(crate) fn start(working_dir: PathBuf) {
    use tracing_subscriber::{fmt, prelude::*};
    // if RUST_LOG is not set, we will use the following filters
    tracing_subscriber::registry().with(fmt::layer()).init();

    let config = load_config(&working_dir);
    egui_overlay::start(Overlay {
        width_i32: config.overlay_size[0] as i32,
        width_f32: config.overlay_size[0] as f32,
        height_i32: config.overlay_size[0] as i32,
        height_f32: config.overlay_size[1] as f32,
        cell_width: (config.overlay_size[0] / 3) as f32,
        cell_height: (config.overlay_size[1] / 3) as f32,
        statistics_enabled: config.statistics_display.enabled,
        statistics_font_size: config.statistics_display.font_size,
        statistics_text_color: get_color(&config.statistics_display.text_color),
        statistics_grid_position: config.statistics_display.grid_position,
        statistics_vertical_offset: config.statistics_display.vertical_offset,
        display_enabled: config.message_display.enabled,
        display_grid_position: config.message_display.grid_position,
        display_vertical_offset: config.message_display.vertical_offset,
    });
}

pub struct Overlay {
    width_i32: i32,
    width_f32: f32,
    height_i32: i32,
    height_f32: f32,
    cell_width: f32,
    cell_height: f32,
    statistics_enabled: bool,
    statistics_font_size: u8,
    statistics_text_color: Color32,
    statistics_grid_position: u8,
    statistics_vertical_offset: f32,
    display_enabled: bool,
    display_grid_position: u8,
    display_vertical_offset: f32,
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
            .anchor(Align2::CENTER_TOP, [0.0, 0.0])
            .frame(Frame {
                inner_margin: Margin::ZERO,
                outer_margin: Margin::ZERO,
                rounding: egui::Rounding::ZERO,
                shadow: Shadow::NONE,
                fill: Color32::TRANSPARENT,
                stroke: egui::Stroke::NONE,
            })
            .show(egui_context, |ui| {
                egui::Grid::new("overlay_grid")
                    .min_col_width(self.cell_width)
                    .min_row_height(self.cell_height)
                    .show(ui, |ui| {
                        for i in 1..=9 {
                            if self.statistics_enabled && i == self.statistics_grid_position {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(self.statistics_vertical_offset);
                                    for w in generated_stats_widgets(&self, &stats) {
                                        ui.add(w);
                                    }
                                });
                            } else if self.display_enabled && i == self.display_grid_position {
                                ui.vertical_centered_justified(|ui| {
                                    ui.add_space(self.display_vertical_offset);
                                    for w in generate_message_widgets(&messages) {
                                        ui.add(w);
                                    }
                                });
                            } else {
                                ui.label("");
                            }

                            if i % 3 == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });

        egui_context.request_repaint();
    }
}

fn generated_stats_widgets(config: &Overlay, stats: &SessionStats) -> Vec<Label> {
    let mut stats_widgets = Vec::<Label>::new();
    stats_widgets.push(Label::new(
        RichText::new(format!(
            "DPS total: {} 5m: {}",
            stats.total_dps.to_formatted_string(&Locale::en),
            stats.dps_5.to_formatted_string(&Locale::en)
        ))
        .text_style(TextStyle::Monospace)
        .color(config.statistics_text_color)
        .size(config.statistics_font_size.into())
        .strong()
        .background_color(Color32::TRANSPARENT),
    ));
    stats_widgets.push(egui::Label::new(
        RichText::new(format!(
            "EXP total: {} 5m: {}",
            stats.total_exp.to_formatted_string(&Locale::en),
            stats.exp_5.to_formatted_string(&Locale::en)
        ))
        .text_style(TextStyle::Monospace)
        .color(config.statistics_text_color)
        .size(config.statistics_font_size.into())
        .background_color(Color32::TRANSPARENT),
    ));
    stats_widgets.push(egui::Label::new(
        RichText::new(format!(
            "Inf total: {} 5m: {}",
            stats.total_inf.to_formatted_string(&Locale::en),
            stats.inf_5.to_formatted_string(&Locale::en)
        ))
        .text_style(TextStyle::Monospace)
        .color(config.statistics_text_color)
        .size(config.statistics_font_size.into())
        .background_color(Color32::TRANSPARENT),
    ));

    stats_widgets
}

fn generate_message_widgets(messages: &Vec<MonitorMessage>) -> Vec<Label> {
    let mut message_widgets = Vec::<Label>::new();

    for m in messages {
        let color = get_color(&m.color);
        message_widgets.push(Label::new(
            RichText::new(m.output_text.clone())
                .text_style(TextStyle::Monospace)
                .color(color)
                .size(m.font_size as f32)
                .strong()
                .background_color(Color32::TRANSPARENT),
        ));
    }

    message_widgets
}

fn get_color(color_str: &String) -> Color32 {
    match color_str.as_str() {
        "green" => Color32::GREEN,
        "yellow" => Color32::YELLOW,
        "red" => Color32::RED,
        "orange" => Color32::ORANGE,
        "blue" => Color32::BLUE,
        "indigo" => Color32::from_rgb(75, 0, 130),
        "violet" => Color32::from_rgb(148, 0, 211),
        _ => Color32::GRAY,
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayConfig {
    #[serde(rename = "overlay_size")]
    pub overlay_size: Vec<i64>,
    #[serde(rename = "statistics_display")]
    pub statistics_display: StatisticsDisplay,
    #[serde(rename = "message_display")]
    pub message_display: MessageDisplay,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsDisplay {
    pub enabled: bool,
    #[serde(rename = "grid_position")]
    pub grid_position: u8,
    #[serde(rename = "vertical_offset")]
    pub vertical_offset: f32,
    #[serde(rename = "font_size")]
    pub font_size: u8,
    #[serde(rename = "text_color")]
    pub text_color: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageDisplay {
    pub enabled: bool,
    #[serde(rename = "grid_position")]
    pub grid_position: u8,
    #[serde(rename = "vertical_offset")]
    pub vertical_offset: f32,
}

fn _get_alignment(align_str: &str) -> Align2 {
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
