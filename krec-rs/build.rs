fn main() {
    prost_build::compile_protos(&["proto/krec.proto"], &["proto/"]).unwrap();
}
