use crate::{errors::Result, VulkanApp};
use ash::vk;
use parking_lot::MutexGuard;

impl VulkanApp {
    pub(crate) unsafe fn alloc_transient_transfer_cmd_buffer(
        &self,
    ) -> Result<(MutexGuard<vk::CommandPool>, vk::CommandBuffer)> {
        let pool = self.queues.transfer().pool.lock();
        let cmd = self.allocate_primary_buffers_from_pool(*pool, 1)?[0];

        Ok((pool, cmd))
    }

    #[inline]
    pub(crate) unsafe fn allocate_primary_buffers_from_pool(
        &self,
        pool: vk::CommandPool,
        count: u32,
    ) -> Result<Vec<vk::CommandBuffer>> {
        Ok(self.device.allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::builder()
                .command_pool(pool)
                .command_buffer_count(count)
                .level(vk::CommandBufferLevel::PRIMARY),
        )?)
    }
}
