<div align="center">
  <img src="https://raw.githubusercontent.com/RuriChoco/Photopaper/main/data/icons/hicolor/scalable/apps/com.github.RuriChoco.Photopaper.svg" width="128" height="128" alt="Photopaper Logo">
  <h1>Photopaper</h1>
  <p><strong>A simple, elegant tool to create perfectly sized passport and ID photos with AI background removal.</strong></p>
  
  <br/>
  <img src="https://raw.githubusercontent.com/RuriChoco/Photopaper/main/screenshots/main-window.png" alt="Photopaper Main Window" width="800">
</div>

---

## 📸 Overview

Generating ID photos often requires tedious manual formatting in complex, heavyweight photo editing software. **Photopaper** eliminates this friction by providing a streamlined, purpose-built interface to automatically crop, format, enhance, and generate print-ready layouts tailored for standard photo paper sizes.

Built from the ground up in Rust using GTK4 and Libadwaita, Photopaper seamlessly integrates into the modern GNOME desktop environment, providing a beautiful, native user experience combined with high-performance, multi-threaded image processing.

---

## ✨ Key Features

### Intelligent Layout Generation
- **Standard ID Sizes:** Automatically crop and scale photos to precise dimensions: 1x1 inch, 2x2 inch, and Passport (35x45mm).
- **Print Canvases:** Generate high-resolution, 300 DPI print layouts tailored for popular photo paper sizes including 4R (4x6 inches), 5R, and standard A4.
- **Smart Packing:** Automatically packs the maximum number of photos onto the selected canvas size.

### Advanced Photo Editing
- **Rule of Thirds Cropping:** An intuitive, interactive cropping tool with a built-in Rule of Thirds grid to ensure perfect portrait composition.
- **AI-Style Background Removal:** Advanced boundary-fill algorithms allow you to instantly strip complex backgrounds and replace them with standard ID white.
- **Color Correction:** Fine-tune Brightness, Contrast, and Image Sharpness to ensure your prints look professional and vibrant.

### High-Performance Core
- **Multi-Core Acceleration:** Powered by the `rayon` engine, heavy image manipulation (like mathematical dilation and blurring) is distributed across all available CPU cores for zero-latency processing.
- **Lossless EXIF Injection:** Ensures crucial camera metadata is seamlessly transferred from the original photo directly into the exported print layout.
- **Dynamic Exporting:** Export your final layouts in either PNG or JPEG formats instantly.

---

## 🛠️ Technical Stack

Photopaper is engineered for safety, speed, and native Linux integration.

- **Language:** [Rust](https://www.rust-lang.org/) — Ensures memory safety and fearless concurrency.
- **UI Toolkit:** [GTK4](https://gtk.org/) + [Libadwaita](https://gitlab.gnome.org/GNOME/libadwaita) — Provides hardware-accelerated rendering and modern GNOME Human Interface Guidelines (HIG) components.
- **Image Processing Engine:** The [`image`](https://crates.io/crates/image) crate, accelerated by [`rayon`](https://crates.io/crates/rayon) for parallel data-level execution.
- **Binary Metadata:** [`img-parts`](https://crates.io/crates/img-parts) and [`kamadak-exif`](https://crates.io/crates/kamadak-exif) for zero-copy EXIF extraction and injection.

---

## 🚀 Installation & Building

Photopaper utilizes the standard `cargo` build system.

### Prerequisites
Ensure you have the Rust toolchain and the required GTK4 development libraries installed on your system.

**For Fedora/RHEL:**
```bash
sudo dnf install rust cargo gtk4-devel libadwaita-devel
```

**For Ubuntu/Debian:**
```bash
sudo apt install cargo libgtk-4-dev libadwaita-1-dev
```

### Compiling from Source
```bash
# Clone the repository
git clone https://github.com/RuriChoco/Photopaper.git
cd Photopaper

# Build and run the application
cargo run --release
```

---

## 📜 License & Credits

**Photopaper** is licensed under the [GNU General Public License v3.0 (GPL-3.0)](LICENSE). 

- **Author & Concept Design:** RuriChoco
- **Open Source Dependencies:** Massive thanks to the maintainers of `image`, `gtk4-rs`, `libadwaita`, `rayon`, `img-parts`, and `kamadak-exif` for making this application possible.
