use clap::Parser;
use image::{DynamicImage, ImageReader, Rgb};
use std::fmt::{Display, Formatter};
use std::path::Path;

mod resizing;
mod styling;

use resizing::*;
use styling::*;

/// Single pixel value.
type Pixel = Rgb<f32>;

#[derive(clap::Parser, Debug)]
struct Args {
    /// Files to display.
    filenames: Vec<String>,

    /// Optional width to scale the image to before displaying it.
    /// When height is also given, aspect ratio is not preserved.
    /// When neither are given, a default width of 80 is used.
    #[clap(short, long)]
    width: Option<u32>,

    /// Optional height to scale the image to before displaying it.
    /// When width is also given, aspect ratio is not preserved.
    #[clap(short = 'y', long)]
    height: Option<u32>,

    /// Optional filter to use for scaling.
    #[clap(short, long)]
    filter: Option<Filter>,

    /// Optional display style.
    #[clap(short, long, group = "display_style", default_value = "color")]
    style: Style,

    /// Gradient string to use.
    /// First is darkest, right is the lightest color.
    /// Cannot be combined with another style.
    #[clap(short, long, group = "display_style")]
    gradient: Option<String>,

    /// Print version info.
    #[clap(short, long)]
    version: bool,
}

#[derive(Debug)]
enum Error {
    IO(std::io::Error),
    Decode(image::ImageError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(err) => write!(f, "{err}"),
            Self::Decode(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

fn build_display_string(image: &DynamicImage, style: &Style) -> String {
    let mut image = image.to_rgb32f();
    style.apply(&mut image)
}

fn load_image(path: impl AsRef<Path>) -> Result<DynamicImage, Error> {
    ImageReader::open(path)
        .map_err(Error::IO)?
        .decode()
        .map_err(Error::Decode)
}

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() {
    let mut args = Args::parse();
    if args.version {
        let version = built_info::PKG_VERSION;
        println!("ttview {}", version);
    }
    if let Some(gradient) = args.gradient {
        let gradient = gradient.chars().collect();
        args.style = Style::Gradient(gradient);
    }
    let style = args.style;
    let filter = args.filter.unwrap_or_default();
    let dim = match (args.width, args.height) {
        (None, None) => (Some(80), None),
        other => other,
    };
    for filename in &args.filenames {
        let image = match load_image(filename) {
            Ok(img) => img,
            Err(err) => {
                println!("{filename}: {err}");
                continue;
            }
        };
        let image = resize(image, dim, filter);
        println!("{filename}:\n{}", build_display_string(&image, &style));
    }
}
