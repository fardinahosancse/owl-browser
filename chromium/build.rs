fn main() {
    // the Owl. icon on the executable, the same file the WebView2 build uses
    println!("cargo:rerun-if-changed=owl.rc");
    println!("cargo:rerun-if-changed=../assets/owl.ico");
    let _ = embed_resource::compile("owl.rc", embed_resource::NONE);
}
