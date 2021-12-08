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

    let in_data = [1, 2, 3, 4];
    info!("Creating buffer containing {:?}", in_data);
    let buffer = app.new_cpu_buffer(&in_data, ash::vk::BufferUsageFlags::UNIFORM_BUFFER)?;

    let mut out_data = [0; 4];
    buffer.read(&mut out_data, 0);
    info!("Read from buffer: {:?}", out_data);

    assert_eq!(&out_data, &in_data);

    Ok(())
}
