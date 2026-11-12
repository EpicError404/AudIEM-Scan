//! Komut satırı (CLI) argüman ayrıştırma, cihaz listeleme ve terminal üzerinden test çalıştırma.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;

use crate::audio::{
    build_stream, sanitize_level, ChannelMode, SignalGenerator, SignalType,
};

/// CLI çalıştırma yapılandırması.
#[derive(Debug, Clone, PartialEq)]
pub struct RunConfig {
    pub channel_mode: ChannelMode,
    pub signal_type: SignalType,
    pub duration: f64,
    pub level: f32,
    pub device_index: Option<usize>,
}

/// CLI komutları.
#[derive(Debug, Clone, PartialEq)]
pub enum CliCommand {
    List,
    Help,
    Run(RunConfig),
    Gui,
}

/// Komut satırı argümanlarını ayrıştırır.
pub fn parse_args(args: &[String]) -> Result<CliCommand, String> {
    if args.is_empty() {
        return Ok(CliCommand::Gui);
    }

    let first = args[0].as_str();
    if first == "-h" || first == "--help" || first == "help" {
        return Ok(CliCommand::Help);
    }
    if first == "list" {
        if args.len() > 1 {
            return Err("'list' komutu ek parametre almaz.".to_string());
        }
        return Ok(CliCommand::List);
    }
    if first == "gui" || first == "--gui" {
        return Ok(CliCommand::Gui);
    }

    let mut device_index: Option<usize> = None;
    let mut positional: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--device" {
            if i + 1 >= args.len() {
                return Err("'--device' bayrağından sonra bir cihaz indeksi belirtilmelidir.".to_string());
            }
            let idx = args[i + 1]
                .parse::<usize>()
                .map_err(|_| format!("Geçersiz cihaz indeksi: '{}'", args[i + 1]))?;
            device_index = Some(idx);
            i += 2;
        } else if let Some(stripped) = arg.strip_prefix("--device=") {
            let idx = stripped
                .parse::<usize>()
                .map_err(|_| format!("Geçersiz cihaz indeksi: '{}'", stripped))?;
            device_index = Some(idx);
            i += 1;
        } else if arg.starts_with('-') {
            return Err(format!("Bilinmeyen seçenek: '{}'", arg));
        } else {
            positional.push(arg);
            i += 1;
        }
    }

    if positional.len() < 2 {
        return Err("Kanal ve sinyal türü zorunludur.".to_string());
    }
    if positional.len() > 4 {
        return Err("Fazla sayıda parametre girildi.".to_string());
    }

    let channel_mode = match positional[0] {
        "left" => ChannelMode::Left,
        "right" => ChannelMode::Right,
        "both" => ChannelMode::Both,
        other => {
            return Err(format!(
                "Geçersiz kanal: '{}'. Desteklenenler: 'left', 'right', 'both'.",
                other
            ))
        }
    };

    let signal_type = match positional[1] {
        "tone" => SignalType::Tone,
        "sweep" => SignalType::Sweep,
        "bass" => SignalType::BassSweep,
        "pink" => SignalType::PinkNoise,
        other => {
            return Err(format!(
                "Geçersiz sinyal türü: '{}'. Desteklenenler: 'tone', 'sweep', 'bass', 'pink'.",
                other
            ))
        }
    };

    let duration: f64 = if positional.len() >= 3 {
        let d = positional[2]
            .parse::<f64>()
            .map_err(|_| format!("Geçersiz süre değeri: '{}'", positional[2]))?;
        if d <= 0.0 {
            return Err("Süre 0'dan büyük bir değer olmalıdır.".to_string());
        }
        d
    } else {
        20.0
    };

    let level: f32 = if positional.len() >= 4 {
        positional[3]
            .parse::<f32>()
            .map_err(|_| format!("Geçersiz seviye değeri: '{}'", positional[3]))?
    } else {
        0.1
    };

    Ok(CliCommand::Run(RunConfig {
        channel_mode,
        signal_type,
        duration,
        level,
        device_index,
    }))
}

