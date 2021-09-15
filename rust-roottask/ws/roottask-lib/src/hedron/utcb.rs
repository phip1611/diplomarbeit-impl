/// User Thread Control Block.
/// Data-Buffer for IPC.
#[derive(Debug)]
#[repr(C)]
pub struct UtcbData {
    mtd: u64,
    inst_len: u64,
    rip: u64,
    rflags: u64,
    intr_state: u32,
    actv_state: u32,
    intr_info: u32,
    intr_errir: u32,
    vect_info: u32,
    vect_error: u32,
    rax: u64,
    rcx: u64,
    rdx: u64,
    rbx: u64,
    rsp: u64,
    rbp: u64,
    rsi: u64,
    rdi: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    qual: [u64; 2],
    ctrl: [u32; 2],
    xrc0: u64,
    cr0: u64,
    cr2: u64,
    cr3: u64,
    cr4: u64,
    pdpte: [u64; 4],
    cr8: u64,
    efer: u64,
    pat: u64,
    star: u64,
    lstar: u64,
    fmask: u64,
    kernel_gs_base: u64,
    dr7: u64,
    sysenter_cs: u64,
    sysenter_rsp: u64,
    sysenter_rip: u64,
    es: UtcbSegment,
    cs: UtcbSegment,
    ss: UtcbSegment,
    ds: UtcbSegment,
    fs: UtcbSegment,
    gs: UtcbSegment,
    ld: UtcbSegment,
    tr: UtcbSegment,
    gd: UtcbSegment,
    id: UtcbSegment,
    tsc_val: u64,
    tsc_off: u64,
    tsc_aux: u32,
    exc_bitmap: u32,
    tpr_threshold: u32,
    reserved2: u32,

    eoi_bitmap: [u64; 4],

    vintr_status: u16,
    reserved_array: [u16; 3],

    cr0_mon: u64,
    cr4_mon: u64,
    spec_ctrl: u64,
}

impl UtcbData {
    pub fn mtd(&self) -> u64 {
        self.mtd
    }
    pub fn inst_len(&self) -> u64 {
        self.inst_len
    }
    pub fn rip(&self) -> u64 {
        self.rip
    }
    pub fn rflags(&self) -> u64 {
        self.rflags
    }
    pub fn intr_state(&self) -> u32 {
        self.intr_state
    }
    pub fn actv_state(&self) -> u32 {
        self.actv_state
    }
    pub fn intr_info(&self) -> u32 {
        self.intr_info
    }
    pub fn intr_errir(&self) -> u32 {
        self.intr_errir
    }
    pub fn vect_info(&self) -> u32 {
        self.vect_info
    }
    pub fn vect_error(&self) -> u32 {
        self.vect_error
    }
    pub fn rax(&self) -> u64 {
        self.rax
    }
    pub fn rcx(&self) -> u64 {
        self.rcx
    }
    pub fn rdx(&self) -> u64 {
        self.rdx
    }
    pub fn rbx(&self) -> u64 {
        self.rbx
    }
    pub fn rsp(&self) -> u64 {
        self.rsp
    }
    pub fn rbp(&self) -> u64 {
        self.rbp
    }
    pub fn rsi(&self) -> u64 {
        self.rsi
    }
    pub fn rdi(&self) -> u64 {
        self.rdi
    }
    pub fn r8(&self) -> u64 {
        self.r8
    }
    pub fn r9(&self) -> u64 {
        self.r9
    }
    pub fn r10(&self) -> u64 {
        self.r10
    }
    pub fn r11(&self) -> u64 {
        self.r11
    }
    pub fn r12(&self) -> u64 {
        self.r12
    }
    pub fn r13(&self) -> u64 {
        self.r13
    }
    pub fn r14(&self) -> u64 {
        self.r14
    }
    pub fn r15(&self) -> u64 {
        self.r15
    }
    pub fn qual(&self) -> &[u64; 2] {
        &self.qual
    }
    pub fn ctrl(&self) -> &[u32; 2] {
        &self.ctrl
    }
    pub fn xrc0(&self) -> u64 {
        self.xrc0
    }
    pub fn cr0(&self) -> u64 {
        self.cr0
    }
    pub fn cr2(&self) -> u64 {
        self.cr2
    }
    pub fn cr3(&self) -> u64 {
        self.cr3
    }
    pub fn cr4(&self) -> u64 {
        self.cr4
    }
    pub fn pdpte(&self) -> &[u64; 4] {
        &self.pdpte
    }
    pub fn cr8(&self) -> u64 {
        self.cr8
    }
    pub fn efer(&self) -> u64 {
        self.efer
    }
    pub fn pat(&self) -> u64 {
        self.pat
    }
    pub fn star(&self) -> u64 {
        self.star
    }
    pub fn lstar(&self) -> u64 {
        self.lstar
    }
    pub fn fmask(&self) -> u64 {
        self.fmask
    }
    pub fn kernel_gs_base(&self) -> u64 {
        self.kernel_gs_base
    }
    pub fn dr7(&self) -> u64 {
        self.dr7
    }
    pub fn sysenter_cs(&self) -> u64 {
        self.sysenter_cs
    }
    pub fn sysenter_rsp(&self) -> u64 {
        self.sysenter_rsp
    }
    pub fn sysenter_rip(&self) -> u64 {
        self.sysenter_rip
    }
    pub fn es(&self) -> &UtcbSegment {
        &self.es
    }
    pub fn cs(&self) -> &UtcbSegment {
        &self.cs
    }
    pub fn ss(&self) -> &UtcbSegment {
        &self.ss
    }
    pub fn ds(&self) -> &UtcbSegment {
        &self.ds
    }
    pub fn fs(&self) -> &UtcbSegment {
        &self.fs
    }
    pub fn gs(&self) -> &UtcbSegment {
        &self.gs
    }
    pub fn ld(&self) -> &UtcbSegment {
        &self.ld
    }
    pub fn tr(&self) -> &UtcbSegment {
        &self.tr
    }
    pub fn gd(&self) -> &UtcbSegment {
        &self.gd
    }
    pub fn id(&self) -> &UtcbSegment {
        &self.id
    }
    pub fn tsc_val(&self) -> u64 {
        self.tsc_val
    }
    pub fn tsc_off(&self) -> u64 {
        self.tsc_off
    }
    pub fn tsc_aux(&self) -> u32 {
        self.tsc_aux
    }
    pub fn exc_bitmap(&self) -> u32 {
        self.exc_bitmap
    }
    pub fn tpr_threshold(&self) -> u32 {
        self.tpr_threshold
    }
    pub fn reserved2(&self) -> u32 {
        self.reserved2
    }
    pub fn eoi_bitmap(&self) -> &[u64; 4] {
        &self.eoi_bitmap
    }
    pub fn vintr_status(&self) -> u16 {
        self.vintr_status
    }
    pub fn reserved_array(&self) -> &[u16; 3] {
        &self.reserved_array
    }
    pub fn cr0_mon(&self) -> u64 {
        self.cr0_mon
    }
    pub fn cr4_mon(&self) -> u64 {
        self.cr4_mon
    }
    pub fn spec_ctrl(&self) -> u64 {
        self.spec_ctrl
    }
}

#[derive(Debug)]
pub struct UtcbSegment {
    sel: u16,
    ar: u16,
    limit: u32,
    base: u64,
}
