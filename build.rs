fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .out_dir("src/zrs/")
        .compile(&["proto/pb.proto"], &["proto"])
        .unwrap();
    Ok(())
}
