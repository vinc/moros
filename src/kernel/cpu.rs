use raw_cpuid::CpuId;
use crate::{print, kernel};

pub fn init() {
    let cpuid = CpuId::new();

    if let Some(vendor_info) = cpuid.get_vendor_info() {
        print!("[{:.6}] CPU {}\n", kernel::clock::uptime(), vendor_info);
    }

    if let Some(extended_function_info) = cpuid.get_extended_function_info() {
        if let Some(processor_brand_string) = extended_function_info.processor_brand_string() {
            print!("[{:.6}] CPU {}\n", kernel::clock::uptime(), processor_brand_string);
        }
    }

    if let Some(processor_frequency_info) = cpuid.get_processor_frequency_info() {
        let processor_base_frequency = processor_frequency_info.processor_base_frequency();
        print!("[{:.6}] CPU {} MHz\n", kernel::clock::uptime(), processor_base_frequency);
    }
}
