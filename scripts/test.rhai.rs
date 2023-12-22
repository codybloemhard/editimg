thumbnail(0, -1, 64, 64);

while true {
    let e = get_input_event();
    if e.key == "termination" { break; }
    if e.key == "return" { break; }
    else if e.key == "left" {
        show_prev();
    }
    else if e.key == "right" {
        show_next();
    }
}

kill();

