mod window;
mod timer;

use crate::{
    window::EIWindow,
    timer::Timer,
};

// use mlua::prelude::*;
use rhai::Engine;

use sdl2::{
    event::{ Event, WindowEvent },
    keyboard::Keycode,
    mouse::MouseButton,
};

pub fn main() -> Result<(), String> {
    let mut test = 0;

    // let lua = Lua::new();
    // lua.scope(|scope| {
    //     lua.globals().set(
    //         "set_test",
    //         scope.create_function_mut(|_, ()| {
    //             test = 32;
    //             Ok(())
    //         })?,
    //     )?;
    //     lua.load("set_test()").exec()
    // }).map_err(|err| err.to_string())?;

    // Channel: Script -> Master
    let (tx_script, rx_master) = std::sync::mpsc::channel();
    // Channel: Master -> Script
    let (tx_master, rx_script) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let mut engine = Engine::new();

        // Notice that the API functions are blocking
        engine
            .register_fn("get", move || rx_script.recv().unwrap_or_default())
            .register_fn("put", move |v: i64| tx_script.send(v).unwrap());

        engine.run(
            r#"
                print("Starting script loop...");
                loop {
                    let x = get();
                    print(`Script Read: ${x}`);
                    x += 2;
                    print(`Script Write: ${x}`);
                    put(x);
                }
            "#,
        )
        .unwrap();
    });

    println!("Starting main loop...");

    let mut value: i64 = 1;

    while value < 10 {
        println!("Value: {}", value);
        // Send value to script
        tx_master.send(value).unwrap();
        // Receive value from script
        value = rx_master.recv().unwrap();
        if value == 5 { value = 2; }
    }

    panic!("yeetus");

    // This is the main processing thread

    println!("Starting main loop...");

    let mut value: i64 = 0;

    while value < 10 {
        println!("Value: {}", value);
        // Send value to script
        tx_master.send(value).unwrap();
        // Receive value from script
        value = rx_master.recv().unwrap();
    }

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

