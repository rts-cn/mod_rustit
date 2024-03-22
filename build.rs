fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(feature = "proto"))]
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/grpc/")
        .compile(&["src/grpc/pb.proto"], &["proto"])
        .unwrap();
    Ok(())
}
