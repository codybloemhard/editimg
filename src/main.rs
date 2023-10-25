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
    mouse::MouseButton,
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
                while true{
                    let e = get_input_event();
                    if e.is_click {
                        print(`  Script Read: ${e.u}, ${e.v}`);
                        draw_rect_xy(e.x, e.y, e.x + 50, e.y + 50);
                    } else if e.key == "Return"{
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
        for rhai_call in from_rhai.try_iter(){
            use HostMsg::*;
            match rhai_call{
                Kill => break 'running,
                DrawRectUV(r) => rects_uv.push(r),
                DrawRectXY(r) => rects_xy.push(r),
                GetInputEvent => polling = true,
                msg => println!("{:?}", msg),
            }
            let mut drawn = false;
            if rects_uv.len() + rects_xy.len() > 0 { drawn = true; }
            while let Some(r) = rects_uv.pop(){ window.draw_rect_uv(r)?; }
            while let Some(r) = rects_xy.pop(){ window.draw_rect_xy(r)?; }
            if drawn { window.redraw(); }
        }
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
                    inputs.push_back(Input::key(format!("{:?}", kc)));
                },
                Event::MouseButtonDown{ mouse_btn: MouseButton::Left, clicks: 1, x, y, .. } => {
                    inputs.push_back(Input::click(window.screen_to_click(x, y)));
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

