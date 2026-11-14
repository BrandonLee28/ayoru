fn main() {
    println!("cargo:rerun-if-changed=native/macos.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        cc::Build::new()
            .file("native/macos.m")
            .flag("-fobjc-arc")
            .flag("-mmacosx-version-min=12.0")
            .compile("ayoru_native");
        for framework in ["AppKit", "AVKit", "AVFoundation", "CoreMedia"] {
            println!("cargo:rustc-link-lib=framework={framework}");
        }
    }
}
