use crate::cap_space::user::UserAppCapSpace;
#[cfg(feature = "foreign_rust_rt")]
use crate::rt::hybrid_rt::syscalls::sys_hybrid_call;
use crate::rt::services::allocate::AllocRequest;
use crate::rt::user_load_utcb::user_load_utcb_mut;
use core::alloc::Layout;
#[cfg(feature = "native_rust_rt")]
use libhedron::syscall::sys_call;

/// Allocates memory from the roottask allocator.
#[cfg(any(feature = "foreign_rust_rt", feature = "native_rust_rt"))]
pub fn alloc_service(layout: Layout) -> *mut u8 {
    let utcb = user_load_utcb_mut();
    utcb.store_data(&AllocRequest::new_alloc(layout)).unwrap();

    #[cfg(feature = "native_rust_rt")]
    sys_call(UserAppCapSpace::AllocatorServicePT.val()).unwrap();
    #[cfg(feature = "foreign_rust_rt")]
    sys_hybrid_call(UserAppCapSpace::AllocatorServicePT.val()).unwrap();

    utcb.load_data::<u64>().unwrap() as *mut u8
}

/// Allocates memory from the roottask allocator.
#[cfg(any(feature = "foreign_rust_rt", feature = "native_rust_rt"))]
pub unsafe fn dealloc_service(ptr: u64, layout: Layout) {
    let utcb = user_load_utcb_mut();
    utcb.store_data(&AllocRequest::new_delloc(ptr, layout))
        .unwrap();

    #[cfg(feature = "native_rust_rt")]
    sys_call(UserAppCapSpace::AllocatorServicePT.val()).unwrap();
    #[cfg(feature = "foreign_rust_rt")]
    sys_hybrid_call(UserAppCapSpace::AllocatorServicePT.val()).unwrap();
}
