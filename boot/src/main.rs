#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;

// These use statements should be collated together
use log::info;
use uefi::prelude::*;

use uefi::{CStr16, Char16};

use uefi::proto::media::file::{File, FileInfo};

use uefi::proto::media::file::FileHandle;

use uefi::proto::console::gop::{PixelFormat, FrameBuffer, GraphicsOutput};

// My crates
mod bdf_loader;

use bdf_loader::load_bdf_to_mono_font;

use graphics::CPUFrameBuffer;

/// Reads a file (passed as a handle) to an owned heap array and returns it
fn read_file_from_handle(file_handle: FileHandle) -> Result<Vec<u8>, Status>{
    let mut file = match file_handle.into_regular_file() {
        Some(file) => file,
        None => return Err(Status::ABORTED)
    };

    let mut file_ram_location = {
        let file_size = match file.get_boxed_info::<FileInfo>() {
            Ok(info) => info.file_size(),
            Err(err) => return Err(err.status())
        };

        let mut ret = Vec::<u8>::with_capacity(file_size as usize);

        ret.resize(file_size as usize, 0);

        ret
    };

    if file.read(file_ram_location.as_mut_slice()).is_err() {
        return Err(Status::ABORTED)
    }

    return Ok(file_ram_location);
}

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
 
    let boot_services = system_table.boot_services();

    // Read the relavent information files into ram
   
    info!("Getting file system handle");

    let fs_handle = match boot_services.get_image_file_system(image_handle){
        Ok(handle) => handle,
        Err(err) => return err.status()
    };

    info!("Getting root directory");
    
    let mut root_dir = match fs_handle.get_mut() {
        Some(fs) => match fs.open_volume() {
            Ok(dir) => dir,
            Err(err) => return err.status()
        },
        None => return Status::ABORTED
    };
    
    let mut CStr16_buffer = [0u16; 20];
   
    // Loading the font info first to test it
    
    let font_path = match CStr16::from_str_with_buf("font.bdf", &mut CStr16_buffer) {
        Ok(string) => string,
        Err(_) => return Status::ABORTED
    };

    info!("Opening the font file handle");
    
    let font_handle = match root_dir.open(
        font_path,
        uefi::proto::media::file::FileMode::Read,
        // This only applies to creating a file but is needed since it is an argument
        uefi::proto::media::file::FileAttribute::READ_ONLY
    ) {
        Ok(handle) => handle,
        Err(err) => return err.status()
    };

    info!("Reading font file into memory");
    
    let font_bdf = match read_file_from_handle(font_handle) {
        Ok(font) => font,
        Err(status) => return status
    };

    info!("Loading the bdf file into a MonoFont struct");

    // It turns out I haven't finished writing this yet so I will just comment it out 
    /*
    let mono_font = load_bdf_to_mono_font(
        match alloc::str::from_utf8(font_bdf.as_slice()) {
            Ok(string) => string,
            Err(_) => return Status::ABORTED
        }
    );
    */

    info!("Loaded the font\nGrabbing the GOP framebuffer"); 
    boot_services.stall(1_000_000);

    let mut gop_handle = match boot_services.get_handle_for_protocol
        ::<GraphicsOutput>() {
        Ok(handle) => handle,
        Err(err) => return err.status()
    };

    info!("Loaded the gop handle. Now opening the protocol. The end of messages since open protocol exclusive, in fact exclusively opens the graphics protocol");

    let mut gop_protocol = match boot_services.open_protocol_exclusive
        ::<GraphicsOutput>(gop_handle){
        Ok(proto) => proto,
        Err(err) => return err.status()
    };

    let mut mode = match gop_protocol.query_mode(0, boot_services) {
        Ok(mode) => mode,
        Err(err) => return err.status()
    };

    // On my laptop the pixel format is either RGB or BGR, and I can't be bothered checking. Since
    // I initially only support black and white there's no reason to check anyway, it just can't be
    // one of the special formats
    // (eta: when I wrote text pixels to the screen (0xff32) they were blue (and 0xff000000 was
    // black, presumably the reserved bit) this implies that the pixel format is one byte of
    // reserved memory, and one byte each for red, green and blue, if my understanding of the
    // possible formats is correct
    match mode.info().pixel_format() {
        PixelFormat::Bitmask | PixelFormat::BltOnly => return Status::ABORTED,
        _ => {}
    }

    let mut frame_buffer = gop_protocol.frame_buffer(); 

    const TEST_PIXEL: u32 = 0xffu32;
    
    for offset_one in 0..200 {
        for offset_two in 0..200 {
            unsafe {
                (frame_buffer.as_mut_ptr() as *mut u32)
                    .add(offset_one * mode.info().stride() + offset_two)
                    .write_volatile(TEST_PIXEL)
            }
        }
    }

    // let mut cpu_frame_buffer = CPUFrameBuffer {
    //    width: mode.info().
    //};

    // For testing I want to exit before trying to load the kernel
    boot_services.stall(10_000_000);
    return Status::ABORTED;

    let kernel_path = match CStr16::from_str_with_buf("kernel.elf", &mut CStr16_buffer) {
        Ok(string) => string,
        Err(_) => return Status::ABORTED
    };

    let kernel_handle = match root_dir.open(
        kernel_path,
        uefi::proto::media::file::FileMode::Read,
        // This only applies to creating a file but is needed since it is an argument
        uefi::proto::media::file::FileAttribute::READ_ONLY
    ) {
        Ok(handle) => handle,
        Err(err) => return err.status()
    };

    let kernel = match read_file_from_handle(kernel_handle) {
        Ok(kernel) => kernel,
        Err(status) => return status
    };

    system_table.boot_services().stall(10_000_000);
    Status::SUCCESS
}
