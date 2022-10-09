use rhai::Engine;

use std::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostMsg{
    Kill,
    GetMouseClick,
    DebugI64(i64),
    DrawRect(i64, i64, i64, i64),
}

#[derive(Debug, Clone)]
pub struct MouseClick{
    pub x: i64,
    pub y: i64,
}

impl MouseClick{
    fn get_x(&mut self) -> i64 { self.x }
    fn get_y(&mut self) -> i64 { self.y }
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
        .register_fn("draw_rect", move |px: i64, py: i64, qx: i64, qy: i64| {
            th2.send(HostMsg::DrawRect(px, py, qx, qy)).expect(send_err);
        })
        .register_fn("get_mouse_click", move || -> MouseClick {
            th1.send(HostMsg::GetMouseClick).expect(send_err);
            mc_from_host.recv().expect(receive_err)
        })
    ;

    engine
}
