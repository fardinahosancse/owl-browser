fn main() {
    println!("cargo:rerun-if-changed=assets/owl.rc");
    println!("cargo:rerun-if-changed=assets/owl.ico");
    let _ = embed_resource::compile("assets/owl.rc", embed_resource::NONE);
}
