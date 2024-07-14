use clap::{Arg, Command};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage, ImageOutputFormat, Pixel};
use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write, Cursor};

const MAXC: u32 = (1 << 16) - 1;

pub struct Glitch {
    input: DynamicImage,
    output: RgbaImage,
    width: u32,
    height: u32,
    filetype: String,
}

impl Glitch {
    pub fn new(filename: &str) -> Result<Self, io::Error> {
        let file = File::open(filename)?;
        let mut buf_reader = BufReader::new(file);
        let mut buffer = Vec::new();
        buf_reader.read_to_end(&mut buffer)?;
        let image = image::load_from_memory(&buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        let (width, height) = image.dimensions();
        let output = ImageBuffer::new(width, height);
        Ok(Glitch {
            input: image,
            output,
            width,
            height,
            filetype: "png".to_string(),
        })
    }

    pub fn set_bounds(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn seed(&self, seed: u64) {
        let _rng = StdRng::seed_from_u64(seed);
    }

    pub fn write<W: Write>(&self, out: &mut W) -> Result<(), io::Error> {
        if self.filetype == "png" {
            DynamicImage::ImageRgba8(self.output.clone()).write_to(out, ImageOutputFormat::Png)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
        } else {
            DynamicImage::ImageRgba8(self.output.clone()).write_to(out, ImageOutputFormat::Jpeg(80))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
        }
    }

    pub fn copy(&mut self) {
        self.output = self.input.to_rgba8();
    }

    pub fn transpose_input(&mut self, height: u32, width: u32, mut transpose: bool) {
        let (bounds_width, bounds_height) = (self.width, self.height);
        let mut cursor = 0;

        while cursor < bounds_height {
            if transpose {
                let next = cursor + height;
                if next > bounds_height {
                    return;
                }
                for y in cursor..next {
                    for x in 0..bounds_width {
                        let tx = (x + width) % bounds_width;
                        let color = self.input.get_pixel(tx, y);
                        self.output.put_pixel(x, y, color);
                    }
                }
                cursor = next;
            } else {
                cursor += height;
            }
            transpose = !transpose;
        }
    }

    pub fn vertical_transpose_input(&mut self, width: u32, height: u32, mut transpose: bool) {
        let (bounds_width, bounds_height) = (self.width, self.height);
        let mut cursor = 0;

        while cursor < bounds_width {
            if transpose {
                let next = cursor + width;
                if next > bounds_width {
                    return;
                }
                for x in cursor..next {
                    for y in 0..bounds_height {
                        let ty = (y + height) % bounds_height;
                        let color = self.input.get_pixel(x, ty);
                        self.output.put_pixel(x, y, color);
                    }
                }
                cursor = next;
            } else {
                cursor += width;
            }
            transpose = !transpose;
        }
    }

    pub fn channel_shift_left(&mut self) {
        let (bounds_width, bounds_height) = (self.width, self.height);

        for y in 0..bounds_height {
            for x in 0..bounds_width {
                let pixel = self.output.get_pixel(x, y);
                let [r, g, b, a] = pixel.0;
                self.output.put_pixel(x, y, Rgba([g, b, r, a]));
            }
        }
    }

    pub fn channel_shift_right(&mut self) {
        let (bounds_width, bounds_height) = (self.width, self.height);

        for y in 0..bounds_height {
            for x in 0..bounds_width {
                let pixel = self.output.get_pixel(x, y);
                let [r, g, b, a] = pixel.0;
                self.output.put_pixel(x, y, Rgba([b, r, g, a]));
            }
        }
    }

    pub fn half_life_right(&mut self, strikes: u32, length: i32) {
        let mut rng = rand::thread_rng();
        let (bounds_width, bounds_height) = (self.width, self.height);

        for _ in 0..strikes {
            let mut x = rng.gen_range(0..bounds_width);
            let y = rng.gen_range(0..bounds_height);
            let mut kc = self.output.get_pixel(x, y).clone();

            let streak_end = if length < 0 {
                bounds_width
            } else {
                min_int(x + length as u32, bounds_width)
            };

            while x < streak_end {
                let [r1, g1, b1, a1] = kc.0;
                let [r2, g2, b2, a2] = self.output.get_pixel(x, y).0;

                kc = Rgba([
                    c(u32::from(r1) * 3 / 4 + u32::from(r2) / 4),
                    c(u32::from(g1) * 3 / 4 + u32::from(g2) / 4),
                    c(u32::from(b1) * 3 / 4 + u32::from(b2) / 4),
                    c(u32::from(a1) * 3 / 4 + u32::from(a2) / 4),
                ]);

                self.output.put_pixel(x, y, kc);
                x += 1;
            }
        }
    }

    pub fn prism_burst(&mut self) {
        let mut rng = rand::thread_rng();
        let (bounds_width, bounds_height) = (self.width, self.height);
        let offset = rng.gen_range(1..=bounds_height / 10);
        let alpha = rng.gen_range(0..=MAXC);

        let mut out = Rgba([0, 0, 0, 0]);
        for y in 0..bounds_height {
            for x in 0..bounds_width {
                let (sr, sg, sb, sa) = self.output.get_pixel(x, y).0.into();

                let dr = self.output.get_pixel((x + offset).min(bounds_width - 1), (y + offset).min(bounds_height - 1)).0[0];
                let dg = self.output.get_pixel(x.saturating_sub(offset), (y + offset).min(bounds_height - 1)).0[1];
                let db = self.output.get_pixel((x + offset).min(bounds_width - 1), y.saturating_sub(offset)).0[2];
                let da = self.output.get_pixel(x.saturating_sub(offset), y.saturating_sub(offset)).0[3];

                let a = MAXC - (sa as u32 * alpha / MAXC);

                out[0] = c((dr as u32 * a + sr as u32 * alpha) / MAXC);
                out[1] = c((dg as u32 * a + sg as u32 * alpha) / MAXC);
                out[2] = c((db as u32 * a + sb as u32 * alpha) / MAXC);
                out[3] = c((da as u32 * a + sa as u32 * alpha) / MAXC);

                self.output.put_pixel(x, y, out);
            }
        }
    }

    pub fn noise(&mut self, r: f64, g: f64, b: f64, a: f64) {
        let mut rng = rand::thread_rng();
        let (bounds_width, bounds_height) = (self.width, self.height);

        for y in 0..bounds_height {
            for x in 0..bounds_width {
                let mut pixel = self.output.get_pixel(x, y).to_rgba();
                
                let random_r = rng.gen::<f64>() * r;
                let random_g = rng.gen::<f64>() * g;
                let random_b = rng.gen::<f64>() * b;
                
                let blend = rng.gen::<f64>() * a;
                
                pixel[0] = blend_channel(pixel[0], random_r, blend);
                pixel[1] = blend_channel(pixel[1], random_g, blend);
                pixel[2] = blend_channel(pixel[2], random_b, blend);

                self.output.put_pixel(x, y, pixel);
            }
        }
    }

    pub fn compression_ghost(&mut self) {
        let mut rng = rand::thread_rng();
        let quality = rng.gen_range(1..=10);
        let alpha = rng.gen_range(0..=255) as u8;

        let mut buffer = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(self.output.clone())
            .write_to(&mut buffer, ImageOutputFormat::Jpeg(quality))
            .expect("Failed to compress image");

        let compressed = image::load_from_memory(&buffer.into_inner())
            .expect("Failed to load compressed image");

        let mut overlay = ImageBuffer::from_pixel(self.width, self.height, Rgba([255, 255, 255, alpha]));

        for (x, y, pixel) in overlay.enumerate_pixels_mut() {
            let compressed_pixel = compressed.get_pixel(x, y);
            pixel.blend(&compressed_pixel);
        }

        for (x, y, pixel) in self.output.enumerate_pixels_mut() {
            let overlay_pixel = overlay.get_pixel(x, y);
            pixel.blend(&overlay_pixel);
        }
    }

}

// Utils
fn c(a: u32) -> u8 {
    ((a as f64 / 255.0) * 255.0) as u8
}

fn min_int(a: u32, b: u32) -> u32 {
    if a < b {
        a
    } else {
        b
    }
}

fn blend_channel(original: u8, random: f64, blend: f64) -> u8 {
    let blended = (original as f64 * (1.0 - blend) + random * 255.0 * blend).min(255.0).max(0.0);
    blended as u8
}

fn main() -> Result<(), io::Error> {
    let matches = Command::new("Image Glitcher")
        .version("1.0")
        .about("Applies glitch effects to images")
        .arg(Arg::new("file")
            .short('f')
            .long("file")
            .value_name("FILE")
            .help("Sets the input image file")
            .required(true))
        .arg(Arg::new("effect")
            .short('e')
            .long("effect")
            .value_name("EFFECT")
            .help("Specifies the glitch effect to apply")
            .required(true))
        .get_matches();

    let filename = matches.get_one::<String>("file").unwrap();
    let effect = matches.get_one::<String>("effect").unwrap();

    let mut glitch = Glitch::new(filename)?;
    glitch.copy();

    let mut valid_effect = true;

    match effect.as_str() {
        "copy" => (),
        "transpose_input" => glitch.transpose_input(50, 100, true),
        "vertical_transpose_input" => glitch.vertical_transpose_input(30, 80, true),
        "channel_shift_left" => glitch.channel_shift_left(),
        "channel_shift_right" => glitch.channel_shift_right(),
        "half_life_right" => glitch.half_life_right(1000, 1000),
        "prism_burst" => glitch.prism_burst(),
        "noise" => glitch.noise(0.75, 0.75, 0.75, 0.2),
        "compression_ghost" => glitch.compression_ghost(),
        "all" => {
            glitch.transpose_input(50, 100, true);
            glitch.vertical_transpose_input(30, 80, true);
            glitch.channel_shift_left();
            glitch.channel_shift_right();
            glitch.half_life_right(1000, 1000);
            glitch.prism_burst();
            glitch.noise(0.75, 0.75, 0.75, 0.2);
            glitch.compression_ghost();
        }
        _ => {
            eprintln!("Unknown effect: {}", effect);
            valid_effect = false;
        }
    }

    if valid_effect {
        let output_filename = format!("./pngs/output_{}.png", effect);
        let output_file = File::create(output_filename)?;
        let mut writer = BufWriter::new(output_file);
        glitch.write(&mut writer)?;
    } else {
        eprintln!("No image saved due to invalid effect.");
    }

    Ok(())
}
