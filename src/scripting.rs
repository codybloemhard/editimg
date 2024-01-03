use rhai::Engine;
use sdl2::keyboard::Mod;

use std::{
    sync::mpsc,
    io::Write,
};

#[derive(Debug, Clone)]
pub enum HostMsg {
    Kill,
    GetInputEvent,
    GetWH,
    ClearRects,
    DrawRectUV(RectUV),
    DrawRectXY(RectXY),
    Crop(i64, i64, i64, i64, i64, i64),
    Save(i64, String),
    FlipH(i64, i64),
    FlipV(i64, i64),
    Rot90(i64, i64),
    Rot180(i64, i64),
    Rot270(i64, i64),
    Invert(i64, i64),
    Grayscale(i64, i64),
    Blur(i64, i64, f64),
    Unsharpen(i64, i64, f64, i64),
    Filter3x3(i64, i64, [f64; 9]),
    AdjustContrast(i64, i64, f64),
    Brighten(i64, i64, i64),
    Huerotate(i64, i64, i64),
    Resize(i64, i64, i64, i64, String),
    ResizeExact(i64, i64, i64, i64, String),
    ResizeFill(i64, i64, i64, i64, String),
    Thumbnail(i64, i64, i64, i64),
    ThumbnailExact(i64, i64, i64, i64),
    Show(i64),
    ShowNext,
    ShowPrev,
    Create(i64, i64),
    Copy(i64, i64, i64, i64),
}

#[derive(Debug, Clone)]
pub struct RectUV {
    pub px: f32,
    pub py: f32,
    pub qx: f32,
    pub qy: f32,
}

impl RectUV {
    fn new(px: f32, py: f32, qx: f32, qy: f32) -> Self {
        Self{ px, py, qx, qy }
    }
}

#[derive(Debug, Clone)]
pub struct RectXY {
    pub px: i32,
    pub py: i32,
    pub qx: i32,
    pub qy: i32,
}

impl RectXY {
    fn new(px: i32, py: i32, qx: i32, qy: i32) -> Self {
        Self{ px, py, qx, qy }
    }
}

#[derive(Debug, Clone)]
pub enum RhaiMsg {
    Killed,
    Input(Input),
    Int(i64),
}

#[derive(Debug, Clone)]
pub struct Input {
    pub is_click: bool,
    pub key: String,
    pub u: f32,
    pub v: f32,
    pub x: i32,
    pub y: i32,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub nummod: bool,
    pub capsmod: bool,
}

impl Input {
    pub fn click(c: (f32, f32, i32, i32), key: String) -> Self {
        Self {
            is_click: true, key, u: c.0, v: c.1, x: c.2, y: c.3,
            shift: false, control: false, alt: false, nummod: false, capsmod: false,
        }
    }

    pub fn key(key: String, keymod: Mod) -> Self {
        let shift = keymod.contains(Mod::LSHIFTMOD) || keymod.contains(Mod::RSHIFTMOD);
        let control = keymod.contains(Mod::LCTRLMOD) || keymod.contains(Mod::RCTRLMOD);
        let alt = keymod.contains(Mod::LALTMOD) || keymod.contains(Mod::RALTMOD);
        let nummod = keymod.contains(Mod::NUMMOD);
        let capsmod = keymod.contains(Mod::CAPSMOD);

        Self {
            is_click: false, key, u: 0.0, v: 0.0, x: 0, y: 0,
            shift, control, alt, nummod, capsmod,
        }
    }

    fn get_is_click(&mut self) -> bool { self.is_click }
    fn get_key(&mut self) -> String { self.key.clone() }
    fn get_u(&mut self) -> f64 { self.u as f64 }
    fn get_v(&mut self) -> f64 { self.v as f64 }
    fn get_x(&mut self) -> i64 { self.x as i64 }
    fn get_y(&mut self) -> i64 { self.y as i64 }
    fn get_shift(&mut self) -> bool { self.shift }
    fn get_control(&mut self) -> bool { self.control }
    fn get_alt(&mut self) -> bool { self.alt }
    fn get_nummod(&mut self) -> bool { self.nummod }
    fn get_capsmod(&mut self) -> bool { self.capsmod }
}

#[derive(Debug, Clone)]
pub struct WH {
    pub w: i64,
    pub h: i64,
}

impl WH {
    fn get_w(&mut self) -> i64 { self.w }
    fn get_h(&mut self) -> i64 { self.h }
}

pub struct HostPortals {
    pub to_host: mpsc::Sender<HostMsg>,
    pub from_host: spmc::Receiver<RhaiMsg>,
}

pub struct RhaiPortals {
    pub from_rhai: mpsc::Receiver<HostMsg>,
    pub to_rhai: spmc::Sender<RhaiMsg>,
}

