use color_eyre::eyre::Result;
use egui::{ColorImage, TextureHandle, Ui, Vec2};
use image::{imageops::FilterType, io::Reader as ImageReader, DynamicImage};
use lith::gen::{
    cylinder_mesh::CylinderMeshGenerator, filter_image::FilterImagePreprocessor,
    flat_mesh::FlatMeshGenerator, standard_image::StandardImagePreprocessor, ImagePreprocessor,
    LithophaneGenerator,
};
use std::{fmt::Display, path::PathBuf};

use eframe::egui;

struct App {
    path: Option<PathBuf>,
    display_image: Option<TextureHandle>,
    dyn_image: Option<DynamicImage>,
    res: Option<Result<usize, &'static str>>,
    processor: Processor,
    generator: Generator,
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

    fn generate_lithophane(&mut self) {
        let map = match self.processor {
            Processor::Standard(width) => StandardImagePreprocessor::default()
                .width(width)
                .transform(self.dyn_image.as_ref().unwrap()),
            Processor::Filter(width, filter) => FilterImagePreprocessor::default()
                .width(width)
                .filter(FILTER_TYPES[filter])
                .transform(self.dyn_image.as_ref().unwrap()),
        };
        let mesh = match self.generator {
            Generator::FlatMesh(scaling) => {
                FlatMeshGenerator::default().scaling(scaling).generate(map)
            }
            Generator::Cylinder(scaling, radius, height) => CylinderMeshGenerator::default()
                .scaling(scaling)
                .radius(radius)
                .height(height)
                .generate(map),
        };

        let r = std::fs::write(
            self.path.as_ref().unwrap().with_extension("stl"),
            mesh.as_stl_bytes(),
        );

        if let Err(err) = r {
            println!("{:?}", err);
            self.res = Some(Err("Please check the console for more information..."));
        } else {
            self.res = Some(Ok(10));
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            path: None,
            display_image: None,
            dyn_image: None,
            res: None,
            processor: Processor::Standard(80),
            generator: Generator::FlatMesh(2.0),
        }
    }
}

static FILE_FORMATS: &[&str] = &["png", "jpg", "jpeg", "bmp", "qoi", "tiff"];
static FILTER_TYPES: &[FilterType] = &[
    FilterType::Nearest,
    FilterType::Triangle,
    FilterType::Gaussian,
    FilterType::CatmullRom,
    FilterType::Lanczos3,
];
static FILTER_NAMES: &[&str] = &[
    "Nearest Neighbor",
    "Linear",
    "Gaussian",
    "Bicubic",
    "Lanczos",
];

enum Processor {
    Standard(usize),
    Filter(usize, usize),
}

impl Display for Processor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Processor::Standard(_) => "Standard",
                Processor::Filter(_, _) => "Filtered",
            }
        )
    }
}

enum Generator {
    FlatMesh(f32),
    Cylinder(f32, f32, f32),
}

impl Display for Generator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Generator::FlatMesh(_) => "Flat Mesh",
                Generator::Cylinder(_, _, _) => "Cylindrical",
            }
        )
    }
}

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

            ui.menu_button(format!("Image Processor: {}", self.processor), |ui| {
                if ui.button("Standard").clicked() {
                    self.processor = Processor::Standard(80);
                    ui.close_menu();
                }
                if ui.button("Filtered").clicked() {
                    self.processor = Processor::Filter(80, 3);
                    ui.close_menu();
                }
            });

            match self.processor {
                Processor::Standard(ref mut width) => {
                    ui.horizontal(|ui| {
                        ui.label("Width");
                        ui.add(egui::Slider::new(width, 20..=720))
                    });
                }
                Processor::Filter(ref mut width, ref mut filter) => {
                    ui.horizontal(|ui| {
                        ui.label("Width");
                        ui.add(egui::Slider::new(width, 20..=720))
                    });
                    ui.menu_button(format!("Filter: {}", FILTER_NAMES[*filter]), |ui| {
                        for (i, name) in FILTER_NAMES.iter().enumerate() {
                            if ui.button(*name).clicked() {
                                *filter = i;
                                ui.close_menu();
                            }
                        }
                    });
                }
            }

            ui.menu_button(format!("Mesh Generator: {}", self.generator), |ui| {
                if ui.button("Flat Mesh").clicked() {
                    self.generator = Generator::FlatMesh(2.0);
                    ui.close_menu();
                }
                if ui.button("Cylindrical").clicked() {
                    self.generator = Generator::Cylinder(2.0, 20.0, 20.0);
                    ui.close_menu();
                }
            });

            match self.generator {
                Generator::FlatMesh(ref mut scaling) => {
                    ui.horizontal(|ui| {
                        ui.label("Scaling");
                        ui.add(egui::Slider::new(scaling, 0.0..=5.0))
                    });
                }
                Generator::Cylinder(ref mut scaling, ref mut radius, ref mut height) => {
                    ui.horizontal(|ui| {
                        ui.label("Scaling");
                        ui.add(egui::Slider::new(scaling, 0.0..=5.0))
                    });
                    ui.horizontal(|ui| {
                        ui.label("Radius");
                        ui.add(egui::Slider::new(radius, 0.0..=50.0))
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height");
                        ui.add(egui::Slider::new(height, 0.0..=100.0))
                    });
                }
            }

            if let Some(ref texture) = self.display_image {
                let s = texture.size();
                let h = ui.available_height() - 80.0;
                let w = s[0] as f32 * h / s[1] as f32;
                ui.vertical_centered(|ui| ui.image((texture.id(), Vec2 { x: w, y: h })));
            }

            if !self.dyn_image.is_none() {
                ui.vertical_centered(|ui| {
                    if ui.button("Generate Lithophane").clicked() {
                        self.res = None;
                        self.generate_lithophane();
                    }
                });
            }

            if let Some(ref mut res) = self.res {
                match res {
                    Ok(ref mut countdown) => {
                        if *countdown > 0 {
                            *countdown -= 1;
                            ui.ctx().request_repaint();
                        } else {
                            ui.label("Lithophane successfully generated...");
                        }
                    }
                    Err(msg) => {
                        ui.label(&format!("ERROR: {msg}"));
                    }
                };
            }
        });
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
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
