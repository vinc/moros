use crate::sys;

use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use acpi::platform::{Processor, ProcessorState};
use alloc::boxed::Box;
use aml::value::AmlValue;
use aml::{AmlContext, AmlName, DebugVerbosity, Handler};
use core::ptr::NonNull;
use x86_64::instructions::port::Port;
use x86_64::PhysAddr;

static mut PM1A_CNT_BLK: u32 = 0;
static mut SLP_TYPA: u16 = 0;
static SLP_LEN: u16 = 1 << 13;

pub fn init() {
    let res = unsafe { AcpiTables::search_for_rsdp_bios(MorosAcpiHandler) };
    match res {
        Ok(acpi) => {
            if let Ok(info) = acpi.platform_info() {
                if let Some(info) = info.processor_info {
                    log_cpu(&info.boot_processor);
                    for processor in info.application_processors.iter() {
                        log_cpu(&processor);
                    }
                }
            }
            if let Ok(fadt) = acpi.find_table::<acpi::fadt::Fadt>() {
                if let Ok(block) = fadt.pm1a_control_block() {
                    unsafe {
                        PM1A_CNT_BLK = block.address as u32;
                    }
                }
            }
            if let Ok(dsdt) = acpi.dsdt() {
                let phys_addr = PhysAddr::new(dsdt.address as u64);
                let virt_addr = sys::mem::phys_to_virt(phys_addr);
                let ptr = virt_addr.as_ptr();
                let table = unsafe {
                    core::slice::from_raw_parts(ptr , dsdt.length as usize)
                };
                let handler = Box::new(MorosAmlHandler);
                let mut aml = AmlContext::new(handler, DebugVerbosity::None);
                if aml.parse_table(table).is_ok() {
                    let name = AmlName::from_str("\\_S5").unwrap();
                    let res = aml.namespace.get_by_path(&name);
                    if let Ok(AmlValue::Package(s5)) = res {
                        if let AmlValue::Integer(value) = s5[0] {
                            unsafe {
                                SLP_TYPA = value as u16;
                            }
                        }
                    }
                } else {
                    debug!("ACPI: Could not parse AML in DSDT");
                    // FIXME: AML parsing works on QEMU and Bochs but not
                    // on VirtualBox at the moment, so we use the following
                    // hardcoded value:
                    unsafe {
                        SLP_TYPA = (5 & 7) << 10;
                    }
                }
            } else {
                debug!("ACPI: Could not find DSDT in BIOS");
            }
        }
        Err(_e) => {
            debug!("ACPI: Could not find RDSP in BIOS");
        }
    };
}

pub fn shutdown() {
    log!("ACPI Shutdown");
    unsafe {
        let mut port: Port<u16> = Port::new(PM1A_CNT_BLK as u16);
        port.write(SLP_TYPA | SLP_LEN);
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
        let phys_addr = PhysAddr::new(addr as u64);
        let virt_addr = sys::mem::phys_to_virt(phys_addr);
        let ptr = NonNull::new(virt_addr.as_mut_ptr()).unwrap();
        PhysicalMapping::new(addr, ptr, size, size, Self)
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

struct MorosAmlHandler;

impl Handler for MorosAmlHandler {
    fn read_u8(&self, address: usize) -> u8 {
        read_addr::<u8>(address)
    }
    fn read_u16(&self, address: usize) -> u16 {
        read_addr::<u16>(address)
    }
    fn read_u32(&self, address: usize) -> u32 {
        read_addr::<u32>(address)
    }
    fn read_u64(&self, address: usize) -> u64 {
        read_addr::<u64>(address)
    }

    fn write_u8(&mut self, _: usize, _: u8) {
        unimplemented!()
    }
    fn write_u16(&mut self, _: usize, _: u16) {
        unimplemented!()
    }
    fn write_u32(&mut self, _: usize, _: u32) {
        unimplemented!()
    }
    fn write_u64(&mut self, _: usize, _: u64) {
        unimplemented!()
    }
    fn read_io_u8(&self, _: u16) -> u8 {
        unimplemented!()
    }
    fn read_io_u16(&self, _: u16) -> u16 {
        unimplemented!()
    }
    fn read_io_u32(&self, _: u16) -> u32 {
        unimplemented!()
    }
    fn write_io_u8(&self, _: u16, _: u8) {
        unimplemented!()
    }
    fn write_io_u16(&self, _: u16, _: u16) {
        unimplemented!()
    }
    fn write_io_u32(&self, _: u16, _: u32) {
        unimplemented!()
    }
    fn read_pci_u8(&self, _: u16, _: u8, _: u8, _: u8, _: u16) -> u8 {
        unimplemented!()
    }
    fn read_pci_u16(&self, _: u16, _: u8, _: u8, _: u8, _: u16) -> u16 {
        unimplemented!()
    }
    fn read_pci_u32(&self, _: u16, _: u8, _: u8, _: u8, _: u16) -> u32 {
        unimplemented!()
    }
    fn write_pci_u8(&self, _: u16, _: u8, _: u8, _: u8, _: u16, _: u8) {
        unimplemented!()
    }
    fn write_pci_u16(&self, _: u16, _: u8, _: u8, _: u8, _: u16, _: u16) {
        unimplemented!()
    }
    fn write_pci_u32(&self, _: u16, _: u8, _: u8, _: u8, _: u16, _: u32) {
        unimplemented!()
    }
}

fn read_addr<T>(addr: usize) -> T where T: Copy {
    let virtual_address = sys::mem::phys_to_virt(PhysAddr::new(addr as u64));
    unsafe { *virtual_address.as_ptr::<T>() }
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
