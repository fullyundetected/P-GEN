#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use eframe::icon_data;
use egui::{Button, Color32, FontId, Label, Margin, Pos2, Stroke, TextEdit, Vec2};
use generator::GeneratePasswordParams;

mod gui;
mod crypto;
mod entropy;
mod ui_theme;
mod generator;

pub struct PigeonInstance {
    should_init: bool,
    last_used_entropy: Mutex<Vec<u8>>,
    mouse_position_entropy: Arc<Mutex<Vec<u8>>>,
    program_start_time: u128,
    generated_streams_count: u32,
    password_size_str: String,
    password_size_str_last: String,
    password_params: GeneratePasswordParams,
    password_textedit: String,
}

impl Default for PigeonInstance {
    fn default() -> Self {
        let mut default_mouse_position_entropy = Vec::new();
        default_mouse_position_entropy.append(&mut crypto::hashes::whirlpool_512_compute(b"P-GEN DEFAULT MOUSE POSITION ENTROPY"));
        default_mouse_position_entropy.append(&mut vec![0; 192]);
        
        Self {
            should_init: true,
            last_used_entropy: Mutex::new(Vec::new()),
            mouse_position_entropy: Arc::from(Mutex::new(default_mouse_position_entropy)),
            program_start_time: entropy::get_current_time_ns(),
            generated_streams_count: 0,
            password_size_str: "20".to_string(),
            password_size_str_last: "20".to_string(),
            password_params: GeneratePasswordParams {
                size: 20,
                use_uppercase_chars: true,
                use_lowercase_chars: true,
                use_numbers: true,
                use_logograms: false,
                use_punctuation: false,
                use_quotation_marks: false,
                use_dashes_and_slashes: false,
                use_maths_symbols: false,
                use_brackets: false
            },
            password_textedit: "Before generating a password, you should move your mouse around randomly to make your password more secure".to_string()
        }
    }
}

impl PigeonInstance {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let instance = PigeonInstance::default();

        let mp_entropy = instance.mouse_position_entropy.clone();
        std::thread::spawn(move || {
            entropy::mouse_position_entropy_updater_thread(mp_entropy);
        });

        return instance;
    }
}

impl eframe::App for PigeonInstance {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.should_init {
            egui_extras::install_image_loaders(ctx);
            ui_theme::load_theme(ctx);
        }

        let main_panel_frame = egui::containers::Frame {
            fill: Color32::from_rgb(47, 47, 48),
            ..Default::default()
        };
        
