use clap::Parser;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader, Pixel, Rgba};

#[derive(clap::Parser, Debug)]
struct Args {
    /// Path to image to display.
    filename: String,

    /// Optional display width.
    #[clap(short, long, default_value = "80")]
    width: u32,
}

#[derive(Debug)]
enum Error {
    IO(std::io::Error),
    Decode(image::ImageError),
}

fn build_display_string(image: &DynamicImage) -> String {
    let mut string = String::default();
    for y in (0..image.height()).step_by(2) {
        for x in 0..image.width() {
            let upper = image.get_pixel(x, y);
            string += &format!(
                "\x1B[38;2;{};{};{}m",
                upper.channels()[0],
                upper.channels()[1],
                upper.channels()[2]
            );

            if y + 1 < image.height() {
                let lower = image.get_pixel(x, y + 1);
                string += &format!(
                    "\x1B[48;2;{};{};{}m",
                    lower.channels()[0],
                    lower.channels()[1],
                    lower.channels()[2]
                );
            }
            string += "▀\x1B[0m";
        }
        string += "\n";
    }
    string
}

fn resize(image: DynamicImage, width: u32) -> DynamicImage {
    let (w, h) = image.dimensions();
    let scale = width as f64 / w as f64;
    let h = (scale * h as f64) as u32;
    image.resize(width, h, FilterType::Gaussian)
}

fn display_image(path: &str, width: u32) -> Result<(), Error> {
    let reader = ImageReader::open(path).map_err(Error::IO)?;
    let image = reader.decode().map_err(Error::Decode)?;
    let image = resize(image, width);
    let string = build_display_string(&image);
    println!("Your image:\n{}", string);
    Ok(())
}

fn main() {
    let args = Args::parse();
    match display_image(&args.filename, args.width) {
        Ok(_) => (),
        Err(err) => panic!("error: {err:?}"),
    }
}
