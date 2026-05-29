// client/build.rs
// Чтобы логгер создавался в правильной папке

fn main() {
    println!("cargo:rerun-if-changed=src/");
}