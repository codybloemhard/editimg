use crate::timer::Timer;

use sdl2::{
    EventPump,
    video::{ Window, WindowContext },
    render::{ Canvas, TextureCreator, Texture },
    pixels::{ PixelFormatEnum, Color },
    rect::{ Rect, Point },
};

use image::io::Reader as IR;

pub struct EIWindow{
    pub canvas: Canvas<Window>,
    pub texture_creator: TextureCreator<WindowContext>,
    pub texture: Option<(Texture, u32, u32)>,
}

impl EIWindow{
    pub fn create(timer: &Timer) -> Result<(Self, EventPump), String>{
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("editimg", 512, 512)
            .resizable()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();
        let event_pump = sdl_context.event_pump()?;

        println!("Window: {:?}ms", timer.elapsed());
        Ok((Self{canvas, texture_creator, texture: None}, event_pump))
    }

    pub fn set_texture(&mut self, file: &str, timer: &Timer) -> Result<(), String>
    {
        let img = IR::open(file)
            .map_err(|e| e.to_string())?
            .decode()
            .map_err(|e| e.to_string())?
            .into_rgba8();

        let imgw = img.width();
        let imgh = img.height();

        println!("Image: {:?}ms", timer.elapsed());

        let mut texture = self
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, imgw, imgh)
            .map_err(|e| e.to_string())?;
        texture.update(None, &img, 4 * imgw as usize).map_err(|e| e.to_string())?;
        self.texture = Some((texture, imgw, imgh));

        println!("Texture: {:?}ms", timer.elapsed());

        Ok(())
    }

    pub fn redraw(&mut self){
        self.canvas.present();
    }

    pub fn draw_texture(&mut self, winw: u32, winh: u32) -> Result<(), String>{
        self.canvas.clear();
        if let Some((texture, imgw, imgh)) = &self.texture{
            let (x, y, w, h) = resize_dims(*imgw, *imgh, winw, winh);
            self.canvas.copy(texture, None, Some(Rect::new(x, y, w, h)))?;
            Ok(())
        } else {
            Err("Editimg error: window redraw with no valid texture available.".to_string())
        }
    }

    pub fn draw_rect(&mut self, px: i64, py: i64, qx: i64, qy: i64) -> Result<(), String>{
        let dc = self.canvas.draw_color();
        let draw_point_box = |skip: usize, canvas: &mut Canvas<Window>| -> Result<(), String>{
            let t = (px..qx).into_iter().skip(skip).step_by(2)
                .map(|x| Point::new(x as i32, py as i32)).collect::<Vec<_>>();
            let b = (px..qx).into_iter().skip(skip)
                .step_by(2).map(|x| Point::new(x as i32, qy as i32)).collect::<Vec<_>>();
            let l = (py..qy).into_iter().skip(skip)
                .step_by(2).map(|y| Point::new(px as i32, y as i32)).collect::<Vec<_>>();
            let r = (py..qy).into_iter().skip(skip)
                .step_by(2).map(|y| Point::new(qx as i32, y as i32)).collect::<Vec<_>>();
            canvas.draw_points(t.as_slice())?;
            canvas.draw_points(b.as_slice())?;
            canvas.draw_points(l.as_slice())?;
            canvas.draw_points(r.as_slice())?;
            Ok(())
        };
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        draw_point_box(0, &mut self.canvas)?;
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        draw_point_box(1, &mut self.canvas)?;
        self.canvas.set_draw_color(dc);
        Ok(())
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
