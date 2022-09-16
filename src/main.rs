use image::io::Reader as IR;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

use std::time::Instant;

pub fn main() -> Result<(), String> {
    let start = Instant::now();
    let ta = start.elapsed().as_millis();

    let img = IR::open("/home/cody/img/collections/janitor-pics/14_cracked_stones.png")
        .expect("dsnt").decode().expect("hoed");
    let pxs = img.as_rgba8().unwrap();
    //let pxs = img.to_rgba8();
    // let pxs = IR::open("/home/cody/img/collections/cltracer/q-bright-sky-rough-copper.png")
    //     .expect("dsnt").decode().expect("hoed");
    // let pxs = IR::open("/home/cody/img/collections/cltracer/t-microfacets-dielectrics-conductors.png")
    //     .expect("dsnt").decode().expect("hoed");

    println!("image: {:?}ms", start.elapsed().as_millis() - ta);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 500, 500)
        .resizable()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    println!("window: {:?}ms", start.elapsed().as_millis() - ta);

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA32, img.width(), img.height())
        .map_err(|e| e.to_string())?;
    texture.update(None, pxs, 4 * img.width() as usize).unwrap();

    println!("texture: {:?}ms", start.elapsed().as_millis() - ta);

    canvas.copy(&texture, None, Some(Rect::new(0, 0, 512, 256)))?;
    canvas.present();

    println!("present: {:?}ms", start.elapsed().as_millis() - ta);

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown { .. } => {
                    println!("yeet");
                    canvas.copy(&texture, None, Some(Rect::new(100, 100, 256, 256)))?;
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...
    }

    Ok(())
}
