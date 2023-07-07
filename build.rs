fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../game-panda-client/Networking/pool.proto")?;
    Ok(())
}