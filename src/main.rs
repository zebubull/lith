use color_eyre::eyre::Result;
use egui::{ColorImage, TextureHandle, Ui, Vec2};
use image::{io::Reader as ImageReader, DynamicImage};
use lith::gen::{flat_image::FlatImageGenerator, LithophaneGenerator};
use std::path::PathBuf;

use eframe::egui;

struct App {
    path: Option<PathBuf>,
    scaling: f32,
    width: usize,
    display_image: Option<TextureHandle>,
    dyn_image: Option<DynamicImage>,
    res: Option<Result<(), &'static str>>,
}

impl App {
    fn try_load_image(&mut self, path: PathBuf, ui: &Ui) -> Result<()> {
        let raw_image = ImageReader::open(&path)?.decode()?;
        self.path = Some(path);
        let image = ColorImage::from_rgba_unmultiplied(
            [raw_image.width() as usize, raw_image.height() as usize],
            &raw_image.to_rgba8(),
        );
        self.display_image = Some(ui.ctx().load_texture("image", image, Default::default()));
        self.dyn_image = Some(raw_image);

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            path: None,
            scaling: 3.0,
            width: 80,
            display_image: None,
            dyn_image: None,
            res: None,
        }
    }
}

static FILE_FORMATS: &[&str] = &["png", "jpg", "jpeg", "bmp", "qoi", "tiff"];

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Select image...").clicked() {
                    self.display_image = None;
                    self.path = None;
                    let path = rfd::FileDialog::new()
                        .add_filter("Image Files", &FILE_FORMATS)
                        .add_filter("All Files", &[""])
                        .pick_file();
                    if let Some(p) = path {
                        let err = self.try_load_image(p, ui);
                        if let Err(e) = err {
                            println!("{:?}", e);
                            self.res =
                                Some(Err("Please check the console for more information..."));
                        }
                    }
                }

                match &self.path {
                    Some(path) => ui.label(path.display().to_string()),
                    None => ui.label("No image selected"),
                };
            });

            ui.horizontal(|ui| {
                ui.label("Scaling");
                ui.add(egui::Slider::new(&mut self.scaling, 0.0..=5.0))
            });

            ui.horizontal(|ui| {
                ui.label("Width");
                ui.add(egui::Slider::new(&mut self.width, 0..=480))
            });

            if let Some(ref texture) = self.display_image {
                let s = texture.size();
                let h = ui.available_height() - 80.0;
                let w = s[0] as f32 * h / s[1] as f32;
                ui.vertical_centered(|ui| ui.image((texture.id(), Vec2 { x: w, y: h })));
            }

            if !self.dyn_image.is_none() {
                ui.vertical_centered(|ui| {
                    if ui.button("Generate Lithophane").clicked() {
                        let generator =
                            FlatImageGenerator::from(self.dyn_image.as_ref().unwrap().clone())
                                .width(self.width)
                                .scaling(self.scaling);
                        let mesh = generator.generate();
                        let r = std::fs::write(
                            self.path.as_ref().unwrap().with_extension("stl"),
                            mesh.as_stl_bytes(),
                        );
                        if let Err(err) = r {
                            println!("{:?}", err);
                            self.res =
                                Some(Err("Please check the console for more information..."));
                        } else {
                            self.res = Some(Ok(()));
                        }
                    }
                });
            }

            if let Some(res) = self.res {
                match res {
                    Ok(_) => ui.label("Lithophane successfully generated..."),
                    Err(msg) => ui.label(&format!("ERROR: {msg}")),
                };
            }
        });
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 600.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Lithophane Generator",
        options,
        Box::new(|c| {
            egui_extras::install_image_loaders(&c.egui_ctx);
            Box::<App>::default()
        }),
    );
    Ok(())
}
