use image::GenericImageView;
use ravif::{Encoder, RGBA8};
use imgref::Img;
use std::fs;
use std::path::Path;

use crate::ndl::debug::log;

pub fn to_avif(path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        return Err(format!("Input file does not exist: '{}'", path).into());
    }

    let img = image::open(path).map_err(|e| {
        format!("Failed to open image '{}': {}", path, e)
    })?;
    
    let (width, height) = img.dimensions();
    
    if width == 0 || height == 0 {
        return Err(format!("Invalid image dimensions: {}x{}", width, height).into());
    }
    
    let pixels = img.to_rgba8();
    
    let data: Vec<RGBA8> = pixels
        .chunks_exact(4)
        .map(|p| RGBA8::new(p[0], p[1], p[2], p[3]))
        .collect();

    let img_ref = Img::new(data.as_slice(), width as usize, height as usize);

    let encoder = Encoder::new().with_quality(80.0);
    let encoded = encoder.encode_rgba(img_ref).map_err(|e| {
        format!("Failed to encode image: {}", e)
    })?;

    let out = Path::new(output_path);
    if let Some(parent) = out.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!("Failed to create output directory '{}': {}", parent.display(), e)
            })?;
        }
    }

    fs::write(out, encoded.avif_file).map_err(|e| {
        format!("Failed to write output file '{}': {}", out.display(), e)
    })?;

    log::info(&format!("Image converted successfully: '{}'", out.display()));

    Ok(())
}
