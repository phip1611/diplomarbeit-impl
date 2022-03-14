//! Everything related to extract the runtime environment from the Tar file which is provided
//! in a Multiboot boot module.

use crate::mem::{
    MappedMemory,
    ROOT_MEM_MAPPER,
    VIRT_MEM_ALLOC,
};
use crate::process::Process;
use crate::process::SyscallAbi;
use crate::process::PROCESS_MNG;
use alloc::rc::Rc;
use alloc::string::String;
use core::alloc::Layout;
use libhrstd::cstr::CStr;
use libhrstd::libhedron::mem::PAGE_SIZE;
use libhrstd::libhedron::MemCapPermissions;
use libhrstd::libhedron::{
    HipMem,
    HipMemType,
    HIP,
};
use libhrstd::mem::calc_page_count;
use tar_no_std::TarArchiveRef;

/// Contains all files of the userland (runtime services + user applications) that
/// are provided by the userland tarball. The tarball is provided as multiboot boot module.
/// Some files appear twice as "debug" and as "release" version to cope with situations
/// where the release build doesn't work in QEMU (due to fancy CPU features) but should be
/// executed on real hardware.
#[derive(Debug)]
#[allow(unused)]
pub struct InitialUserland {
    /// Release-version (=maximum optimized + fancy CPU features) of `hedron_native_hello_world_rust_debug_elf`
    hedron_native_hello_world_rust_elf: MappedMemory,
    /// Statically compiled Hello World for Linux (C + musl/gcc)
    linux_c_hello_world_elf: MappedMemory,
    /// Statically compiled Hello World for Linux (Rust + musl/LLVM)
    linux_rust_hello_world_elf: MappedMemory,
    /// Statically compiled Hello World for Linux (Rust + musl/LLVM) + hybrid part (native Hedron syscalls)
    linux_rust_hello_world_hybrid_elf: MappedMemory,
    /// Statically compiled Linux Application with Hybrid Parts that will act as my Evaluation Benchmark.
    /// It will output all relevant information to serial. (debug)
    linux_rust_hybrid_benchmark_elf: MappedMemory,
    // /// statically compiled Hello World for Linux (Zig)
    // Statically compiled Matrix Multiplication in C that allocates matrices on the heap.
    linux_c_matrix_mult_elf: MappedMemory,
    // Statically compiled AUX Vec Dump tool.
    linux_c_aux_dump_elf: MappedMemory,
}

