use image::io::Reader as IR;

use sdl2::{
    event::{ Event, WindowEvent },
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    rect::Rect,
    mouse::MouseButton,
};

use std::time::Instant;

pub fn main() -> Result<(), String> {
    // let img = IR::open("/home/cody/img/collections/cltracer/q-bright-sky-rough-copper.png")
    //     .expect("dsnt").decode().expect("hoed");
    // let img = IR::open("/home/cody/img/collections/cltracer/t-microfacets-dielectrics-conductors.png")
    //     .expect("dsnt").decode().expect("hoed");

    let mut timer = Timer::new();

    let img = IR::open("/home/cody/img/collections/janitor-pics/14_cracked_stones.png")
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?
        .into_rgba8();

    let imgw = img.width();
    let imgh = img.height();

    println!("Image: {:?}ms", timer.elapsed());

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("editimg", 512, 512)
        .resizable()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    println!("Window: {:?}ms", timer.elapsed());

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA32, imgw, imgh)
        .map_err(|e| e.to_string())?;
    texture.update(None, &img, 4 * imgw as usize).map_err(|e| e.to_string())?;

    println!("Texture: {:?}ms", timer.elapsed());

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::Window{ win_event: WindowEvent::Resized(winw, winh), .. } => {
                    println!("Resized: ({}, {})", winw, winh);
                    timer.checkpoint();
                    canvas.clear();
                    let winw = winw.max(0).unsigned_abs();
                    let winh = winh.max(0).unsigned_abs();
                    let (x, y, w, h) = resize_dims(imgw, imgh, winw, winh);
                    canvas.copy(&texture, None, Some(Rect::new(x, y, w, h)))?;
                    canvas.present();
                    println!("Resizing: {:?}ms", timer.elapsed());
                },
                Event::KeyDown { .. } => {
                    println!("yeet");
                },
                Event::MouseButtonDown{ mouse_btn: MouseButton::Left, clicks: 1, x, y, .. } => {
                    println!("Click: ({}, {})", x, y);
                },
                _ => {}
            }
        }
    }

    Ok(())
}

struct Timer{
    time: Instant,
    prev: u128,
}

impl Timer{
    fn new() -> Self{
        let time = Instant::now();
        let prev = time.elapsed().as_millis();
        Self{ time, prev }
    }

    fn elapsed(&self) -> u128{
        self.time.elapsed().as_millis() - self.prev
    }

    fn checkpoint(&mut self){
        self.prev = self.time.elapsed().as_millis();
    }
}

fn resize_dims(imgw: u32, imgh: u32, winw: u32, winh: u32) -> (i32, i32, u32, u32){
    let wfac = winw as f32 / imgw as f32;
    let hfac = winh as f32 / imgh as f32;
    let fac = wfac.min(hfac);
    let w = (imgw as f32 * fac) as u32;
    let h = (imgh as f32 * fac) as u32;
    let x = if w < winw - 2 { (winw - w) / 2 } else { 0 } as i32;
    let y = if h < winh - 2 { (winh - h) / 2 } else { 0 } as i32;
    (x, y, w, h)
}

#[cfg(test)]
mod tests{

    use super::*;

    #[test]
    fn test_resize_dims(){
        let (x, y, w, h) = resize_dims(100, 100, 100, 100);
        assert_eq!((x, y, w, h), (0, 0, 100, 100));

        let (x, y, w, h) = resize_dims(50, 50, 100, 100);
        assert_eq!((x, y, w, h), (0, 0, 100, 100));

        let (x, y, w, h) = resize_dims(200, 200, 100, 100);
        assert_eq!((x, y, w, h), (0, 0, 100, 100));

        let (x, y, w, h) = resize_dims(100, 50, 100, 100);
        assert_eq!((x, y, w, h), (0, 25, 100, 50));

        let (x, y, w, h) = resize_dims(50, 100, 100, 100);
        assert_eq!((x, y, w, h), (25, 0, 50, 100));
    }
}
