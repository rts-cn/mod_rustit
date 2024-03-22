fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(feature = "proto"))]
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/zrs/")
        .compile(&["src/zrs/pb.proto"], &["proto"])
        .unwrap();
    Ok(())
}
