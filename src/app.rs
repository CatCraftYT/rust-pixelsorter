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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
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
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                    if ui.button("Open").clicked() {
                        let img = OpenImgFileWithDialog();
                        if img.is_some() {
                            self.originalImageBytes = img;
                            self.processedImage = self.originalImageBytes.clone();
                            self.UpdateImage(ctx);
                        }
                    }

                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("settings_panel").show(ctx, |ui| {
            ui.heading("Pixel Sorter Settings");

            ui.add_enabled_ui(self.originalImageBytes.is_some(), |ui| {
                ui.checkbox(&mut self.settings.vertical, "Vertical").on_hover_text("If this is true, pixels will be sorted vertically. If not, the pixels will be sorted horizontally.");
                
                ui.add(egui::Slider::new(&mut self.settings.threshold.end, self.settings.threshold.start..=255).clamp_to_range(true).text("Threshold max"));
                ui.add(egui::Slider::new(&mut self.settings.threshold.start, 0..=self.settings.threshold.end).clamp_to_range(true).text("Threshold min"));
                ui.checkbox(&mut self.settings.showThresholds, "Show thresholds").on_hover_text("Show the thresholds for sorting. Rows of white pixels will be sorted, black pixels will not be affected.");

                if ui.button("Sort!").clicked() {
                    self.processedImage = self.originalImageBytes.clone();
                    crate::sorter::SortImage(self.processedImage.as_mut().unwrap(), &self.settings);
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

    }
}

fn OpenImgFileWithDialog() -> Option<image::DynamicImage> {
    let path = FileDialog::new()
    //.add_filter("Image files", &["PNG, JPEG, JPG, BMP, GIF, WEBP, TIFF, TGA"])
    .show_open_single_file().unwrap();

    if path.is_none() { return None; }

    let image = ImageReader::open(path.unwrap()).unwrap().decode();
    if image.is_err() {
        return None;
    }
    return Some(image.unwrap());
}
