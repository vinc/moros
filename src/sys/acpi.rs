use crate::sys;
use crate::sys::x86::addr::PhysAddr;
use crate::sys::x86::port::*;

use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use acpi::platform::{Processor, ProcessorState};
use core::ptr::NonNull;

static mut PM1A_CNT_BLK: u16 = 0;
static mut SLP_TYPA: u16 = 0;
static SLP_LEN: u16 = 1 << 13;

fn find_pm1a_control_block(acpi: &AcpiTables<MorosAcpiHandler>) -> Option<u16> {
    if let Ok(fadt) = acpi.find_table::<acpi::fadt::Fadt>() {
        if let Ok(block) = fadt.pm1a_control_block() {
            debug_assert!(block.address <= u16::MAX as u64);
            return Some(block.address as u16);
        }
    }
    None
}

fn find_s5(acpi: &AcpiTables<MorosAcpiHandler>) -> Option<u16> {
    if let Ok(dsdt) = acpi.dsdt() {
        let phys_addr = PhysAddr::new(dsdt.address);
        let virt_addr = sys::mem::phys_to_virt(phys_addr);
        let ptr = virt_addr.as_ptr();
        let aml = unsafe {
            core::slice::from_raw_parts(ptr , dsdt.length as usize)
        };
        //debug!("AML dump");
        //printk!("{}", crate::api::hex::format_hex(aml));
        if let Some(i) = aml.windows(4).position(|w| w == b"_S5_") {
            debug_assert!(i + 5 < aml.len());
            if aml[i + 4] == 0x12 { // PackageOp
                let len = aml[i + 5] as usize; // PkgLength
                debug_assert!(i + 5 + len <= aml.len());
                let num = aml[i + 6] as usize; // NumElements
                debug_assert!(num > 0);
                //let j = i + 5 + len;
                //debug!("AML S5 dump");
                //printk!("{}", crate::api::hex::format_hex_at(&aml[i..j], i));
                let b = aml[i + 7];
                match b {
                    0x00 => return Some(0), // ZeroOp
                    0x01 => return Some(1), // OneOp
                    0x0A => return Some(aml[i + 8] as u16), // BytePrefix
                       _ => debug!("ACPI: Could not parse '{:#X}' in AML", b),
                }
            }
        }
    } else {
        debug!("ACPI: Could not find DSDT in BIOS");
    }
    None
}

pub fn init() {
    let res = unsafe { AcpiTables::search_for_rsdp_bios(MorosAcpiHandler) };
    match res {
        Ok(acpi) => {
            if let Ok(info) = acpi.platform_info() {
                if let Some(info) = info.processor_info {
                    log_cpu(&info.boot_processor);
                    for processor in info.application_processors.iter() {
                        log_cpu(processor);
                    }
                }
            }
            if let Some(block) = find_pm1a_control_block(&acpi) {
                unsafe {
                    PM1A_CNT_BLK = block;
                }
            }
            if let Some(value) = find_s5(&acpi) {
                unsafe {
                    SLP_TYPA = ((value & 7) as u16) << 10;
                }
            }
        }
        Err(_) => {
            debug!("ACPI: Could not find RDSP in BIOS");
        }
    };
}

pub fn shutdown() {
    log!("ACPI Shutdown");
    unsafe {
        outw(PM1A_CNT_BLK, SLP_TYPA | SLP_LEN);
    }
}

#[derive(Clone)]
pub struct MorosAcpiHandler;

impl AcpiHandler for MorosAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        addr: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let phys_addr = PhysAddr::new(addr);
        let virt_addr = sys::mem::phys_to_virt(phys_addr);
        let ptr = NonNull::new(virt_addr.as_mut_ptr()).unwrap();
        PhysicalMapping::new(addr, ptr, size, size, Self)
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

fn log_cpu(processor: &Processor) {
    let kind = if processor.is_ap { "AP" } else { "BP" };
    let state = match processor.state {
        ProcessorState::Disabled       => "disabled",
        ProcessorState::Running        => "running",
        ProcessorState::WaitingForSipi => "waiting",
    };
    log!("CPU {}:{} {}", kind, processor.processor_uid, state);
}
