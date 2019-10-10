#[cfg(feature = "ocr")]
use std::sync::Arc;

use dssim::{ToRGBAPLU, RGBAPLU};
use image::{GenericImageView, Rgba};
use imgref::ImgVec;
use rgb::RGBA;
#[cfg(feature = "ocr")]
use tempfile::TempPath;

/// Convert the given generic image to an `ImgVec` used with DSSIM for image comparing.
pub fn to_imgvec(input: &impl GenericImageView<Pixel = Rgba<u8>>) -> ImgVec<RGBAPLU> {
    let pixels = input
        .pixels()
        .map(|(_x, _y, Rgba([r, g, b, a]))| RGBA::new(r, g, b, a))
        .collect::<Vec<_>>()
        .to_rgbaplu();
    ImgVec::new(pixels, input.width() as usize, input.height() as usize)
}

/// Read text from image at given path.
#[cfg(feature = "ocr")]
pub async fn read_text(path: Arc<TempPath>) -> Result<String, ()> {
    // Run OCR to get text from image in threadpool
    tokio_executor::blocking::run(move || {
        // Get the path as a string
        let path = match path.to_str() {
            Some(path) => path,
            None => {
                error!("Failed to parse image path for OCR check");
                return Err(());
            }
        };

        // Construct OCR reader
        let mut lt = leptess::LepTess::new(None, "eng").unwrap();
        lt.set_image(path);

        // Read text from image
        lt.get_utf8_text().map_err(|err| {
            error!("Failed to OCR image: {}", err);
            ()
        })
    }).await
}
