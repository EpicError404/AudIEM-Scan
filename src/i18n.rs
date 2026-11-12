//! Çoklu dil desteği (Türkçe ve İngilizce) için yerelleştirme modülü.

use crate::audio::{ChannelMode, SignalType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Turkish,
    English,
}

impl Language {
    pub fn app_title(self) -> &'static str {
        match self {
            Language::Turkish => "AudIEM Scan - Kulaklık Kanal ve Cızırtı Test Aracı",
            Language::English => "AudIEM Scan - Headphone Channel & Crackle Test Tool",
        }
    }

    pub fn heading(self) -> &'static str {
        match self {
            Language::Turkish => "🎧 AudIEM Scan - Kulaklık Kanal & Cızırtı Testi",
            Language::English => "🎧 AudIEM Scan - Headphone Channel & Crackle Test",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Language::Turkish => "Kablo temassızlığı, sürücü zırıltısı ve kanal dengesini test edin.",
            Language::English => "Test for cable dropouts, driver rattle, and channel balance.",
        }
    }

    pub fn safety_title(self) -> &'static str {
        match self {
            Language::Turkish => "GÜVENLİK UYARISI:",
            Language::English => "SAFETY WARNING:",
        }
    }

    pub fn safety_desc(self) -> &'static str {
        match self {
            Language::Turkish => "Kulaklığı takmadan önce sistem sesini düşük tutun ve seviyeyi kademeli artırın.",
            Language::English => "Keep system volume low before wearing headphones, then increase level gradually.",
        }
    }

    pub fn section_device(self) -> &'static str {
        match self {
            Language::Turkish => "1. Ses Çıkış Cihazı",
            Language::English => "1. Audio Output Device",
        }
    }

    pub fn default_tag(self) -> &'static str {
        match self {
            Language::Turkish => "(Varsayılan)",
            Language::English => "(Default)",
        }
    }

    pub fn select_device_placeholder(self) -> &'static str {
        match self {
            Language::Turkish => "Cihaz Seçin",
            Language::English => "Select Device",
        }
    }

    pub fn refresh_btn(self) -> &'static str {
        match self {
            Language::Turkish => "🔄 Yenile",
            Language::English => "🔄 Refresh",
        }
    }

    pub fn section_channel(self) -> &'static str {
        match self {
            Language::Turkish => "2. Kanal Yönlendirme",
            Language::English => "2. Channel Routing",
        }
    }

    pub fn channel_left(self) -> &'static str {
        match self {
            Language::Turkish => "🔊 Yalnız Sol (L)",
            Language::English => "🔊 Left Only (L)",
        }
    }

    pub fn channel_right(self) -> &'static str {
        match self {
            Language::Turkish => "🔊 Yalnız Sağ (R)",
            Language::English => "🔊 Right Only (R)",
        }
    }

    pub fn channel_both(self) -> &'static str {
        match self {
            Language::Turkish => "🎧 Her İki Kanal (L+R)",
            Language::English => "🎧 Both Channels (L+R)",
        }
    }

    pub fn channel_desc(self, mode: ChannelMode) -> &'static str {
        match (self, mode) {
            (Language::Turkish, ChannelMode::Left) => "Sinyal yalnızca SOL kanala gider, sağ kanal tam 0.0 (sessiz).",
            (Language::Turkish, ChannelMode::Right) => "Sinyal yalnızca SAĞ kanala gider, sol kanal tam 0.0 (sessiz).",
            (Language::Turkish, ChannelMode::Both) => "Sinyal hem SOL hem SAĞ kanala aynı anda gider.",
            (Language::English, ChannelMode::Left) => "Signal is sent ONLY to LEFT channel; right channel is completely silent (0.0).",
            (Language::English, ChannelMode::Right) => "Signal is sent ONLY to RIGHT channel; left channel is completely silent (0.0).",
            (Language::English, ChannelMode::Both) => "Signal is sent to BOTH left and right channels simultaneously.",
        }
    }

    pub fn section_signal(self) -> &'static str {
        match self {
            Language::Turkish => "3. Test Sinyali",
            Language::English => "3. Test Signal",
        }
    }

    pub fn signal_tone(self) -> &'static str {
        match self {
            Language::Turkish => "🎵 1 kHz Sinüs Tonu",
            Language::English => "🎵 1 kHz Sine Tone",
        }
    }

    pub fn signal_sweep(self) -> &'static str {
        match self {
            Language::Turkish => "🌊 20 Hz - 20 kHz Sweep",
            Language::English => "🌊 20 Hz - 20 kHz Sweep",
        }
    }

    pub fn signal_bass_sweep(self) -> &'static str {
        match self {
            Language::Turkish => "🔊 20 Hz - 250 Hz Bas Sweep",
            Language::English => "🔊 20 Hz - 250 Hz Bass Sweep",
        }
    }

    pub fn signal_pink_noise(self) -> &'static str {
        match self {
            Language::Turkish => "📻 Pembe Gürültü (Pink Noise)",
            Language::English => "📻 Pink Noise",
        }
    }

    pub fn signal_desc(self, sig: SignalType) -> &'static str {
        match (self, sig) {
            (Language::Turkish, SignalType::Tone) => "1 kHz Sabit Sinüs: Sürücü ses dengesi, sabit ton cızırtısı ve kanal eşitliği.",
            (Language::Turkish, SignalType::Sweep) => "20 Hz - 20 kHz Tam Tarama: Akustik rezonanslar, kablo paraziti ve frekans tepkisi.",
            (Language::Turkish, SignalType::BassSweep) => "20 Hz - 250 Hz Bas Taraması: Diyafram esnemesi (driver flex), bas titreşimi ve gövde zırıltısı.",
            (Language::Turkish, SignalType::PinkNoise) => "Pembe Gürültü (-3 dB/oktav): Kablo bükme, soket (MMCX/2-pin) oynatma ve statik çıtırtı tespiti.",
            (Language::English, SignalType::Tone) => "1 kHz Constant Sine: Driver audio balance, static tone crackling, and channel matching.",
            (Language::English, SignalType::Sweep) => "20 Hz - 20 kHz Full Sweep: Acoustic resonance, cable noise, and overall frequency response.",
            (Language::English, SignalType::BassSweep) => "20 Hz - 250 Hz Bass Sweep: Driver flex, low-frequency excursion, and housing rattle.",
            (Language::English, SignalType::PinkNoise) => "Pink Noise (-3 dB/oct): Cable flex, connector (MMCX/2-pin) wobble, and intermittent contact test.",
        }
    }

    pub fn section_params(self) -> &'static str {
        match self {
            Language::Turkish => "4. Süre ve Çıkış Seviyesi",
            Language::English => "4. Duration & Output Level",
        }
    }

    pub fn duration_label(self) -> &'static str {
        match self {
            Language::Turkish => "Test Süresi:",
            Language::English => "Duration:",
        }
    }

    pub fn duration_suffix(self) -> &'static str {
        match self {
            Language::Turkish => " sn",
            Language::English => " s",
        }
    }

    pub fn level_label(self) -> &'static str {
        match self {
            Language::Turkish => "Ses Seviyesi:",
            Language::English => "Volume Level:",
        }
    }

    pub fn safety_limit_note(self) -> &'static str {
        match self {
            Language::Turkish => "Güvenlik sınırı: 0.30 (-10.5 dBFS). Varsayılan: 0.30 (-10.5 dBFS).",
            Language::English => "Safety limit: 0.30 (-10.5 dBFS). Default: 0.30 (-10.5 dBFS).",
        }
    }

    pub fn presets_header(self) -> &'static str {
        match self {
            Language::Turkish => "⚡ Hızlı Hazır Ayarlar (Presets)",
            Language::English => "⚡ Quick Presets",
        }
    }

    pub fn preset_left_tone(self) -> &'static str {
        match self {
            Language::Turkish => "Sol: 1 kHz Ton (30s, 0.1)",
            Language::English => "Left: 1 kHz Tone (30s, 0.1)",
        }
    }

    pub fn preset_right_sweep(self) -> &'static str {
        match self {
            Language::Turkish => "Sağ: Sweep (20s, 0.1)",
            Language::English => "Right: Sweep (20s, 0.1)",
        }
    }

    pub fn preset_both_bass(self) -> &'static str {
        match self {
            Language::Turkish => "Her İkisi: Bas Sweep (15s, 0.15)",
            Language::English => "Both: Bass Sweep (15s, 0.15)",
        }
    }

    pub fn preset_both_pink(self) -> &'static str {
        match self {
            Language::Turkish => "Her İkisi: Pembe Gürültü (30s, 0.08)",
            Language::English => "Both: Pink Noise (30s, 0.08)",
        }
    }

    pub fn btn_start(self) -> &'static str {
        match self {
            Language::Turkish => "▶️ TESTİ BAŞLAT",
            Language::English => "▶️ START TEST",
        }
    }

    pub fn btn_stop(self) -> &'static str {
        match self {
            Language::Turkish => "⏹️ TESTİ DURDUR",
            Language::English => "⏹️ STOP TEST",
        }
    }

    pub fn elapsed_label(self) -> &'static str {
        match self {
            Language::Turkish => "Geçen:",
            Language::English => "Elapsed:",
        }
    }

    pub fn channel_tag(self, mode: ChannelMode) -> &'static str {
        match (self, mode) {
            (Language::Turkish, ChannelMode::Left) => "SOL (L)",
            (Language::Turkish, ChannelMode::Right) => "SAĞ (R)",
            (Language::Turkish, ChannelMode::Both) => "SOL + SAĞ (L+R)",
            (Language::English, ChannelMode::Left) => "LEFT (L)",
            (Language::English, ChannelMode::Right) => "RIGHT (R)",
            (Language::English, ChannelMode::Both) => "LEFT + RIGHT (L+R)",
        }
    }

    pub fn status_ready(self) -> &'static str {
        match self {
            Language::Turkish => "Hazır. Testi başlatmak için aşağıdaki butona basın.",
            Language::English => "Ready. Press the button below to start the test.",
        }
    }

    pub fn status_playing(self) -> &'static str {
        match self {
            Language::Turkish => "Sinyal çalınıyor... Lütfen kulaklığınızı dinleyin.",
            Language::English => "Signal playing... Please listen to your headphones.",
        }
    }

    pub fn status_stopped(self) -> &'static str {
        match self {
            Language::Turkish => "Oynatma durduruldu.",
            Language::English => "Playback stopped.",
        }
    }

    pub fn status_completed(self) -> &'static str {
        match self {
            Language::Turkish => "✅ Test tamamlandı. Cızırtı veya kesilme fark ettiniz mi?",
            Language::English => "✅ Test completed. Did you notice any crackle or dropouts?",
        }
    }

    pub fn status_hardware_error(self) -> &'static str {
        match self {
            Language::Turkish => "Hata: Ses aygıtında donanım hatası oluştu!",
            Language::English => "Error: Hardware/driver error occurred in audio device!",
        }
    }
}

