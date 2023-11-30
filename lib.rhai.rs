fn select() {
    let wh = get_wh();
    let w = wh.w;
    let h = wh.h;
    let px = w/2;
    let py = h/2;
    let qx = w/2;
    let qy = h/2;
    while true{
        let opx = px;
        let opy = py;
        let oqx = qx;
        let oqy = qy;
        let e = get_input_event();
        if e.key == "termination" { break; }
        if e.is_click {
            print(`  Script Read: ${e.x}, ${e.y}`);
            if e.key == "left" {
                px = e.x;
                py = e.y;
            } else if e.key == "right" {
                qx = e.x;
                qy = e.y;
            }
        } else if e.key == "return"{
            break;
        }
        else if e.key == "s"{
            px -= 10;
            qx -= 10;
        }
        else if e.key == "n"{
            px += 10;
            qx += 10;
        }
        else if e.key == "m"{
            py -= 10;
            qy -= 10;
        }
        else if e.key == "t"{
            py += 10;
            qy += 10;
        }
        else if e.key == "a"{
            px += 10;
            qx -= 10;
        }
        else if e.key == "o"{
            px -= 10;
            qx += 10;
        }
        else if e.key == "u"{
            py -= 10;
            qy += 10;
        }
        else if e.key == "e"{
            py += 10;
            qy -= 10;
        }
        if px != opx || py != opy || qx != oqx || qy != oqy{
            clear_rects();
            draw_rect_xy(px, py, qx, qy);
        }
    }
    return [px, py, qx, qy];
}

