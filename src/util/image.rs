use dssim::{ToRGBAPLU, RGBAPLU};
use image::{GenericImageView, Rgba};
use imgref::ImgVec;
use rgb::RGBA;

/// Convert the given generic image to an `ImgVec` used with DSSIM for image comparing.
pub fn to_imgvec(input: &impl GenericImageView<Pixel = Rgba<u8>>) -> ImgVec<RGBAPLU> {
    let pixels = input
        .pixels()
        .map(|(_x, _y, Rgba([r, g, b, a]))| RGBA::new(r, g, b, a))
        .collect::<Vec<_>>()
        .to_rgbaplu();
    ImgVec::new(pixels, input.width() as usize, input.height() as usize)
}
