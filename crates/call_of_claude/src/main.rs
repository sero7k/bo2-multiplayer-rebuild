fn main() {
    std::process::exit(call_of_claude::run(std::env::args().skip(1).collect()));
}
