use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};

#[derive(Debug, Default, Copy, Clone, clap::ValueEnum)]
pub enum Filter {
    /// Nearest Neighbor
    Nearest,

    /// Linear Filter
    Triangle,

    /// Cubic Filter
    CatmullRom,

    /// Gaussian Filter
    #[default]
    Gaussian,

    /// Lanczos with window 3
    Lanczos3,
}

pub fn resize(
    image: DynamicImage,
    (width, height): (u32, Option<u32>),
    filter: Filter,
) -> DynamicImage {
    let filter = match filter {
        Filter::Nearest => FilterType::Nearest,
        Filter::Triangle => FilterType::Triangle,
        Filter::CatmullRom => FilterType::CatmullRom,
        Filter::Gaussian => FilterType::Gaussian,
        Filter::Lanczos3 => FilterType::Lanczos3,
    };
    let (w, h) = image.dimensions();
    let scale = width as f64 / w as f64;
    let height = height.unwrap_or((scale * h as f64) as u32);
    image.resize_exact(width, height, filter)
}
