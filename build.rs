fn main() {
    prost_build::compile_protos(&["proto/state.proto"], &["proto/"])
        .expect("Failed to compile proto files");
}
