use crate::ndl::debug::log;
use crate::ndl::utils::image;
use regex::Regex;
use std::path::Path;

pub const PUBLIC_PATH: &str = "public";

fn convert_images() {
    log::info("Converting images...");
    let image_formats = ["png"]; //TODO: add more formats
    
    if let Err(e) = convert_images_recursive(PUBLIC_PATH, &image_formats) {
        log::error(&format!("Error during image conversion: {}", e));
    }
}

fn convert_images_recursive(dir: &str, image_formats: &[&str]) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            convert_images_recursive(path.to_str().unwrap_or(dir), image_formats)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if image_formats.contains(&ext.to_lowercase().as_str()) {
                    if let Some(path_str) = path.to_str() {
                        if let Err(e) = image::to_avif(path_str) {
                            log::error(&format!("Failed to convert image '{}': {}", path.display(), e));
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}


fn update_html_references() {
    log::info("Updating HTML image references...");
    if let Err(e) = update_html_recursive(PUBLIC_PATH) {
        log::error(&format!("Error updating HTML references: {}", e));
    }
}

fn update_html_recursive(dir: &str) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            update_html_recursive(path.to_str().unwrap_or(dir))?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("html") {
            if let Err(e) = update_image_refs_in_file(&path) {
                log::error(&format!("Failed to update '{}': {}", path.display(), e));
            }
        }
    }
    Ok(())
}

fn update_image_refs_in_file(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let mut updated = content.clone();
    let mut changes_made = false;
    
    let patterns = [
        (r#"src=["']([^"']+\.png)["']"#, "src"),
        (r#"href=["']([^"']+\.png)["']"#, "href"),
        (r#"url\(['"]*([^)'"]+\.png)['"]*\)"#, "url"),
    ];
    
    for (pattern, _attr_name) in patterns {
        let re = Regex::new(pattern)?;
        let matches: Vec<_> = re.captures_iter(&content).collect();
        
        for cap in matches {
            if let Some(img_match) = cap.get(1) {
                let img_path = img_match.as_str();
                
                if is_local_path(img_path) {
                    let resolved_path = resolve_image_path(file_path, img_path)?;
                    let avif_path = resolved_path.with_extension("avif");
                    
                    if avif_path.exists() {
                        let new_img_path = img_path.replace(".png", ".avif");
                        let full_match = cap.get(0).unwrap().as_str();
                        let new_match = full_match.replace(img_path, &new_img_path);
                        updated = updated.replace(full_match, &new_match);
                        changes_made = true;
                    }
                }
            }
        }
    }
    
    if changes_made {
        std::fs::write(file_path, updated)?;
        log::info(&format!("Updated references in: '{}'", file_path.display()));
    }
    
    Ok(())
}

fn is_local_path(path: &str) -> bool {
    if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("//") {
        return false;
    }
    
    if path.starts_with("data:") {
        return false;
    }
    
    if path.len() >= 3 {
        let chars: Vec<char> = path.chars().collect();
        if chars.len() >= 2 && chars[1] == ':' && chars[0].is_ascii_alphabetic() {
            return false;
        }
    }
    
    return true;
}

fn resolve_image_path(html_file: &Path, img_path: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let html_dir = html_file.parent()
        .ok_or("Could not determine HTML file directory")?;
    
    let resolved = if img_path.starts_with('/') {
        Path::new(PUBLIC_PATH).join(&img_path[1..])
    } else {
        html_dir.join(img_path)
    };
    
    Ok(resolved)
}

pub fn preprocess() {
    log::info("Preprocessing started.");
    convert_images();
    update_html_references(); 
    log::info("Preprocessing completed.");
}
