use crate::{
    timer::Timer,
    scripting::MouseClick,
};

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
    rects: Vec<(f32, f32, f32, f32)>,
    winw: u32,
    winh: u32,
    imgx: i32,
    imgy: i32,
    imgw: u32,
    imgh: u32,
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
        Ok((
            Self{
                canvas,
                texture_creator,
                texture: None,
                rects: Vec::new(),
                winw: 1,
                winh: 1,
                imgx: 0,
                imgy: 0,
                imgw: 1,
                imgh: 1,
            },
            event_pump
        ))
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

    pub fn screen_to_click(&self, x: i32, y: i32) -> MouseClick{
        let u = (x as f32 - self.imgx as f32) / self.imgw as f32;
        let v = (y as f32 - self.imgy as f32) / self.imgh as f32;
        MouseClick{
            u, v,
            x: (u * self.imgw as f32) as i32,
            y: (v * self.imgh as f32) as i32,
        }
    }

    pub fn redraw(&mut self){
        self.canvas.present();
    }

    pub fn resize_redraw(&mut self, winw: u32, winh: u32) -> Result<(), String>{
        self.winw = winw;
        self.winh = winh;
        self.draw_texture(winw, winh)?;
        let rects = std::mem::take(&mut self.rects);
        for (px, py, qx, qy) in &rects{
            self._draw_rect(*px, *py, *qx, *qy)?;
        }
        self.rects = rects;
        self.redraw();
        Ok(())
    }

    pub fn draw_texture(&mut self, winw: u32, winh: u32) -> Result<(), String>{
        self.canvas.clear();
        if let Some((texture, imgw, imgh)) = &self.texture{
            let (x, y, w, h) = resize_dims(*imgw, *imgh, winw, winh);
            self.canvas.copy(texture, None, Some(Rect::new(x, y, w, h)))?;
            self.imgx = x;
            self.imgy = y;
            self.imgw = w;
            self.imgh = h;
            Ok(())
        } else {
            Err("Editimg error: window redraw with no valid texture available.".to_string())
        }
    }

    pub fn draw_rect_xy(&mut self, px: i32, py: i32, qx: i32, qy: i32) -> Result<(), String>{
        let px = px as f32 / self.imgw as f32;
        let py = py as f32 / self.imgh as f32;
        let qx = qx as f32 / self.imgw as f32;
        let qy = qy as f32 / self.imgh as f32;
        self.draw_rect_uv(px, py, qx, qy)
    }

    pub fn draw_rect_uv(&mut self, px: f32, py: f32, qx: f32, qy: f32) -> Result<(), String>{
        self.rects.push((px, py, qx, qy));
        self._draw_rect(px, py, qx, qy)
    }

    pub fn _draw_rect(&mut self, px: f32, py: f32, qx: f32, qy: f32) -> Result<(), String>{
        let dc = self.canvas.draw_color();
        let px = (px * self.imgw as f32 + self.imgx as f32) as i32;
        let py = (py * self.imgh as f32 + self.imgy as f32) as i32;
        let qx = (qx * self.imgw as f32 + self.imgx as f32) as i32;
        let qy = (qy * self.imgh as f32 + self.imgy as f32) as i32;
        let draw_point_box = |skip: usize, canvas: &mut Canvas<Window>| -> Result<(), String>{
            let t = (px..qx).into_iter().skip(skip).step_by(2)
                .map(|x| Point::new(x, py)).collect::<Vec<_>>();
            let b = (px..qx).into_iter().skip(skip)
                .step_by(2).map(|x| Point::new(x, qy)).collect::<Vec<_>>();
            let l = (py..qy).into_iter().skip(skip)
                .step_by(2).map(|y| Point::new(px, y)).collect::<Vec<_>>();
            let r = (py..qy).into_iter().skip(skip)
                .step_by(2).map(|y| Point::new(qx, y)).collect::<Vec<_>>();
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
