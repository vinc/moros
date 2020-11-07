use crate::{kernel, log};
use acpi::{AcpiHandler, PhysicalMapping, AcpiTables};
use alloc::boxed::Box;
use aml::{AmlContext, AmlName, DebugVerbosity, Handler};
use x86_64::instructions::port::Port;

pub fn poweroff() {
    let mut pm1a_control_block = 0;
    let mut slp_typa = 0;
    let slp_len = 1 << 13;

    let mut aml = AmlContext::new(Box::new(MorosAmlHandler), false, DebugVerbosity::None);
    let res = unsafe { AcpiTables::search_for_rsdp_bios(MorosAcpiHandler) };
    match res {
        Ok(acpi) => {
            //log!("ACPI init successful\n");
            for (sign, sdt) in acpi.sdts {
                if sign.as_str() == "FACP" {
                    //log!("{:?}\n", sign);
                    let addr = kernel::mem::phys_mem_offset() + (sdt.physical_address + 64) as u64;
                    let ptr = addr.as_ptr::<u32>();
                    pm1a_control_block = unsafe { *ptr };
                    //log!("{:?}\n", pm1a_control_block);
                }
            }
            match &acpi.dsdt {
                Some(dsdt) => {
                    //log!("{:?}\n", dsdt);
                    let addr = kernel::mem::phys_mem_offset() + dsdt.address as u64;
                    let stream = unsafe { core::slice::from_raw_parts(addr.as_ptr(), dsdt.length as usize) };
                    //let stream = unsafe { core::slice::from_raw_parts(addr.as_u64() as *mut u8, dsdt.length as usize) };
                    if aml.parse_table(stream).is_err() {
                        //log!("Failed to parse AML in DSDT");
                        return;
                    }
                    //let name = AmlName::from_str("\\_S5.SLP_TYPa").unwrap();
                    let name = AmlName::from_str("\\_S5").unwrap();
                    if let Ok(aml::value::AmlValue::Package(s5)) = aml.namespace.get_by_path(&name) {
                        //log!("{:?}\n", s5);
                        if let aml::value::AmlValue::Integer(value) = s5[0] {
                            slp_typa = value;
                            //log!("{:?}\n", slp_typa);
                        }
                    }
                },
                None => {},
            }
        }
        Err(_e) => {
            //log!("ACPI init unsuccessful {:?}\n", e);
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
