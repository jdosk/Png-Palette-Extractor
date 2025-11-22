use anyhow::{Result, bail};
use std::{fs, fs::File, io::BufReader, path::PathBuf};
use png::{ColorType, Info};

// Returns selected file
fn open_file_dialog() -> Result<PathBuf> {
    let file = rfd::FileDialog::new() // file = some .png file
        .add_filter("PNG images", &["png"]) //filter for only png files
        .pick_file(); // open file picker

    // check if file is .png otherwise abort
    match file { 
        Some(path) => Ok(path), 
        None => bail!("No file selected"), 
    }
}

fn load_png(path: &PathBuf) -> Result<(png::Info, Vec<u8>)> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let decoder = png::Decoder::new(reader);
    let mut reader = decoder.read_info()?;

    // Grab the metadata including palette BEFORE reading pixels
    let info = reader.info().clone();

    // Allocate pixel buffer
    let buf_size = reader
        .output_buffer_size()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine PNG buffer size"))?;

    let mut buf = vec![0u8; buf_size];

    // Read pixel data
    reader.next_frame(&mut buf)?;

    Ok((info, buf))
}

// Checks if indexed
fn ensure_indexed(info: &png::Info) -> Result<()> {
    if info.color_type != ColorType::Indexed {
        bail!("PNG is not indexed.");
    }
    Ok(())
}

fn extract_palette(info: &png::Info) -> Result<Vec<(u8, u8, u8)>> {
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

fn save_pal(original_path: &PathBuf, palette: &[(u8, u8, u8)]) -> Result<()> {
    let mut out = String::new();
    out.push_str("JASC-PAL\n");
    out.push_str("0100\n");
    out.push_str(&format!("{}\n", palette.len()));

    for (r, g, b) in palette.iter() {
        out.push_str(&format!("{} {} {}\n", r, g, b));
    }

    // Build output filename: same path, but with .pal
    let mut new_path = original_path.clone();
    new_path.set_extension("pal");

    fs::write(&new_path, out)?;
    println!("Saved palette to {}", new_path.display());

    Ok(())
}

fn main() -> Result<()> {
    let path = open_file_dialog()?;
    println!("Selected: {}", path.display());

    let (info, _) = load_png(&path)?;
    ensure_indexed(&info)?;

    let palette = extract_palette(&info)?;
    println!("Extracted {} colors", palette.len());

    save_pal(&path, &palette)?;

    Ok(())
}