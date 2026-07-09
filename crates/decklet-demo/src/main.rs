use decklet_demo::runtime_capability_demo_snapshot;
use decklet_host_sdl::{SdlHost, SdlHostConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SdlHostConfig::from_env_and_args();
    SdlHost::new(config).run(runtime_capability_demo_snapshot())?;
    Ok(())
}
