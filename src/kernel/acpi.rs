use crate::{kernel, log};
use acpi::{AcpiHandler, PhysicalMapping, AcpiTables};
use alloc::boxed::Box;
use aml::{AmlContext, AmlName, DebugVerbosity, Handler};
use x86_64::instructions::port::Port;

pub fn shutdown() {
    let mut pm1a_control_block = 0;
    let mut slp_typa = 0;
    let slp_len = 1 << 13;

    log!("ACPI Executing shutdown\n");
    let mut aml = AmlContext::new(Box::new(MorosAmlHandler), false, DebugVerbosity::None);
    let res = unsafe { AcpiTables::search_for_rsdp_bios(MorosAcpiHandler) };
    match res {
        Ok(acpi) => {
            //log!("ACPI Found RDSP in BIOS\n");
            for (sign, sdt) in acpi.sdts {
                if sign.as_str() == "FACP" {
                    //log!("ACPI Found FACP at {}\n", sdt.physical_address);
                    let addr = kernel::mem::phys_mem_offset() + (sdt.physical_address + 64) as u64;
                    let ptr = addr.as_ptr::<u32>();
                    pm1a_control_block = unsafe { *ptr };
                    //log!("ACPI Found PM1a Control Block: {}\n", pm1a_control_block);
                }
            }
            match &acpi.dsdt {
                Some(dsdt) => {
                    //log!("ACPI Found DSDT at {}\n", dsdt.address);
                    let addr = kernel::mem::phys_mem_offset() + dsdt.address as u64;
                    let stream = unsafe { core::slice::from_raw_parts(addr.as_ptr(), dsdt.length as usize) };
                    if aml.parse_table(stream).is_err() {
                        log!("ACPI Failed to parse AML in DSDT\n");
                        return;
                    }
                    let name = AmlName::from_str("\\_S5").unwrap();
                    if let Ok(aml::value::AmlValue::Package(s5)) = aml.namespace.get_by_path(&name) {
                        //log!("ACPI Found \\_S5 in DSDT\n");
                        if let aml::value::AmlValue::Integer(value) = s5[0] {
                            slp_typa = value;
                            //log!("ACPI Found SLP_TYPa in \\_S5: {}\n", slp_typa);
                        }
                    }
                },
                None => {},
            }
        }
        Err(_e) => {
            log!("ACPI Could not find RDSP in BIOS\n");
        }
    };

    let mut port: Port<u16> = Port::new(pm1a_control_block as u16);
    unsafe {
        port.write(slp_typa as u16 | slp_len as u16);
    }
}

#[derive(Clone)]
pub struct MorosAcpiHandler;

impl AcpiHandler for MorosAcpiHandler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        let virtual_address = kernel::mem::phys_mem_offset() + physical_address as u64;
        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: core::ptr::NonNull::new(virtual_address.as_mut_ptr()).unwrap(),
            region_length: size,
            mapped_length: size,
            handler: Self,
        }
    }

    fn unmap_physical_region<T>(&self, _region: &PhysicalMapping<Self, T>) {
    }
}

macro_rules! def_read_handler {
    ($name:ident, $size:ty) => {
        fn $name(&self, address: usize) -> $size {
            let virtual_address = kernel::mem::phys_mem_offset() + address as u64;
            let ptr = virtual_address.as_ptr::<$size>();
            unsafe { *ptr }
        }
    };
}

struct MorosAmlHandler;

impl Handler for MorosAmlHandler {
    def_read_handler!(read_u8, u8);
    def_read_handler!(read_u16, u16);
    def_read_handler!(read_u32, u32);
    def_read_handler!(read_u64, u64);
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
