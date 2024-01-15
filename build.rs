fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("proto/")
        .compile(&["proto/pb.proto"], &["proto"])
        .unwrap();
    Ok(())
}
