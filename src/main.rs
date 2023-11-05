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

use std::collections::VecDeque;

pub fn main() -> Result<(), String> {
    let (
        host_portals,
        RhaiPortals{ from_rhai, input_to_rhai, int_to_rhai },
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
    window.set_texture(file, &timer)?;

    enum PollType { Input, WH }
    let mut polls = VecDeque::new();
    let mut inputs = VecDeque::new();
    let mut rects_uv = Vec::new();
    let mut rects_xy = Vec::new();

    'running: loop {
        let mut drawn = false;
        for rhai_call in from_rhai.try_iter(){
            use HostMsg::*;
            match rhai_call{
                Kill => break 'running,
                ClearRects => {
                    window.clear_rects();
                    window.redraw_texture()?;
                },
                DrawRectUV(r) => rects_uv.push(r),
                DrawRectXY(r) => rects_xy.push(r),
                GetInputEvent => polls.push_back(PollType::Input),
                GetWH => polls.push_back(PollType::WH),
                msg => println!("{:?}", msg),
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
                } => break 'running,
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
            }
        }
    }

    println!("Editimg: finished.");
    Ok(())
}

