# AudIEM Scan 🎧

**AudIEM Scan**, kulak içi kulaklıklar (IEM) ve kafa üstü kulaklıklar için geliştirilmiş profesyonel bir kanal dengesi, sürücü rezonansı ve cızırtı test aracıdır.

---

## 🚀 Özellikler (Features)

- **🎧 Kanal Yönlendirme:**
  - Yalnız Sol (L)
  - Yalnız Sağ (R)
  - Her İki Kanal (L+R)
- **🔊 4 Farklı Özel Test Sinyali:**
  - **1 kHz Sinüs Tonu:** Kanal dengesi ve süreklilik testi (1 kHz Saf Sinüs).
  - **20 Hz - 20 kHz Logaritmik Frekans Taraması (Sweep):** Tüm duyulabilir spektrum boyunca sürücü rezonansı ve çatlama tespiti.
  - **20 Hz - 250 Hz Bas Taraması:** Alt frekanslarda sürücü titremesi (rattle), gevşek armatür ve gövde rezonansı tespiti.
  - **Pembe Gürültü (Pink Noise):** Paul Kellet -3 dB/oktav filtresiyle eşit enerji dağılımı; kablo bükme ve temassızlık tespiti.
- **🛡️ Kulak ve Sürücü Koruma Güvenliği:**
  - Maksimum çıkış seviyesi **0.30 (-10.5 dBFS)** ile sınırlandırılmıştır.
  - Her sinyalin başlangıcında ve bitişinde **50 ms yumuşak fade-in / fade-out** uygulanır (ani patlama veya klik sesi oluşmaz).
- **🌐 Çift Dil Desteği (Dual Language):**
  - Türkçe (Varsayılan)
  - English
- **🖥️ İki Arayüz Bir Arada:**
  - Kullanımı kolay, görselleştirilmiş modern Masaüstü UI (GUI).
  - Terminal ve otomasyon için tam yetenekli Komut Satırı Arayüzü (CLI).

---

## 📥 İndirme (Download)

En güncel tek parça çalıştırılabilir dosyayı **[Releases](https://github.com/Zgn/AudIEM-Scan/releases)** sayfasından `AudIEM_Scan.exe` olarak doğrudan indirip kurmadan çalıştırabilirsiniz.

---

## 💻 Komut Satırı Kullanımı (CLI Usage)

```bash
# Ses çıkış cihazlarını listeleme
AudIEM_Scan.exe list

# Yalnızca sol kanalda 1 kHz sinüs tonu (30 saniye, 0.1 genlik)
AudIEM_Scan.exe left tone 30 0.1

# Yalnızca sağ kanalda 20 Hz - 20 kHz frekans taraması (20 saniye, 0.1 genlik)
AudIEM_Scan.exe right sweep 20 0.1

# Her iki kanalda 20 Hz - 250 Hz bas taraması (15 saniye, 0.15 genlik)
AudIEM_Scan.exe both bass-sweep 15 0.15

# Her iki kanalda pembe gürültü (30 saniye, 0.08 genlik)
AudIEM_Scan.exe both pink-noise 30 0.08

# Belirli bir ses kartı cihazını hedefleyerek çalıştırma
AudIEM_Scan.exe both tone 10 0.05 --device 2
```

---

## 🛠️ Kaynak Koddan Derleme (Building from Source)

Gereksinimler:
- [Rust & Cargo](https://rustup.rs/) (Rust 2021 edition)

```bash
# Depoyu klonlayın
git clone https://github.com/Zgn/AudIEM-Scan.git
cd AudIEM-Scan

# Testleri çalıştırın
cargo test

# Optimize edilmiş sürümü derleyin
cargo build --release
```

---

## 📄 Lisans

MIT License.
