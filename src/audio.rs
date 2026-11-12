//! Ses işleme, sinyal üretimi ve generic cpal ses akışı yönetimi.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use cpal::traits::DeviceTrait;
use cpal::{FromSample, Sample, SizedSample, Stream, StreamConfig};

/// Test edilecek kanal yönlendirme modu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelMode {
    Left,
    Right,
    Both,
}

/// Üretilecek test sinyali türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    Tone,      // 1 kHz sabit sinüs tonu
    Sweep,     // 20 Hz - 20 kHz tam frekans taraması
    BassSweep, // 20 Hz - 250 Hz bas / sub-bass taraması (Sürücü zırıltısı & rezonans)
    PinkNoise, // Pembe gürültü (Kablo bükme & statik temassızlık testi)
}

/// Güvenli seviye sınırlaması uygular.
pub fn sanitize_level(input: f32) -> (f32, Option<&'static str>) {
    if input > 0.3 {
        (
            0.3,
            Some("Uyarı: Girilen seviye güvenli üst sınır olan 0.3'ten büyük olduğu için 0.3'e sabitlendi."),
        )
    } else if input < 0.0 {
        (
            0.0,
            Some("Uyarı: Negatif seviye girildiği için 0.0'a sabitlendi."),
        )
    } else {
        (input, None)
    }
}

/// 20 Hz - 20 kHz logaritmik sweep frekansı hesabı.
pub fn sweep_frequency(t: f64, total_duration: f64) -> f64 {
    if total_duration <= 0.0 {
        return 20.0;
    }
    let ratio = (t / total_duration).clamp(0.0, 1.0);
    20.0 * 1000.0_f64.powf(ratio)
}

/// 20 Hz - 250 Hz bas / sub-bass logaritmik tarama frekansı hesabı.
pub fn bass_sweep_frequency(t: f64, total_duration: f64) -> f64 {
    if total_duration <= 0.0 {
        return 20.0;
    }
    let ratio = (t / total_duration).clamp(0.0, 1.0);
    20.0 * (250.0 / 20.0_f64).powf(ratio)
}

/// Yumuşak başlangıç ve bitiş (fade-in / fade-out) zarfı.
pub fn fade_factor(t: f64, total_duration: f64, fade_duration: f64) -> f64 {
    if t < 0.0 || t >= total_duration || total_duration <= 0.0 {
        return 0.0;
    }

    let effective_fade = fade_duration.min(total_duration / 2.0);
    if effective_fade <= 0.0 {
        return 1.0;
    }

    if t < effective_fade {
        0.5 * (1.0 - (std::f64::consts::PI * t / effective_fade).cos())
    } else if t > total_duration - effective_fade {
        let t_rem = total_duration - t;
        0.5 * (1.0 - (std::f64::consts::PI * t_rem / effective_fade).cos())
    } else {
        1.0
    }
}

/// Cpal'den bağımsız, saf ve test edilebilir sinyal üretici.
pub struct SignalGenerator {
    signal_type: SignalType,
    duration_secs: f64,
    level: f32,
    sample_rate: f64,
    current_sample: Arc<AtomicU64>,
    total_samples: u64,
    phase: f64,
    // Pembe gürültü için Paul Kellet filtresi durum değişkenleri
    rng_state: u32,
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
}

impl SignalGenerator {
    pub fn new(
        signal_type: SignalType,
        duration_secs: f64,
        level: f32,
        sample_rate: f64,
        current_sample: Arc<AtomicU64>,
    ) -> Self {
        let total_samples = (duration_secs * sample_rate).round().max(0.0) as u64;
        Self {
            signal_type,
            duration_secs,
            level,
            sample_rate,
            current_sample,
            total_samples,
            phase: 0.0,
            rng_state: 123456789,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
        }
    }

