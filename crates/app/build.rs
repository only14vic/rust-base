use {
    dotenv::dotenv,
    std::{env, ffi::OsStr, path::PathBuf}
};

fn main() {
    dotenv().ok();

    //
    // Configuration
    //
    let src_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let inc_dir = PathBuf::from_iter([&src_dir, "include"]);
    let target_dir = format!(
        "{}/../../{}",
        env::var("OUT_DIR").unwrap(),
        env::var("PROFILE").unwrap()
    );

    println!("cargo:rerun-if-changed={src_dir}/build.rs");
    println!("cargo:rerun-if-changed={src_dir}/src/lib.rs");
    println!("cargo:rerun-if-changed={src_dir}/cbindgen.toml");

    //
    // Linking libraries
    //
    println!("cargo::rustc-link-search={target_dir}");
    //println!("cargo::rustc-link-lib=inih");

    //
    // Binding Rust code
    //
    let cbindgens_filename = PathBuf::from_iter([
        inc_dir.as_os_str(),
        OsStr::new(&format!(
            "lib{}.h",
            env::var("CARGO_PKG_NAME").unwrap().replace("-", "_")
        ))
    ]);

    cbindgen::Builder::new()
        .with_config(cbindgen::Config::from_file("cbindgen.toml").unwrap())
        .with_crate(env::var("CARGO_MANIFEST_DIR").unwrap())
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(cbindgens_filename);
}
