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
    dim: (Option<u32>, Option<u32>),
    filter: Filter,
) -> DynamicImage {
    let filter = match filter {
        Filter::Nearest => FilterType::Nearest,
        Filter::Triangle => FilterType::Triangle,
        Filter::CatmullRom => FilterType::CatmullRom,
        Filter::Gaussian => FilterType::Gaussian,
        Filter::Lanczos3 => FilterType::Lanczos3,
    };
    let (img_width, img_height) = image.dimensions();
    match dim {
        (Some(width), None) => {
            let scale = (width as f32) / (img_width as f32);
            let height = (img_height as f32 * scale) as u32;
            image.resize(width, height, filter)
        }
        (None, Some(height)) => {
            let scale = (height as f32) / (img_height as f32);
            let width = (img_width as f32 * scale) as u32;
            image.resize(width, height, filter)
        }
        (Some(width), Some(height)) => image.resize_exact(width, height, filter),
        _ => unreachable!("impossible dimensions for resize!"),
    }
}
