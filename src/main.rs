mod window;
mod timer;
mod scripting;

use crate::{
    window::EIWindow,
    timer::Timer,
    scripting::*,
};

use sdl2::{
    event::{ Event, WindowEvent },
    keyboard::Keycode,
};

use simpleio as sio;

use image::{
    io::Reader as IR,
    DynamicImage,
    GenericImage,
    imageops::FilterType,
    RgbImage,
};

use clap::Parser;

use std::{
    collections::VecDeque,
    sync::mpsc,
    path::PathBuf,
};

use rhai::{
    Module,
    Scope,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    input: PathBuf,
    command: String,
}

pub fn main() -> Result<(), String> {
    let args = Args::parse();

    let (
        host_portals,
        RhaiPortals {
            from_rhai, mut to_rhai,
        },
    ) = create_channels();
    let (to_host, from_thread) = mpsc::channel();

    let mut lpath = sio::get_home().vital("Editimg: could not get home directory");
    let mut rpath = lpath.clone();
    lpath.push(".config/editimg/lib.rhai.rs");
    rpath.push(".config/editimg");
    rpath.push(args.command);
    rpath.set_extension("rhai.rs");
    println!("{:?}", rpath);
    let lib_code = sio::read_file_into_string(&lpath).vital("Editimg: could not load library");
    let run_code = sio::read_file_into_string(&rpath).vital("Editimg: could not load command");

    std::thread::spawn(move || {
        let mut engine = construct_rhai_engine(host_portals);
        match engine.compile(&lib_code) {
            Ok(ast) => {
                match Module::eval_ast_as_new(Scope::new(), &ast, &engine) {
                    Ok(module) => {
                        engine.register_global_module(module.into());
                    },
                    Err(e) => {
                        to_host.send(Some(e.to_string()))
                            .expect("Editimg: compilation verification send error");
                    },
                }
            },
            Err(e) => {
                to_host.send(Some(e.to_string()))
                    .expect("Editimg: compilation verification send error");
            },
        }
        match engine.compile(&run_code) {
            Ok(ast) => {
                to_host.send(None).expect("Editimg: compilation verification send error");
                engine.run_ast(&ast).expect("Editimg: rhai run error");
            },
            Err(e) => {
                to_host.send(Some(e.to_string()))
                    .expect("Editimg: compilation verification send error");
            },
        }
    });

    if let Some(e) = from_thread.recv().vital("Editimg: compilation verification receive error") {
        println!("Rhai Compile error: {}", e);
        return Err("Editimg: could not compile, aborting".to_string());
    }

    println!("Starting main loop...");

    let mut timer = Timer::new();
    let (mut window, mut event_pump) = EIWindow::create(&timer)?;

    let img = IR::open(args.input)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    println!("Image: {:?}ms", timer.elapsed());
    window.set_texture(&img, &mut timer)?;

    let mut inputs = VecDeque::new();
    let mut polls = VecDeque::new();
    let mut rects_uv = Vec::new();
    let mut rects_xy = Vec::new();
    let mut images = vec![img];
    let mut show = 0;

    loop {
        let mut drawn = false;
        let mut redraw = false;
        let mut die = false;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    die = true;
                    break;
                },
                Event::Window{ win_event: WindowEvent::Resized(winw, winh), .. } => {
                    println!("Resized: ({winw}, {winh})");
                    let winw = winw.max(0).unsigned_abs();
                    let winh = winh.max(0).unsigned_abs();
                    timer.checkpoint();
                    window.resize_redraw(winw, winh)?;
                    println!("Resizing: {:?}ms", timer.elapsed());
                },
                Event::KeyDown { keycode: Some(kc), keymod, .. } => {
                    inputs.push_back(Input::key(format!("{:?}", kc).to_lowercase(), keymod));
                },
                Event::MouseButtonDown{ mouse_btn, clicks: 1, x, y, .. } => {
                    let button = format!("{:?}", mouse_btn).to_lowercase();
                    inputs.push_back(Input::click(window.screen_to_click(x, y), button));
                },
                _ => {}
            }
        }

        for rhai_call in from_rhai.try_iter() {
            polls.push_back(rhai_call);
        }

        if let Some(pt) = polls.iter().next() {
            let mut pop = true;
            use HostMsg::*;
            match pt {
                Kill => {
                    die = true;
                },
                GetInputEvent => {
                    if let Some(i) = inputs.pop_front() {
                        to_rhai.send(RhaiMsg::Input(i)).map_err(|_| "Editimg: cannot push input")?;
                    } else {
                        pop = false;
                    }
                },
                GetWH => {
                    let (w, h) = window.get_wh();
                    to_rhai.send(RhaiMsg::Int(w as i64)).map_err(|_| "Editimg: cannot push width")?;
                    to_rhai.send(RhaiMsg::Int(h as i64)).map_err(|_| "Editimg: cannot push height")?;
                },
                ClearRects => {
                    window.clear_rects();
                    window.redraw_texture()?;
                    drawn = true;
                },
                DrawRectUV(r) => {
                    rects_uv.push(r.clone());
                },
                DrawRectXY(r) => {
                    rects_xy.push(r.clone());
                },
                Crop(src, dst, px, py, qx, qy) => {
                    let s = img_index(src, &images);
                    let (px, py, qx, qy) = img_crop(*px, *py, *qx, *qy);
                    let img = images[s].crop(px, py, qx - px, qy - py);
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push crop dst")?;
                },
                Save(source, path) => {
                    let s = img_index(source, &images);
                    images[s].save(path).map_err(|e| e.to_string())?;
                },
                FlipH(src, dst) => {
                    img_action(src, dst, &mut images, &mut redraw, show, &mut to_rhai,
                        "fliph", DynamicImage::fliph)?;
                },
                FlipV(src, dst) => {
                    img_action(src, dst, &mut images, &mut redraw, show, &mut to_rhai,
                        "flipv", DynamicImage::flipv)?;
                },
                Rot90(src, dst) => {
                    img_action(src, dst, &mut images, &mut redraw, show, &mut to_rhai,
                        "rotate90", DynamicImage::rotate90)?;
                },
                Rot180(src, dst) => {
                    img_action(src, dst, &mut images, &mut redraw, show, &mut to_rhai,
                        "rotate180", DynamicImage::rotate180)?;
                },
                Rot270(src, dst) => {
                    img_action(src, dst, &mut images, &mut redraw, show, &mut to_rhai,
                        "rotate270", DynamicImage::rotate270)?;
                },
                Invert(src, dst) => {
                    let s = img_index(src, &images);
                    let mut r = images[s].clone();
                    r.invert();
                    let d = put_img(dst, r, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d as i64))
                        .map_err(|_| "Editimg: cannot push invert dst")?;
                },
                Grayscale(src, dst) => {
                    img_action(src, dst, &mut images, &mut redraw, show, &mut to_rhai,
                        "grayscale", DynamicImage::grayscale)?;
                },
                Blur(src, dst, sigma) => {
                    let s = img_index(src, &images);
                    let img = images[s].blur(*sigma as f32);
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push blur dst")?;
                },
                Unsharpen(src, dst, sigma, threshold) => {
                    let s = img_index(src, &images);
                    let img = images[s].unsharpen(*sigma as f32, *threshold as i32);
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push unsharpen dst")?;
                },
                Filter3x3(src, dst, fltr) => {
                    let s = img_index(src, &images);
                    let f = fltr.iter().map(|v| *v as f32).collect::<Vec<_>>();
                    let img = images[s].filter3x3(&f);
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push filter dst")?;
                },
                AdjustContrast(src, dst, c) => {
                    let s = img_index(src, &images);
                    let img = images[s].adjust_contrast(*c as f32);
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push contrast dts")?;
                },
                Brighten(src, dst, v) => {
                    let s = img_index(src, &images);
                    let img = images[s].brighten(*v as i32);
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push brighten dst")?;
                },
                Huerotate(src, dst, v) => {
                    let s = img_index(src, &images);
                    let img = images[s].huerotate(*v as i32);
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push huerotate dst")?;
                },
                Resize(src, dst, w, h, ft) => {
                    let s = img_index(src, &images);
                    let img = images[s].resize(clamp(w), clamp(h), filtertype(ft));
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push resize dst")?;
                },
                ResizeExact(src, dst, w, h, ft) => {
                    let s = img_index(src, &images);
                    let img = images[s].resize_exact(clamp(w), clamp(h), filtertype(ft));
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push resize_exact dst")?;
                },
                ResizeFill(src, dst, w, h, ft) => {
                    let s = img_index(src, &images);
                    let img = images[s].resize_to_fill(clamp(w), clamp(h), filtertype(ft));
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push resize_fill dst")?;
                },
                Thumbnail(src, dst, w, h) => {
                    let s = img_index(src, &images);
                    let img = images[s].thumbnail(clamp(w), clamp(h));
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push thumbnail dst")?;
                },
                ThumbnailExact(src, dst, w, h) => {
                    let s = img_index(src, &images);
                    let img = images[s].thumbnail_exact(clamp(w), clamp(h));
                    let d = put_img(dst, img, &mut images, &mut redraw, show);
                    to_rhai.send(RhaiMsg::Int(d))
                        .map_err(|_| "Editimg: cannot push thumbnail_exact dst")?;
                },
                Show(img) => {
                    let i = img_index(img, &images);
                    let old = show;
                    show = i;
                    if old != show { redraw = true; }
                    to_rhai.send(RhaiMsg::Int(show as i64))
                        .map_err(|_| "Editimg: cannot push show dst")?;
                },
                ShowNext => {
                    let old = show;
                    show = (show + 1) % images.len();
                    if old != show { redraw = true; }
                    to_rhai.send(RhaiMsg::Int(show as i64))
                        .map_err(|_| "Editimg: cannot push show dst")?;
                },
                ShowPrev => {
                    let old = show;
                    show = if show == 0 { images.len() - 1 } else { show - 1 };
                    if old != show { redraw = true; }
                    to_rhai.send(RhaiMsg::Int(show as i64))
                        .map_err(|_| "Editimg: cannot push show dst")?;
                },
                Create(w, h) => {
                    let img = RgbImage::new(clamp(w), clamp(h));
                    images.push(DynamicImage::ImageRgb8(img));
                    to_rhai.send(RhaiMsg::Int((images.len() - 1) as i64))
                        .map_err(|_| "Editimg: cannot push create dst")?;
                },
                Copy(src, dst, x, y) => {
                    let src = img_index(src, &images);
                    let dst = img_index(dst, &images);
                    let mut d = std::mem::take(&mut images[dst]);
                    let res = d.copy_from(&images[src], clamp(x), clamp(y));
                    images[dst] = d;
                    if dst == show { redraw = true; }
                    to_rhai.send(RhaiMsg::Int(if res.is_ok() { 1 } else { 0 }))
                        .map_err(|_| "Editimg: cannot push create dst")?;
                },
            }
            if pop {
                polls.pop_front();
            }
        }

        if die {
            // do not yield error on purpose: channel maybe closed, and that is alright.
            let _ = to_rhai.send(RhaiMsg::Killed).map_err(|_| "Editimg: cannot push die signal");
            break;
        }

        if rects_uv.len() + rects_xy.len() > 0 {
            drawn = true;
            while let Some(r) = rects_uv.pop() { window.draw_rect_uv(r)?; }
            while let Some(r) = rects_xy.pop() { window.draw_rect_xy(r)?; }
        }

        if redraw {
            window.set_texture(&images[show], &mut timer)?;
            window.redraw_texture()?;
            drawn = true;
        }
        if drawn {
            window.redraw();
        }
    }

    println!("Editimg: finished.");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn img_action(
    src: &i64, dst: &i64, images: &mut Vec<DynamicImage>, redraw: &mut bool, show: usize,
    to_rhai: &mut spmc::Sender<scripting::RhaiMsg>, err: &str, f: fn(&DynamicImage) -> DynamicImage
) -> Result<(), String> {
    let s = img_index(src, images);
    let img = f(&images[s]);
    let d = put_img(dst, img, images, redraw, show);
    to_rhai.send(RhaiMsg::Int(d as i64))
        .map_err(|_| format!("Editimg: cannot push {} dst", err))
}