pub fn print_help() {
    println!(
        r#"AudIEM Scan - Kulaklık Kanal ve Cızırtı Test Aracı

KULLANIM:
  audiem_scan                           Grafik Arayüzü (GUI) başlatır.
  audiem_scan list                      Mevcut ses çıkış cihazlarını listeler.
  audiem_scan <left|right|both> <tone|sweep|bass|pink> [saniye=20] [seviye=0.3] [--device N]
  audiem_scan help (veya -h)

PARAMETRELER:
  <left|right|both>           left (yalnız sol), right (yalnız sağ), both (her ikisi)
  <tone|sweep|bass|pink>      tone  : 1 kHz sabit sinüs tonu (sürücü dengesi/çıtırtı)
                              sweep : 20 Hz - 20 kHz tam frekans taraması (rezonans)
                              bass  : 20 Hz - 250 Hz bas taraması (sürücü zırıltısı & diyafram)
                              pink  : Pembe gürültü (kablo oynatma & statik çıtırtı)
  [saniye]                    Test süresi (varsayılan: 20)
  [seviye]                    Sinyal genliği (varsayılan: 0.1, güvenli üst sınır: 0.3)
  [--device N]                Belirtilen ses cihazı indeksi

ÖRNEKLER:
  cargo run --release                   # Grafik Arayüzü (GUI) açar
  cargo run --release -- list           # Cihazları listeler
  cargo run --release -- left tone 30 0.1
  cargo run --release -- right sweep 20 0.1
  cargo run --release -- both bass 15 0.15
  cargo run --release -- both pink 30 0.08 --device 2
"#
    );
}

pub fn print_short_usage(err_msg: &str) {
    eprintln!("Hata: {}\n", err_msg);
    eprintln!("Kullanım:");
    eprintln!("  audiem_scan (GUI modunda açmak için)");
    eprintln!("  audiem_scan list");
    eprintln!("  audiem_scan <left|right|both> <tone|sweep|bass|pink> [saniye=20] [seviye=0.3] [--device N]");
    eprintln!("  audiem_scan help (veya -h)\n");
    eprintln!("Detaylı bilgi için 'audiem_scan help' komutunu çalıştırın.");
}

pub fn list_devices(host: &cpal::Host) -> Result<(), String> {
    let devices = host
        .output_devices()
        .map_err(|e| format!("Çıkış cihazları listelenemedi: {}", e))?;

    let default_device_name = host.default_output_device().and_then(|d| d.name().ok());

    println!("Mevcut Ses Çıkış Cihazları:");
    let mut count = 0;
    for (index, device) in devices.enumerate() {
        let name = device
            .name()
            .unwrap_or_else(|_| "Bilinmeyen Cihaz".to_string());
        if let Some(ref def_name) = default_device_name {
            if def_name == &name {
                println!("  [{}] {} (Varsayılan)", index, name);
                count += 1;
                continue;
            }
        }
        println!("  [{}] {}", index, name);
        count += 1;
    }

    if count == 0 {
        println!("  Hiçbir çıkış cihazı bulunamadı.");
    }
    Ok(())
}

pub fn select_device(host: &cpal::Host, device_index: Option<usize>) -> Result<cpal::Device, String> {
    match device_index {
        Some(idx) => {
            let devices: Vec<cpal::Device> = host
                .output_devices()
                .map_err(|e| format!("Çıkış cihazları taranamadı: {}", e))?
                .collect();

            if idx >= devices.len() {
                return Err(format!(
                    "Geçersiz cihaz indeksi: {}. Mevcut cihaz sayısı: {}. 'iem_test list' komutu ile indeksleri görebilirsiniz.",
                    idx,
                    devices.len()
                ));
            }
            devices
                .into_iter()
                .nth(idx)
                .ok_or_else(|| "Cihaz seçilemedi.".to_string())
        }
        None => host
            .default_output_device()
            .ok_or_else(|| "Sistemde varsayılan ses çıkış cihazı bulunamadı.".to_string()),
    }
}

