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
    let bstep = step;

    while true {
        let opx = px;
        let opy = py;
        let oqx = qx;
        let oqy = qy;
        let e = get_input_event();
        if e.key == "termination" { break; }
        if e.shift {
            step = 1;
        } else {
            step = bstep;
        }
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

fn nat_num_input() {
    let number = "0";
    print("natural input: 0");

    while true {
        let e = get_input_event();
        if e.key == "termination" { break; }
        if e.key == "num0" {
            number += "0";
            print("0");
        } else if e.key == "num1" {
            number += "1";
            print("1");
        } else if e.key == "num2" {
            number += "2";
            print("2");
        } else if e.key == "num3" {
            number += "3";
            print("3");
        } else if e.key == "num4" {
            number += "4";
            print("4");
        } else if e.key == "num5" {
            number += "5";
            print("5");
        } else if e.key == "num6" {
            number += "6";
            print("6");
        } else if e.key == "num7" {
            number += "7";
            print("7");
        } else if e.key == "num8" {
            number += "8";
            print("8");
        } else if e.key == "num9" {
            number += "9";
            print("9");
        } else if e.key == "backspace" && number.len() > 1{
            number.pop();
            print("\x08");
            print(" ");
            print("\x08");
        } else if e.key == "return" {
            break;
        }
    }

    let parsed = parse_int(number);
    let res = if type_of(parsed) == "i64" {
        print(" [parsed]");
        parsed
    } else {
        print(" [failed!]");
        -1
    };
    print("\n");

    return res;
}

fn kill_on(key) {
    while true {
        let e = get_input_event();
        if e.key == key {
            kill();
        }
    }
}

