use image::GenericImageView;
use ravif::{Encoder, RGBA8};
use imgref::Img;
use std::fs;
use std::path::Path;

use crate::ndl::debug::log;

pub fn to_avif(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(path)?;
    let (width, height) = img.dimensions();
    let pixels = img.to_rgba8();
    
    let data: Vec<RGBA8> = pixels
        .chunks_exact(4)
        .map(|p| RGBA8::new(p[0], p[1], p[2], p[3]))
        .collect();

    let img_ref = Img::new(data.as_slice(), width as usize, height as usize);

    let encoder = Encoder::new().with_quality(80.0);
    let encoded = encoder.encode_rgba(img_ref)?;

    let output_path = Path::new(path).with_extension("avif");
    
    fs::write(&output_path, encoded.avif_file)?;
    
    log::info(&format!("Image converted successfully: '{}'", output_path.display()));
    
    let remove_result = fs::remove_file(path);
    if let Err(e) = remove_result {
        log::warn(&format!("Failed to remove original image '{}': {}", path, e));
    }

    Ok(())
}