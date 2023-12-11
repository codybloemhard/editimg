
while true {
    let e = get_input_event();
    if e.is_click { continue; }
    if e.key == "termination" {
        break;
    }
    else if e.key == 'i' {
        invert(0, 0);
    }
    else if e.key == 'g' {
        blur(0, 0, 1.0);
    }
    else if e.key == 'u' {
        unsharpen(0, 0, 1.0, 1);
    }
    else if e.key == 'c' && !e.shift {
        adjust_contrast(0, 0, 4.0);
    }
    else if e.key == 'c' && e.shift {
        adjust_contrast(0, 0, -4.0);
    }
    else if e.key == 'b' && !e.shift {
        brighten(0, 0, 4);
    }
    else if e.key == 'b' && e.shift {
        brighten(0, 0, -4);
    }
    else if e.key == 'h' && !e.shift {
        huerotate(0, 0, 4);
    }
    else if e.key == 'h' && e.shift {
        huerotate(0, 0, -4);
    }
    else if e.key == "return" {
        save(0, "outp.jpg");
        print("haha");
        break;
    }
}

kill();

