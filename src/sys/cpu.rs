use raw_cpuid::CpuId;

pub fn init() {
    let cpuid = CpuId::new();

    if let Some(vendor_info) = cpuid.get_vendor_info() {
        log!("CPU {}", vendor_info);
    }

    if let Some(processor_brand_string) = cpuid.get_processor_brand_string() {
        log!("CPU {}", processor_brand_string.as_str().trim());
    }

    if let Some(info) = cpuid.get_processor_frequency_info() {
        let frequency = info.processor_base_frequency();
        log!("CPU {} MHz", frequency);
    }
}
