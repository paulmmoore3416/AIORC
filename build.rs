fn main() {
    // Compile proto files
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &["proto/orchestrator.proto"],
            &["proto/"],
        )
        .expect("Failed to compile proto files");
}
