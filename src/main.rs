use color_eyre::eyre::{Result, WrapErr, Report};
use image::{imageops::FilterType, io::Reader as ImageReader, DynamicImage, ImageBuffer};
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use eframe::egui;

#[derive(Default, Clone)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

impl Into<[u8; 12]> for &Vertex {
    fn into(self) -> [u8; 12] {
        let x = self.x.to_le_bytes();
        let y = self.y.to_le_bytes();
        let z = self.z.to_le_bytes();
        [
            x[0], x[1], x[2], x[3], y[0], y[1], y[2], y[3], z[0], z[1], z[2], z[3],
        ]
    }
}

impl std::ops::Sub<&Vertex> for &Vertex {
    type Output = Vertex;

    fn sub(self, rhs: &Vertex) -> Self::Output {
        Vertex {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

struct LightnessMap {
    lights: Vec<f32>,
    width: usize,
    height: usize,
}

impl From<&DynamicImage> for LightnessMap {
    fn from(value: &DynamicImage) -> Self {
        let gray = value.to_luma8();

        let data: Vec<_> = gray
            .iter()
            .map(|p| {
                let y = *p as f32 / 255.0;
                if y < 216.0 / 24389.0 {
                    y * 24389.0 / 27.0
                } else {
                    (y.powf(1.0 / 3.0) * 116.0) - 16.0
                }
            })
            .collect();

        Self {
            lights: data,
            width: value.width() as usize,
            height: value.height() as usize,
        }
    }
}

struct HeightMap {
    heights: Vec<f32>,
    width: usize,
    height: usize,
}

impl HeightMap {
    pub fn from_lightness(light: LightnessMap, scale: f32) -> Self {
        let (width, height) = (light.width, light.height);
        let mut data = light.lights;
        data.iter_mut().for_each(|p| *p *= scale);
        Self {
            heights: data,
            width,
            height,
        }
    }

    pub fn get_vertex(&self, x: usize, y: usize) -> Vertex {
        Vertex {
            x: x as f32,
            y: y as f32,
            z: self.heights[y * self.width + x],
        }
    }
}

fn normal(a: &Vertex, b: &Vertex, c: &Vertex) -> Vertex {
    let u = b - a;
    let v = c - a;
    Vertex {
        x: u.y * v.z - u.z * v.y,
        y: u.z * v.x - u.x * v.z,
        z: u.x * v.y - u.y * v.x,
    }
}

struct Mesh {
    tris: Vec<[Vertex; 4]>,
}

impl Mesh {
    fn save(&self, path: &Path) -> Result<()> {
        let f = OpenOptions::new().create(true).write(true).open(path)?;
        let mut stream = BufWriter::new(f);

        stream.write_all(&[0; 80])?;
        stream.write(&(self.tris.len() as u32).to_le_bytes())?;

        for t in self.tris.iter() {
            let (a, b, c, n) = (&t[0], &t[1], &t[2], &t[3]);
            let a: [u8; 12] = a.into();
            let b: [u8; 12] = b.into();
            let c: [u8; 12] = c.into();
            let n: [u8; 12] = n.into();
            stream.write(&n)?;
            stream.write(&a)?;
            stream.write(&b)?;
            stream.write(&c)?;
            stream.write(&[0; 2])?;
        }

        stream.flush()?;

        Ok(())
    }

    fn from_heights(map: HeightMap) -> Self {
        let mut tris = vec![];
        for y in 1..map.height {
            for x in 1..map.width {
                let top_left = map.get_vertex(x - 1, y - 1);
                let bottom_left = map.get_vertex(x - 1, y);
                let top_right = map.get_vertex(x, y - 1);
                let bottom_right = map.get_vertex(x, y);

                let tri1 = [
                    bottom_right.clone(),
                    bottom_left.clone(),
                    top_left.clone(),
                    normal(&top_left, &bottom_left, &bottom_right),
                ];
                let tri2 = [
                    bottom_right.clone(),
                    top_left.clone(),
                    top_right.clone(),
                    normal(&top_right, &top_left, &bottom_right),
                ];
                tris.push(tri1);
                tris.push(tri2);
            }
        }

        Mesh { tris }
    }
}

struct Args<'a> {
    input: &'a Path,
    scaling: f32,
    width: u32,
}

fn do_lith(args: Args) -> Result<()> {
    let img = ImageReader::open(&args.input)
        .wrap_err_with(|| format!("failed to find file '{}'", args.input.display()))?
        .decode()
        .wrap_err_with(|| format!("failed to decode image '{}'", args.input.display()))?;

    let new_height = (img.height() as f32 * args.width as f32 / img.width() as f32).ceil() as u32;
    let img = img.resize(args.width, new_height, FilterType::CatmullRom);
    let mut bg = ImageBuffer::from_pixel(
        img.width() + 4,
        img.height() + 4,
        image::Rgba([255, 255, 255, 255]),
    );
    image::imageops::overlay(&mut bg, &img, 2, 2);
    let img = DynamicImage::from(bg);

    println!("generating lightmap...");
    let lightness = LightnessMap::from(&img);
    println!("calculating mesh heights...");
    let heights = HeightMap::from_lightness(lightness, -args.scaling);
    println!("building mesh...");
    let mesh = Mesh::from_heights(heights);
    println!("saving lithophane...");
    mesh.save(&PathBuf::from(
        PathBuf::from(args.input).with_extension("stl"),
    ))?;

    Ok(())
}

#[derive(Default)]
struct App {
    path: Option<PathBuf>,
    scaling: f32,
    width: u32,
    err: Option<Report>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Select image...").clicked() {
                    self.path = rfd::FileDialog::new().pick_file();
                }

                match &self.path {
                    Some(path) => ui.label(path.display().to_string()),
                    None => ui.label("No image selected"),
                };
            });
            
            ui.horizontal(|ui| {
                ui.label("Scaling");
                ui.add(egui::Slider::new(&mut self.scaling, -1.0..=1.0))
            });

            ui.horizontal(|ui| {
                ui.label("Width");
                ui.add(egui::Slider::new(&mut self.width, 0..=480))
            });

            if ui.button("Generate Lithophane").clicked() && self.path.is_some() {
                let p = self.path.as_ref().unwrap();
                let r = do_lith(Args {
                    input: p,
                    scaling: self.scaling,
                    width: self.width,
                });
                self.err = match r {
                    Ok(()) => None,
                    Err(r) => {
                        println!("{:?}", r);
                        Some(r)
                    },
                };
            }

            if let Some(_) = self.err {
                ui.label("ERROR: Please check console for details");
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
    }));
    Ok(())
}
