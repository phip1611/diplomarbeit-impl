//! Everything related to extract the runtime environment from the Tar file which is provided
//! in a Multiboot boot module.

use crate::mem::{
    MappedMemory,
    ROOT_MEM_MAPPER,
    VIRT_MEM_ALLOC,
};
use crate::process_mng::manager::PROCESS_MNG;
use crate::process_mng::syscall_abi::SyscallAbi;
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
    /// Hedron-native Rust App that acts as my testing playground.
    hedron_native_hello_world_rust_debug_elf: MappedMemory,
    /// Release-version (=maximum optimized + fancy CPU features) of `hedron_native_hello_world_rust_debug_elf`
    hedron_native_hello_world_rust_release_elf: MappedMemory,
    /// The file system service compiled as Hedron-native Rust application.
    hedron_native_fs_service_rust_debug_elf: MappedMemory,
    /// Release-version (=maximum optimized + fancy CPU features) of `hedron_native_fs_service_rust_debug_elf`
    hedron_native_fs_service_rust_release_elf: MappedMemory,
    /// Statically compiled Hello World for Linux (C + musl/gcc)
    linux_c_hello_world_elf: MappedMemory,
    /// Statically compiled Hello World for Linux (Rust + musl/LLVM)
    linux_rust_hello_world_debug_elf: MappedMemory,
    /// Statically compiled Hello World for Linux (Rust + musl/LLVM) + hybrid part (native Hedron syscalls)
    linux_rust_hello_world_hybrid_debug_elf: MappedMemory,
    /// Release-version (=maximum optimized + fancy CPU features) of `linux_rust_hybrid_benchmark_debug_elf`
    linux_rust_hello_world_hybrid_release_elf: MappedMemory,
    /// Statically compiled Linux Application with Hybrid Parts that will act as my Evaluation Benchmark.
    /// It will output all relevant information to serial. (debug)
    linux_rust_hybrid_benchmark_debug_elf: MappedMemory,
    /// Release-version (=maximum optimized + fancy CPU features) of `linux_rust_hybrid_benchmark_debug_elf`
    linux_rust_hybrid_benchmark_release_elf: MappedMemory,
    // /// statically compiled Hello World for Linux (Zig)
    // linux_zig_hello_world_elf: MappedMemory,
}

