//! Grafik Kullanıcı Arayüzü (GUI) - eframe / egui uygulaması.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use eframe::egui;

use crate::audio::{
    bass_sweep_frequency, build_stream, sanitize_level, sweep_frequency, ChannelMode,
    SignalGenerator, SignalType,
};
use crate::cli::select_device;
use crate::i18n::Language;

#[derive(Clone)]
pub struct UiDeviceInfo {
    pub index: usize,
    pub name: String,
    pub is_default: bool,
}

pub struct ActivePlayback {
    pub _stream: Stream,
    pub current_sample: Arc<AtomicU64>,
    pub sample_rate: f64,
    pub duration: f64,
    pub signal_type: SignalType,
    pub channel_mode: ChannelMode,
    pub finished: Arc<AtomicBool>,
    pub error_flag: Arc<AtomicBool>,
}

pub struct IemTestApp {
    pub host: cpal::Host,
    pub devices: Vec<UiDeviceInfo>,
    pub selected_device_idx: usize,
    pub channel_mode: ChannelMode,
    pub signal_type: SignalType,
    pub duration: f64,
    pub level: f32,
    pub active_playback: Option<ActivePlayback>,
    pub status_message: String,
    pub is_error: bool,
    pub language: Language,
}

impl IemTestApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Arial yazı fontunu yükle ve font boyutlarını %20 büyüt
        let mut fonts = egui::FontDefinitions::default();
        let arial_candidates = [
            "C:\\Windows\\Fonts\\arial.ttf",
            "/Library/Fonts/Arial.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/usr/share/fonts/truetype/msttcorefonts/arial.ttf",
            "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        ];

        for path in &arial_candidates {
            if let Ok(font_data) = std::fs::read(path) {
                fonts.font_data.insert(
                    "Arial".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "Arial".to_owned());
                break;
            }
        }
        cc.egui_ctx.set_fonts(fonts);

        let mut style = (*cc.egui_ctx.style()).clone();
        for (_, font_id) in style.text_styles.iter_mut() {
            font_id.size *= 1.20;
        }
        cc.egui_ctx.set_style(style);

        let host = cpal::default_host();
        let lang = Language::Turkish;
        let mut app = Self {
            host,
            devices: Vec::new(),
            selected_device_idx: 0,
            channel_mode: ChannelMode::Both,    // Varsayılan: Her İki Kanal (L+R)
            signal_type: SignalType::BassSweep, // Varsayılan: 20 Hz - 250 Hz Bas Sweep
            duration: 20.0,
            level: 0.3,                         // Varsayılan seviye: 0.3
            active_playback: None,
            status_message: lang.status_ready().to_string(),
            is_error: false,
            language: lang,
        };
        app.refresh_devices();
        app
    }

    pub fn refresh_devices(&mut self) {
        self.devices.clear();
        let default_name = self.host.default_output_device().and_then(|d| d.name().ok());
        if let Ok(devs) = self.host.output_devices() {
            for (idx, dev) in devs.enumerate() {
                let name = dev.name().unwrap_or_else(|_| "Bilinmeyen Cihaz".to_string());
                let is_default = default_name.as_ref().map(|d| d == &name).unwrap_or(false);
                if is_default {
                    self.selected_device_idx = idx;
                }
                self.devices.push(UiDeviceInfo {
                    index: idx,
                    name,
                    is_default,
                });
            }
        }
    }

    pub fn start_playback(&mut self) {
        self.stop_playback();

        let device = match select_device(&self.host, Some(self.selected_device_idx)) {
            Ok(d) => d,
            Err(e) => {
                self.status_message = format!("Hata: Cihaz seçilemedi ({})", e);
                self.is_error = true;
                return;
            }
        };

        let supported_config = match device.default_output_config() {
            Ok(c) => c,
            Err(e) => {
                self.status_message = format!("Hata: Cihaz yapılandırması alınamadı ({})", e);
                self.is_error = true;
                return;
            }
        };

        if supported_config.channels() < 2 {
            self.status_message = format!(
                "Hata: Bu cihaz stereo (en az 2 kanal) desteklemiyor (Mevcut: {} kanal).",
                supported_config.channels()
            );
            self.is_error = true;
            return;
        }

        let sample_rate = supported_config.sample_rate().0 as f64;
        let sample_format = supported_config.sample_format();
        let stream_config: StreamConfig = supported_config.into();

        let (sanitized_lvl, _) = sanitize_level(self.level);
        let current_sample = Arc::new(AtomicU64::new(0));
        let finished = Arc::new(AtomicBool::new(false));
        let error_flag = Arc::new(AtomicBool::new(false));

        let generator = SignalGenerator::new(
            self.signal_type,
            self.duration,
            sanitized_lvl,
            sample_rate,
            Arc::clone(&current_sample),
        );

        let stream_res = match sample_format {
            cpal::SampleFormat::F32 => build_stream::<f32>(
                &device,
                &stream_config,
                generator,
                self.channel_mode,
                Arc::clone(&finished),
                Arc::clone(&error_flag),
            ),
            cpal::SampleFormat::I16 => build_stream::<i16>(
                &device,
                &stream_config,
                generator,
                self.channel_mode,
                Arc::clone(&finished),
                Arc::clone(&error_flag),
            ),
            cpal::SampleFormat::I32 => build_stream::<i32>(
                &device,
                &stream_config,
                generator,
                self.channel_mode,
                Arc::clone(&finished),
                Arc::clone(&error_flag),
            ),
            cpal::SampleFormat::U16 => build_stream::<u16>(
                &device,
                &stream_config,
                generator,
                self.channel_mode,
                Arc::clone(&finished),
                Arc::clone(&error_flag),
            ),
            other => {
                self.status_message = format!("Hata: Desteklenmeyen örnek formatı: {:?}", other);
                self.is_error = true;
                return;
            }
        };

        let stream = match stream_res {
            Ok(s) => s,
            Err(e) => {
                self.status_message = format!("Hata: Ses akışı oluşturulamadı: {}", e);
                self.is_error = true;
                return;
            }
        };

        if let Err(e) = stream.play() {
            self.status_message = format!("Hata: Ses akışı başlatılamadı: {}", e);
            self.is_error = true;
            return;
        }

        self.active_playback = Some(ActivePlayback {
            _stream: stream,
            current_sample,
            sample_rate,
            duration: self.duration,
            signal_type: self.signal_type,
            channel_mode: self.channel_mode,
            finished,
            error_flag,
        });

        self.is_error = false;
        self.status_message = self.language.status_playing().to_string();
    }

    pub fn stop_playback(&mut self) {
        if self.active_playback.is_some() {
            self.active_playback = None;
            self.status_message = self.language.status_stopped().to_string();
        }
    }
}

