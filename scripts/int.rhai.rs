
print("[NORMAL]\n");
while true {
    let e = get_input_event();
    if e.key == "termination" { break; }
    if e.key == "a" {
        show_prev();
        continue;
    } else if e.key == "o" {
        show_next();
        continue;
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
    else if e.key == "quote" {
        print("[HELP]\n");
        print("a_prev, o_next, s_ave, f_x\n");
    }
    // print(`${e.key}`);
}

kill();

fn src() { return nat_num_input("src: "); }
fn dst() { return nat_num_input("dst: "); }

fn handle_fx() {
    print("[FX]");
    let e = get_input_event();
    if e.is_click { return; }
    if e.key == "termination" { return; }
    else if e.key == 'i' {
        print("[INVERT]\n");
        invert(src(), dst());
    } else if e.key == 'g' && !e.shift {
        print("[GRAYSCALE]\n");
        grayscale(src(), dst());
    } else if e.key == 'g' && e.shift {
        print("[GAUSSIAN]\n");
        blur(src(), dst(), 1.0);
    } else if e.key == 'u' {
        print("[UNSHARPEN]\n");
        unsharpen(src(), dst(), 1.0, nat_num_input("threshold: "));
    } else if e.key == 'c' {
        print("[CONTRAST]\n");
        adjust_contrast(src(), dst(), 4.0);
    } else if e.key == 'b' {
        print("[BRIGHTEN]\n");
        brighten(src(), dst(), nat_num_input("val: "));
    } else if e.key == 'h' {
        print("[HUEROTATE]\n");
        huerotate(src(), dst(), nat_num_input("val: "));
    } else if e.key == "quote" {
        print("[HELP]\n");
        print("i_nvert, g_rayscale, G_aussian, u_nsharpen, c_ontrast, b_righten, h_uerotate\n");
    }
}

