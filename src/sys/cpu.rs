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
        if frequency > 0 {
            log!("CPU {} MHz", frequency);
        }
    }
}

// RDRAND has been available since 2012 for Intel (Ivy Bridge) processors
// and 2015 for AMD.
pub fn has_rdrand() -> bool {
    CpuId::new().get_feature_info().is_some_and(|info| info.has_rdrand())
}
