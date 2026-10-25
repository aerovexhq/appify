fn main() {
    println!("cargo:rerun-if-changed=ui/dist");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    tauri_build::build()
}