pub fn run_cli_test(run_config: RunConfig, host: &cpal::Host) {
    let device = match select_device(host, run_config.device_index) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Hata: {}", e);
            std::process::exit(1);
        }
    };

    let device_name = device
        .name()
        .unwrap_or_else(|_| "Bilinmeyen Cihaz".to_string());

    let supported_config = match device.default_output_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Hata: Cihazın varsayılan ses yapılandırması alınamadı: {}", e);
            std::process::exit(1);
        }
    };

    let channels = supported_config.channels();
    if channels < 2 {
        eprintln!(
            "Hata: Cihaz en az 2 kanal (stereo) desteklemelidir. Bu cihaz yalnızca {} kanal sunuyor.",
            channels
        );
        std::process::exit(1);
    }

    let sample_rate = supported_config.sample_rate().0 as f64;
    let sample_format = supported_config.sample_format();
    let stream_config: StreamConfig = supported_config.into();

    let (sanitized_level, level_warning) = sanitize_level(run_config.level);

    let dbfs_str = if sanitized_level <= 0.0 {
        "-sonsuz dBFS".to_string()
    } else {
        format!("{:.1} dBFS", 20.0 * sanitized_level.log10())
    };

    let channel_desc = match run_config.channel_mode {
        ChannelMode::Left => "Sol Kanal (Yalnızca SOL kulaklık çalıyor, SAĞ kulaklık sessiz)",
        ChannelMode::Right => "Sağ Kanal (Yalnızca SAĞ kulaklık çalıyor, SOL kulaklık sessiz)",
        ChannelMode::Both => "Her İki Kanal (SOL ve SAĞ kulaklıklar birlikte çalıyor)",
    };

    let signal_desc = match run_config.signal_type {
        SignalType::Tone => "1 kHz Sabit Sinüs Tonu".to_string(),
        SignalType::Sweep => "20 Hz - 20 kHz Logaritmik Frekans Taraması (Tek Sefer)".to_string(),
        SignalType::BassSweep => "20 Hz - 250 Hz Bas / Sub-Bass Taraması (Sürücü Zırıltısı Testi)".to_string(),
        SignalType::PinkNoise => "Pembe Gürültü (Kablo Oynatma & Statik Çıtırtı Testi)".to_string(),
    };

    println!("============================================================");
    println!("              AudIEM Scan - KANAL VE CIZIRTI TESTİ          ");
    println!("============================================================");
    println!("GÜVENLİK UYARISI:");
    println!("  Kulaklığı takmadan önce sistem sesini düşük tut ve seviyeyi kademeli artır.\n");

    if let Some(warn) = level_warning {
        eprintln!("{}\n", warn);
    }

    println!("Cihaz Bilgileri:");
    println!("  Cihaz Adı      : {}", device_name);
    println!("  Örnekleme Hızı : {} Hz", sample_rate as u32);
    println!("  Kanal Sayısı   : {}", channels);
    println!("  Örnek Formatı  : {:?}", sample_format);
    println!("------------------------------------------------------------");
    println!("Test Parametreleri:");
    println!("  Kanal Seçimi   : {}", channel_desc);
    println!("  Sinyal Türü    : {}", signal_desc);
    println!("  Test Süresi    : {:.1} saniye", run_config.duration);
    println!("  Çıkış Seviyesi : {:.4} ({})", sanitized_level, dbfs_str);
    println!("============================================================");
    println!("Sinyal çalınıyor... (Durdurmak için Ctrl+C)\n");

    let current_sample = Arc::new(AtomicU64::new(0));
    let generator = SignalGenerator::new(
        run_config.signal_type,
        run_config.duration,
        sanitized_level,
        sample_rate,
        Arc::clone(&current_sample),
    );

    let finished = Arc::new(AtomicBool::new(false));
    let error_flag = Arc::new(AtomicBool::new(false));

    let stream_res = match sample_format {
        cpal::SampleFormat::F32 => build_stream::<f32>(
            &device,
            &stream_config,
            generator,
            run_config.channel_mode,
            Arc::clone(&finished),
            Arc::clone(&error_flag),
        ),
        cpal::SampleFormat::I16 => build_stream::<i16>(
            &device,
            &stream_config,
            generator,
            run_config.channel_mode,
            Arc::clone(&finished),
            Arc::clone(&error_flag),
        ),
        cpal::SampleFormat::I32 => build_stream::<i32>(
            &device,
            &stream_config,
            generator,
            run_config.channel_mode,
            Arc::clone(&finished),
            Arc::clone(&error_flag),
        ),
        cpal::SampleFormat::U16 => build_stream::<u16>(
            &device,
            &stream_config,
            generator,
            run_config.channel_mode,
            Arc::clone(&finished),
            Arc::clone(&error_flag),
        ),
        other => {
            eprintln!(
                "Hata: Desteklenmeyen örnek formatı: {:?}. Desteklenenler: F32, I16, I32, U16.",
                other
            );
            std::process::exit(1);
        }
    };

    let stream = match stream_res {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Hata: Ses akışı oluşturulamadı: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = stream.play() {
        eprintln!("Hata: Ses akışı başlatılamadı: {}", e);
        std::process::exit(1);
    }

    while !finished.load(Ordering::Acquire) {
        if error_flag.load(Ordering::Acquire) {
            eprintln!("\nHata: Ses akışı sırasında donanım hatası oluştu.");
            std::process::exit(1);
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    std::thread::sleep(Duration::from_millis(100));
    drop(stream);

    println!("Test başarıyla tamamlandı.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_all_commands() {
        assert_eq!(parse_args(&[]).unwrap(), CliCommand::Gui);
        assert_eq!(parse_args(&["help".to_string()]).unwrap(), CliCommand::Help);
        assert_eq!(parse_args(&["list".to_string()]).unwrap(), CliCommand::List);

        let cmd_run = parse_args(&[
            "left".to_string(),
            "bass".to_string(),
            "15".to_string(),
            "0.2".to_string(),
            "--device".to_string(),
            "3".to_string(),
        ])
        .unwrap();

        assert_eq!(
            cmd_run,
            CliCommand::Run(RunConfig {
                channel_mode: ChannelMode::Left,
                signal_type: SignalType::BassSweep,
                duration: 15.0,
                level: 0.2,
                device_index: Some(3),
            })
        );
    }
}

