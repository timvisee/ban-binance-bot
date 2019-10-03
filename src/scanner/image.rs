use std::fs;
use std::path::Path;
use std::sync::Arc;

use dssim::Dssim;
use futures::{future::ok, Future};
use image::GenericImageView;
use image::{imageops, FilterType};
use tempfile::TempPath;

use crate::{config::*, scanner, util};

/// Check whether the given image is illegal.
pub fn is_illegal_image(path: Arc<TempPath>) -> Box<dyn Future<Item = bool, Error = ()>> {
    println!("Checking image {:?}...", path);

    // Set up a return future
    let mut future: Box<dyn Future<Item = _, Error = _>> = Box::new(ok(false));

    // Check for illegal text in images
    #[cfg(feature = "ocr")]
    {
        let path = path.clone();
        future = Box::new(future.and_then(
            move |illegal| -> Box<dyn Future<Item = _, Error = _>> {
                if !illegal {
                    Box::new(has_illegal_text(path))
                } else {
                    Box::new(ok(illegal))
                }
            },
        ));
    }

    // TODO: make the following async as well

    // Create a directory reader
    let read_dir = match fs::read_dir(ILLEGAL_IMAGES_DIR) {
        Ok(read_dir) => read_dir,
        Err(err) => {
            println!(
                "failed create directory reader for illegal image templates: {}",
                err
            );
            return Box::new(ok(false));
        }
    };

    // Check if image is illegal by testing against all illegal templates
    let illegal = read_dir
        .filter_map(|template_path| {
            template_path
                .or_else(|err| {
                    println!("failed to read illegal image template: {}", err);
                    Err(())
                })
                .ok()
        })
        .any(|template_path| match_image(&path, &template_path.path()));
    if illegal {
        future = Box::new(ok(true));
    }

    future
}

/// Check whether the images contains any illegal text, with an OCR check.
#[cfg(feature = "ocr")]
fn has_illegal_text(path: Arc<TempPath>) -> Box<dyn Future<Item = bool, Error = ()>> {
    // Get the path as a string
    let path = match path.to_str() {
        Some(path) => path,
        None => {
            println!("failed to obtain image path as text for OCR check, ignoring...");
            return Box::new(ok(false));
        }
    };

    // Construct OCR reader
    let mut lt = leptess::LepTess::new(None, "eng").unwrap();
    lt.set_image(path);

    // Read text from image
    println!("Scanning for illegal text in image with OCR...");
    let text = match lt.get_utf8_text() {
        Ok(text) => text,
        Err(err) => {
            println!("failed to OCR: {}", err);
            return Box::new(ok(false));
        }
    };

    // Trim and lowercase
    let text = text.trim().to_lowercase();

    // Match the URL against a list of banned host parts
    let illegal = ILLEGAL_IMAGE_TEXT
        .iter()
        .any(|illegal_text| text.contains(&illegal_text.to_lowercase()));
    if illegal {
        println!("Found illegal text in image!");
        return Box::new(ok(true));
    }

    // Scan for generic illegal text as well, return the result
    Box::new(scanner::text::is_illegal_text(text))
}

/// Check whether the images at the given two paths match.
fn match_image(path: &Path, template_path: &Path) -> bool {
    print!(
        "Matching illegal template '{}'...",
        template_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
    );

    // Load the images
    let template_image = image::open(template_path).expect("failed to open base");
    let image = match image::open(path) {
        Ok(image) => image,
        Err(err) => {
            eprintln!("failed to open downloaded image, ignoring: {}", err);
            return false;
        }
    };

    // Make the image we're testing the same size
    let (x, y) = template_image.dimensions();
    let image = imageops::resize(&image, x, y, FilterType::Triangle);

    // Create a DSSIM instance
    let mut dssim = Dssim::new();

    let template_image = util::image::to_imgvec(&template_image);
    let template_image = dssim
        .create_image(&template_image)
        .expect("failed to load base image");

    let image = util::image::to_imgvec(&image);
    let image = dssim.create_image(&image).expect("failed to load image");

    // Compare the images, obtain the score
    let result = dssim.compare(&template_image, image);
    let score = result.0;
    let is_similar = score <= IMAGE_BAN_THRESHOLD;

    if is_similar {
        println!(" Illegal! (score: {})", score);
    } else {
        println!(" (score: {})", score);
    }

    is_similar
}
