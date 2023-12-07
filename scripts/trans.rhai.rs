
while true {
    let e = get_input_event();
    if e.is_click { continue; }
    if e.key == "termination" {
        break;
    } else if e.key == 'o' {
        rotate90(0);
    } else if e.key == 'a' {
        rotate270(0);
    } else if e.key == 'e' {
        fliph(0);
    } else if e.key == 'u' {
        flipv(0);
    } else if e.key == "return" {
        save(0, "transformed.jpg");
    }
}
