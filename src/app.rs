#![allow(non_snake_case)]
use image;
use eframe;
use eframe::egui;
use native_dialog::FileDialog;
use image::io::Reader as ImageReader;

pub struct PixelSorter {
    displayedImage: Option<egui::TextureHandle>,
    processedImage: Option<image::DynamicImage>,
    originalImageBytes: Option<image::DynamicImage>,
    settings: crate::PixelSorterSettings
}

impl Default for PixelSorter {
    fn default() -> Self {
        Self {
            originalImageBytes: None,
            processedImage: None,
            displayedImage: None,
            settings: crate::PixelSorterSettings::default()
        }
    }
}

impl PixelSorter {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn UpdateImage(&mut self, ctx: &egui::Context) {
        let img = self.processedImage.as_ref().unwrap();
        let imgWidth = img.width();
        let imgHeight = img.height();
        self.displayedImage = Some(ctx.load_texture(
            "displayed_image",
            egui::ColorImage::from_rgba_unmultiplied([imgWidth as usize, imgHeight as usize],
                img.to_rgba8().as_flat_samples().as_slice()
            ),
            egui::TextureOptions::default()
        ));
    }
}

impl eframe::App for PixelSorter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.processedImage.is_some() && self.displayedImage.is_none() { self.UpdateImage(ctx); }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Open").clicked() {
                    let img = OpenImgFileWithDialog();
                    if img.is_some() {
                        self.originalImageBytes = img;
                        self.processedImage = self.originalImageBytes.clone();
                        self.UpdateImage(ctx);
                    }
                }

                if ui.button("Save").clicked() && self.processedImage.is_some() {
                    let path = GetSavePath();
                    if path.is_some() {
                        self.processedImage.as_ref().unwrap().save(path.unwrap());
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("settings_panel").resizable(false).show(ctx, |ui| {
            ui.add_enabled_ui(self.originalImageBytes.is_some(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Sorting direction");
                    ui.separator();
                    ui.radio_value(&mut self.settings.vertical, false, "Horizontal");
                    ui.radio_value(&mut self.settings.vertical, true, "Vertical");
                });

                ui.horizontal(|ui| {
                    ui.label("Sorting mode");
                    ui.add_space(16.5);
                    ui.separator();
                    // no label so that we can add custom label on the left
                    egui::ComboBox::from_id_source("Sorting mode")
                    .selected_text(format!("{:?}", self.settings.sortMode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.settings.sortMode, crate::sorter::SortMode::Average, "Average");
                        ui.selectable_value(&mut self.settings.sortMode, crate::sorter::SortMode::Red, "Red");
                        ui.selectable_value(&mut self.settings.sortMode, crate::sorter::SortMode::Green, "Green");
                        ui.selectable_value(&mut self.settings.sortMode, crate::sorter::SortMode::Blue, "Blue");
                        ui.selectable_value(&mut self.settings.sortMode, crate::sorter::SortMode::Hue, "Hue");
                        ui.selectable_value(&mut self.settings.sortMode, crate::sorter::SortMode::Saturation, "Saturation");
                        ui.selectable_value(&mut self.settings.sortMode, crate::sorter::SortMode::Lightness, "Lightness");
                    });
                });

                ui.checkbox(&mut self.settings.invert, "Invert sorting order").on_hover_text("If this is unchecked, pixels will be sorted darkest to lightest. If it is checked, they will be sorted lightest to darkest.");

                ui.separator();
                
                ui.add(egui::Slider::new(&mut self.settings.threshold.max, self.settings.threshold.min..=255).clamp_to_range(true).text("Threshold max"));
                ui.add(egui::Slider::new(&mut self.settings.threshold.min, 0..=self.settings.threshold.max).clamp_to_range(true).text("Threshold min"));

                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.settings.showThresholds, "Show thresholds").on_hover_text("Show the thresholds for sorting. Rows of white pixels will be sorted, black pixels will not be affected.").changed() {
                        self.processedImage = self.originalImageBytes.clone();
                        if self.settings.showThresholds {
                            crate::sorter::ThresholdImage(self.processedImage.as_mut().unwrap(), &self.settings);
    
                        }
                        self.UpdateImage(ctx);
                    };
                    ui.separator();
                    if ui.button("Update").clicked() && self.settings.showThresholds {
                        self.processedImage = self.originalImageBytes.clone();
                        crate::sorter::ThresholdImage(self.processedImage.as_mut().unwrap(), &self.settings);
                        self.UpdateImage(ctx);
                    }
                });

                ui.separator();

                let sortButton = egui::Button::new("Sort!");
                if ui.add_sized(egui::vec2(ui.available_width(), 10.0), sortButton).clicked() {
                    self.settings.showThresholds = false;
                    self.processedImage = self.originalImageBytes.clone();
                    crate::sorter::SortImage(self.processedImage.as_mut().unwrap(), &self.settings);
                    self.UpdateImage(ctx);
                }

                let resetButton = egui::Button::new("Reset");
                if ui.add_sized(egui::vec2(ui.available_width(), 10.0), resetButton).clicked() {
                    self.processedImage = self.originalImageBytes.clone();
                    self.UpdateImage(ctx);
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.displayedImage.is_some() {
                let img = self.displayedImage.as_ref().unwrap();
                let width = f32::min(ui.available_width(), ui.available_height() * img.aspect_ratio());
                let displaySize = egui::vec2(width, width / img.aspect_ratio());
                ui.image(img, displaySize);
            }
        });

        // https://github.com/emilk/egui/blob/8ce0e1c5206780e76234842b94ceb0edf5bb8b75/examples/file_dialog/src/main.rs#L67
        ctx.input(|input| {
            if !input.raw.dropped_files.is_empty() {
                let img = OpenImgFile(input.raw.dropped_files[0].path.as_ref().unwrap());
                if img.is_some() {
                    self.originalImageBytes = img;
                    self.processedImage = self.originalImageBytes.clone();
                    self.displayedImage = None;
                }
            }
        });

    }
}

fn OpenImgFileWithDialog() -> Option<image::DynamicImage> {
    let path = FileDialog::new()
    .add_filter("Image files", &["png", "jpeg", "jpg", "bmp", "gif", "webp", "tiff", "tga"])
    .show_open_single_file().unwrap();

    if path.is_none() { return None; }

    return OpenImgFile(&path.unwrap());
}

fn OpenImgFile(path: &std::path::PathBuf) -> Option<image::DynamicImage> {
    let image = ImageReader::open(path).unwrap().decode();
    if image.is_err() {
        return None;
    }

    // probably inefficient but the image needs to be in rgba8 format otherwise the sorter shits itself and dies (and so do i)
    return Some(image::DynamicImage::from(image.unwrap().into_rgba8()));
}

fn GetSavePath() -> Option<std::path::PathBuf> {
    return FileDialog::new()
    .add_filter("Image files", &["png", "jpeg", "jpg", "bmp", "gif", "webp", "tiff", "tga"])
    .show_save_single_file().unwrap();
}
