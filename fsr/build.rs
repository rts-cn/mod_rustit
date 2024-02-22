#[cfg(target_os = "linux")]
fn build() {
    let headers_path_str = "/usr/local/freeswitch/include/freeswitch/switch.h";

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", "/usr/local/freeswitch/lib");

    // Tell cargo to look for shared libraries  irectory
    println!(
        "cargo:rustc-link-arg={}",
        "-Wl,-rpath=/usr/local/freeswitch/lib"
    );

    // Tell cargo to tell rustc to link our `freeswitch` library. Cargo will
    // automatically know it must look for a `libfreeswitch.a` file.
    println!("cargo:rustc-link-lib=freeswitch");

    // Tell cargo to invalidate the built crate whenever the header changes.
    println!("cargo:rerun-if-changed={}", headers_path_str);

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .layout_tests(false)
        // The input header we would like to generate
        // bindings for.
        .header(headers_path_str)
        .allowlist_file(".*freeswitch.*")
        .newtype_enum(".*")
        .derive_hash(false)
        .generate_comments(false)
        .clang_arg("-I/usr/local/freeswitch/include/freeswitch")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = std::path::PathBuf::from("src/");
    bindings
        .write_to_file(out_path.join("fs.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    #[cfg(target_os = "linux")]
    build();
}
