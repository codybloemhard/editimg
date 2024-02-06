
print("[NORMAL]\n");
while true {
    let e = get_input_event();
    if e.key == "termination" { break; }
    if e.key == "lshift" || e.key == "rshift" {
        continue;
    }
    if e.key == "a" {
        show_prev();
    }
    else if e.key == "o" {
        show_next();
    }
    else if e.key == "s" {
        let image = src();
        let fname = file_input("file name: ");
        if fname != "" {
            save(image, fname);
        }
    }
    else if e.key == "f" {
        handle_fx();
    }
    else if e.key == "t" {
        handle_transform();
    }
    else if e.key == "c" {
        handle_crop();
    }
    else if e.key == "slash" && e.shift {
        print("[HELP]\n");
        print("a_prev, o_next, s_ave, f_x, t_ransform, c_rop, ._repeat\n");
    }
    else if e.key == "period" {
        print("[REPEAT]\n");
        repeat();
    }
    // print(`${e.key}`);
}

kill();

fn src() { return nat_num_input("src: "); }
fn dst() { return nat_num_input("dst: "); }

fn handle_fx() {
    print("[FX]");
    while true {
        let e = get_input_event();
        if e.is_click { return; }
        if e.key == "termination" { return; }
        else if e.key == "lshift" || e.key == "rshift" {
            continue;
        } else if e.key == 'i' {
            print("[INVERT]\n");
            invert(src(), dst());
        } else if e.key == 'g' && !e.shift {
            print("[GRAYSCALE]\n");
            grayscale(src(), dst());
        } else if e.key == 'g' && e.shift {
            print("[GAUSSIAN]\n");
            blur(src(), dst(), float_input("sigma: "));
        } else if e.key == 'u' {
            print("[UNSHARPEN]\n");
            unsharpen(src(), dst(), float_input("sigma: "), nat_num_input("threshold: "));
        } else if e.key == 'c' {
            print("[CONTRAST]\n");
            adjust_contrast(src(), dst(), float_input("contrast: "));
        } else if e.key == 'b' {
            print("[BRIGHTEN]\n");
            brighten(src(), dst(), nat_num_input("val: "));
        } else if e.key == 'h' {
            print("[HUEROTATE]\n");
            huerotate(src(), dst(), nat_num_input("val: "));
        } else if e.key == "slash" && e.shift {
            print("[HELP]\n");
            print("i_nvert, g_rayscale, G_aussian, u_nsharpen, c_ontrast, b_righten, h_uerotate\n");
            continue;
        }
        break;
    }
}

fn handle_transform() {
    print("[TRANSFORM]");
    while true {
        let e = get_input_event();
        if e.is_click { return; }
        if e.key == "termination" { return; }
        else if e.key == "lshift" || e.key == "rshift" {
            continue;
        } else if e.key == 'o' {
            print("[ROTATE90]\n");
            rotate90(0, 0);
        } else if e.key == 'a' {
            print("[ROTATE270]\n");
            rotate270(0, 0);
        } else if e.key == 'e' {
            print("[FLIPH]\n");
            fliph(0, 0);
        } else if e.key == 'u' {
            print("[FLIPV]\n");
            flipv(0, 0);
        } else if e.key == "slash" && e.shift {
            print("[HELP]\n");
            print("o_rotate90, a_rotate270, e_fliph, u_flipv\n");
            continue;
        }
        break;
    }
}

fn handle_crop() {
    print("[CROP]\n");
    let d = dst();
    let square = bool_input("Square selection y/n: ");

    let pq = select(0, square);
    crop(shown(), d, pq[0], pq[1], pq[2], pq[3]);
    show(d);
}

