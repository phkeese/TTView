use clap::Parser;
use image::{DynamicImage, ImageReader, Rgb};
use std::fmt::{Display, Formatter};

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

    /// Optional display width.
    #[clap(short, long, default_value = "80")]
    width: u32,

    /// Optional display height.
    /// If specified, the image is stretched to that height.
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

fn display_image(
    path: &str,
    dimensions: (u32, Option<u32>),
    style: &Style,
    filter: Filter,
) -> Result<(), Error> {
    let reader = ImageReader::open(path).map_err(Error::IO)?;
    let image = reader.decode().map_err(Error::Decode)?;
    let image = resize(image, dimensions, filter);
    let string = build_display_string(&image, style);
    println!("{path}:\n{}", string);
    Ok(())
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
    let dim = (args.width, args.height);
    for filename in &args.filenames {
        match display_image(filename, dim, &style, filter) {
            Ok(_) => (),
            Err(err) => println!("{filename}: {err}"),
        }
    }
}
