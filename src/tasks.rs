use crate::{
    errors::{Result, VulkanError},
    mem::RawAllocation,
    VulkanApp,
};
use ash::vk;

use std::{
    future::Future,
    pin::Pin,
    slice::from_ref,
    task::{Context, Poll},
};

impl VulkanApp {
    pub(crate) unsafe fn create_copy_buffer_cmd(
        &self,
        src: (vk::Buffer, &RawAllocation),
        dst: (vk::Buffer, &RawAllocation),
    ) -> Result<WaitForFenceFuture> {
        let cmd = {
            // Lock pool for the whole recording session
            let pool = self.queues.transfer().pool.lock();
            let cmd = self.device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::builder()
                    .command_pool(*pool)
                    .command_buffer_count(1)
                    .level(vk::CommandBufferLevel::PRIMARY),
            )?[0];

            self.device.begin_command_buffer(
                cmd,
                &vk::CommandBufferBeginInfo::builder()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;

            let copy = vk::BufferCopy::builder()
                .size(src.1.size)
                .src_offset(0)
                .dst_offset(0);

            self.device
                .cmd_copy_buffer(cmd, src.0, dst.0, from_ref(&copy));

            self.device.end_command_buffer(cmd)?;

            cmd
        };

        let submit_info = vk::SubmitInfo::builder().command_buffers(from_ref(&cmd));

        self.queues
            .submit_to_transfer(&self.device, from_ref(&submit_info))
    }
}

pub(crate) struct WaitForFenceFuture<'a> {
    pub(crate) device: &'a ash::Device,
    pub(crate) fence: vk::Fence,
}

impl Future for WaitForFenceFuture<'_> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            match self.device.get_fence_status(self.fence) {
                Ok(true) => {
                    self.device.destroy_fence(self.fence, None);
                    Poll::Ready(Ok(()))
                }
                Ok(false) => {
                    ctx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(e) => Poll::Ready(Err(VulkanError::VkError(e))),
            }
        }
    }
}
