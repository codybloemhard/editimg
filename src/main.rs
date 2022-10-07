mod window;
mod timer;

use crate::{
    window::EIWindow,
    timer::Timer,
};

use rhai::Engine;

use sdl2::{
    event::{ Event, WindowEvent },
    keyboard::Keycode,
    mouse::MouseButton,
};

use std::sync::mpsc;

#[derive(Debug, Clone)]
struct RHAIMouseClick{
    x: i64,
    y: i64,
}

impl RHAIMouseClick{
    fn get_x(&mut self) -> i64 { self.x }
    fn get_y(&mut self) -> i64 { self.y }
}

pub fn main() -> Result<(), String> {
    let (to_host, from_rhai) = mpsc::channel();
    let (to_rhai, from_host) = mpsc::channel();

    let _rhai_thread_handle = std::thread::spawn(move || {
        let mut engine = Engine::new();

        engine.register_type_with_name::<RHAIMouseClick>("MouseClick")
            .register_get("x", RHAIMouseClick::get_x)
            .register_get("y", RHAIMouseClick::get_y);

        let receive_err = "Editimg: rhai thread could not receive from host.";
        let send_err = "Editimg: rhai thread could not send to host.";
        let th0 = to_host.clone();
        let th1 = to_host.clone();

        // Notice that the API functions are blocking
        engine
            .register_fn("get", move || -> RHAIMouseClick {
                th0.send(-1).expect(send_err);
                from_host.recv().expect(receive_err)
            })
            .register_fn("put", move |v: i64| {
                th1.send(v).expect(send_err);
            })
            ;

        engine.run(
            r#"
                print("Starting script loop...");
                for _i in 0..2{
                    let mc = get();
                    print(`  Script Read: ${mc.x}, ${mc.y}`);
                    put(mc.x);
                    put(mc.y);
                }
            "#,
        ).expect("Editimg: rhai error");
    });

    println!("Starting main loop...");

    let mut timer = Timer::new();
    let (mut window, mut event_pump) = EIWindow::create(&timer)?;
    let file = "/home/cody/img/collections/janitor-pics/14_cracked_stones.png";
    window.set_texture(file, &timer)?;

    let mut send_next_click = false;

    'running: loop {
        for rhai_call in from_rhai.try_iter(){
            println!("Call: {rhai_call}");
            if rhai_call == -1 { send_next_click = true; }
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
                    window.redraw(winw, winh)?;
                    println!("Resizing: {:?}ms", timer.elapsed());
                },
                Event::KeyDown { .. } => {
                    println!("yeet");
                },
                Event::MouseButtonDown{ mouse_btn: MouseButton::Left, clicks: 1, x, y, .. } => {
                    if send_next_click{
                        send_next_click = false;
                        to_rhai.send(RHAIMouseClick{ x: x as i64, y: y as i64 })
                            .map_err(|e| e.to_string())?;
                    }
                },
                _ => {}
            }
        }
    }

    Ok(())
}

