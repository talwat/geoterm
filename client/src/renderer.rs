use image::{DynamicImage, GenericImageView};

fn get_ansi(r: u8, g: u8, b: u8, bg: bool) -> String {
    return format!("\x1b[{};2;{};{};{}m", if bg { "48" } else { "38" }, r, g, b);
}

pub fn trim(img: &DynamicImage) -> DynamicImage {
    // Get the dimensions of the image
    let (width, height) = img.dimensions();

    // Find the cropping boundaries
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;

    for (x, y, pixel) in img.pixels() {
        if pixel[3] != 0 {
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }

            if y < min_y {
                min_y = y
            }
            if y > max_y {
                max_y = y;
            }
        }
    }

    // Calculate the cropping dimensions
    let crop_width = max_x - min_x + 1;
    let crop_height = max_y - min_y + 1;

    // Crop the image
    img.crop_imm(min_x, min_y, crop_width, crop_height)
}

// Max image size on standard sized terminal: 80x96
pub fn render(img: &DynamicImage) -> String {
    let mut out = String::new();

    let (width, height) = img.dimensions();

    // Get every other row
    for y in (0..height - 1).step_by(2) {
        for x in 0..width {
            // The first pixel.
            let pixel = img.get_pixel(x, y);

            // The next pixel under the first. It's done like this because pixels are printed
            // twice at a time; one on top, and one on the bottom. This is so we can make the sprite
            // as small as possible, leaving more room.
            let next = img.get_pixel(x, y + 1);

            // If it's the start of a row, and it isn't the first, print a newline.
            if x == 0 && y != 0 {
                out.push('\n')
            }

            // If both pixels are transparent, just print a space.
            if pixel[3] <= 0 && next[3] <= 0 {
                out.push(' ');
                continue;
            }

            if pixel[3] != 0 && next[3] != 0 {
                out.push_str(&get_ansi(pixel[0], pixel[1], pixel[2], true));
                out.push_str(&get_ansi(next[0], next[1], next[2], false));
            } else if pixel[3] == 0 {
                out.push_str(&get_ansi(next[0], next[1], next[2], false));
            } else if next[3] == 0 {
                out.push_str(&get_ansi(pixel[0], pixel[1], pixel[2], false));
            }

            if pixel[3] != 0 && next[3] == 0 {
                out.push_str("▀\x1b[0m");
            } else {
                out.push_str("▄\x1b[0m");
            }
        }
    }

    // If the height is odd, we have to also print the very bottom row in a special way.
    if height % 2 != 0 {
        out.push('\n');

        for x in 0..width {
            let pixel = img.get_pixel(x, height - 1);

            if pixel[3] <= 0 {
                out.push_str(" ");
                continue;
            }

            out.push_str(&get_ansi(pixel[0], pixel[1], pixel[2], false));
            out.push_str("▀\x1b[0m");
        }
    }

    return out;
}
