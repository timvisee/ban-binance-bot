use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use dssim::Dssim;
use futures::{future, prelude::*};
use image::GenericImageView;
use image::{imageops, FilterType};
use tempfile::TempPath;

use crate::{
    config::*,
    util::{self, future::select_true},
};
#[cfg(feature = "ocr")]
use crate::scanner;

/// Check whether the given image is illegal.
pub async fn is_illegal_image(path: Arc<TempPath>) -> bool {
    println!("Checking image {:?}...", path);

    let mut checks: Vec<Pin<Box<dyn Future<Output = bool> + Send>>> = vec![];

    // Compare images against database of banned images
    if AUDIT_IMAGE_COMPARE {
        // TODO: this seems to leak memory when used a lot, investigate and fix
        checks.push(matches_illegal_template(path.clone()).boxed());
    }

    // Check for illegal text in images
    #[cfg(feature = "ocr")]
    checks.push(has_illegal_text(path.clone()).boxed());

    // Run checks
    select_true(checks).await
}

/// Check whether the images contains any illegal text, with an OCR check.
#[cfg(feature = "ocr")]
async fn has_illegal_text(path: Arc<TempPath>) -> bool {
    // Read text from image
    let text = match util::image::read_text(path).await {
        Ok(text) => text,
        Err(_) => return false,
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

    // Scan for generic illegal text as well, return the result
    scanner::text::is_illegal_text(text).await
}

/// Check whether an image matches an illegal image template.
///
/// This checks whether the image at the given path matches any of the images in the illegal image
/// templates directory.
///
/// True is returned if the image is illegal, false if not.
/// On error, false is returned as it is assumed the image is allowed.
async fn matches_illegal_template(path: Arc<TempPath>) -> bool {
    // Create a directory reader to list all image templates
    let read_dir = match tokio::fs::read_dir(ILLEGAL_IMAGES_DIR).await {
        Ok(read_dir) => read_dir,
        Err(err) => {
            println!(
                "failed create directory reader for illegal image templates: {}",
                err
            );
            return false;
        }
    };

    // Test image for matches with templates, return on first match
    read_dir
            .filter_map(|template_path| {
                future::ready(
                    template_path
                        .or_else(|err| {
                            println!("failed to read illegal image template: {}", err);
                            Err(())
                        })
                        .ok(),
                )
            })
            .map(|template_path| {
                let path = path.clone();
                tokio_executor::blocking::run(move || match_image(path, template_path.path())).boxed()
            })
            .buffer_unordered(*IMAGE_CONCURRENT_MATCHES)
            .filter(|illegal| future::ready(*illegal))
            .next()
            .await
            .is_some()
}

/// Check whether the images at the given two paths match.
///
/// This operation is expensive.
fn match_image(path: Arc<TempPath>, template_path: PathBuf) -> bool {
    println!(
        "Matching illegal template '{}'...",
        template_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
    );

    // Load the user image, return if it's too small
    let image = match image::open(path.as_ref()) {
        Ok(image) => image,
        Err(err) => {
            println!("Failed to open downloaded image, ignoring: {}", err);
            return false;
        }
    };
    let (x, y) = image.dimensions();
    if x < IMAGE_MIN_SIZE || y < IMAGE_MIN_SIZE {
        println!("Image too small to match against banned templates, ignoring");
        return false;
    }

    // Load the template image
    let template_image = image::open(template_path).expect("failed to open base");

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
        println!("Matched image is illegal! (score: {})", score);
    } else {
        println!("Matched image is legal (score: {})", score);
    }

    is_similar
}
