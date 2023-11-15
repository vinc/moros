use crate::sys;

use acpi::{AcpiHandler, PhysicalMapping, AcpiTables};
use alloc::boxed::Box;
use aml::value::AmlValue;
use aml::{AmlContext, AmlName, DebugVerbosity, Handler};
use core::ptr::NonNull;
use x86_64::PhysAddr;
use x86_64::instructions::port::Port;

fn read_addr<T>(physical_address: usize) -> T where T: Copy {
    let virtual_address = sys::mem::phys_to_virt(PhysAddr::new(physical_address as u64));
    unsafe { *virtual_address.as_ptr::<T>() }
}

pub fn shutdown() {
    let mut pm1a_control_block = 0;
    let mut slp_typa = 0;
    let slp_len = 1 << 13;

    log!("ACPI Shutdown\n");
    let mut aml = AmlContext::new(Box::new(MorosAmlHandler), DebugVerbosity::None);
    let res = unsafe { AcpiTables::search_for_rsdp_bios(MorosAcpiHandler) };
    match res {
        Ok(acpi) => {
            if let Ok(fadt) = acpi.find_table::<acpi::fadt::Fadt>() {
                if let Ok(block) = fadt.pm1a_control_block() {
                    pm1a_control_block = block.address as u32;
                    //debug!("ACPI Found PM1a Control Block: {:#x}", pm1a_control_block);
                }
            }
            if let Ok(dsdt) = &acpi.dsdt() {
                let address = sys::mem::phys_to_virt(PhysAddr::new(dsdt.address as u64));
                //debug!("ACPI Found DSDT at {:#x} {:#x}", dsdt.address, address);
                let table = unsafe { core::slice::from_raw_parts(address.as_ptr(), dsdt.length as usize) };
                if aml.parse_table(table).is_ok() {
                    let name = AmlName::from_str("\\_S5").unwrap();
                    if let Ok(AmlValue::Package(s5)) = aml.namespace.get_by_path(&name) {
                        //debug!("ACPI Found \\_S5 in DSDT");
                        if let AmlValue::Integer(value) = s5[0] {
                            slp_typa = value as u16;
                            //debug!("ACPI Found SLP_TYPa in \\_S5: {}", slp_typa);
                        }
                    }
                } else {
                    debug!("ACPI Failed to parse AML in DSDT");
                    // FIXME: AML parsing works on QEMU and Bochs but not
                    // on VirtualBox at the moment, so we use the following
                    // hardcoded value:
                    slp_typa = (5 & 7) << 10;
                }
            }
        }
        Err(_e) => {
            debug!("ACPI Could not find RDSP in BIOS\n");
        }
    };

    let mut port: Port<u16> = Port::new(pm1a_control_block as u16);
    //debug!("ACPI shutdown");
    unsafe {
        port.write(slp_typa | slp_len);
    }
}

#[derive(Clone)]
pub struct MorosAcpiHandler;

impl AcpiHandler for MorosAcpiHandler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        let virtual_address = sys::mem::phys_to_virt(PhysAddr::new(physical_address as u64));
        //debug!("ACPI mapping phys {:#x} -> virt {:#x}", physical_address, virtual_address);
        PhysicalMapping::new(physical_address, NonNull::new(virtual_address.as_mut_ptr()).unwrap(), size, size, Self)
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

struct MorosAmlHandler;

impl Handler for MorosAmlHandler {
    fn read_u8(&self, address: usize) -> u8 { read_addr::<u8>(address) }
    fn read_u16(&self, address: usize) -> u16 { read_addr::<u16>(address) }
    fn read_u32(&self, address: usize) -> u32 { read_addr::<u32>(address) }
    fn read_u64(&self, address: usize) -> u64 { read_addr::<u64>(address) }
    fn write_u8(&mut self, _address: usize, _value: u8) { unimplemented!() }
    fn write_u16(&mut self, _address: usize, _value: u16) { unimplemented!() }
    fn write_u32(&mut self, _address: usize, _value: u32) { unimplemented!() }
    fn write_u64(&mut self, _address: usize, _value: u64) { unimplemented!() }
    fn read_io_u8(&self, _port: u16) -> u8 { unimplemented!() }
    fn read_io_u16(&self, _port: u16) -> u16 { unimplemented!() }
    fn read_io_u32(&self, _port: u16) -> u32 { unimplemented!() }
    fn write_io_u8(&self, _port: u16, _value: u8) { unimplemented!() }
    fn write_io_u16(&self, _port: u16, _value: u16) { unimplemented!() }
    fn write_io_u32(&self, _port: u16, _value: u32) { unimplemented!() }
    fn read_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u8 { unimplemented!() }
    fn read_pci_u16(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u16 { unimplemented!() }
    fn read_pci_u32(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u32 { unimplemented!() }
    fn write_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u8) { unimplemented!() }
    fn write_pci_u16(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u16) { unimplemented!() }
    fn write_pci_u32(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16, _value: u32) { unimplemented!() }
}
