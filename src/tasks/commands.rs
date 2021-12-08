use crate::setup::QueueWithPool;
use crate::{
    errors::Result, mem::RawAllocation, tasks::WaitForFenceFuture, DeviceQueues, VulkanApp,
};
use ash::vk;
use parking_lot::MutexGuard;
use std::slice::from_ref;

impl VulkanApp {
    pub(crate) unsafe fn cmd_copy_buffer(
        &self,
        src: (vk::Buffer, &RawAllocation),
        dst: (vk::Buffer, &RawAllocation),
    ) -> Result<WaitForFenceFuture> {
        self.execute_commands(
            |qs| qs.transfer(),
            |pool| Ok(self.allocate_primary_buffers_from_pool(pool, 1)?[0]),
            |device, cmd| {
                device.begin_command_buffer(
                    cmd,
                    &vk::CommandBufferBeginInfo::builder()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )?;

                let copy = vk::BufferCopy::builder()
                    .size(src.1.size)
                    .src_offset(0)
                    .dst_offset(0);

                device.cmd_copy_buffer(cmd, src.0, dst.0, from_ref(&copy));
                Ok(())
            },
        )
    }

    #[inline]
    pub(crate) unsafe fn execute_commands(
        &self,
        queue_chooser: impl FnOnce(&DeviceQueues) -> &QueueWithPool,
        cmd_creator: impl FnOnce(vk::CommandPool) -> Result<vk::CommandBuffer>,
        recorder: impl FnOnce(&ash::Device, vk::CommandBuffer) -> Result<()>,
    ) -> Result<WaitForFenceFuture> {
        let queue_pool = queue_chooser(&self.queues);

        let cmd = {
            let pool = queue_pool.pool.lock();
            let cmd = cmd_creator(*pool)?;
            recorder(&self.device, cmd);

            unsafe {
                self.device.end_command_buffer(cmd)?;
            }
            cmd
        };

        self.queues.submit_to_queue(
            &self.device,
            &queue_pool.queue,
            from_ref(&vk::SubmitInfo::builder().command_buffers(from_ref(&cmd))),
        )
    }
}
