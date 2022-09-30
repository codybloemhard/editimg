mod window;
mod timer;

use crate::{
    window::EIWindow,
    timer::Timer,
};

// use mlua::prelude::*;

use sdl2::{
    event::{ Event, WindowEvent },
    keyboard::Keycode,
    mouse::MouseButton,
};

pub fn main() -> Result<(), String> {
    let mut timer = Timer::new();
    let (mut window, mut event_pump) = EIWindow::create(&timer)?;
    let file = "/home/cody/img/collections/janitor-pics/14_cracked_stones.png";
    window.set_texture(file, &timer)?;

    'running: loop {
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
                    window.redraw(winw, winh)?;
                    println!("Resizing: {:?}ms", timer.elapsed());
                },
                Event::KeyDown { .. } => {
                    println!("yeet");
                },
                Event::MouseButtonDown{ mouse_btn: MouseButton::Left, clicks: 1, x, y, .. } => {
                    println!("Click: ({x}, {y})");
                },
                _ => {}
            }
        }
    }

    Ok(())
}

