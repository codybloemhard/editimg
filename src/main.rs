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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    input: PathBuf,
}

pub fn main() -> Result<(), String> {

    let args = Args::parse();

    let (
        host_portals,
        RhaiPortals{
            from_rhai, mut to_rhai,
        },
    ) = create_channels();
    let (to_host, from_thread) = mpsc::channel();

    let mut fpath = sio::get_home().unwrap();
    fpath.push(".config/editimg/crop.rhai.rs");
    let rhai_code = sio::read_file_into_string(&fpath).unwrap();

    std::thread::spawn(move || {
        let engine = construct_rhai_engine(host_portals);
        match engine.compile(&rhai_code) {
            Ok(ast) => {
                to_host.send(None).expect("Editimg: compilation verification send error");
                engine.run_ast(&ast).expect("Editimg: rhai run error");
            },
            Err(e) => {
                to_host.send(Some(e)).expect("Editimg: compilation verification send error");
            },
        }
    });

    if let Some(e) = from_thread.recv().expect("Editimg: compilation verification receive error") {
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

    enum PollType {
        Input,
        WH,
        Crop(i64, i64, i64, i64, i64, i64),
        Save(i64, String),
    }

    let mut polls = VecDeque::new();
    let mut inputs = VecDeque::new();
    let mut rects_uv = Vec::new();
    let mut rects_xy = Vec::new();
    let mut images = vec![img];

    loop {
        let mut drawn = false;
        let mut die = false;

        for rhai_call in from_rhai.try_iter(){
            use HostMsg::*;
            match rhai_call{
                Kill => { die = true; break; },
                GetInputEvent => polls.push_back(PollType::Input),
                GetWH => polls.push_back(PollType::WH),
                DebugI64(i) => println!("{:?}", i),
                ClearRects => {
                    window.clear_rects();
                    window.redraw_texture()?;
                },
                DrawRectUV(r) => rects_uv.push(r),
                DrawRectXY(r) => rects_xy.push(r),
                Crop(s, d, px, py, qx, qy) => polls.push_back(PollType::Crop(s, d, px, py, qx, qy)),
                Save(s, p) => polls.push_back(PollType::Save(s, p)),
            }
            if rects_uv.len() + rects_xy.len() > 0 { drawn = true; }
            while let Some(r) = rects_uv.pop(){ window.draw_rect_uv(r)?; }
            while let Some(r) = rects_xy.pop(){ window.draw_rect_xy(r)?; }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
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
                Event::KeyDown { keycode: Some(kc), .. } => {
                    inputs.push_back(Input::key(format!("{:?}", kc).to_lowercase()));
                },
                Event::MouseButtonDown{ mouse_btn, clicks: 1, x, y, .. } => {
                    let button = format!("{:?}", mouse_btn).to_lowercase();
                    inputs.push_back(Input::click(window.screen_to_click(x, y), button));
                },
                _ => {}
            }
        }

        if let Some(pt) = polls.iter().next() {
            match pt {
                PollType::Input => if let Some(i) = inputs.pop_front() {
                    to_rhai.send(RhaiMsg::Input(i)).map_err(|_| "Editimg: cannot push input")?;
                    polls.pop_front();
                },
                PollType::WH => {
                    let (w, h) = window.get_wh();
                    to_rhai.send(RhaiMsg::Int(w as i64)).map_err(|_| "Editimg: cannot push width")?;
                    to_rhai.send(RhaiMsg::Int(h as i64)).map_err(|_| "Editimg: cannot push height")?;
                    polls.pop_front();
                },
                PollType::Crop(s, d, px, py, qx, qy) => {
                    let (px, py, qx, qy) = img_crop(*px, *py, *qx, *qy);
                    let s = ((*s).max(0) as usize).min(images.len() - 1);
                    let c = images[s].crop(px, py, qx - px, qy - py);
                    let d = if *d < 0 {
                        images.push(c);
                        images.len() - 1
                    } else {
                        let d = ((*d).max(0) as usize).min(images.len() - 1);
                        images[d] = c;
                        d
                    };
                    to_rhai.send(RhaiMsg::Int(d as i64)).map_err(|_| "Editimg: cannot push crop")?;
                    if d == 0 {
                        window.set_texture(&images[0], &mut timer)?;
                        window.redraw_texture()?;
                        drawn = true;
                    }
                    polls.pop_front();
                },
                PollType::Save(source, path) => {
                    let s = ((*source).max(0) as usize).min(images.len() - 1);
                    images[s].save(path).map_err(|e| e.to_string())?;
                    polls.pop_front();
                },
            }
        }

        if die {
            // do not yield error on purpose: channel maybe closed, and that is alright.
            let _ = to_rhai.send(RhaiMsg::Killed).map_err(|_| "Editimg: cannot push die signal");
            break;
        }

        if drawn { window.redraw(); }
    }

    println!("Editimg: finished.");
    Ok(())
}

fn img_crop(px: i64, py: i64, qx: i64, qy: i64) -> (u32, u32, u32, u32) {
    let npx = px.min(qx).max(0) as u32;
    let npy = py.min(qy).max(0) as u32;
    let nqx = px.max(qx).max(0) as u32;
    let nqy = py.max(qy).max(0) as u32;
    (npx, npy, nqx, nqy)
}