        egui::TopBottomPanel::top("main_panel").exact_height(510.0)
        .show_separator_line(false)
        .resizable(false)
        .frame(main_panel_frame).show(ctx, |ui| {
            let visuals = ui.visuals_mut();
            visuals.selection.stroke = Stroke::NONE;
            visuals.extreme_bg_color = Color32::from_rgb(35, 35, 38);
            ui.spacing_mut().slider_width = 560.0;
            ui.allocate_ui_at_rect(egui::Rect::from_two_pos(Pos2::new(20.0, 20.0), Pos2::new(800.0, 200.0)), |ui| {
                let text_edit_resp = ui.add(TextEdit::singleline(&mut self.password_textedit).desired_width(654.0).margin(Margin::symmetric(6.0, 4.0)));
                gui::draw_rect_stroke(ui.painter(), text_edit_resp.rect.shrink2(Vec2::new(1.0, 0.0)), Color32::from_rgb(35, 35, 38));
                gui::draw_rect_stroke(ui.painter(), text_edit_resp.rect, Color32::from_rgb(54, 98, 54));
            });
            ui.allocate_ui_at_rect(egui::Rect::from_two_pos(Pos2::new(693.0, 20.0), Pos2::new(800.0, 200.0)), |ui| {
                let regenerate_password_button = ui.add(Button::new("  🔄").rounding(0.0).min_size(Vec2::new(28.0, 25.0)));
                gui::draw_rect_stroke(ui.painter(), regenerate_password_button.rect, Color32::from_rgb(54, 98, 54));
                if regenerate_password_button.clicked() { self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1; }
            });

            ui.allocate_ui_at_rect(egui::Rect::from_two_pos(Pos2::new(20.0, 60.0), Pos2::new(744.0, 140.0)), |ui| {
                ui.add(Label::new("Password Length").selectable(false));
            });

            ui.allocate_ui_at_rect(egui::Rect::from_two_pos(Pos2::new(120.0, 60.0), Pos2::new(744.0, 140.0)), |ui| {
                let previous_fill = ui.style().visuals.widgets.inactive.bg_fill;
                ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 35, 38); 
                ui.add(egui::Slider::new(&mut self.password_params.size, 0..=100).show_value(false).handle_shape(egui::style::HandleShape::Rect { aspect_ratio: 1.0 }));
                ui.style_mut().visuals.widgets.inactive.bg_fill = previous_fill;
                ui.allocate_ui_at_rect(egui::Rect::from_two_pos(Pos2::new(685.0, 60.0), Pos2::new(744.0, 140.0)), |ui| {
                    ui.add(TextEdit::singleline(&mut self.password_size_str).char_limit(3).font(FontId::proportional(10.0)).desired_width(30.0));
                });
            });

            let mut textedit_updated = self.password_size_str != self.password_size_str_last;
            let mut reverted = false;
            if textedit_updated {
                if self.password_size_str.len() == 0 {
                    self.password_size_str = self.password_size_str_last.clone();
                    textedit_updated = false;
                } else {
                    for c in self.password_size_str.chars() {
                        if !"0123456789".contains(c) {
                            self.password_size_str = self.password_size_str_last.clone();
                            reverted = true;
                            break;
                        }
                    }
                }
            }
            self.password_size_str_last = self.password_size_str.clone();
            if textedit_updated {
                self.password_params.size = self.password_size_str.parse::<u32>().unwrap();
                if !reverted {
                    self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1;
                }
            } else {
                if self.password_params.size != self.password_size_str.parse::<u32>().unwrap() {
                    self.password_size_str = self.password_params.size.to_string();
                    self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1;
                }
            }

            ui.allocate_ui_at_rect(egui::Rect::from_two_pos(Pos2::new(20.0, 90.0), Pos2::new(744.0, 140.0)), |ui| {
                ui.horizontal(|ui| {
                    let uppercase_chars_before = self.password_params.use_uppercase_chars.clone();
                    let lowercase_chars_before = self.password_params.use_lowercase_chars.clone();
                    let numbers_before = self.password_params.use_numbers.clone();
                    let logograms_before = self.password_params.use_logograms.clone();
                    let punctuation_before = self.password_params.use_punctuation.clone();
                    let quotation_marks_before = self.password_params.use_quotation_marks.clone();
                    let dashes_and_slashes_before = self.password_params.use_dashes_and_slashes.clone();
                    let maths_symbols_before = self.password_params.use_maths_symbols.clone();
                    let brackets_before = self.password_params.use_brackets.clone();
    
                    if uppercase_chars_before {
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(33, 63, 33);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                    }
                    let uppercase_characters_button = ui.add(Button::new("A-Z").rounding(0.0).min_size(Vec2::new(60.0, 30.0)));
                    if uppercase_chars_before {
                        gui::draw_rect_stroke(ui.painter(), uppercase_characters_button.rect, Color32::from_rgb(54, 98, 54));
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(35, 35, 38);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(42, 42, 46);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(50, 50, 55);
                    } else {
                        gui::draw_rect_stroke(ui.painter(), uppercase_characters_button.rect, Color32::from_rgb(50, 50, 55));
                    }
                    if lowercase_chars_before {
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(33, 63, 33);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                    }
                    let lowercase_characters_button = ui.add(Button::new("a-z").rounding(0.0).min_size(Vec2::new(60.0, 30.0)));
                    if lowercase_chars_before {
                        gui::draw_rect_stroke(ui.painter(), lowercase_characters_button.rect, Color32::from_rgb(54, 98, 54));
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(35, 35, 38);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(42, 42, 46);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(50, 50, 55);
                    } else {
                        gui::draw_rect_stroke(ui.painter(), lowercase_characters_button.rect, Color32::from_rgb(50, 50, 55));
                    }
                    if numbers_before {
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(33, 63, 33);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                    }
                    let numbers_button = ui.add(Button::new("0-9").rounding(0.0).min_size(Vec2::new(60.0, 30.0)));
                    if numbers_before {
                        gui::draw_rect_stroke(ui.painter(), numbers_button.rect, Color32::from_rgb(54, 98, 54));
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(35, 35, 38);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(42, 42, 46);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(50, 50, 55);
                    } else {
                        gui::draw_rect_stroke(ui.painter(), numbers_button.rect, Color32::from_rgb(50, 50, 55));
                    }
                    if logograms_before {
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(33, 63, 33);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                    }
                    let logograms_button = ui.add(Button::new("# $ % & @ ^ ` ~").rounding(0.0).min_size(Vec2::new(120.0, 30.0)));
                    if logograms_before {
                        gui::draw_rect_stroke(ui.painter(), logograms_button.rect, Color32::from_rgb(54, 98, 54));
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(35, 35, 38);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(42, 42, 46);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(50, 50, 55);
                    } else {
                        gui::draw_rect_stroke(ui.painter(), logograms_button.rect, Color32::from_rgb(50, 50, 55));
                    }
                    if punctuation_before {
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(33, 63, 33);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                    }
                    let punctuation_button = ui.add(Button::new(". , : ;").rounding(0.0).min_size(Vec2::new(60.0, 30.0)));
                    if punctuation_before {
                        gui::draw_rect_stroke(ui.painter(), punctuation_button.rect, Color32::from_rgb(54, 98, 54));
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(35, 35, 38);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(42, 42, 46);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(50, 50, 55);
                    } else {
                        gui::draw_rect_stroke(ui.painter(), punctuation_button.rect, Color32::from_rgb(50, 50, 55));
                    }
                    if quotation_marks_before {
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(33, 63, 33);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                    }
                    let quotation_marks_button = ui.add(Button::new("\" '").rounding(0.0).min_size(Vec2::new(60.0, 30.0)));
                    if quotation_marks_before {
                        gui::draw_rect_stroke(ui.painter(), quotation_marks_button.rect, Color32::from_rgb(54, 98, 54));
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(35, 35, 38);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(42, 42, 46);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(50, 50, 55);
                    } else {
                        gui::draw_rect_stroke(ui.painter(), quotation_marks_button.rect, Color32::from_rgb(50, 50, 55));
                    }
                    if dashes_and_slashes_before {
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(33, 63, 33);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                    }
                    let dashes_and_slashes_button = ui.add(Button::new("\\ / | _ -").rounding(0.0).min_size(Vec2::new(60.0, 30.0)));
                    if dashes_and_slashes_before {
                        gui::draw_rect_stroke(ui.painter(), dashes_and_slashes_button.rect, Color32::from_rgb(54, 98, 54));
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(35, 35, 38);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(42, 42, 46);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(50, 50, 55);
                    } else {
                        gui::draw_rect_stroke(ui.painter(), dashes_and_slashes_button.rect, Color32::from_rgb(50, 50, 55));
                    }
                    if maths_symbols_before {
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(33, 63, 33);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                    }
                    let maths_symbols_button = ui.add(Button::new("< > * + ! ? =").rounding(0.0).min_size(Vec2::new(90.0, 30.0)));
                    if maths_symbols_before {
                        gui::draw_rect_stroke(ui.painter(), maths_symbols_button.rect, Color32::from_rgb(54, 98, 54));
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(35, 35, 38);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(42, 42, 46);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(50, 50, 55);
                    } else {
                        gui::draw_rect_stroke(ui.painter(), maths_symbols_button.rect, Color32::from_rgb(50, 50, 55));
                    }
                    if brackets_before {
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(33, 63, 33);
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(41, 79, 41);
                    }
                    let use_brackets_button = ui.add(Button::new("( ) [ ] { }").rounding(0.0).min_size(Vec2::new(69.0, 30.0)));
                    if brackets_before {
                        gui::draw_rect_stroke(ui.painter(), use_brackets_button.rect, Color32::from_rgb(54, 98, 54));
                    } else {
                        gui::draw_rect_stroke(ui.painter(), use_brackets_button.rect, Color32::from_rgb(50, 50, 55));
                    }
                    if uppercase_characters_button.clicked() {self.password_params.use_uppercase_chars = !self.password_params.use_uppercase_chars; self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1; self.generated_streams_count += 1;};
                    if lowercase_characters_button.clicked() {self.password_params.use_lowercase_chars = !self.password_params.use_lowercase_chars; self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1};
                    if numbers_button.clicked() {self.password_params.use_numbers = !self.password_params.use_numbers; self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1};
                    if logograms_button.clicked() {self.password_params.use_logograms = !self.password_params.use_logograms; self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1};
                    if punctuation_button.clicked() {self.password_params.use_punctuation = !self.password_params.use_punctuation; self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1};
                    if quotation_marks_button.clicked() {self.password_params.use_quotation_marks = !self.password_params.use_quotation_marks; self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1};
                    if dashes_and_slashes_button.clicked() {self.password_params.use_dashes_and_slashes = !self.password_params.use_dashes_and_slashes; self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1};
                    if maths_symbols_button.clicked() {self.password_params.use_maths_symbols = !self.password_params.use_maths_symbols; self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1};
                    if use_brackets_button.clicked() {self.password_params.use_brackets = !self.password_params.use_brackets; self.password_textedit = generator::generate_password(self, &self.password_params); self.generated_streams_count += 1};
                });
            });
        });
    }
}

fn main() {
    let native_options: eframe::NativeOptions = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([744.0, 140.0])
            .with_max_inner_size([744.0, 140.0])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_minimize_button(false)
            .with_icon(Arc::from(icon_data::from_png_bytes(include_bytes!("../P-GEN.png")).unwrap())),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "P-GEN",
        native_options,
        Box::new(|cc| Box::new(PigeonInstance::new(cc))),
    );
}
