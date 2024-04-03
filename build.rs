fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(feature = "proto"))]
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/api/")
        .compile(&["src/api/zrapi.proto"], &["src/api"])
        .unwrap();
    Ok(())
}
