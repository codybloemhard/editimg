use rhai::Engine;
use sdl2::keyboard::Mod;

use std::sync::mpsc;

#[derive(Debug, Clone)]
pub enum HostMsg {
    Kill,
    GetInputEvent,
    GetWH,
    DebugI64(i64),
    ClearRects,
    DrawRectUV(RectUV),
    DrawRectXY(RectXY),
    Crop(i64, i64, i64, i64, i64, i64),
    Save(i64, String),
    FlipH(i64),
    FlipV(i64),
    Rot90(i64),
    Rot180(i64),
    Rot270(i64),
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

impl RectXY{
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

    let th_debug = to_host.clone();
    let th_input = to_host.clone();
    let th_ruv = to_host.clone();
    let th_rxy = to_host.clone();
    let th_clear = to_host.clone();
    let th_wh = to_host.clone();
    let th_crop = to_host.clone();
    let th_save = to_host.clone();
    let th_fliph = to_host.clone();
    let th_flipv = to_host.clone();
    let th_rot90 = to_host.clone();
    let th_rot180 = to_host.clone();
    let th_rot270 = to_host.clone();

    let fh_input = from_host.clone();
    let fh_wh = from_host.clone();
    let fh_crop = from_host.clone();

    engine
        .register_fn("kill", move || {
            to_host.send(HostMsg::Kill).expect(send_err);
        })
        .register_fn("put", move |v: i64| {
            th_debug.send(HostMsg::DebugI64(v)).expect(send_err);
        })
        .register_fn("clear_rects", move || {
            th_clear.clone().send(HostMsg::ClearRects).expect(send_err);
        })
        .register_fn("draw_rect_uv", move |px: f64, py: f64, qx: f64, qy: f64| {
            th_ruv.send(HostMsg::DrawRectUV(RectUV::new(px as f32, py as f32, qx as f32, qy as f32)))
                .expect(send_err);
        })
        .register_fn("draw_rect_xy", move |px: i64, py: i64, qx: i64, qy: i64| {
            th_rxy.send(HostMsg::DrawRectXY(RectXY::new(px as i32, py as i32, qx as i32, qy as i32)))
                .expect(send_err);
        })
        .register_fn("get_input_event", move || -> Input {
            th_input.send(HostMsg::GetInputEvent).expect(send_err);
            if let RhaiMsg::Input(input) = fh_input.recv().expect(receive_err) {
                input
            } else {
                quit("Editimg: rhai thread expected input but received otherwise.");
            }
        })
        .register_fn("get_wh", move || -> WH {
            th_wh.send(HostMsg::GetWH).expect(send_err);
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
            th_crop.send(HostMsg::Crop(s, d, px, py, qx, qy)).expect(send_err);
            if let RhaiMsg::Int(i) = fh_crop.recv().expect(receive_err) {
                i
            } else {
                quit("Editimg: rhai thread expected crop buffer but received otherwise.");
            }
        })
        .register_fn("save", move |s: i64, p: String| {
            th_save.send(HostMsg::Save(s, p)).expect(send_err);
        })
        .register_fn("fliph", move |s: i64| {
            th_fliph.send(HostMsg::FlipH(s)).expect(send_err);
        })
        .register_fn("flipv", move |s: i64| {
            th_flipv.send(HostMsg::FlipV(s)).expect(send_err);
        })
        .register_fn("rotate90", move |s: i64| {
            th_rot90.send(HostMsg::Rot90(s)).expect(send_err);
        })
        .register_fn("rotate180", move |s: i64| {
            th_rot180.send(HostMsg::Rot180(s)).expect(send_err);
        })
        .register_fn("rotate270", move |s: i64| {
            th_rot270.send(HostMsg::Rot270(s)).expect(send_err);
        })
    ;

    engine
}

fn quit(msg: &str) -> ! {
    println!("{msg}");
    std::process::exit(0)
}

