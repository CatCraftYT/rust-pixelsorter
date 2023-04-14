#![allow(non_snake_case)]
use image::DynamicImage;
use rayon::prelude::*;

#[derive(Debug, PartialEq)]
pub enum SortMode {
    Red,
    Green,
    Blue,
    Hue,
    Saturation,
    Lightness
}

/*
impl std::fmt::Display for SortMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortMode::Red => { write!(f, "Red") }
            SortMode::Green => { write!(f, "Green") }
            SortMode::Blue => { write!(f, "Blue") }
            SortMode::Hue => { write!(f, "Hue") }
            SortMode::Saturation => { write!(f, "Saturation") }
            SortMode::Lightness => { write!(f, "Lightness") }
        }
    }
}
*/

pub struct PixelSorterSettings {
    pub vertical: bool,  // whether the image should be sorted horizontally or vertically
    pub threshold: std::ops::Range<u8>,
    pub showThresholds: bool,
    pub sortMode: SortMode
}

impl Default for PixelSorterSettings {
    fn default() -> Self {
        Self {
            vertical: false,
            threshold: std::ops::Range {start: 127, end: 223},
            showThresholds: false,
            sortMode: SortMode::Lightness
        }
    }
}

pub fn ThresholdImage(img: &mut DynamicImage, settings : &PixelSorterSettings) {
    let rgbImg = img.as_mut_rgba8().unwrap();
    rgbImg.pixels_mut().par_bridge().for_each(|pixel| {
        let px = if settings.threshold.contains(&GetPixelLuminance(&*pixel)) {255} else {0};
        *pixel = image::Rgba::from([px,px,px,pixel[3]]);
    })
}

pub fn SortImage(img: &mut DynamicImage, settings : &PixelSorterSettings) {
    if settings.vertical { *img = img.rotate90(); }
    let rgbImg = img.as_mut_rgba8().unwrap();
    let lines = rgbImg.rows_mut();
    
    lines.par_bridge().for_each(|line| {
        SortPixelsInLine(&mut line.collect(), settings);
    });

    if settings.vertical { *img = img.rotate270(); }
}

// this function creates multiple copies of each span. this could probably be improved if i wasn't a dumdum
fn SortPixelsInLine(line: &mut Vec<&mut image::Rgba<u8>>, settings : &PixelSorterSettings) {
    let mut currentSpan: Vec<usize> = Vec::new();
    let mut spans : Vec<Vec<usize>> = Vec::new();

    for pixel in 0..line.len() {
        if settings.threshold.contains(&GetPixelLuminance(&*line[pixel])) {
            currentSpan.push(pixel);
        }
        else if !currentSpan.is_empty() {
            spans.push(std::mem::take(&mut currentSpan));
        }
    }

    let mut transformations : Vec<(usize, usize)> = Vec::new();
    for span in spans.iter_mut() {
        let presortSpan = span.clone();
        // each span is a list of indices into line, not pixel values
        span.sort_unstable_by(|a, b| { GetPixelSortingNumber(&*line[*a], settings).cmp(&GetPixelSortingNumber(&*line[*b], settings)) });
        for i in 0..span.len() {
            transformations.push((presortSpan[i], span[i]));
        }
    }

    let mut newLine : Vec<image::Rgba<u8>> = line.iter().map(|pixel| { **pixel.clone() }).collect();
    for transformation in transformations.iter() {
        newLine[transformation.0] = line[transformation.1].clone();
    }

    for pixel in 0..newLine.len() {
        *line[pixel] = newLine[pixel];
    }
}

fn GetPixelLuminance(pixel: &image::Rgba<u8>) -> u8 {
    // don't consider alpha
    // cast to u32 to stop overflows
    return ((pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3) as u8;
}

/// Returns the number that a pixel should be sorted by.
fn GetPixelSortingNumber(pixel: &image::Rgba<u8>, settings : &PixelSorterSettings) -> u32 {
    match settings.sortMode {
        SortMode::Red => {
            return pixel[0] as u32;
        },
        SortMode::Green => {
            return pixel[1] as u32;
        },
        SortMode::Blue => {
            return pixel[2] as u32;
        },
        SortMode::Hue => {
            let hsl = rgba_to_hsl(pixel);
            return hsl.0;
        },
        SortMode::Saturation => {
            let hsl = rgba_to_hsl(pixel);
            return hsl.1;
        },
        _ => {
            let hsl = rgba_to_hsl(pixel);
            return hsl.2;
        }
    }
}

fn rgba_to_hsl(rgba: &image::Rgba<u8>) -> (u32, u32, u32) {
    // Normalize R, G, and B values to the range 0..1
    let r = rgba.0[0] as f32 / 255.0;
    let g = rgba.0[1] as f32 / 255.0;
    let b = rgba.0[2] as f32 / 255.0;

    // Find the maximum and minimum values of R, G, and B
    let cmax = r.max(g).max(b);
    let cmin = r.min(g).min(b);
    let delta = cmax - cmin;

    // Calculate H, S, and L
    let h = if delta == 0.0 {
        0.0
    } else if cmax == r {
        60.0 * ((g - b) / delta % 6.0)
    } else if cmax == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let l = 0.5 * (cmax + cmin);
    let s = if delta == 0.0 {
        0.0
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };

    // Return the values as a tuple
    (h as u32, (s * 100.0) as u32, (l * 100.0) as u32)
}


