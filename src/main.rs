//! AudIEM Scan - Kulak İçi Kulaklık Kanal ve Cızırtı Test Aracı
//!
//! Hem Komut Satırı (CLI) hem de Grafik Kullanıcı Arayüzü (GUI) ile
//! kulak içi kulaklıklarda sürücü veya kablo kaynaklı cızırtı/çıtırtı testleri yapar.

pub mod audio;
pub mod cli;
pub mod i18n;
pub mod ui;

use cli::{parse_args, print_help, print_short_usage, run_cli_test, list_devices, CliCommand};
use eframe::egui;
use ui::IemTestApp;

fn main() {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    let command = match parse_args(&raw_args) {
        Ok(cmd) => cmd,
        Err(err) => {
            print_short_usage(&err);
            std::process::exit(1);
        }
    };

    let host = cpal::default_host();

    match command {
        CliCommand::Help => {
            print_help();
        }
        CliCommand::List => {
            if let Err(e) = list_devices(&host) {
                eprintln!("Hata: {}", e);
                std::process::exit(1);
            }
        }
        CliCommand::Run(run_config) => {
            run_cli_test(run_config, &host);
        }
        CliCommand::Gui => {
            let native_options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([580.0, 750.0])
                    .with_min_inner_size([500.0, 640.0])
                    .with_title("AudIEM Scan - Kulaklık Kanal ve Cızırtı Test Aracı"),
                ..Default::default()
            };

            let _ = eframe::run_native(
                "AudIEM Scan",
                native_options,
                Box::new(|cc| Box::new(IemTestApp::new(cc))),
            );
        }
    }
}
