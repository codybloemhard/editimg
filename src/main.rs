use show_image::{ImageView, ImageInfo, create_window, event};
use image::{open};
use image::io::Reader as IR;
use std::time::Instant;

#[show_image::main]
fn main(){
    let start = Instant::now();
    let ta = start.elapsed().as_millis();

    let pxs = IR::open("/home/cody/img/collections/cltracer/q-bright-sky-rough-copper.png")
        .expect("dsnt").decode().expect("hoed");

    println!("{:?}", start.elapsed().as_millis() - ta);

    let window = create_window("image", Default::default()).expect("oof");
    window.set_image("image-001", pxs).expect("auw");

    println!("{:?}", start.elapsed().as_millis() - ta);

    for event in window.event_channel().map_err(|e| e.to_string()).expect("rip") {
        if let event::WindowEvent::KeyboardInput(event) = event {
            if !event.is_synthetic
                && event.input.key_code == Some(event::VirtualKeyCode::Escape)
                && event.input.state.is_pressed() {
                println!("Escape pressed!");
                break;
            }
            if !event.is_synthetic
                && event.input.key_code == Some(event::VirtualKeyCode::A)
                && event.input.state.is_pressed() {
                let pxs = open("/home/cody/img/collections/cltracer/t-microfacets-dielectrics-conductors.png")
                    .unwrap();
                window.set_image("image-001", pxs).expect("auw");
            }
            if !event.is_synthetic
                && event.input.key_code == Some(event::VirtualKeyCode::E)
                && event.input.state.is_pressed() {
                let pxs = open("/home/cody/img/collections/cltracer/q-bright-sky-rough-copper.png")
                    .unwrap();
                window.set_image("image-001", pxs).expect("auw");
            }
        }
    }
}

