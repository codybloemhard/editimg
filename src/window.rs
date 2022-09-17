use crate::timer::Timer;

use sdl2::{
    EventPump,
    video::{ Window, WindowContext },
    render::{ Canvas, TextureCreator },
};

pub struct EIWindow{
    pub canvas: Canvas<Window>,
    pub texture_creator: TextureCreator<WindowContext>,
    pub event_pump: EventPump,
}

impl EIWindow{
    pub fn create(timer: &Timer) -> Result<Self, String>{
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
        Ok(Self{
            canvas, event_pump, texture_creator
        })
    }
}

