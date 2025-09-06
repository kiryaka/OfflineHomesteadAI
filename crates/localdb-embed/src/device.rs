use candle_core::Device;

pub fn select_device() -> Device {
    #[cfg(feature = "metal")]
    {
        if let Ok(dev) = Device::new_metal(0) { println!("🚀 Device: Metal (MPS)"); return dev; }
    }
    println!("🖥️  Device: CPU");
    Device::Cpu
}
