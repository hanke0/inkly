use candle_core::{Device, Result};

pub fn pick_device(force_cpu: bool) -> Result<Device> {
    if force_cpu {
        return Ok(Device::Cpu);
    }
    if candle_core::utils::cuda_is_available() {
        return Device::new_cuda(0);
    }
    if candle_core::utils::metal_is_available() {
        return Device::new_metal(0);
    }
    Ok(Device::Cpu)
}
