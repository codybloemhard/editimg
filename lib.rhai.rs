fn select(step, square) {
    let wh = get_wh();
    let w = wh.w;
    let h = wh.h;
    let px = w / 2;
    let py = h / 2;
    let qx = w / 2;
    let qy = h / 2;
    if step == 0 {
        step = min(w, h) / 32;
    } else {
        step /= 2;
    }

    while true {
        let opx = px;
        let opy = py;
        let oqx = qx;
        let oqy = qy;
        let e = get_input_event();
        if e.key == "termination" { break; }
        if e.is_click {
            if e.key == "left" && !square {
                px = e.x;
                py = e.y;
            } else if e.key == "right" && !square {
                qx = e.x;
                qy = e.y;
            }
        } else if e.key == "return" {
            break;
        }
        else if e.key == "s" { // move left
            px -= step;
            qx -= step;
        }
        else if e.key == "n" { // move right
            px += step;
            qx += step;
        }
        else if e.key == "m" { // move down
            py -= step;
            qy -= step;
        }
        else if e.key == "t" { // move up
            py += step;
            qy += step;
        }
        else if square {
            if e.key == "e" { // shrink width and height
                let gap = min(qx - px, qy - py);
                if gap < 2 * step {
                    px = (px + qx) / 2;
                    qx = px;
                    py = (py + qy) / 2;
                    qy = py;
                } else {
                    px += step;
                    qx -= step;
                    py += step;
                    qy -= step;
                }
            } else if e.key == "u" { // grow width and height
                px -= step;
                qx += step;
                py -= step;
                qy += step;
            }
        }
        else {
            if e.key == "a" { // shrink width
                if qx - px < 2 * step {
                    px = (px + qx) / 2;
                    qx = px;
                } else {
                    px += step;
                    qx -= step;
                }
            }
            else if e.key == "o" { // grow width
                px -= step;
                qx += step;
            }
            else if e.key == "u" { // grow height
                py -= step;
                qy += step;
            }
            else if e.key == "e" { // shrink height
                if qy - py < 2 * step {
                    py = (py + qy) / 2;
                    qy = py;
                } else {
                    py += step;
                    qy -= step;
                }
            }
        }
        if px != opx || py != opy || qx != oqx || qy != oqy {
            clear_rects();
            draw_rect_xy(px, py, qx, qy);
        }
    }
    return [px, py, qx, qy];
}