impl InitialUserland {
    pub fn load(hip: &HIP) -> Self {
        let hip_mem = Self::find_userland_tar_mem_desc(hip)
            .ok_or(HedronUserlandError::FileNotFound)
            .unwrap();
        // all permissions; I reduce the permissions to the minimum when I start the dedicated processes
        let mapped_mem = ROOT_MEM_MAPPER.lock().mmap(
            hip_mem.addr(),
            calc_page_count(hip_mem.size()),
            MemCapPermissions::all(),
        );

        let tar_file = TarArchiveRef::new(mapped_mem.mem_as_slice(hip_mem.size() as usize));
        log::trace!("userland tar contains files:");
        tar_file
            .entries()
            .for_each(|e| log::trace!("    {} ({} bytes)", e.filename(), e.size()));

        Self {
            hedron_native_hello_world_rust_debug_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "helloworld-bin--debug",
            )
            .unwrap(),
            hedron_native_hello_world_rust_release_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "helloworld-bin--release",
            )
            .unwrap(),
            hedron_native_fs_service_rust_debug_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "fileserver-bin--debug",
            )
            .unwrap(),
            hedron_native_fs_service_rust_release_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "fileserver-bin--release",
            )
            .unwrap(),
            linux_c_hello_world_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_c_hello_world_musl",
            )
            .unwrap(),
            linux_rust_hello_world_debug_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_rust_hello_world_musl--debug",
            )
            .unwrap(),
            linux_rust_hello_world_hybrid_debug_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_rust_hello_world_hybrid_musl--debug",
            )
            .unwrap(),
            linux_rust_hello_world_hybrid_release_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_rust_hello_world_hybrid_musl--release",
            )
            .unwrap(),
            linux_rust_hybrid_benchmark_debug_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_rust_hybrid_benchmark--debug",
            )
            .unwrap(),
            linux_rust_hybrid_benchmark_release_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux_rust_hybrid_benchmark--release",
            )
            .unwrap(),
            /*linux_rust_hello_world_elf: Self::map_tar_entry_to_page_aligned_dest(
                &tar_file,
                "linux-rust-hello-world-bin",
            )
            .unwrap(),*/
        }
    }

    /// Finds the HipMem descriptor that holds the Tar file with the userland.
    fn find_userland_tar_mem_desc(hip: &HIP) -> Option<&HipMem> {
        let mut userland_tar_mem_desc = None;
        let mb_modules = hip
            .mem_desc_iterator()
            .filter(|x| x.typ() == HipMemType::MbModule);
        for hipmem in mb_modules {
            let cmdline = hipmem.cmdline().unwrap() as u64;
            let cmdline_page = cmdline & !0xfff;
            log::debug!("mapping memory for MB mod cmdline ptr");
            let mem = ROOT_MEM_MAPPER
                .lock()
                .mmap(cmdline_page, 1, MemCapPermissions::READ);
            let cmdline = mem.old_to_new_addr(cmdline);
            let cmdline = CStr::try_from(cmdline as *const u8).unwrap();
            let cmdline = cmdline.as_str();
            assert!(cmdline.len() > 0,
                    "cmdline must be something. If empty, there is some bigger problem with the memory mapping?!"
            );
            let first_arg = cmdline.split_once(' ').map(|(_file, first_arg)| first_arg);

            // cmdline-string is sometihng like: "./build/roottask-bin_debug.elf roottask"
            // I want to check if the first parameter (after first space) is "userland".
            if first_arg.is_some() && first_arg.unwrap() == MB_MODULE_ARGUMENT {
                userland_tar_mem_desc.replace(hipmem);
                break;
            }
        }
        userland_tar_mem_desc
    }

    /// Extracts an ELF from the TarArchive and maps it to a page-aligned destination with
    /// RWX rights, if the given filename pattern matches one of the files.
    fn map_tar_entry_to_page_aligned_dest(
        tar: &TarArchiveRef,
        filename: &str,
    ) -> Option<MappedMemory> {
        let entry = tar.entries().find(|e| e.filename().contains(filename))?;
        // looks a bit weird, but is fine for a quick & dirty solution. I need some destination, where I can map the new memory too!
        let phys_src = VIRT_MEM_ALLOC
            .lock()
            .next_addr(Layout::from_size_align(entry.size(), PAGE_SIZE).unwrap());

        log::debug!("mapping memory for Userland file: {}", filename);
        let mut mapped_mem = ROOT_MEM_MAPPER.lock().mmap(
            phys_src,
            calc_page_count(entry.size() as u64),
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
        /*let elf_file = if runs_inside_qemu().is_very_likely() {
            self.linux_rust_hybrid_benchmark_debug_elf
        } else {
            self.linux_rust_hybrid_benchmark_release_elf
        };
        PROCESS_MNG.lock().start_process(
            elf_file,
            String::from("My Diplom thesis evaluation benchmark."),
            SyscallAbi::Linux,
        );*/

        /*PROCESS_MNG.lock().start_process(
            self.hedron_native_hello_world_rust_release_elf.clone(),
            String::from("Hedron-native Hello World Rust+libhrstd [RELEASE]"),
            SyscallAbi::NativeHedron,
        );*/

        /*PROCESS_MNG.lock().start_process(
            self.linux_c_hello_world_elf.clone(),
            String::from("Linux C Hello World Musl"),
            SyscallAbi::Linux,
        );*/
        /*PROCESS_MNG.lock().start_process(
            self.linux_rust_hello_world_hybrid_debug_elf.clone(),
            String::from("Linux Hello World Hybrid (Rust + musl) [DEBUG]"),
            SyscallAbi::Linux,
        );*/
        PROCESS_MNG.lock().start_process(
            self.linux_rust_hello_world_hybrid_release_elf.clone(),
            String::from("Linux Hello World Hybrid (Rust + musl) [RELEASE]"),
            SyscallAbi::Linux,
        );

        /*PROCESS_MNG.lock().start_process(
            self.linux_rust_hybrid_benchmark_release_elf.clone(),
            String::from("My Diplom thesis evaluation benchmark. [RELEASE]"),
            SyscallAbi::Linux,
        );*/
    }
}

#[derive(Debug, Copy, Clone)]
pub enum HedronUserlandError {
    FileNotFound,
}

/// The first argument describing the given payload as userland file.
const MB_MODULE_ARGUMENT: &str = "userland";
