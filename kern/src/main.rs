#![no_std]
#![no_main]

#[used]
#[link_section = ".requests"]
pub static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::new();

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: limine::request::FramebufferRequest = limine::request::FramebufferRequest::new();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    assert!(BASE_REVISION.is_supported());

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            for i in 0..500_u64 {
                // Calculate the pixel offset using the framebuffer information we obtained above.
                // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
                let pixel_offset = i * framebuffer.pitch() + i * 4;

                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                unsafe { *(framebuffer.addr().add(pixel_offset as usize) as *mut u32) = 0x0000FFFF };
            }
        }
    }

    loop {}
}
