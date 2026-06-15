fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.as_slice() == ["--version"] {
        println!("gws {}", gws_core::version());
        return;
    }

    eprintln!("gws: no command implemented yet");
    std::process::exit(2);
}
