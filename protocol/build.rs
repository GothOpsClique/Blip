fn main() {
    let protoc =
        protoc_bin_vendored::protoc_bin_path().expect("Failed to locate vendored protoc binary");
    let protoc_path = protoc.to_string_lossy().into_owned();
    unsafe {
        std::env::set_var("PROTOC", &protoc_path);
    }

    prost_build::Config::new()
        .compile_protos(&["proto/chat.proto"], &["proto"])
        .expect("Failed to compile protobuf definitions");
}
