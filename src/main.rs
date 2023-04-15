#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    //tracing_subscriber::fmt::init(); 

    // use this to convert to right format https://convertio.co/rgba-converter/
    let icon = eframe::IconData { rgba: include_bytes!(r"../resources/icon.rgba").to_vec(), width: 128, height: 128 };

    let mut native_options = eframe::NativeOptions::default();
    native_options.icon_data = Some(icon);
    native_options.maximized = true;

    eframe::run_native(
        "Pixel Sorter",
        native_options,
        Box::new(|cc| Box::new(pixelsorter::PixelSorter::new(cc))),
    )
}
