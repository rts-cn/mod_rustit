fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/zrs/")
        .compile(&["proto/pb.proto"], &["proto"])
        .unwrap();
    Ok(())
}
