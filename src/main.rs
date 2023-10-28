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
        RhaiPortals{ mc_to_rhai, from_rhai },
    ) = create_channels();

    std::thread::spawn(move || {
        let engine = construct_rhai_engine(host_portals);

        engine.run(
            r#"
                print("Starting script loop...");
                let px = 0;
                let py = 0;
                let qx = 0;
                let qy = 0;
                while true{
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
                        clear_rects();
                        draw_rect_xy(px, py, qx, qy);
                    } else if e.key == "return"{
                        kill();
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

    let mut polling = false;
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
                GetInputEvent => polling = true,
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

        if polling {
            if let Some(e) = inputs.pop_front() {
                mc_to_rhai.send(e).map_err(|e| e.to_string())?;
                polling = false;
            }
        }
    }

    println!("Editimg: finished.");
    Ok(())
}

