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

use image::io::Reader as IR;

use std::collections::VecDeque;

pub fn main() -> Result<(), String> {
    let (
        host_portals,
        RhaiPortals{
            from_rhai, input_to_rhai, int_to_rhai, crop_to_rhai,
        },
    ) = create_channels();

    std::thread::spawn(move || {
        let engine = construct_rhai_engine(host_portals);

        engine.run(
            r#"
                print("Starting script loop...");
                let wh = get_wh();
                let w = wh.w;
                let h = wh.h;
                let px = w/2;
                let py = h/2;
                let qx = w/2;
                let qy = h/2;
                while true{
                    let opx = px;
                    let opy = py;
                    let oqx = qx;
                    let oqy = qy;
                    let e = get_input_event();
                    if e.key == "termination" { break; }
                    if e.is_click {
                        print(`  Script Read: ${e.x}, ${e.y}`);
                        if e.key == "left" {
                            px = e.x;
                            py = e.y;
                        } else if e.key == "right" {
                            qx = e.x;
                            qy = e.y;
                        }
                    } else if e.key == "return"{
                        crop(0, px, py, qx, qy);
                        kill();
                    }
                    else if e.key == "s"{
                        px -= 10;
                        qx -= 10;
                    }
                    else if e.key == "n"{
                        px += 10;
                        qx += 10;
                    }
                    else if e.key == "m"{
                        py -= 10;
                        qy -= 10;
                    }
                    else if e.key == "t"{
                        py += 10;
                        qy += 10;
                    }
                    else if e.key == "a"{
                        px += 10;
                        qx -= 10;
                    }
                    else if e.key == "o"{
                        px -= 10;
                        qx += 10;
                    }
                    else if e.key == "u"{
                        py -= 10;
                        qy += 10;
                    }
                    else if e.key == "e"{
                        py += 10;
                        qy -= 10;
                    }
                    if px != opx || py != opy || qx != oqx || qy != oqy{
                        clear_rects();
                        draw_rect_xy(px, py, qx, qy);
                    }
                }
                kill();
            "#,
        ).expect("Editimg: rhai error");
    });

    println!("Starting main loop...");

    let mut timer = Timer::new();
    let (mut window, mut event_pump) = EIWindow::create(&timer)?;

    let file = "/home/cody/img/janitor-pics/14_cracked_stones.png";
    let img = IR::open(file)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    println!("Image: {:?}ms", timer.elapsed());
    window.set_texture(&img, &timer)?;

    enum PollType {
        Input,
        WH,
        Crop(i64, i64, i64, i64, i64),
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
                Crop(b, px, py, qx, qy) => polls.push_back(PollType::Crop(b, px, py, qx, qy)),
            }
            if rects_uv.len() + rects_xy.len() > 0 { drawn = true; }
            while let Some(r) = rects_uv.pop(){ window.draw_rect_uv(r)?; }
            while let Some(r) = rects_xy.pop(){ window.draw_rect_xy(r)?; }
        }
        if drawn { window.redraw(); }

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
                PollType::Input => if let Some(e) = inputs.pop_front() {
                    input_to_rhai.send(e).map_err(|e| e.to_string())?;
                    polls.pop_front();
                },
                PollType::WH => {
                    let (w, h) = window.get_wh();
                    int_to_rhai.send(w as i64).map_err(|e| e.to_string())?;
                    int_to_rhai.send(h as i64).map_err(|e| e.to_string())?;
                    polls.pop_front();
                },
                PollType::Crop(b, px, py, qx, qy) => {
                    let (px, py, qx, qy) = img_crop(*px, *py, *qx, *qy);
                    let c = images[*b as usize].crop(px, py, qx - px, qy - py);
                    crop_to_rhai.send(1).map_err(|e| e.to_string())?;
                    polls.pop_front();
                    c.save("output.jpg").map_err(|e| e.to_string())?;
                },
            }
        }

        if die {
            input_to_rhai.send(Input::key("termination".to_string())).map_err(|e| e.to_string())?;
            int_to_rhai.send(-1).map_err(|e| e.to_string())?;
            break;
        }
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

