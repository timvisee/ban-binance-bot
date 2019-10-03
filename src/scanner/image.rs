use std::fs;
use std::path::Path;

use dssim::Dssim;
use futures::{future::ok, Future};
use image::GenericImageView;
use image::{imageops, FilterType};

use crate::{config::*, util};

/// Check whether the given image is illegal.
// TODO: make this async!
pub fn is_illegal_image(image: &Path) -> impl Future<Item = bool, Error = ()> {
    println!("Checking image {:?}...", image);

    // Check for illegal text in images
    #[cfg(feature = "ocr")]
    {
        if has_illegal_text(image) {
            return ok(true);
        }
    }

    // Create a directory reader
    let read_dir = match fs::read_dir(ILLEGAL_IMAGES_DIR) {
        Ok(read_dir) => read_dir,
        Err(err) => {
            println!(
                "failed create directory reader for illegal image templates: {}",
                err
            );
            return ok(false);
        }
    };

    // Check if image is illegal by testing against all illegal templates
    let illegal = read_dir
        .filter_map(|path| {
            path.or_else(|err| {
                println!("failed to read illegal image template: {}", err);
                Err(())
            })
            .ok()
        })
        .any(|path| match_image(image, &path.path()));

    ok(illegal)
}

/// Check whether the images contains any illegal text, with an OCR check.
#[cfg(feature = "ocr")]
fn has_illegal_text(image: &Path) -> bool {
    // Get the path as a string
    let path = match image.to_str() {
        Some(path) => path,
        None => {
            println!("failed to obtain image path as text for OCR check, ignoring...");
            return false;
        }
    };

    // Construct OCR reader
    let mut lt = leptess::LepTess::new(None, "eng").unwrap();
    lt.set_image(path);

    // Read text from image
    println!("Reading text from image with OCR...");
    let text = match lt.get_utf8_text() {
        Ok(text) => text,
        Err(err) => {
            println!("failed to OCR: {}", err);
            return false;
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
        return true;
    }

    false
}

/// Check whether the images at the given two paths match.
fn match_image(image: &Path, other: &Path) -> bool {
    print!(
        "Matching illegal template '{}'...",
        other
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
    );

    // Load the images
    let base_image = image::open(other).expect("failed to open base");
    let image = match image::open(image) {
        Ok(image) => image,
        Err(err) => {
            eprintln!("failed to open downloaded image, ignoring: {}", err);
            return false;
        }
    };

    // Make the image we're testing the same size
    let (x, y) = base_image.dimensions();
    let image = imageops::resize(&image, x, y, FilterType::Triangle);

    // Create a DSSIM instance
    let mut dssim = Dssim::new();

    let base_image = util::image::to_imgvec(&base_image);
    let base_image = dssim
        .create_image(&base_image)
        .expect("failed to load base image");

    let image = util::image::to_imgvec(&image);
    let image = dssim.create_image(&image).expect("failed to load image");

    // Compare the images, obtain the score
    let result = dssim.compare(&base_image, image);
    let score = result.0;
    let is_similar = score <= IMAGE_BAN_THRESHOLD;

    if is_similar {
        println!(" Illegal! (score: {})", score);
    } else {
        println!(" (score: {})", score);
    }

    is_similar
}
