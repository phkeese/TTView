use clap::Parser;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader, Pixel as ImagePixel, Rgb};
use std::fmt::{Display, Formatter};

/// Single pixel value.
type Pixel = Rgb<f32>;

fn brightness(pixel: &Pixel) -> f32 {
    0.299 * pixel.channels()[0] + 0.587 * pixel.channels()[1] + 0.114 * pixel.channels()[2]
}

#[derive(Debug, Default, Copy, Clone, clap::ValueEnum)]
enum Filter {
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

/// Display style.
#[derive(Debug, Default, Clone, clap::ValueEnum)]
enum Style {
    /// Default style, 24 bit color with upper half block character.
    #[default]
    Default,

    /// Greyscale style, uses a weighted average for the final pixel value.
    Greyscale,

    /// Display in greyscale using a gradient.
    #[clap(skip)]
    Gradient(Vec<char>),
}

fn fg(color: &Pixel) -> String {
    format!(
        "\x1B[38;2;{};{};{}m",
        (color.channels()[0] * 255.0) as u8,
        (color.channels()[1] * 255.0) as u8,
        (color.channels()[2] * 255.0) as u8,
    )
}

fn bg(color: &Pixel) -> String {
    format!(
        "\x1B[48;2;{};{};{}m",
        (color.channels()[0] * 255.0) as u8,
        (color.channels()[1] * 255.0) as u8,
        (color.channels()[2] * 255.0) as u8,
    )
}

impl Style {
    fn apply(&self, top: &Pixel, bottom: Option<&Pixel>) -> String {
        let mut string = String::default();
        match self {
            Self::Default => {
                string += &fg(top);
                if let Some(lower) = bottom {
                    string += &bg(lower);
                }
                string += "▀\x1B[0m";
            }
            Self::Gradient(gradient) => {
                let mut avg = top.0;
                if let Some(bottom) = bottom {
                    for i in 0..3 {
                        avg[i] = (avg[i] + bottom.channels()[i]) / 2.0;
                    }
                }
                let avg = Pixel::from(avg);
                let b = brightness(&avg);
                let char_index = ((gradient.len() - 1) as f32 * b) as usize;
                string += &format!("{}\x1B[0m", gradient[char_index]);
            }
            Self::Greyscale => {
                let b = brightness(&top);
                string += &fg(&Pixel::from([b, b, b]));

                if let Some(lower) = bottom {
                    let b = brightness(lower);
                    string += &bg(&Pixel::from([b, b, b]));
                }
                string += "▀\x1B[0m";
            }
        }
        string
    }
}

#[derive(clap::Parser, Debug)]
struct Args {
    /// Files to display.
    filenames: Vec<String>,

    /// Optional display width.
    #[clap(short, long, default_value = "80")]
    width: u32,

    /// Optional filter to use for scaling.
    #[clap(short, long)]
    filter: Option<Filter>,

    /// Optional display style.
    #[clap(short, long, group = "display_style")]
    style: Option<Style>,

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
    let image = image.to_rgb32f();
    let mut string = String::default();
    for y in (0..image.height()).step_by(2) {
        for x in 0..image.width() {
            let top = image.get_pixel(x, y);
            let bottom = image.get_pixel_checked(x, y + 1);
            string += &style.apply(top, bottom);
        }
        string += "\n";
    }
    string
}

fn resize(image: DynamicImage, width: u32, filter: Filter) -> DynamicImage {
    let filter = match filter {
        Filter::Nearest => FilterType::Nearest,
        Filter::Triangle => FilterType::Triangle,
        Filter::CatmullRom => FilterType::CatmullRom,
        Filter::Gaussian => FilterType::Gaussian,
        Filter::Lanczos3 => FilterType::Lanczos3,
    };
    let (w, h) = image.dimensions();
    let scale = width as f64 / w as f64;
    let h = (scale * h as f64) as u32;
    image.resize(width, h, filter)
}

fn display_image(path: &str, width: u32, style: &Style, filter: Filter) -> Result<(), Error> {
    let reader = ImageReader::open(path).map_err(Error::IO)?;
    let image = reader.decode().map_err(Error::Decode)?;
    let image = resize(image, width, filter);
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
        args.style = Some(Style::Gradient(gradient));
    }
    let style = args.style.unwrap_or_default();
    let filter = args.filter.unwrap_or_default();
    for filename in &args.filenames {
        match display_image(filename, args.width, &style, filter) {
            Ok(_) => (),
            Err(err) => println!("{filename}: {err}"),
        }
    }
}
