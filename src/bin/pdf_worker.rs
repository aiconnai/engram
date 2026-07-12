use std::alloc::System;

#[global_allocator]
static ALLOCATOR: cap::Cap<System> = cap::Cap::new(System, 256 * 1024 * 1024);

fn main() {
    if let Err(error) = engram::intelligence::pdf_worker::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
