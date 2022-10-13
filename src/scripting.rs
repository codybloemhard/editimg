use rhai::Engine;

use std::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum HostMsg{
    Kill,
    GetMouseClick,
    DebugI64(i64),
    DrawRect(f32, f32, f32, f32),
}

#[derive(Debug, Clone)]
pub struct MouseClick{
    pub x: f32,
    pub y: f32,
}

impl MouseClick{
    fn get_x(&mut self) -> f64 { self.x as f64 }
    fn get_y(&mut self) -> f64 { self.y as f64 }
}

pub struct HostPortals{
    pub to_host: mpsc::Sender<HostMsg>,
    pub mc_from_host: mpsc::Receiver<MouseClick>,
}

pub struct RhaiPortals{
    pub mc_to_rhai: mpsc::Sender<MouseClick>,
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

    engine.register_type_with_name::<MouseClick>("MouseClick")
        .register_get("x", MouseClick::get_x)
        .register_get("y", MouseClick::get_y);

    let receive_err = "Editimg: rhai thread could not receive from host.";
    let send_err = "Editimg: rhai thread could not send to host.";
    let th0 = to_host.clone();
    let th1 = to_host.clone();
    let th2 = to_host.clone();

    engine
        .register_fn("kill", move || {
            to_host.send(HostMsg::Kill).expect(send_err);
        })
        .register_fn("put", move |v: i64| {
            th0.send(HostMsg::DebugI64(v)).expect(send_err);
        })
        .register_fn("draw_rect", move |px: f64, py: f64, qx: f64, qy: f64| {
            th2.send(HostMsg::DrawRect(px as f32, py as f32, qx as f32, qy as f32))
                .expect(send_err);
        })
        .register_fn("get_mouse_click", move || -> MouseClick {
            th1.send(HostMsg::GetMouseClick).expect(send_err);
            mc_from_host.recv().expect(receive_err)
        })
    ;

    engine
}
