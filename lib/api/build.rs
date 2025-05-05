fn main() {
    tonic_build::configure()
        .build_server(false)
        .compile_protos(&["src/notify.proto"], &["src"])
        .unwrap();
}
