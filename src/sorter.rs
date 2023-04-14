#![allow(non_snake_case)]
use image::{DynamicImage, GenericImage, GenericImageView};
use rayon::prelude::*;

pub struct PixelSorterSettings {
    pub vertical: bool,  // whether the image should be sorted horizontally or vertically
    pub threshold: std::ops::Range<u8>,
    pub showThresholds: bool
}

impl Default for PixelSorterSettings {
    fn default() -> Self {
        Self {
            vertical: false,
            threshold: std::ops::Range {start: 127, end: 223},
            showThresholds: false
        }
    }
}

pub fn ThresholdImage(img: &mut DynamicImage, settings : &PixelSorterSettings) {
    let rgbImg = img.as_mut_rgb8().unwrap();
    rgbImg.pixels_mut().par_bridge().for_each(|pixel| {
        let px = if settings.threshold.contains(&GetPixelLuminance(&*pixel)) {255} else {0};
        *pixel = image::Rgb::from([px,px,px]);
    })
}

pub fn SortImage(img: &mut DynamicImage, settings : &PixelSorterSettings) {
    if settings.vertical { *img = img.rotate90(); }
    let rgbImg = img.as_mut_rgb8().unwrap();
    let lines = rgbImg.rows_mut();
    
    lines.par_bridge().for_each(|line| {
        SortPixelsInLine(&mut line.collect(), settings);
    });

    if settings.vertical { *img = img.rotate270(); }
}

// this function creates multiple copies of each span. this could probably be improved if i wasn't a dumdum
fn SortPixelsInLine(line: &mut Vec<&mut image::Rgb<u8>>, settings : &PixelSorterSettings) {
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
        // sort by pixel luminance. each span is a list of indices into line, not pixel values
        span.sort_unstable_by(|a, b| { GetPixelLuminance(&*line[*a]).cmp(&GetPixelLuminance(&*line[*b])) });
        for i in 0..span.len() {
            transformations.push((presortSpan[i], span[i]));
        }
    }

    let mut newLine : Vec<image::Rgb<u8>> = line.iter().map(|pixel| { **pixel.clone() }).collect();
    for transformation in transformations.iter() {
        newLine[transformation.0] = line[transformation.1].clone();
    }

    for pixel in 0..newLine.len() {
        *line[pixel] = newLine[pixel];
    }
}

fn GetPixelLuminance(pixel: &image::Rgb<u8>) -> u8 {
    // don't consider alpha
    // cast to u32 to stop overflows
    return ((pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3) as u8;
}
