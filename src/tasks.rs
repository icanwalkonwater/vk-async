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

mod alloc;
mod commands;

const WAIT_FOR_FENCE_SPIN_INTERVALS_NS: u64 = 200;

pub(crate) struct WaitForFenceFuture<'a> {
    pub(crate) device: &'a ash::Device,
    pub(crate) fence: vk::Fence,
}

impl Future for WaitForFenceFuture<'_> {
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            // Wait for a little time while, then let other things run and spin again
            let res = self.device.wait_for_fences(
                from_ref(&self.fence),
                true,
                WAIT_FOR_FENCE_SPIN_INTERVALS_NS,
            );

            match res {
                Ok(_) => {
                    self.device.destroy_fence(self.fence, None);
                    Poll::Ready(Ok(()))
                }
                Err(vk::Result::TIMEOUT) => {
                    ctx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(e) => Poll::Ready(Err(VulkanError::VkError(e))),
            }
        }
    }
}