pub fn create_channels() -> (HostPortals, RhaiPortals) {
    let (to_host, from_rhai) = mpsc::channel();
    let (to_rhai, from_host) = spmc::channel();
    (
        HostPortals {
            to_host, from_host,
        },
        RhaiPortals {
            from_rhai, to_rhai,
        }
    )
}

pub fn construct_rhai_engine(host_portals: HostPortals) -> Engine {
    let mut engine = Engine::new();
    engine.set_max_expr_depths(50, 50);
    engine.on_print(|msg| {
        print!("{msg}");
        let _ = std::io::stdout().flush();
    });

    let HostPortals{
        to_host, from_host,
    } = host_portals;

    engine.register_type_with_name::<Input>("Input")
        .register_get("is_click", Input::get_is_click)
        .register_get("key", Input::get_key)
        .register_get("u", Input::get_u)
        .register_get("v", Input::get_v)
        .register_get("x", Input::get_x)
        .register_get("y", Input::get_y)
        .register_get("shift", Input::get_shift)
        .register_get("control", Input::get_control)
        .register_get("alt", Input::get_alt)
        .register_get("nummod", Input::get_nummod)
        .register_get("capsmod", Input::get_capsmod);
    engine.register_type_with_name::<WH>("WH")
        .register_get("w", WH::get_w)
        .register_get("h", WH::get_h);

    let receive_err = "Editimg: rhai thread could not receive from host.";
    let send_err = "Editimg: rhai thread could not send to host.";

    macro_rules! def_clones {
        ( $clonee:ident, $( $name:ident ), * ) => { $( let $name = $clonee.clone(); )* }
    }
    def_clones!( to_host,
        th_input, th_ruv, th_rxy, th_clear, th_wh, th_crop, th_save, th_fliph, th_flipv, th_rot90,
        th_rot180, th_rot270, th_invert, th_grayscale, th_blur, th_unsharpen, th_filter3x3,
        th_adjust_contrast, th_brighten, th_huerotate, th_resize, th_resize_exact, th_resize_fill,
        th_thumbnail, th_thumbnail_exact, th_show, th_show_next, th_show_prev, th_create,
        th_copy
    );
    def_clones!( from_host,
        fh_input, fh_wh, fh_crop, fh_fliph, fh_flipv, fh_rotate90, fh_rotate180, fh_rotate270,
        fh_invert, fh_grayscale, fh_blur, fh_unsharpen, fh_filter, fh_contrast, fh_brighten,
        fh_huerotate, fh_resize, fh_resize_exact, fh_resize_fill, fh_thumbnail, fh_thumbnail_exact,
        fh_show, fh_show_next, fh_show_prev, fh_create, fh_copy
    );

    macro_rules! recv_buf {
        ($fh: ident) => {
            if let RhaiMsg::Int(i) = $fh.recv().expect(receive_err) { i }
            else { quit("Editimg: rhai thread expected crop buffer but received otherwise."); }
        }
    }

    use HostMsg::*;

    engine
        .register_fn("kill", move || {
            to_host.send(Kill).expect(send_err);
        })
        .register_fn("clear_rects", move || {
            th_clear.clone().send(ClearRects).expect(send_err);
        })
        .register_fn("draw_rect_uv", move |px: f64, py: f64, qx: f64, qy: f64| {
            th_ruv.send(DrawRectUV(RectUV::new(px as f32, py as f32, qx as f32, qy as f32)))
                .expect(send_err);
        })
        .register_fn("draw_rect_xy", move |px: i64, py: i64, qx: i64, qy: i64| {
            th_rxy.send(DrawRectXY(RectXY::new(px as i32, py as i32, qx as i32, qy as i32)))
                .expect(send_err);
        })
        .register_fn("get_input_event", move || -> Input {
            th_input.send(GetInputEvent).expect(send_err);
            if let RhaiMsg::Input(input) = fh_input.recv().expect(receive_err) {
                input
            } else {
                quit("Editimg: rhai thread expected input but received otherwise.");
            }
        })
        .register_fn("get_wh", move || -> WH {
            th_wh.send(GetWH).expect(send_err);
            let w = if let RhaiMsg::Int(i) = fh_wh.recv().expect(receive_err) {
                i
            } else {
                quit("Editimg: rhai thread expected width but received otherwise.");
            };
            let h = if let RhaiMsg::Int(i) = fh_wh.recv().expect(receive_err) {
                i
            } else {
                quit("Editimg: rhai thread expected height but received otherwise.");
            };
            WH {
                w, h
            }
        })
        .register_fn("crop", move |s: i64, d: i64, px: i64, py: i64, qx: i64, qy: i64| -> i64 {
            th_crop.send(Crop(s, d, px, py, qx, qy)).expect(send_err);
            recv_buf!(fh_crop)
        })
        .register_fn("save", move |s: i64, p: String| {
            th_save.send(Save(s, p)).expect(send_err);
        })
        .register_fn("fliph", move |s: i64, d: i64| {
            th_fliph.send(FlipH(s, d)).expect(send_err);
            recv_buf!(fh_fliph)
        })
        .register_fn("flipv", move |s: i64, d: i64| {
            th_flipv.send(FlipV(s, d)).expect(send_err);
            recv_buf!(fh_flipv)
        })
        .register_fn("rotate90", move |s: i64, d: i64| {
            th_rot90.send(Rot90(s, d)).expect(send_err);
            recv_buf!(fh_rotate90)
        })
        .register_fn("rotate180", move |s: i64, d: i64| {
            th_rot180.send(Rot180(s, d)).expect(send_err);
            recv_buf!(fh_rotate180)
        })
        .register_fn("rotate270", move |s: i64, d: i64| {
            th_rot270.send(Rot270(s, d)).expect(send_err);
            recv_buf!(fh_rotate270)
        })
        .register_fn("invert", move |s: i64, d: i64| {
            th_invert.send(Invert(s, d)).expect(send_err);
            recv_buf!(fh_invert)
        })
        .register_fn("grayscale", move |s: i64, d: i64| {
            th_grayscale.send(Grayscale(s, d)).expect(send_err);
            recv_buf!(fh_grayscale)
        })
        .register_fn("blur", move |s: i64, d: i64, sigma: f64| {
            th_blur.send(Blur(s, d, sigma)).expect(send_err);
            recv_buf!(fh_blur)
        })
        .register_fn("unsharpen", move |s: i64, d: i64, sigma: f64, threshold: i64| {
            th_unsharpen.send(Unsharpen(s, d, sigma, threshold)).expect(send_err);
            recv_buf!(fh_unsharpen)
        })
        .register_fn("filter3x3", move |s: i64, d: i64, filter: [f64; 9]| {
            th_filter3x3.send(Filter3x3(s, d, filter)).expect(send_err);
            recv_buf!(fh_filter)
        })
        .register_fn("adjust_contrast", move |s: i64, d: i64, c: f64| {
            th_adjust_contrast.send(AdjustContrast(s, d, c)).expect(send_err);
            recv_buf!(fh_contrast)
        })
        .register_fn("brighten", move |s: i64, d: i64, v: i64| {
            th_brighten.send(Brighten(s, d, v)).expect(send_err);
            recv_buf!(fh_brighten)
        })
        .register_fn("huerotate", move |s: i64, d: i64, v: i64| {
            th_huerotate.send(Huerotate(s, d, v)).expect(send_err);
            recv_buf!(fh_huerotate)
        })
        .register_fn("resize", move |s: i64, d: i64, w: i64, h: i64, f: String| {
            th_resize.send(Resize(s, d, w, h, f)).expect(send_err);
            recv_buf!(fh_resize)
        })
        .register_fn("resize_exact", move |s: i64, d: i64, w: i64, h: i64, f: String| {
            th_resize_exact.send(ResizeExact(s, d, w, h, f)).expect(send_err);
            recv_buf!(fh_resize_exact)
        })
        .register_fn("resize_fill", move |s: i64, d: i64, w: i64, h: i64, f: String| {
            th_resize_fill.send(ResizeFill(s, d, w, h, f)).expect(send_err);
            recv_buf!(fh_resize_fill)
        })
        .register_fn("thumbnail", move |s: i64, d: i64, w: i64, h: i64| {
            th_thumbnail.send(Thumbnail(s, d, w, h)).expect(send_err);
            recv_buf!(fh_thumbnail)
        })
        .register_fn("thumbnail_exact", move |s: i64, d: i64, w: i64, h: i64| {
            th_thumbnail_exact.send(ThumbnailExact(s, d, w, h)).expect(send_err);
            recv_buf!(fh_thumbnail_exact)
        })
        .register_fn("show", move |i: i64| {
            th_show.send(Show(i)).expect(send_err);
            recv_buf!(fh_show)
        })
        .register_fn("show_next", move || {
            th_show_next.send(ShowNext).expect(send_err);
            recv_buf!(fh_show_next)
        })
        .register_fn("show_prev", move || {
            th_show_prev.send(ShowPrev).expect(send_err);
            recv_buf!(fh_show_prev)
        })
        .register_fn("create", move |w: i64, h: i64| {
            th_create.send(Create(w, h)).expect(send_err);
            recv_buf!(fh_create)
        })
        .register_fn("copy", move |s: i64, d: i64, x: i64, y: i64| {
            th_copy.send(Copy(s, d, x, y)).expect(send_err);
            if let RhaiMsg::Int(i) = fh_copy.recv().expect(receive_err) {
                i == 1
            } else {
                quit("Editimg: rhai thread expected crop buffer but received otherwise.");
            }
        })
    ;

    engine
}

fn quit(msg: &str) -> ! {
    println!("{msg}");
    std::process::exit(0)
}

