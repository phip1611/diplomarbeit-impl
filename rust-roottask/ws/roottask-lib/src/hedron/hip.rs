use crate::hedron::acpi_gas::AcpiGas;
use crate::hedron::capability::CapSel;
use crate::hedron::consts::{
    NUM_CPUS,
    NUM_IOAPICS,
};
use crate::hedron::cpu::LapicInfo;
use core::fmt::{
    Debug,
    Formatter,
};

/// Hypervisor Information Page.
#[repr(C)]
pub struct HIP {
    /// A value of 0x41564f4e identified NOVA/Hedron.
    signature: u32,
    /// The checksum is valid if 16bit-wise addition the HIP contents produces a value of 0
    checksum: u16,
    /// Length of HIP in bytes. This includes all CPU and memory descriptors.
    length: u16,
    /// Offset of first CPU descriptor in bytes. Relative to HIP base.
    cpu_offs: u16,
    /// Size of one CPU descriptor in bytes.
    cpu_size: u16,

    ioapic_offs: u16,
    ioapic_size: u16,

    /// Offset of first memory descriptor in bytes. Relative to HIP base.
    mem_offs: u16,
    /// Size of one memory descriptor in bytes.
    mem_size: u16,

    api_flg: HipFeatureFlags,
    /// API version number.
    api_ver: u32,
    /// Number of available capability selectors in each object space. Specifying a capability selector
    /// beyond the maximum number supported wraps around to the beginning of the object space.
    sel_num: u32,

    /// Number of capability selectors used for exception handling
    num_exc_sel: u32,
    /// Number of capability selectors used for virtual-machine intercept handling
    num_vmi_sel: u32,
    /// Number of available global system interrupts selectors.
    num_gsi_sel: u32,
    /// If bit n is set, the implementation supports memory pages of size 2^n bytes.
    cfg_page: u32,
    /// If bit n is set, the implementation supports UTCBs of size 2^n bytes.
    cfg_utcb: u32,
    /// Time Stamp Counter Frequency in kHz.
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
    pub fn api_flg(&self) -> HipFeatureFlags {
        self.api_flg
    }
    pub fn api_ver(&self) -> u32 {
        self.api_ver
    }
    pub fn sel_num(&self) -> u32 {
        self.sel_num
    }
    pub fn num_exc_sel(&self) -> u32 {
        self.num_exc_sel
    }
    pub fn num_vmi_sel(&self) -> u32 {
        self.num_vmi_sel
    }
    pub fn num_gsi_sel(&self) -> u32 {
        self.num_gsi_sel
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
        self.num_exc_sel as u64 + 0
    }
    /// Returns the cap selector for the root EC.
    /// See spec pdf 6.1.2.3 Object Space
    pub fn root_ec(&self) -> CapSel {
        self.num_exc_sel as u64 + 1
    }
    /// Returns the cap selector for the root SC.
    /// See spec pdf 6.1.2.3 Object Space
    pub fn root_sc(&self) -> CapSel {
        self.num_exc_sel as u64 + 2
    }
}

impl Debug for HIP {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HIP")
            .field("signature", &self.signature)
            .field("checksum", &self.checksum)
            .field("length", &self.length)
            .field("cpu_offs", &self.cpu_offs)
            .field("cpu_size", &self.cpu_size)
            .field("ioapic_offs", &self.ioapic_offs)
            .field("ioapic_size", &self.ioapic_size)
            .field("mem_offs", &self.mem_offs)
            .field("mem_size", &self.mem_size)
            .field("api_flg", &self.api_flg)
            .field("api_ver", &self.api_ver)
            .field("sel_num", &self.sel_num)
            .field("sel_exc", &self.num_exc_sel)
            .field("sel_vmi", &self.num_vmi_sel)
            .field("sel_gsi", &self.num_gsi_sel)
            .field("cfg_page", &self.cfg_page)
            .field("cfg_utcb", &self.cfg_utcb)
            .field("freq_tsc", &self.freq_tsc)
            .field("freq_bus", &self.freq_bus)
            .field("pci_bus_start", &self.pci_bus_start)
            .field("mcfg_base", &self.mcfg_base)
            .field("msfg_size", &self.msfg_size)
            .field("dmar_tables", &self.dmar_tables)
            .field("hpet_base", &self.hpet_base)
            .field("cap_vmx_sec_exec", &self.cap_vmx_sec_exec)
            .field("xsdt_rdst_table", &self.xsdt_rdst_table)
            .field("pm1a_cnt", &self.pm1a_cnt)
            .field("pm2b_cnt", &self.pm2b_cnt)
            .field("cpu_desc", &"<cpu_desc array>")
            .field("ioapic_desc", &"<ioapic_desc array>")
            .field("mem_desc", &"<mem_desc array>")
            .finish()
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

bitflags::bitflags! {
    pub struct HipFeatureFlags: u32 {
        /// The platform provides an IOMMU and the feature has been activated.
        const IOM = 1 << 0;
        /// The platform supports Intel Virtual Machine Extension and the feature has been activated.
        const VMX = 1 << 1;
        /// The platform supports AMD Secure Virtual Machine, and the feature has been activated.
        const SVM = 1 << 2;
    }
}
