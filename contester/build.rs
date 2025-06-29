fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .file_descriptor_set_path("proto_descriptor.bin")
        .compile_protos(&["proto/testing_service.proto"], &["proto/"])?;
    //tonic_build::compile_protos("proto/testing_service.proto")?;
    Ok(())
}
