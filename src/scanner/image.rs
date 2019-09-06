use std::path::Path;

use dssim::Dssim;
use futures::{Future, future::ok};
use image::GenericImageView;
use image::{imageops, FilterType};

use crate::{
    config::*,
    util
};

/// Check whether the given image is illegal.
pub fn is_illegal_image(image: &Path) -> impl Future<Item = bool, Error = ()> {
    eprintln!("Checking image...");

    // Load the images
    let base_image = image::open("./res/illegal/binance.jpg").expect("failed to open base");
    let image = image::open(image).expect("failed to open downloaded image");

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
        println!("Illegal image! (score: {})", score);
    } else {
        println!("Allowed image (score: {})", score);
    }

    ok(is_similar)
}
