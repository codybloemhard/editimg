
print("Stack two images on top of each other.\n");

let n = get_buffers_len();

print("Found ");
print(`${n}`);
print(" inputs.\n");

let max_w = 0;

for i in 0..n {
    let wh = get_wh(i);
    max_w = max_w.max(wh.w);
}

let height = 0;

for i in 0..n {
    let wh = get_wh(i);
    height += wh.h * (max_w.to_float() / wh.w.to_float());
}

height = height.ceiling().to_int();

let result = create(max_w, height);
let h = 0;

for i in 0..n {
    resize(i, result + 1, max_w, 1000000, "lanczos");
    let ok = copy(result + 1, result, 0, h);
    if !ok {
        print("OOF\n");
    }
    let wh = get_wh(result + 1);
    h += wh.h;
}

show(result);

print("Final size: ");
print(`${max_w}`);
print(" x ");
print(`${height}`);
print("\n");

let square = bool_input("Save result y/n: ");

if square {
    let fname = file_input("file name: ");
    if fname != "" {
        save(result, fname);
    }
}

kill();

fn src() { return nat_num_input("src: "); }

