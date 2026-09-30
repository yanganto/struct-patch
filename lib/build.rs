fn main() {
    println!("cargo:rustc-check-cfg=cfg(use_alloc)");

    let on = |name: &str| std::env::var_os(format!("CARGO_FEATURE_{name}")).is_some();

    if on("ALLOC") || (on("NESTING") && on("LOG")) {
        println!("cargo:rustc-cfg=use_alloc");
    }
}
