use anyhow::{Result, bail};
use std::{fs, fs::File, io::BufReader, path::PathBuf};
use png::{ColorType, Info};
use rfd::FileDialog;
use eframe::egui;

fn main() -> Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "PNG to PAL Converter",
        options,
        Box::new(|_cc| Ok(Box::new(AppState::default()))),
    );

    Ok(())
}

// App State
struct AppState {
    input_path: Option<PathBuf>,
    status: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            input_path: None,
            status: "".to_owned(),
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PNG to PAL Converter");

            // Select file
            if ui.button("Select PNG File").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter("PNG", &["png"])
                    .pick_file()
                {
                    self.input_path = Some(path);
                    self.status = "".to_owned();
                }
            }

            if let Some(path) = &self.input_path {
                ui.label(format!("Selected: {}", path.display()));
            } else {
                ui.label("No file selected");
            }

            ui.add_space(10.0);

            // Convert button 
            if ui.button("Convert to .pal").clicked() {
                match &self.input_path {
                    Some(path) => {
                        self.status = match convert_png_to_pal(path) {
                            Ok(_) => "Conversion successful!".to_owned(),
                            Err(e) => format!("Error: {}", e),
                        };
                    }
                    None => {
                        self.status = "No file selected.".to_owned();
                    }
                }
            }

            ui.add_space(10.0);

            // Status area
            ui.label(&self.status);
        });
    }
}

// Conversion logic
fn convert_png_to_pal(path: &PathBuf) -> Result<()> {
    let (info, _) = load_png(path)?;
    ensure_indexed(&info)?;
    let palette = extract_palette(&info)?;
    save_pal(path, &palette)?;
    Ok(())
}

// Load PNG file
fn load_png(path: &PathBuf) -> Result<(Info, Vec<u8>)> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let decoder = png::Decoder::new(reader);
    let mut reader = decoder.read_info()?;
    let info = reader.info().clone();

    let buf_size = reader
        .output_buffer_size()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine PNG buffer size"))?;
    let mut buf = vec![0u8; buf_size];
    reader.next_frame(&mut buf)?;

    Ok((info, buf))
}

// Check if indexed
fn ensure_indexed(info: &Info) -> Result<()> {
    if info.color_type != ColorType::Indexed {
        bail!("PNG is not indexed.");
    }
    Ok(())
}

// Extract palette
fn extract_palette(info: &Info) -> Result<Vec<(u8, u8, u8)>> {
    let palette = match &info.palette {
        Some(p) => p,
        None => bail!("No palette found."),
    };
    if palette.len() % 3 != 0 {
        bail!("Invalid palette length.");
    }
    let mut out = Vec::new();
    for chunk in palette.chunks(3) {
        out.push((chunk[0], chunk[1], chunk[2]));
    }
    Ok(out)
}

// Save palette to PAL
fn save_pal(original_path: &PathBuf, palette: &[(u8, u8, u8)]) -> Result<()> {
    let mut out = String::new();
    out.push_str("JASC-PAL\n0100\n");
    out.push_str(&format!("{}\n", palette.len()));

    for (r, g, b) in palette.iter() {
        out.push_str(&format!("{} {} {}\n", r, g, b));
    }

    let mut new_path = original_path.clone();
    new_path.set_extension("pal");

    fs::write(&new_path, out)?;
    Ok(())
}
