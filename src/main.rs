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

use image::io::Reader as IR;

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
                Crop(s, d, px, py, qx, qy) => {
                    let (px, py, qx, qy) = img_crop(*px, *py, *qx, *qy);
                    let s = ((*s).max(0) as usize).min(images.len() - 1);
                    let c = images[s].crop(px, py, qx - px, qy - py);
                    let d = if *d < 0 {
                        images.push(c);
                        images.len() - 1
                    } else {
                        let d = img_index(d, &images);
                        images[d] = c;
                        d
                    };
                    to_rhai.send(RhaiMsg::Int(d as i64)).map_err(|_| "Editimg: cannot push crop")?;
                    if d == 0 { redraw = true; }
                },
                Save(source, path) => {
                    let s = img_index(source, &images);
                    images[s].save(path).map_err(|e| e.to_string())?;
                },
                FlipH(image) => {
                    let i = img_index(image, &images);
                    images[i] = images[i].fliph();
                    if i == 0 { redraw = true; }
                },
                FlipV(image) => {
                    let i = img_index(image, &images);
                    images[i] = images[i].flipv();
                    if i == 0 { redraw = true; }
                },
                Rot90(image) => {
                    let i = img_index(image, &images);
                    images[i] = images[i].rotate90();
                    if i == 0 { redraw = true; }
                },
                Rot180(image) => {
                    let i = img_index(image, &images);
                    images[i] = images[i].rotate180();
                    if i == 0 { redraw = true; }
                },
                Rot270(image) => {
                    let i = img_index(image, &images);
                    images[i] = images[i].rotate270();
                    if i == 0 { redraw = true; }
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
            window.set_texture(&images[0], &mut timer)?;
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

fn img_index(i: &i64, images: &Vec<image::DynamicImage>) -> usize {
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