impl eframe::App for IemTestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let lang = self.language;

        if let Some(ref pb) = self.active_playback {
            ctx.request_repaint();

            if pb.error_flag.load(Ordering::Acquire) {
                self.active_playback = None;
                self.status_message = lang.status_hardware_error().to_string();
                self.is_error = true;
            } else if pb.finished.load(Ordering::Acquire) {
                self.active_playback = None;
                self.status_message = lang.status_completed().to_string();
                self.is_error = false;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 9.0);

            // Üst Çubuk: Dil Seçimi (Ayrı üst satırda sağa yaslı)
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.selectable_label(self.language == Language::English, "🇬🇧 English").clicked() {
                        self.language = Language::English;
                        if self.active_playback.is_none() && !self.is_error {
                            self.status_message = self.language.status_ready().to_string();
                        }
                    }
                    if ui.selectable_label(self.language == Language::Turkish, "🇹🇷 Türkçe").clicked() {
                        self.language = Language::Turkish;
                        if self.active_playback.is_none() && !self.is_error {
                            self.status_message = self.language.status_ready().to_string();
                        }
                    }
                    ui.label(egui::RichText::new("🌐").size(14.0));
                });
            });

            // Başlık ve Açıklama (Ortalı)
            ui.vertical_centered(|ui| {
                ui.heading(lang.heading());
                ui.label(lang.subtitle());
            });

            ui.separator();

            // Güvenlik Uyarısı (Sarı Vurgulu Kutu)
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(45, 38, 20))
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(230, 180, 50)))
                .rounding(6.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("⚠️").size(22.0));
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(lang.safety_title())
                                    .strong()
                                    .color(egui::Color32::from_rgb(255, 215, 80)),
                            );
                            ui.label(lang.safety_desc());
                        });
                    });
                });

            // 1. Çıkış Cihazı Seçimi
            ui.group(|ui| {
                ui.label(egui::RichText::new(lang.section_device()).strong());
                ui.horizontal(|ui| {
                    let current_name = self
                        .devices
                        .iter()
                        .find(|d| d.index == self.selected_device_idx)
                        .map(|d| {
                            if d.is_default {
                                format!("[{}] {} {}", d.index, d.name, lang.default_tag())
                            } else {
                                format!("[{}] {}", d.index, d.name)
                            }
                        })
                        .unwrap_or_else(|| lang.select_device_placeholder().to_string());

                    egui::ComboBox::from_id_source("device_combo")
                        .width(ui.available_width() - 95.0)
                        .selected_text(current_name)
                        .show_ui(ui, |ui| {
                            for dev in &self.devices {
                                let label = if dev.is_default {
                                    format!("[{}] {} {}", dev.index, dev.name, lang.default_tag())
                                } else {
                                    format!("[{}] {}", dev.index, dev.name)
                                };
                                ui.selectable_value(&mut self.selected_device_idx, dev.index, label);
                            }
                        });

                    if ui.button(lang.refresh_btn()).clicked() {
                        self.refresh_devices();
                    }
                });
            });

            // 2. Kanal Seçimi
            ui.group(|ui| {
                ui.label(egui::RichText::new(lang.section_channel()).strong());
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.channel_mode, ChannelMode::Left, lang.channel_left());
                    ui.selectable_value(&mut self.channel_mode, ChannelMode::Right, lang.channel_right());
                    ui.selectable_value(&mut self.channel_mode, ChannelMode::Both, lang.channel_both());
                });
                ui.label(egui::RichText::new(lang.channel_desc(self.channel_mode)).weak().size(11.5));
            });

            // 3. Test Sinyali Seçimi
            ui.group(|ui| {
                ui.label(egui::RichText::new(lang.section_signal()).strong());
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(&mut self.signal_type, SignalType::Tone, lang.signal_tone());
                    ui.selectable_value(&mut self.signal_type, SignalType::Sweep, lang.signal_sweep());
                    ui.selectable_value(&mut self.signal_type, SignalType::BassSweep, lang.signal_bass_sweep());
                    ui.selectable_value(&mut self.signal_type, SignalType::PinkNoise, lang.signal_pink_noise());
                });
                ui.label(egui::RichText::new(lang.signal_desc(self.signal_type)).weak().size(11.5));
            });

            // 4. Parametreler (Süre ve Seviye)
            ui.group(|ui| {
                ui.label(egui::RichText::new(lang.section_params()).strong());

                ui.horizontal(|ui| {
                    ui.label(lang.duration_label());
                    ui.add(egui::Slider::new(&mut self.duration, 1.0..=60.0).suffix(lang.duration_suffix()));
                });

                let (sanitized, warn) = sanitize_level(self.level);
                let dbfs = if sanitized <= 0.0 {
                    "-sonsuz dBFS".to_string()
                } else {
                    format!("{:.1} dBFS", 20.0 * sanitized.log10())
                };

                ui.horizontal(|ui| {
                    ui.label(lang.level_label());
                    ui.add(egui::Slider::new(&mut self.level, 0.0..=0.30).step_by(0.01));
                    ui.label(format!("(≈ {})", dbfs));
                });

                if let Some(w) = warn {
                    ui.label(egui::RichText::new(w).color(egui::Color32::from_rgb(255, 120, 80)));
                } else {
                    ui.label(
                        egui::RichText::new(lang.safety_limit_note())
                            .weak()
                            .size(11.0),
                    );
                }
            });

            // 5. Hızlı Hazır Ayarlar (Presets)
            ui.collapsing(lang.presets_header(), |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button(lang.preset_left_tone()).clicked() {
                        self.channel_mode = ChannelMode::Left;
                        self.signal_type = SignalType::Tone;
                        self.duration = 30.0;
                        self.level = 0.1;
                    }
                    if ui.button(lang.preset_right_sweep()).clicked() {
                        self.channel_mode = ChannelMode::Right;
                        self.signal_type = SignalType::Sweep;
                        self.duration = 20.0;
                        self.level = 0.1;
                    }
                    if ui.button(lang.preset_both_bass()).clicked() {
                        self.channel_mode = ChannelMode::Both;
                        self.signal_type = SignalType::BassSweep;
                        self.duration = 15.0;
                        self.level = 0.15;
                    }
                    if ui.button(lang.preset_both_pink()).clicked() {
                        self.channel_mode = ChannelMode::Both;
                        self.signal_type = SignalType::PinkNoise;
                        self.duration = 30.0;
                        self.level = 0.08;
                    }
                });
            });

            ui.add_space(4.0);

            // 6. Başlat / Durdur Butonu ve Canlı Göstergeler
            let is_playing = self.active_playback.is_some();

            ui.vertical_centered(|ui| {
                if is_playing {
                    let stop_btn = egui::Button::new(
                        egui::RichText::new(lang.btn_stop())
                            .size(18.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(200, 50, 50))
                    .min_size(egui::vec2(220.0, 42.0));

                    if ui.add(stop_btn).clicked() {
                        self.stop_playback();
                    }
                } else {
                    let start_btn = egui::Button::new(
                        egui::RichText::new(lang.btn_start())
                            .size(18.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(34, 139, 34))
                    .min_size(egui::vec2(220.0, 42.0));

                    if ui.add(start_btn).clicked() {
                        self.start_playback();
                    }
                }
            });

            // Canlı İlerleme Çubuğu ve Frekans Göstergesi
            if let Some(ref pb) = self.active_playback {
                let cur = pb.current_sample.load(Ordering::Relaxed);
                let elapsed = (cur as f64 / pb.sample_rate).min(pb.duration);
                let progress = (elapsed / pb.duration).clamp(0.0, 1.0) as f32;

                ui.add_space(3.0);
                ui.add(egui::ProgressBar::new(progress).show_percentage());

                let cur_signal_str = match (lang, pb.signal_type) {
                    (Language::Turkish, SignalType::Tone) => "1,000 Hz Sabit Ton".to_string(),
                    (Language::Turkish, SignalType::Sweep) => {
                        let f = sweep_frequency(elapsed, pb.duration);
                        format!("{:.0} Hz (Sweep)", f)
                    }
                    (Language::Turkish, SignalType::BassSweep) => {
                        let f = bass_sweep_frequency(elapsed, pb.duration);
                        format!("{:.0} Hz (Bas)", f)
                    }
                    (Language::Turkish, SignalType::PinkNoise) => "Pembe Gürültü (-3 dB/oktav)".to_string(),
                    (Language::English, SignalType::Tone) => "1,000 Hz Constant Tone".to_string(),
                    (Language::English, SignalType::Sweep) => {
                        let f = sweep_frequency(elapsed, pb.duration);
                        format!("{:.0} Hz (Sweep)", f)
                    }
                    (Language::English, SignalType::BassSweep) => {
                        let f = bass_sweep_frequency(elapsed, pb.duration);
                        format!("{:.0} Hz (Bass)", f)
                    }
                    (Language::English, SignalType::PinkNoise) => "Pink Noise (-3 dB/oct)".to_string(),
                };

                let ch_str = lang.channel_tag(pb.channel_mode);

                ui.horizontal(|ui| {
                    ui.label(format!("{} {:.1} / {:.1}{}", lang.elapsed_label(), elapsed, pb.duration, lang.duration_suffix()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(ch_str).strong());
                        ui.label(egui::RichText::new(cur_signal_str).color(egui::Color32::LIGHT_BLUE));
                    });
                });
            }

            ui.add_space(3.0);

            // Durum Mesajı
            let status_color = if self.is_error {
                egui::Color32::from_rgb(255, 90, 90)
            } else if is_playing {
                egui::Color32::from_rgb(100, 220, 100)
            } else {
                egui::Color32::from_rgb(200, 200, 200)
            };
            ui.label(egui::RichText::new(&self.status_message).color(status_color));
        });
    }
}