    pub fn next_frame(&mut self, channel_mode: ChannelMode) -> (f32, f32) {
        let cur = self.current_sample.load(Ordering::Relaxed);
        if cur >= self.total_samples {
            return (0.0, 0.0);
        }

        let t = cur as f64 / self.sample_rate;
        let fade = fade_factor(t, self.duration_secs, 0.05) as f32;

        let raw_sample = match self.signal_type {
            SignalType::Tone => {
                let f = 1000.0;
                let phase_step = 2.0 * std::f64::consts::PI * f / self.sample_rate;
                let s = self.phase.sin() as f32;
                self.phase += phase_step;
                if self.phase >= 2.0 * std::f64::consts::PI {
                    self.phase %= 2.0 * std::f64::consts::PI;
                }
                s
            }
            SignalType::Sweep => {
                let f = sweep_frequency(t, self.duration_secs);
                let phase_step = 2.0 * std::f64::consts::PI * f / self.sample_rate;
                let s = self.phase.sin() as f32;
                self.phase += phase_step;
                if self.phase >= 2.0 * std::f64::consts::PI {
                    self.phase %= 2.0 * std::f64::consts::PI;
                }
                s
            }
            SignalType::BassSweep => {
                let f = bass_sweep_frequency(t, self.duration_secs);
                let phase_step = 2.0 * std::f64::consts::PI * f / self.sample_rate;
                let s = self.phase.sin() as f32;
                self.phase += phase_step;
                if self.phase >= 2.0 * std::f64::consts::PI {
                    self.phase %= 2.0 * std::f64::consts::PI;
                }
                s
            }
            SignalType::PinkNoise => {
                // Xorshift32 PRNG
                self.rng_state ^= self.rng_state << 13;
                self.rng_state ^= self.rng_state >> 17;
                self.rng_state ^= self.rng_state << 5;
                let white = (self.rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

                // Paul Kellet pembe gürültü filtresi (-3 dB/oktav)
                self.b0 = 0.99886 * self.b0 + white * 0.0555179;
                self.b1 = 0.99332 * self.b1 + white * 0.0750759;
                self.b2 = 0.96900 * self.b2 + white * 0.1538520;
                self.b3 = 0.86650 * self.b3 + white * 0.3104856;
                self.b4 = 0.55000 * self.b4 + white * 0.5329522;
                self.b5 = -0.7616 * self.b5 - white * 0.0168980;

                let pink = (self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + white * 0.5362) * 0.12;
                pink.clamp(-1.0, 1.0)
            }
        };

        self.current_sample.fetch_add(1, Ordering::Relaxed);
        let sample = raw_sample * self.level * fade;

        match channel_mode {
            ChannelMode::Left => (sample, 0.0),
            ChannelMode::Right => (0.0, sample),
            ChannelMode::Both => (sample, sample),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.current_sample.load(Ordering::Relaxed) >= self.total_samples
    }
}

/// Generic ses akışı oluşturma fonksiyonu.
pub fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    mut generator: SignalGenerator,
    channel_mode: ChannelMode,
    finished: Arc<AtomicBool>,
    error_flag: Arc<AtomicBool>,
) -> Result<Stream, cpal::BuildStreamError>
where
    T: Sample + FromSample<f32> + SizedSample + Send + 'static,
{
    let channel_count = config.channels as usize;
    let zero_sample = T::from_sample(0.0f32);
    let error_flag_cb = Arc::clone(&error_flag);

    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channel_count) {
                let (left_val, right_val) = generator.next_frame(channel_mode);
                if let Some(left) = frame.get_mut(0) {
                    *left = T::from_sample(left_val);
                }
                if let Some(right) = frame.get_mut(1) {
                    *right = T::from_sample(right_val);
                }
                for extra in frame.iter_mut().skip(2) {
                    *extra = zero_sample;
                }
            }
            if generator.is_finished() {
                finished.store(true, Ordering::Release);
            }
        },
        move |_err| {
            error_flag_cb.store(true, Ordering::Release);
        },
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sweep_frequency_endpoints() {
        let duration = 20.0;
        let f_start = sweep_frequency(0.0, duration);
        let f_end = sweep_frequency(duration, duration);
        let f_mid = sweep_frequency(duration / 2.0, duration);

        assert!((f_start - 20.0).abs() < 1e-6);
        assert!((f_end - 20000.0).abs() < 1e-4);
        assert!((f_mid - 632.4555).abs() < 1e-2);
    }

    #[test]
    fn test_bass_sweep_frequency_endpoints() {
        let duration = 15.0;
        let f_start = bass_sweep_frequency(0.0, duration);
        let f_end = bass_sweep_frequency(duration, duration);
        assert!((f_start - 20.0).abs() < 1e-6);
        assert!((f_end - 250.0).abs() < 1e-4);
    }

    #[test]
    fn test_fade_envelope() {
        let duration = 10.0;
        let fade_len = 0.05;

        assert_eq!(fade_factor(0.0, duration, fade_len), 0.0);
        let early = fade_factor(0.001, duration, fade_len);
        assert!(early > 0.0 && early < 0.1);
        assert_eq!(fade_factor(duration / 2.0, duration, fade_len), 1.0);
        assert_eq!(fade_factor(duration, duration, fade_len), 0.0);
    }

    #[test]
    fn test_channel_routing_left_mode() {
        let cur = Arc::new(AtomicU64::new(0));
        let mut gen = SignalGenerator::new(SignalType::Tone, 1.0, 0.1, 44100.0, cur);
        for _ in 0..1000 {
            let (_, right) = gen.next_frame(ChannelMode::Left);
            assert_eq!(right, 0.0f32);
        }
    }

    #[test]
    fn test_channel_routing_right_mode() {
        let cur = Arc::new(AtomicU64::new(0));
        let mut gen = SignalGenerator::new(SignalType::Tone, 1.0, 0.1, 44100.0, cur);
        for _ in 0..1000 {
            let (left, _) = gen.next_frame(ChannelMode::Right);
            assert_eq!(left, 0.0f32);
        }
    }

    #[test]
    fn test_level_sanitization() {
        let (val_high, warn_high) = sanitize_level(0.5);
        assert_eq!(val_high, 0.3);
        assert!(warn_high.is_some());

        let (val_neg, warn_neg) = sanitize_level(-0.2);
        assert_eq!(val_neg, 0.0);
        assert!(warn_neg.is_some());

        let (val_normal, warn_normal) = sanitize_level(0.15);
        assert_eq!(val_normal, 0.15);
        assert!(warn_normal.is_none());
    }

    #[test]
    fn test_sweep_no_periodic_loop() {
        let sr = 1000.0;
        let duration = 0.1;
        let cur = Arc::new(AtomicU64::new(0));
        let mut gen = SignalGenerator::new(SignalType::Sweep, duration, 0.2, sr, cur);

        for _ in 0..100 {
            gen.next_frame(ChannelMode::Both);
        }
        assert!(gen.is_finished());
        let (l, r) = gen.next_frame(ChannelMode::Both);
        assert_eq!(l, 0.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn test_pink_noise_generation() {
        let sr = 44100.0;
        let duration = 1.0;
        let cur = Arc::new(AtomicU64::new(0));
        let mut gen = SignalGenerator::new(SignalType::PinkNoise, duration, 0.1, sr, cur);

        let mut non_zero_count = 0;
        // 50 ms fade-in bittikten sonrasını (yaklaşık 2500 örnek sonrası) test et
        for i in 0..5000 {
            let (l, r) = gen.next_frame(ChannelMode::Both);
            assert_eq!(l, r);
            if i > 2500 && l.abs() > 0.0001 {
                non_zero_count += 1;
            }
        }
        assert!(non_zero_count > 2000, "Fade-in sonrasında pembe gürültü örnekleri üretilmelidir");
    }
}
