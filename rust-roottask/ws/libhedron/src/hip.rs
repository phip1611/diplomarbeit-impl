use crate::acpi_gas::AcpiGas;
use crate::capability::CapSel;
use crate::consts::{
    NUM_CPUS,
    NUM_IOAPICS,
};
use crate::cpu::LapicInfo;
use core::fmt::{
    Debug,
    Formatter,
};
use core::mem::size_of;

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
    /// Equal to `size_of::<HipMem>()`
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

    cpu_desc: [HipCpu; NUM_CPUS],
    ioapic_desc: [HipIoApic; NUM_IOAPICS],
    /// Points to the first item of the dynamically sized array
    /// of memory descriptors. The [`HipMemDescIterator`] iterates
    /// over them safely. As in the Hedron/C++ code, this doesn't
    /// increase the static size of this struct. Also see
    /// https://stackoverflow.com/questions/6732184/
    _mem_desc_arr: [HipMem; 0],
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

    /// Returns the dynamic count of [`HipMem`]-values
    /// at the end of the [`HIP`]. In other words,
    /// the length of the array.
    pub fn mem_desc_count(&self) -> usize {
        assert_eq!(self.mem_size, size_of::<HipMem>() as u16);
        (self.length as usize - size_of::<Self>()) / self.mem_size as usize
    }

    /// Returns an iterator of type [`HipMemDescIterator`].
    pub fn mem_desc_iterator(&self) -> HipMemDescIterator {
        assert_eq!(
            size_of::<HipMem>(),
            self.mem_size as usize,
            "the struct must have an equal size to the struct in Hedron"
        );
        HipMemDescIterator::new(self)
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

/// Identifies all memory that is initially in use. From this, it can be derived
/// what (physical? TODO) memory inside the system is free.
#[repr(C)]
pub struct HipMem {
    /// Base address.
    addr: u64,
    /// Offset from base address.
    size: u64,
    typ: HipMemType,
    /// 0 for [`HipMemType::Hypervisor`], otherwise a pointer to a NULL-terminated `C-String` which
    /// represents the `cmdline` of the Multiboot boot module.
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
    /*pub fn mb_cmdline(&self) -> Option<&'a str> {
        unsafe {
            let ptr = (self.aux as *const u32 as *const u8);
            let slice = core::slice::from_raw_parts(ptr)
            core::str::from_utf8()
        }
    }*/
}

impl Debug for HipMem {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HipMem")
            // print hex
            .field("type", &self.typ)
            .field("addr", &(self.addr as *const u64))
            .field("size", &self.size)
            .field("(end_addr)", &((self.addr + self.size) as *const u64))
            .field("aux", &(self.aux as *const u64))
            .finish()
    }
}

/// Type for [`HipMem`].
#[derive(Debug, Copy, Clone, PartialEq)]
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

/// Iterator over the dynamic (= at compile time unknown) number of [`HipMem`]-descriptors
/// stored at the end of the [`HIP`].
#[derive(Debug)]
pub struct HipMemDescIterator<'a> {
    hip: &'a HIP,
    mem_desc_count: usize,
    iteration_counter: usize,
    slice: &'a [HipMem],
}

impl<'a> HipMemDescIterator<'a> {
    /// Constructs a slice of the memory descriptor array and can iterate through it.
    fn new(hip: &'a HIP) -> Self {
        let count = hip.mem_desc_count();
        let slice = unsafe { core::slice::from_raw_parts(hip._mem_desc_arr.as_ptr(), count) };
        Self {
            hip,
            mem_desc_count: count,
            iteration_counter: 0,
            slice,
        }
    }
}

impl<'a> Iterator for HipMemDescIterator<'a> {
    type Item = &'a HipMem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iteration_counter >= self.mem_desc_count {
            return None;
        }
        let elem = &self.slice[self.iteration_counter];
        self.iteration_counter += 1;
        Some(elem)
    }

    /// Reduces memory allocations; pollutes my log less :D
    fn size_hint(&self) -> (usize, Option<usize>) {
        const EXPECTED_ELEMENTS: usize = 15;
        (EXPECTED_ELEMENTS - self.iteration_counter, None)
    }
}

#[cfg(test)]
mod tests {
    use crate::hip::{
        HipCpu,
        HipIoApic,
        HipMem,
        HipMemType,
        HIP,
    };
    use alloc::vec::Vec;
    use core::mem::size_of;

    /// Helps us to easy identify problems that would effect the dynamic array parsing otherwise.
    /// Took the size values from the C code. It's unfortunate that this is a manual operation.
    #[test]
    fn test_hip_size_as_large_as_in_hedron() {
        assert_eq!(
            size_of::<HipMem>(),
            24,
            "HipMem must be as large as inside Hedron code"
        );
        assert_eq!(
            size_of::<HipCpu>(),
            48,
            "HipCpu must be as large as inside Hedron code"
        );
        assert_eq!(
            size_of::<HipIoApic>(),
            16,
            "HipIoApic must be as large as inside Hedron code"
        );
        assert_eq!(
            size_of::<HIP>(),
            3352,
            "HIP must be as large as inside Hedron code"
        );
    }

    #[test]
    fn test_hip_mem_type() {
        assert_eq!(HipMemType::Hypervisor as u32, 0xffffffff);
        assert_eq!(HipMemType::MbModule as u32, 0xfffffffe);
    }

    #[test]
    fn test_hip_mem_desc_iter() {
        let mut bytes = [0_u8; size_of::<HIP>() + 4 * size_of::<HipMem>()];
        let mut hip = unsafe { &mut *(bytes.as_mut_ptr() as *mut HIP) };
        hip.length = bytes.len() as u16;
        hip.mem_size = size_of::<HipMem>() as u16;

        assert_eq!(
            hip.mem_desc_count(),
            4,
            "must have exactly 4 mem desc items"
        );

        unsafe {
            let arr = bytes.as_ptr().add(size_of::<HIP>()) as *mut HipMem;
            let mut arr = core::slice::from_raw_parts_mut(arr, 4);
            arr[0].typ = HipMemType::Hypervisor;
            arr[0].addr = 0x1337;
            arr[0].size = 42;

            arr[1].typ = HipMemType::Hypervisor;
            arr[1].addr = 0xdeadbeef;
            arr[1].size = 0xc0ffee;

            arr[2].typ = HipMemType::MbModule;
            arr[2].addr = 0xaffeaffe;
            arr[2].size = 73;

            arr[3].typ = HipMemType::MbModule;
            arr[3].addr = 0xbadb001;
            arr[3].size = 0;
        }

        let mem_descs = hip.mem_desc_iterator().collect::<Vec<_>>();
        assert_eq!(mem_descs.len(), 4, "must find 4 hip memory descriptors");
        println!("{:#?}", mem_descs);
        assert_eq!(mem_descs[0].typ, HipMemType::Hypervisor);
        assert_eq!(mem_descs[0].addr, 0x1337);
        assert_eq!(mem_descs[0].size, 42);

        assert_eq!(mem_descs[1].typ, HipMemType::Hypervisor);
        assert_eq!(mem_descs[1].addr, 0xdeadbeef);
        assert_eq!(mem_descs[1].size, 0xc0ffee);

        assert_eq!(mem_descs[2].typ, HipMemType::MbModule);
        assert_eq!(mem_descs[2].addr, 0xaffeaffe);
        assert_eq!(mem_descs[2].size, 73);

        assert_eq!(mem_descs[3].typ, HipMemType::MbModule);
        assert_eq!(mem_descs[3].addr, 0xbadb001);
        assert_eq!(mem_descs[3].size, 0);
    }
}