impl InitialUserland {
    pub fn load(hip: &HIP, root: &Rc<Process>) -> Self {
        let hip_mem = Self::find_userland_tar_mem_desc(hip, root)
            .ok_or(HedronUserlandError::FileNotFound)
            .unwrap();

        // Mep mem with full permissions; I reduce the permissions to the minimum when I start the
        // dedicated processes because rights can't be upgraded when I map them from
        // A to B and A!=B!=ROOTTASK.
        let mapped_mem = ROOT_MEM_MAPPER.lock().mmap(
            root,
            root,
            hip_mem.addr(),
            None,
            calc_page_count(hip_mem.size() as usize) as u64,
            MemCapPermissions::all(),
        );

        let tar_file = TarArchiveRef::new(mapped_mem.mem_as_slice(hip_mem.size() as usize));
        log::trace!("userland tar contains files:");
        tar_file
            .entries()
            .for_each(|e| log::trace!("    {} ({} bytes)", e.filename(), e.size()));

        Self {
            hedron_native_hello_world_rust_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "native-hello-world-rust-bin",
                root,
            )
            .unwrap(),
            linux_c_hello_world_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_c_hello_world_musl",
                root,
            )
            .unwrap(),
            linux_rust_hello_world_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_rust_hello_world_musl",
                root,
            )
            .unwrap(),
            linux_rust_hello_world_hybrid_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_rust_hello_world_hybrid_musl",
                root,
            )
            .unwrap(),
            linux_rust_hybrid_benchmark_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_rust_hybrid_benchmark",
                root,
            )
            .unwrap(),
            linux_c_matrix_mult_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_c_matrix_mult_musl",
                root,
            )
            .unwrap(),
            linux_c_aux_dump_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_c_dump_aux_musl",
                root,
            )
            .unwrap(),
        }
    }

    /// Finds the HipMem descriptor that holds the Tar file with the userland.
    fn find_userland_tar_mem_desc<'a>(hip: &'a HIP, root: &Rc<Process>) -> Option<&'a HipMem> {
        hip.mem_desc_iterator()
            .map(|hipmem| (hipmem, Self::hip_mem_mb_cmd_str(hipmem, root)))
            .filter(|(_, cmdline)| cmdline.is_some())
            .map(|(hipmem, cmdline)| (hipmem, cmdline.unwrap()))
            .filter(|(_, cmdline)| *cmdline == USERLAND_MB_CMDLINE_ARGUMENT)
            .map(|(hipmem, _)| hipmem)
            .next()
    }

    /// Takes a hip mem object of type multiboot and returns the cmdline string
    /// if available.
    fn hip_mem_mb_cmd_str<'a>(hip_mem_mb: &'a HipMem, root: &Rc<Process>) -> Option<&'a str> {
        if hip_mem_mb.typ() != HipMemType::MbModule {
            return None;
        }

        // should never fail, because HipMem objects of type Multiboot boot module
        // always have a cmdline string pointer (but the length might be zero)
        let cmdline_ptr = hip_mem_mb.cmdline()? as u64;

        let cmdline_page = cmdline_ptr & !0xfff;
        log::debug!("mapping memory for MB mod cmdline ptr");
        let mem =
            ROOT_MEM_MAPPER
                .lock()
                .mmap(root, root, cmdline_page, None, 1, MemCapPermissions::READ);
        let cmdline = mem.old_to_new_addr(cmdline_ptr);

        let cmdline = CStr::try_from(cmdline as *const u8).expect("must be valid c string");
        let cmdline = cmdline.as_str();
        if cmdline.is_empty() {
            log::debug!("cmdline string is empty");
            return None;
        } else {
            log::debug!("cmdline string: {}", cmdline);
        }

        // the cmdline arg describes the payload, i.e. "userland"
        let cmdline_arg = if cmdline.contains(' ') {
            // multiboot boot loaders put something like
            // './build/roottask-bin--release.elf roottask'
            // ==> 'roottask'
            cmdline
                .split_once(' ')
                .map(|(_file, first_arg)| first_arg)
                .unwrap()
        } else {
            // SVP UEFI loader put something like
            // 'roottask'
            // ==> 'roottask'
            cmdline
        };

        Some(cmdline_arg)
    }

    /// Extracts an ELF from the TarArchive and maps it to a page-aligned destination with
    /// RWX rights, if the given filename pattern matches one of the files.
    fn map_tar_entry_to_page_aligned_dest(
        tar: &TarArchiveRef,
        filename: &str,
        root: &Rc<Process>,
    ) -> Option<MappedMemory> {
        let entry = tar.entries().find(|e| e.filename().contains(filename))?;
        // looks a bit weird, but is fine for a quick & dirty solution. I need some destination, where I can map the new memory too!
        let phys_src = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(entry.size(), PAGE_SIZE).unwrap());

        log::debug!("mapping memory for Userland file: {}", filename);
        let mut mapped_mem = ROOT_MEM_MAPPER.lock().mmap(
            root,
            root,
            phys_src,
            None,
            calc_page_count(entry.size()) as u64,
            MemCapPermissions::all(),
        );

        // copy data to mapped mem
        unsafe {
            let src_ptr = entry.data().as_ptr();
            let dest_ptr = mapped_mem.mem_as_ptr_mut();
            core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, entry.size());
        }

        Some(mapped_mem)
    }

    /// Bootstraps the userland. Starts processes in the process manager.
    pub fn bootstrap(&self) {
        /*PROCESS_MNG.lock().start_process(
            self.hedron_native_hello_world_rust_elf.clone(),
            String::from("Hedron-native Hello World Rust+libhrstd [RELEASE]"),
            SyscallAbi::NativeHedron,
        );*/

        /*PROCESS_MNG.lock().start_process(
            self.linux_c_hello_world_elf.clone(),
            String::from("Linux C Hello World Musl"),
            SyscallAbi::Linux,
        );*/

        /*PROCESS_MNG.lock().start_process(
            self.linux_rust_hello_world_elf.clone(),
            String::from("Linux Hello World Hybrid (Rust + musl) [RELEASE]"),
            SyscallAbi::Linux,
        );*/

        PROCESS_MNG.lock().start_process(
            self.linux_rust_hybrid_benchmark_elf.clone(),
            String::from("My Diplom thesis evaluation benchmark. [RELEASE]"),
            SyscallAbi::Linux,
        );

        /*PROCESS_MNG.lock().start_process(
            self.linux_c_matrix_mult_elf.clone(),
            String::from("C Matrix Multiplication"),
            SyscallAbi::Linux,
        );*/
    }
}

#[derive(Debug, Copy, Clone)]
pub enum HedronUserlandError {
    FileNotFound,
}

/// The first argument describing the given payload as userland file.
const USERLAND_MB_CMDLINE_ARGUMENT: &str = "userland";
