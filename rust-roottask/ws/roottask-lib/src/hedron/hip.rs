use crate::hedron::acpi_gas::AcpiGas;
use crate::hedron::capability::CapSel;
use crate::hedron::consts::{
    NUM_CPUS,
    NUM_IOAPICS,
};
use crate::hedron::cpu::LapicInfo;

/// Hypervisor Information Page.
#[derive(Debug)]
#[repr(C)]
pub struct HIP {
    signature: u32,
    checksum: u16,
    length: u16,

    cpu_offs: u16,
    cpu_size: u16,
    ioapic_offs: u16,
    ioapic_size: u16,

    mem_offs: u16,
    mem_size: u16,
    api_flg: u32,

    api_ver: u32,
    sel_num: u32,

    sel_exc: u32,
    sel_vmi: u32,

    sel_gsi: u32,
    cfg_page: u32,

    cfg_utcb: u32,
    freq_tsc: u32,

    freq_bus: u32,
    pci_bus_start: u32,

    mcfg_base: u64,
    msfg_size: u64,

    dmar_tables: u64,
    hpet_base: u64,

    cap_vmx_sec_exec: u64,

    xsdt_rdst_table: u64,

    pm1a_cnt: AcpiGas,
    pm2b_cnt: AcpiGas,

    // maximum support tof 64 CPUs
    cpu_desc: [HipCpu; NUM_CPUS],
    ioapic_desc: [HipIoApic; NUM_IOAPICS],
    mem_desc: *const HipMem,
}

impl HIP {
    pub fn signature(&self) -> u32 {
        self.signature
    }
    pub fn checksum(&self) -> u16 {
        self.checksum
    }
    pub fn length(&self) -> u16 {
        self.length
    }
    pub fn cpu_offs(&self) -> u16 {
        self.cpu_offs
    }
    pub fn cpu_size(&self) -> u16 {
        self.cpu_size
    }
    pub fn ioapic_offs(&self) -> u16 {
        self.ioapic_offs
    }
    pub fn ioapic_size(&self) -> u16 {
        self.ioapic_size
    }
    pub fn mem_offs(&self) -> u16 {
        self.mem_offs
    }
    pub fn mem_size(&self) -> u16 {
        self.mem_size
    }
    pub fn api_flg(&self) -> u32 {
        self.api_flg
    }
    pub fn api_ver(&self) -> u32 {
        self.api_ver
    }
    pub fn sel_num(&self) -> u32 {
        self.sel_num
    }
    pub fn sel_exc(&self) -> u32 {
        self.sel_exc
    }
    pub fn sel_vmi(&self) -> u32 {
        self.sel_vmi
    }
    pub fn sel_gsi(&self) -> u32 {
        self.sel_gsi
    }
    pub fn cfg_page(&self) -> u32 {
        self.cfg_page
    }
    pub fn cfg_utcb(&self) -> u32 {
        self.cfg_utcb
    }
    pub fn freq_tsc(&self) -> u32 {
        self.freq_tsc
    }
    pub fn freq_bus(&self) -> u32 {
        self.freq_bus
    }
    pub fn pci_bus_start(&self) -> u32 {
        self.pci_bus_start
    }
    pub fn mcfg_base(&self) -> u64 {
        self.mcfg_base
    }
    pub fn msfg_size(&self) -> u64 {
        self.msfg_size
    }
    pub fn dmar_tables(&self) -> u64 {
        self.dmar_tables
    }
    pub fn hpet_base(&self) -> u64 {
        self.hpet_base
    }
    pub fn cap_vmx_sec_exec(&self) -> u64 {
        self.cap_vmx_sec_exec
    }
    pub fn xsdt_rdst_table(&self) -> u64 {
        self.xsdt_rdst_table
    }
    pub fn pm1a_cnt(&self) -> &AcpiGas {
        &self.pm1a_cnt
    }
    pub fn pm2b_cnt(&self) -> &AcpiGas {
        &self.pm2b_cnt
    }
    pub fn cpu_desc(&self) -> &[HipCpu; 64] {
        &self.cpu_desc
    }
    pub fn ioapic_desc(&self) -> &[HipIoApic; 9] {
        &self.ioapic_desc
    }
    pub fn mem_desc(&self) -> *const HipMem {
        self.mem_desc
    }

    /// Returns the cap selector for the root PD.
    /// See spec pdf 6.1.2.3 Object Space
    pub fn root_pd(&self) -> CapSel {
        self.sel_exc as u64 + 0
    }
    /// Returns the cap selector for the root EC.
    /// See spec pdf 6.1.2.3 Object Space
    pub fn root_ec(&self) -> CapSel {
        self.sel_exc as u64 + 1
    }
    /// Returns the cap selector for the root SC.
    /// See spec pdf 6.1.2.3 Object Space
    pub fn root_sc(&self) -> CapSel {
        self.sel_exc as u64 + 2
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct HipCpu {
    flags: u8,
    thread: u8,
    core: u8,
    package: u8,
    acpi_id: u8,
    _reserved: [u8; 3],
    lapic_info: LapicInfo,
}

impl HipCpu {
    pub fn flags(&self) -> u8 {
        self.flags
    }
    pub fn thread(&self) -> u8 {
        self.thread
    }
    pub fn core(&self) -> u8 {
        self.core
    }
    pub fn package(&self) -> u8 {
        self.package
    }
    pub fn acpi_id(&self) -> u8 {
        self.acpi_id
    }
    pub fn _reserved(&self) -> &[u8; 3] {
        &self._reserved
    }
    pub fn lapic_info(&self) -> &LapicInfo {
        &self.lapic_info
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct HipMem {
    addr: u64,
    size: u64,
    typ: HipMemType,
    aux: u32,
}

impl HipMem {
    pub fn addr(&self) -> u64 {
        self.addr
    }
    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn typ(&self) -> HipMemType {
        self.typ
    }
    pub fn aux(&self) -> u32 {
        self.aux
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum HipMemType {
    Hypervisor = -1_i32 as u32,
    MbModule = -2_i32 as u32,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct HipIoApic {
    id: u32,
    version: u32,
    gsi_base: u32,
    base: u32,
}

impl HipIoApic {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn gsi_base(&self) -> u32 {
        self.gsi_base
    }
    pub fn base(&self) -> u32 {
        self.base
    }
}
