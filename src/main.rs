use color_eyre::eyre::{Report, Result, WrapErr};
use egui::{ColorImage, TextureHandle, Vec2};
use image::{imageops::FilterType, io::Reader as ImageReader, DynamicImage, ImageBuffer};
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use eframe::egui;

#[derive(Default, Clone)]
#[repr(C)]
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

struct HeightMap {
    heights: Vec<f32>,
    width: usize,
    height: usize,
    min: f32,
}

impl HeightMap {
    pub fn from_image(image: &DynamicImage, scale: f32) -> Self {
        let gray = image.to_luma8();
        let mut min = f32::MAX;

        let data: Vec<_> = gray
            .iter()
            .map(|p| {
                // Normalize
                let y = *p as f32 / 255.0;
                // Convert gamma to percieved lightness and scale
                let height = if y < 216.0 / 24389.0 {
                    y * 24389.0 / 27.0 * scale
                } else {
                    ((y.powf(1.0 / 3.0) * 116.0) - 16.0) * scale
                };
                if height < min {
                    min = height;
                }
                height
            })
            .collect();

        Self {
            heights: data,
            width: image.width() as usize,
            height: image.height() as usize,
            min,
        }
    }

    pub fn get_vertex(&self, x: usize, y: usize) -> Vertex {
        Vertex {
            x: x as f32,
            y: y as f32,
            z: self.heights[y * self.width + x],
        }
    }

    pub fn get_min_vertex(&self, x: usize, y: usize) -> Vertex {
        Vertex {
            x: x as f32,
            y: y as f32,
            z: self.min,
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

impl std::ops::Index<(usize, usize)> for HeightMap {
    type Output = f32;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.heights[y * self.width + x]
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
            let (n, a, b, c) = (&t[0], &t[1], &t[2], &t[3]);
            let n: [u8; 12] = n.into();
            let a: [u8; 12] = a.into();
            let b: [u8; 12] = b.into();
            let c: [u8; 12] = c.into();
            stream.write(&n)?;
            stream.write(&a)?;
            stream.write(&b)?;
            stream.write(&c)?;
            stream.write(&[0; 2])?;
        }

        stream.flush()?;

        Ok(())
    }

    fn from_image(image: &DynamicImage, args: &Args) -> Self {
        let map = HeightMap::from_image(image, -args.scaling);
        let mut mesh = Self::from_heights(&map);
        mesh.add_brim(&map);
        mesh
    }

    fn from_heights(map: &HeightMap) -> Self {
        let mut tris = vec![];
        for y in 1..map.height {
            for x in 1..map.width {
                push_quad(
                    &mut tris,
                    map.get_vertex(x - 1, y - 1),
                    map.get_vertex(x - 1, y),
                    map.get_vertex(x, y - 1),
                    map.get_vertex(x, y),
                );
            }
        }

        Mesh { tris }
    }

    fn add_brim(&mut self, map: &HeightMap) {
        for y in 1..map.height {
            push_quad(
                &mut self.tris,
                map.get_vertex(0, y - 1),
                map.get_min_vertex(0, y - 1),
                map.get_vertex(0, y),
                map.get_min_vertex(0, y),
            );

            push_quad(
                &mut self.tris,
                map.get_vertex(map.width - 1, y - 1),
                map.get_min_vertex(map.width - 1, y - 1),
                map.get_vertex(map.width - 1, y),
                map.get_min_vertex(map.width - 1, y),
            );
        }

        for x in 1..map.width {
            push_quad(
                &mut self.tris,
                map.get_vertex(x - 1, 0),
                map.get_min_vertex(x - 1, 0),
                map.get_vertex(x, 0),
                map.get_min_vertex(x, 0),
            );

            push_quad(
                &mut self.tris,
                map.get_vertex(x - 1, map.height - 1),
                map.get_min_vertex(x - 1, map.height - 1),
                map.get_vertex(x, map.height - 1),
                map.get_min_vertex(x, map.height - 1),
            );
        }

        push_quad(
            &mut self.tris,
            map.get_min_vertex(0, map.height - 1),
            map.get_min_vertex(0, 0),
            map.get_min_vertex(map.width - 1, map.height - 1),
            map.get_min_vertex(map.width - 1, 0),
        )
    }
}

fn push_quad(vec: &mut Vec<[Vertex; 4]>, tl: Vertex, bl: Vertex, tr: Vertex, br: Vertex) {
    vec.push([normal(&br, &bl, &tl), br.clone(), bl, tl.clone()]);
    vec.push([normal(&br, &tl, &tr), br, tl, tr]);
}

struct Args<'a> {
    input: &'a Path,
    scaling: f32,
    width: u32,
}

fn load_image(path: &Path, width: u32) -> Result<DynamicImage> {
    let img = ImageReader::open(path)
        .wrap_err_with(|| format!("failed to find file '{}'", path.display()))?
        .decode()
        .wrap_err_with(|| format!("failed to decode image '{}'", path.display()))?;

    let new_height = img.height() * img.width() / 80;
    let img = img.resize(width, new_height, FilterType::CatmullRom);
    let mut bg = ImageBuffer::from_pixel(
        img.width() + 4,
        img.height() + 4,
        image::Rgba([255, 255, 255, 255]),
    );
    image::imageops::overlay(&mut bg, &img, 2, 2);
    Ok(DynamicImage::from(bg))
}

fn do_lith(args: Args) -> Result<()> {
    let image = load_image(args.input, args.width)?;
    let mesh = Mesh::from_image(&image, &args);
    mesh.save(&PathBuf::from(
        PathBuf::from(args.input).with_extension("stl"),
    ))?;

    Ok(())
}

struct App {
    path: Option<PathBuf>,
    scaling: f32,
    width: u32,
    result: Option<Result<String, Report>>,
    display_image: Option<TextureHandle>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            path: None,
            scaling: 0.05,
            width: 80,
            result: None,
            display_image: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Select image...").clicked() {
                    self.result = None;
                    self.display_image = None;
                    self.path = rfd::FileDialog::new().pick_file();
                    if let Some(ref p) = self.path {
                        // I know this code sucks but it was really easy to write
                        let raw_image = ImageReader::open(p);
                        if let Ok(raw_image) = raw_image {
                            let raw_image = raw_image.decode();
                            if let Ok(raw_image) = raw_image {
                                let image = ColorImage::from_rgba_unmultiplied([raw_image.width() as usize, raw_image.height() as usize], &raw_image.to_rgba8());
                                self.display_image = Some(ui.ctx().load_texture("image", image, Default::default()));
                            }
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
                self.result = match r {
                    Ok(()) => Some(Ok("Lithophane generated".to_string())),
                    Err(r) => {
                        println!("{:?}", r);
                        Some(Err(r))
                    }
                };
            }

            if let Some(res) = &self.result {
                ui.label(match res {
                    Ok(ref s) => s,
                    Err(_) => "ERROR: Please check console for details",
                });
            }

            if let Some(ref texture) = self.display_image {
                let s = texture.size();
                let h = ui.available_height();
                let w = s[0] as f32 * h / s[1] as f32;
                ui.image((texture.id(), Vec2 {x: w, y: h}));
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
