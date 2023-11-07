use rhai::Engine;

use std::sync::mpsc;

#[derive(Debug, Clone)]
pub enum HostMsg{
    Kill,
    GetInputEvent,
    GetWH,
    DebugI64(i64),
    ClearRects,
    DrawRectUV(RectUV),
    DrawRectXY(RectXY),
    Crop(i64, i64, i64, i64, i64, i64),
}

#[derive(Debug, Clone)]
pub struct RectUV{
    pub px: f32,
    pub py: f32,
    pub qx: f32,
    pub qy: f32,
}

impl RectUV{
    fn new(px: f32, py: f32, qx: f32, qy: f32) -> Self{
        Self{ px, py, qx, qy }
    }
}

#[derive(Debug, Clone)]
pub struct RectXY{
    pub px: i32,
    pub py: i32,
    pub qx: i32,
    pub qy: i32,
}

impl RectXY{
    fn new(px: i32, py: i32, qx: i32, qy: i32) -> Self{
        Self{ px, py, qx, qy }
    }
}

#[derive(Debug, Clone)]
pub struct Input{
    pub is_click: bool,
    pub key: String,
    pub u: f32,
    pub v: f32,
    pub x: i32,
    pub y: i32,
}

impl Input{
    pub fn click(c: (f32, f32, i32, i32), key: String) -> Self {
        Self { is_click: true, key, u: c.0, v: c.1, x: c.2, y: c.3 }
    }

    pub fn key(key: String) -> Self {
        Self { is_click: false, key, u: 0.0, v: 0.0, x: 0, y: 0 }
    }

    fn get_is_click(&mut self) -> bool { self.is_click }
    fn get_key(&mut self) -> String { self.key.clone() }
    fn get_u(&mut self) -> f64 { self.u as f64 }
    fn get_v(&mut self) -> f64 { self.v as f64 }
    fn get_x(&mut self) -> i64 { self.x as i64 }
    fn get_y(&mut self) -> i64 { self.y as i64 }
}

#[derive(Debug, Clone)]
pub struct WH{
    pub w: i64,
    pub h: i64,
}

impl WH{
    fn get_w(&mut self) -> i64 { self.w }
    fn get_h(&mut self) -> i64 { self.h }
}

pub struct HostPortals{
    pub to_host: mpsc::Sender<HostMsg>,
    pub input_from_host: mpsc::Receiver<Input>,
    pub int_from_host: mpsc::Receiver<i64>,
    pub crop_from_host: mpsc::Receiver<i64>,
}

pub struct RhaiPortals{
    pub from_rhai: mpsc::Receiver<HostMsg>,
    pub input_to_rhai: mpsc::Sender<Input>,
    pub int_to_rhai: mpsc::Sender<i64>,
    pub crop_to_rhai: mpsc::Sender<i64>,
}

pub fn create_channels() -> (HostPortals, RhaiPortals){
    let (to_host, from_rhai) = mpsc::channel();
    let (input_to_rhai, input_from_host) = mpsc::channel();
    let (int_to_rhai, int_from_host) = mpsc::channel();
    let (crop_to_rhai, crop_from_host) = mpsc::channel();
    (
        HostPortals{
            to_host, input_from_host, int_from_host, crop_from_host,
        },
        RhaiPortals{
            from_rhai, input_to_rhai, int_to_rhai, crop_to_rhai,
        }
    )
}

pub fn construct_rhai_engine(host_portals: HostPortals) -> Engine {
    let mut engine = Engine::new();

    let HostPortals{
        to_host, input_from_host, int_from_host, crop_from_host,
    } = host_portals;

    engine.register_type_with_name::<Input>("Input")
        .register_get("is_click", Input::get_is_click)
        .register_get("key", Input::get_key)
        .register_get("u", Input::get_u)
        .register_get("v", Input::get_v)
        .register_get("x", Input::get_x)
        .register_get("y", Input::get_y);
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
            input_from_host.recv().expect(receive_err)
        })
        .register_fn("get_wh", move || -> WH {
            th_wh.send(HostMsg::GetWH).expect(send_err);
            WH{
                w: int_from_host.recv().expect(receive_err),
                h: int_from_host.recv().expect(receive_err),
            }
        })
        .register_fn("crop", move |s: i64, d: i64, px: i64, py: i64, qx: i64, qy: i64| -> i64{
            th_crop.send(HostMsg::Crop(s, d, px, py, qx, qy)).expect(send_err);
            crop_from_host.recv().expect(receive_err)
        })
    ;

    engine
}
