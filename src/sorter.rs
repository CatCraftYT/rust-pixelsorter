#![allow(non_snake_case)]
use image::DynamicImage;
use rayon::prelude::*;

pub struct Threshold {
    pub min: u8,
    pub max: u8
}

impl Threshold {
    fn contains(&self, value: &u8) -> bool {
        return *value >= self.min && *value <= self.max;
    }

    fn new(min: u8, max: u8) -> Self {
        return Self {
            min: min,
            max: max
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SortMode {
    Average,
    Red,
    Green,
    Blue,
    Hue,
    Saturation,
    Lightness
}

pub struct PixelSorterSettings {
    pub vertical: bool,  // whether the image should be sorted horizontally or vertically
    pub threshold: Threshold,
    pub showThresholds: bool,
    pub sortMode: SortMode,
    pub invert: bool
}

impl Default for PixelSorterSettings {
    fn default() -> Self {
        Self {
            vertical: false,
            threshold: Threshold::new(127, 223),
            showThresholds: false,
            sortMode: SortMode::Lightness,
            invert: false
        }
    }
}

pub fn ThresholdImage(img: &mut DynamicImage, settings : &PixelSorterSettings) {
    let rgbImg = img.as_mut_rgba8().unwrap();
    rgbImg.pixels_mut().par_bridge().for_each(|pixel| {
        let px = if settings.threshold.contains(&GetPixelAverage(&*pixel)) {255} else {0};
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
        if settings.threshold.contains(&GetPixelAverage(&*line[pixel])) {
            currentSpan.push(pixel);
        }
        else if !currentSpan.is_empty() {
            spans.push(std::mem::take(&mut currentSpan));
        }
    }
    if !currentSpan.is_empty() { spans.push(std::mem::take(&mut currentSpan)); }

    let mut transformations : Vec<(usize, usize)> = Vec::new();
    for span in spans.iter_mut() {
        let presortSpan = span.clone();
        // each span is a list of indices into line, not pixel values
        span.sort_unstable_by(|a, b| { GetPixelOrdering(&*line[*a], &*line[*b], settings) });
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

fn GetPixelOrdering(a: &image::Rgba<u8>, b: &image::Rgba<u8>, settings : &PixelSorterSettings) -> std::cmp::Ordering {
    let order = GetPixelSortingNumber(a, settings).partial_cmp(&GetPixelSortingNumber(b, settings)).unwrap_or(std::cmp::Ordering::Equal);
    if settings.invert { return order.reverse(); }
    return order;
}

/// Returns the number that a pixel should be sorted by.
fn GetPixelSortingNumber(pixel: &image::Rgba<u8>, settings : &PixelSorterSettings) -> f32 {
    match settings.sortMode {
        SortMode::Red => {
            return pixel[0] as f32;
        },
        SortMode::Green => {
            return pixel[1] as f32;
        },
        SortMode::Blue => {
            return pixel[2] as f32;
        },
        SortMode::Hue => {
            return GetPixelHue(pixel);
        },
        SortMode::Saturation => {
            return GetPixelSaturation(pixel);
        },
        SortMode::Lightness => {
            return GetPixelLightness(pixel);
        }
        _ => {
            return GetPixelAverage(pixel) as f32;
        }
    }
}

fn GetPixelAverage(pixel: &image::Rgba<u8>) -> u8 {
    // don't consider alpha
    // cast to u32 to stop overflows
    return ((pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3) as u8;
}

// adapted from https://en.wikipedia.org/wiki/HSL_and_HSV

fn GetPixelHue(pixel: &image::Rgba<u8>) -> f32 {
    let max = pixel[0].max(pixel[1]).max(pixel[2]) as f32 / 255.0;
    let min = pixel[0].min(pixel[1]).min(pixel[2]) as f32 / 255.0;

    if max == min { return 0.0; }

    let r = pixel[0] as f32 / 255.0;
    let g = pixel[1] as f32 / 255.0;
    let b = pixel[2] as f32 / 255.0;
    
    let mut h : f32 = 0.0;

    // red is max
    if max == r {
        h = ((g - b) / (max - min)).rem_euclid(6.0);
    }

    // green is max
    else if max == g {
        h = 2.0 + (b - r) / (max - min);
    }

    // blue is max
    else if max == b {
        h = 4.0 + (r - g) / (max - min);
    }

    return h * 60.0;
}

fn GetPixelSaturation(pixel: &image::Rgba<u8>) -> f32 {
    let max = pixel[0].max(pixel[1]).max(pixel[2]) as f32 / 255.0;
    let min = pixel[0].min(pixel[1]).min(pixel[2]) as f32 / 255.0;

    let lightness = GetPixelLightness(pixel) / 100.0;
    if lightness == 0.0 { return 0.0 }

    return (max - min) / (1.0 - (2.0 * lightness - 1.0).abs()) * 100.0;
}

fn GetPixelLightness(pixel: &image::Rgba<u8>) -> f32 {
    let min = pixel[0].max(pixel[1]).max(pixel[2]) as f32 / 255.0;
    let max = pixel[0].min(pixel[1]).min(pixel[2]) as f32 / 255.0;
    return (min + max) / 2.0 * 100.0;
}
