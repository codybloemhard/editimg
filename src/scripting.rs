use rhai::Engine;

use std::sync::mpsc;

#[derive(Debug, Clone)]
pub enum HostMsg{
    Kill,
    GetInputEvent,
    DebugI64(i64),
    DrawRectUV(RectUV),
    DrawRectXY(RectXY),
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
    pub fn click(c: (f32, f32, i32, i32)) -> Self {
        Self { is_click: true, key: "".to_string(), u: c.0, v: c.1, x: c.2, y: c.3 }
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

pub struct HostPortals{
    pub to_host: mpsc::Sender<HostMsg>,
    pub mc_from_host: mpsc::Receiver<Input>,
}

pub struct RhaiPortals{
    pub mc_to_rhai: mpsc::Sender<Input>,
    pub from_rhai: mpsc::Receiver<HostMsg>,
}

pub fn create_channels() -> (HostPortals, RhaiPortals){
    let (to_host, from_rhai) = mpsc::channel();
    let (mc_to_rhai, mc_from_host) = mpsc::channel();
    (
        HostPortals{
            to_host, mc_from_host,
        },
        RhaiPortals{
            mc_to_rhai,
            from_rhai,
        }
    )
}

pub fn construct_rhai_engine(host_portals: HostPortals) -> Engine {
    let mut engine = Engine::new();

    let HostPortals{ to_host, mc_from_host } = host_portals;

    engine.register_type_with_name::<Input>("Input")
        .register_get("is_click", Input::get_is_click)
        .register_get("key", Input::get_key)
        .register_get("u", Input::get_u)
        .register_get("v", Input::get_v)
        .register_get("x", Input::get_x)
        .register_get("y", Input::get_y);

    let receive_err = "Editimg: rhai thread could not receive from host.";
    let send_err = "Editimg: rhai thread could not send to host.";
    let th0 = to_host.clone();
    let th1 = to_host.clone();
    let th2 = to_host.clone();
    let th3 = to_host.clone();

    engine
        .register_fn("kill", move || {
            to_host.send(HostMsg::Kill).expect(send_err);
        })
        .register_fn("put", move |v: i64| {
            th0.send(HostMsg::DebugI64(v)).expect(send_err);
        })
        .register_fn("draw_rect_uv", move |px: f64, py: f64, qx: f64, qy: f64| {
            th2.send(HostMsg::DrawRectUV(RectUV::new(px as f32, py as f32, qx as f32, qy as f32)))
                .expect(send_err);
        })
        .register_fn("draw_rect_xy", move |px: i64, py: i64, qx: i64, qy: i64| {
            th3.send(HostMsg::DrawRectXY(RectXY::new(px as i32, py as i32, qx as i32, qy as i32)))
                .expect(send_err);
        })
        .register_fn("get_input_event", move || -> Input {
            th1.send(HostMsg::GetInputEvent).expect(send_err);
            mc_from_host.recv().expect(receive_err)
        })
    ;

    engine
}