fn filtertype(f: &str) -> FilterType {
    match f.to_lowercase().as_ref() {
        "nearest" => FilterType::Nearest,
        "triangle" => FilterType::Triangle,
        "catmullrom" => FilterType::CatmullRom,
        "catmull-rom" => FilterType::CatmullRom,
        "gaussian" => FilterType::Gaussian,
        "lanczos3" => FilterType::Lanczos3,
        "lanczos" => FilterType::Lanczos3,
        _ => FilterType::Triangle,
    }
}

fn clamp(v: &i64) -> u32 {
    (*v).max(0).min(u32::MAX as i64) as u32
}

fn put_img(
    dst: &i64, img: DynamicImage, images: &mut Vec<DynamicImage>, redraw: &mut bool, show: usize
) -> i64 {
    if *dst < 0 {
        images.push(img);
        images.len() as i64 - 1
    } else {
        let d = img_index(dst, images);
        images[d] = img;
        if d == show { *redraw = true; }
        d as i64
    }
}

fn img_index(i: &i64, images: &Vec<DynamicImage>) -> usize {
    ((*i).max(0) as usize).min(images.len() - 1)
}

fn img_crop(px: i64, py: i64, qx: i64, qy: i64) -> (u32, u32, u32, u32) {
    let npx = px.min(qx).max(0) as u32;
    let npy = py.min(qy).max(0) as u32;
    let nqx = px.max(qx).max(0) as u32;
    let nqy = py.max(qy).max(0) as u32;
    (npx, npy, nqx, nqy)
}

trait Vital<T> {
    fn vital(self, msg: &str) -> T;
}

impl<T, U: std::fmt::Display> Vital<T> for Result<T, U> {
    fn vital(self, msg: &str) -> T {
        match self {
            Ok(res) => res,
            Err(err) => {
                println!("{msg}: {err}");
                std::process::exit(-1);
            },
        }
    }
}

