use {
    dotenv::dotenv,
    std::{env, ffi::OsStr, fs::create_dir_all, path::PathBuf, process::Command}
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
    println!("cargo:rerun-if-changed={src_dir}/vendor/inih/ini.h");
    println!("cargo:rerun-if-changed={src_dir}/cbindgen.toml");

    //
    // Linking libraries
    //
    println!("cargo::rustc-link-search={target_dir}");
    //println!("cargo::rustc-link-lib=inih");

    //
    // Binding C code
    //
    let builder = cc::Build::new()
        //.shared_flag(true)
        //.static_flag(true)
        .no_default_flags(false)
        .inherit_rustflags(false)
        .cargo_debug(false)
        .cargo_output(true)
        .out_dir(&target_dir)
        .clone();

    builder.clone()
        .file(src_dir + "/vendor/inih/ini.c")
        .define("INI_USE_STACK", "0")
        .define("INI_ALLOW_REALLOC", "1")
        .define("INI_MAX_LINE", "1000")
        .define("INI_ALLOW_BOM", "0")
        .define("INI_ALLOW_NO_VALUE", "1")
        .define("INI_STOP_ON_FIRST_ERROR", "1")
        .shared_flag(true)
        //.static_flag(true)
        .compile("inih");

    let bindings = bindgen::Builder::default()
        .blocklist_type("__BindgenBitfieldUnit")
        .blocklist_type("_IO_FILE")
        .blocklist_type("_IO_marker")
        .blocklist_type("_IO_codecvt")
        .blocklist_type("_IO_wide_data")
        .blocklist_type("_IO_lock_t")
        .blocklist_type("__off_t")
        .blocklist_type("__off64_t")
        .blocklist_type("FILE")
        .use_core()
        .header("vendor/inih/ini.h")
        .allowlist_item("ini_.*")
        .blocklist_function("ini_parse_file")
        .generate()
        .expect("Unable to generate bindings");

    create_dir_all(inc_dir.as_path())
        .expect(&format!("Couldn't create directory: {inc_dir:?}"));

    let bindings_file =
        PathBuf::from_iter([inc_dir.as_os_str(), OsStr::new("bindings.rs")]);

    bindings
        .write_to_file(&bindings_file)
        .expect("Couldn't write bindings!");

    let output = Command::new("rustup")
        .args(["run", "nightly", "rustfmt", bindings_file.to_str().unwrap()])
        .output()
        .expect("Could not format binding file.");

    assert!(
        output.status.success(),
        "Unsuccessful status code when running `rustfmt`: {output:?}",
    );

    //println!("cargo:warning={:?} was formatted successfully.", &out_path);

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
