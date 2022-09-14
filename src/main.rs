use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    event::{ Event, WindowEvent, ElementState, KeyboardInput, VirtualKeyCode},
};
use pixels::{ Pixels, SurfaceTexture };

use image::io::Reader as IR;
use std::time::Instant;

fn main(){
    let start = Instant::now();
    let ta = start.elapsed().as_millis();

    let img = IR::open("/home/cody/img/collections/cltracer/q-bright-sky-rough-copper.png")
        .expect("bruh").decode().expect("oof");
    //let pxs = open("/home/cody/img/collections/cltracer/t-microfacets-dielectrics-conductors.png")

    println!("img: {:?}", start.elapsed().as_millis() - ta);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("editimg")
        .build(&event_loop)
        .expect("auw");
    let size = window.inner_size();

    println!("window: {:?}", start.elapsed().as_millis() - ta);

    let surface = SurfaceTexture::new(size.width, size.height, &window);
    let mut pixels = Pixels::new(img.width(), img.height(), surface).expect("");
    let image_bytes = img.to_rgba8();
    let image_bytes = image_bytes.as_flat_samples();
    let image_bytes = image_bytes.as_slice();
    let pixels_bytes = pixels.get_frame();
    pixels_bytes.iter_mut().enumerate().for_each(|(i, x)| *x = image_bytes[i]);
    pixels.render();

    println!("surface {:?}", start.elapsed().as_millis() - ta);

    event_loop.run(move |event ,_ , control_flow|{
        *control_flow = ControlFlow::Wait;
        match event{
            Event::WindowEvent { event, window_id } => if window_id == window.id() {
                match event{
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => {
                        pixels.resize_surface(size.width, size.height)
                    },
                    _ => {},
                }
            },
            Event::RedrawRequested { .. } => { pixels.render(); },
            _ => {}
        }
    });
}

