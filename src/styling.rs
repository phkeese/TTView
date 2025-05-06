use crate::Pixel;
use image::Pixel as ImagePixel;
use image::Rgb32FImage;

/// Display style.
#[derive(Debug, Default, Clone, clap::ValueEnum)]
pub enum Style {
    /// Default style, 24 bit color with upper half block character.
    #[default]
    Color,

    /// Greyscale style, uses a weighted average for the final pixel value.
    Greyscale,

    /// Display in greyscale using a gradient.
    #[clap(skip)]
    Gradient(Vec<char>),

    /// Braille style, using a threshold of 0.5 brightness.
    Braille,

    /// Braille but with Floyd-Steinberg dithering.
    DitheredBraille,

    /// Dithered.
    Dithered,
}

impl Style {
    pub fn apply(&self, image: &mut Rgb32FImage) -> String {
        let mut string = String::default();
        match self {
            Self::Color => {
                for y in (0..image.height()).step_by(2) {
                    for x in 0..image.width() {
                        string += &fg(image.get_pixel(x, y));
                        if let Some(bot) = image.get_pixel_checked(x, y + 1) {
                            string += &bg(bot);
                        }
                        string += "▀\x1B[0m";
                    }
                    string += "\n";
                }
            }
            Self::Gradient(gradient) => {
                for y in (0..image.height()).step_by(2) {
                    for x in 0..image.width() {
                        let mut b = brightness(image.get_pixel(x, y));
                        if let Some(bot) = image.get_pixel_checked(x, y + 1) {
                            b = (b + brightness(bot)) / 2.0;
                        }
                        let char_index = ((gradient.len() - 1) as f32 * b) as usize;
                        string += &format!("{}\x1B[0m", gradient[char_index]);
                        string += "\x1B[0m";
                    }
                    string += "\n";
                }
            }
            Self::Greyscale => {
                for y in (0..image.height()).step_by(2) {
                    for x in 0..image.width() {
                        let b = brightness(image.get_pixel(x, y));
                        string += &fg(&Pixel::from([b, b, b]));
                        if let Some(bot) = image.get_pixel_checked(x, y + 1) {
                            let b = brightness(bot);
                            string += &bg(&Pixel::from([b, b, b]));
                        }
                        string += "▀\x1B[0m";
                    }
                    string += "\n";
                }
            }
            Self::DitheredBraille => {
                greyscale(image);
                floyd_steinberg(image);
                string += &Self::Braille.apply(image);
            }
            Self::Dithered => {
                greyscale(image);
                floyd_steinberg(image);
                string += &Self::Greyscale.apply(image);
            }
            Self::Braille => {
                for y in (0..image.height()).step_by(4) {
                    for x in (0..image.width()).step_by(2) {
                        // Coordinate offsets of the braille dots.
                        let offsets = [
                            (0, 0),
                            (0, 1),
                            (0, 2),
                            (1, 0),
                            (1, 1),
                            (1, 2),
                            (0, 3),
                            (1, 3),
                        ];
                        let mut byte = 0u8;
                        for index in 0..offsets.len() {
                            let (i, j) = offsets[index];
                            if let Some(pixel) = image.get_pixel_checked(x + i, y + j) {
                                let b = brightness(pixel);
                                let is_set = b < 0.5;
                                byte = if is_set { byte | (1 << index) } else { byte }
                            }
                        }
                        let char =
                            char::from_u32(0x2800 + byte as u32).expect("failed to encode braille");
                        string += &format!("{}", char);
                    }
                    string += "\n";
                }
            }
        }
        string
    }
}

fn brightness(pixel: &Pixel) -> f32 {
    0.299 * pixel.channels()[0] + 0.587 * pixel.channels()[1] + 0.114 * pixel.channels()[2]
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

fn greyscale(image: &mut Rgb32FImage) {
    for y in 0..image.height() {
        for x in 0..image.width() {
            let b = brightness(image.get_pixel(x, y));
            *image.get_pixel_mut(x, y) = Pixel::from([b; Pixel::CHANNEL_COUNT as usize]);
        }
    }
}

fn quantize(pixel: &mut Pixel) -> Pixel {
    let mut error = Pixel::from([0.0; Pixel::CHANNEL_COUNT as usize]);
    for i in 0..Pixel::CHANNEL_COUNT as usize {
        let value = pixel.channels()[i];
        let q = if value < 0.5 { 0.0 } else { 1.0 };
        pixel.channels_mut()[i] = q;
        error.channels_mut()[i] = value - q;
    }
    error
}

fn floyd_steinberg(image: &mut Rgb32FImage) {
    for y in 0..image.height() {
        for x in 0..image.width() {
            let old_pixel = image.get_pixel_mut(x, y);
            let error = quantize(old_pixel);
            let indices = [(1, 0, 7.0), (-1, 1, 3.0), (0, 1, 5.0), (1, 1, 1.0)];
            for (i, j, f) in indices {
                if x == 0 && i == -1 {
                    continue;
                }
                if let Some(pixel) = image.get_pixel_mut_checked((x as i32 + i) as u32, y + j) {
                    for c in 0..Pixel::CHANNEL_COUNT as usize {
                        pixel.channels_mut()[c] += error.channels()[c] * f / 16.0;
                    }
                }
            }
        }
    }
}
