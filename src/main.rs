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
                for _i in 0..2{
                    let mc = get_mouse_click();
                    print(`  Script Read: ${mc.u}, ${mc.v}`);
                    draw_rect_xy(mc.x, mc.y, mc.x + 50, mc.y + 50);
                }
                kill();
            "#,
        ).expect("Editimg: rhai error");
    });

    println!("Starting main loop...");

    let mut timer = Timer::new();
    let (mut window, mut event_pump) = EIWindow::create(&timer)?;
    let file = "/home/cody/img/collections/janitor-pics/14_cracked_stones.png";
    window.set_texture(file, &timer)?;

    let mut send_next_click = false;
    let mut rects_uv = Vec::new();
    let mut rects_xy = Vec::new();

    'running: loop {
        for rhai_call in from_rhai.try_iter(){
            use HostMsg::*;
            match rhai_call{
                Kill => break 'running,
                DrawRectUV(r) => rects_uv.push(r),
                DrawRectXY(r) => rects_xy.push(r),
                GetMouseClick => send_next_click = true,
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
                Event::KeyDown { .. } => {
                    println!("yeet");
                },
                Event::MouseButtonDown{ mouse_btn: MouseButton::Left, clicks: 1, x, y, .. } => {
                    if send_next_click{
                        send_next_click = false;
                        let mc = window.screen_to_click(x, y);
                        mc_to_rhai.send(mc).map_err(|e| e.to_string())?;
                    }
                },
                _ => {}
            }
        }
    }

    println!("Editimg: finished.");
    Ok(())
}

