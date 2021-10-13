//! Everything related to extract the runtime environment from the Tar file which is provided
//! in a Multiboot boot module.

use crate::mem::MappingHelper;
use libhrstd::cstr::CStr;
use libhrstd::libhedron::capability::MemCapPermissions;
use libhrstd::libhedron::hip::{
    HipMem,
    HipMemType,
    HIP,
};
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::syscall::pd_ctrl::DelegateFlags;
use tar_no_std::TarArchive;

#[derive(Debug, Copy, Clone)]
pub enum HedronUserlandTarError {
    FileNotFound,
}

/// The first argument describing the given payload as userland file.
const MB_MODULE_ARGUMENT: &str = "userland";

/// Finds the Tar archive from the information in the [`HIP`], that represents the whole
/// initial userland/runtime environment the roottask shall set up. The Tar files comes
/// from a Multiboot module.
///
/// The Tar archive is expected to be valid with a flat hierarchy of the relevant
/// files. The expected structure is hard-coded into this function.
///
/// # Safety
/// This function maps physical pages into the heap of the roottask. This function will
/// not mark the Heap pages used for this as protected, but will give the pages back
/// to the Heap, if not used anymore.
///
/// This will probably prevent a second reading of the data, because other heap data could
/// already overwritten it. Furthermore, the memory descriptors describing the location
/// of the Multiboot modules, will overlap with the Heap of the roottask.
///
/// Because only "a few pages" will be lost and I need to read them once only, I
/// take the simple, pragmatic approach, and live with this.
pub unsafe fn find_hedron_userland_tar(hip: &HIP) -> Result<TarArchive, HedronUserlandTarError> {
    let module = find_tar_mb_mod(hip).ok_or(HedronUserlandTarError::FileNotFound)?;
    let tar_archive_copy = map_tar_mb_mod(hip, module);
    Ok(tar_archive_copy)
}

/// Finds the [`HipMem`]-object that describes the Multiboot module
/// that contains the Tar archive with the userland/runtime environment.
///
/// # P
fn find_tar_mb_mod(hip: &HIP) -> Option<&HipMem> {
    for hip_mem_desc in hip
        .mem_desc_iterator()
        .filter(|x| x.typ() == HipMemType::MbModule)
    {
        let cmdline_addr = hip_mem_desc
            .cmdline()
            .expect("MB-Module Descriptor must have a CMD-Line pointer")
            as usize;

        // I expect that the C-Str will never be longer than one page.
        let mut mapping_region = MappingHelper::new(1);
        mapping_region
            .map(
                hip.root_pd(),
                hip.root_pd(),
                cmdline_addr,
                MemCapPermissions::READ | MemCapPermissions::WRITE,
                DelegateFlags::new(false, false, false, true, 0),
            )
            .unwrap();

        // Virtual address that maps to the physical address
        let cmdline_addr = mapping_region.old_to_new_addr(cmdline_addr) as *const u8;
        let cmdline = CStr::try_from(cmdline_addr).unwrap();
        let cmdline = cmdline.as_str();

        assert!(cmdline.len() > 0,
                "cmdline must be something. If empty, there is some bigger problem with the memory mapping?!"
        );

        let first_arg = cmdline.split_once(' ').map(|(_file, first_arg)| first_arg);
        dbg!(cmdline);
        dbg!(first_arg);

        // cmdline-string is sometihng like: "./build/roottask-bin_debug.elf roottask"
        // I want to check if the first parameter (after first space) is "userland".
        if first_arg.is_some() && first_arg.unwrap() == MB_MODULE_ARGUMENT {
            return Some(hip_mem_desc);
        }
    }
    None
}

/// Maps the memory from the Tar file into the heap, and
/// copies the data into an heap-owning Tar archive ([`TarArchive`]).
///
/// # Safety
/// The memory used for the mapping as destination gets freed afterwards.
/// From that, further memory allocations are likely to overwrite
/// the Multiboot module data shortly.
fn map_tar_mb_mod(hip: &HIP, mb_mod: &HipMem) -> TarArchive {
    let div = mb_mod.size() as usize / PAGE_SIZE;
    let remainder = mb_mod.size() as usize % PAGE_SIZE;
    let mut page_count = div;
    if remainder != 0 {
        page_count += 1
    }

    let mut mapping_region = MappingHelper::new(page_count);
    mapping_region
        .map(
            hip.root_pd(),
            hip.root_pd(),
            mb_mod.addr() as usize,
            MemCapPermissions::READ | MemCapPermissions::WRITE,
            DelegateFlags::new(false, false, false, true, 0),
        )
        .unwrap();

    // Reference to the data
    let tar_archive_data = mapping_region.mem_as_slice::<u8>(mb_mod.size() as usize);
    // owned data copy on the heap, independent from mapping_region
    let tar_archive_data = tar_archive_data.to_vec().into_boxed_slice();
    TarArchive::from(tar_archive_data)
}
