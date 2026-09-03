use raw_cpuid::{CpuId, CpuIdReader};

#[cfg(target_arch = "x86")]
pub fn cpuid() -> CpuId<impl CpuIdReader> {
    // The crate requires sse on x86 which we don't have inside the kernel
    // See: https://github.com/gz/rust-cpuid/issues/134
    CpuId::with_cpuid_fn(|leaf, sub_leaf| {
        let res = core::arch::x86::__cpuid_count(leaf, sub_leaf);
        raw_cpuid::CpuIdResult {
            eax: res.eax,
            ebx: res.ebx,
            ecx: res.ecx,
            edx: res.edx,
        }
    })
}

#[cfg(target_arch = "x86_64")]
pub fn cpuid() -> CpuId<impl CpuIdReader> {
    CpuId::new()
}

pub fn init() {
    let cpuid = cpuid();

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
    cpuid().get_feature_info().is_some_and(|info| info.has_rdrand())
}
