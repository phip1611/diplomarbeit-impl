#[derive(Debug, Default)]
#[repr(C)]
pub struct LapicInfo {
    id: u32,
    version: u32,
    svr: u32,
    reserved: u32,
    lvt_timer: u32,
    lvt_lint0: u32,
    lvt_lint1: u32,
    lvt_error: u32,
    lvt_perfm: u32,
    lvt_therm: u32,
}
