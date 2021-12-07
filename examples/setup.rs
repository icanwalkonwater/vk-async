use log::{info, LevelFilter};
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use vk_async::setup::VulkanBuilder;

fn main() -> anyhow::Result<()> {
    TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;
    info!("Logger init ?");

    let mut builder = VulkanBuilder::builder().with_name("Test 1").build()?;

    let mut gpus = builder.list_available_physical_devices()?;
    info!("Available GPUs:");
    for info in &gpus {
        info!(
            "- {} [Discrete: {}, VRAM: {} GiB]",
            info.name(),
            info.is_discrete(),
            info.vram_size() / 1_073_741_824
        );
    }

    let gpu = {
        if gpus.len() > 1 {
            gpus.into_iter()
                .filter(|gpu| gpu.is_discrete())
                .max_by_key(|gpu| gpu.vram_size())
                .unwrap()
        } else {
            gpus.pop().unwrap()
        }
    };

    let app = builder.set_physical_device(gpu).build()?;

    Ok(())
}
