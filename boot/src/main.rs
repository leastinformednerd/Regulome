#![no_main]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use alloc::string::String;

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

use graphics::{CPUFrameBuffer, TextBuffer};

const RED: u32 = 0xff00000;
const GREEN: u32 = 0xff00;
const BLUE: u32 = 0xff;
const BLACK: u32 = 0;
const WHITE: u32 = 0xffffff;

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

fn fill_frame_buffer(frame_buffer: &mut FrameBuffer, pixel: u32, height: usize, stride: usize){
    for y in 0..height {
        let offset = y * stride;
        for x in 0..stride {
            unsafe {
                (frame_buffer.as_mut_ptr() as *mut u32)
                .add(offset + x)
                .write_volatile(pixel)
            }
        }
    }
}

/*unsafe*/ fn print(message: &str, text_buffer: &mut TextBuffer, cpu_frame_buffer: &mut CPUFrameBuffer, frame_buffer_ptr: *mut u8, mono_font: &graphics::MonoFont)
    -> Result<(), &'static str> {
    text_buffer.write_str(message)?;
    text_buffer.write_pixels(cpu_frame_buffer, mono_font, (20,20))?;
    cpu_frame_buffer.flush(frame_buffer_ptr as *mut u32);
    Ok(())
}

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    // Read the relavent information files into ram
   
    info!("Getting file system handle");

    let fs_handle = match system_table.boot_services().get_image_file_system(image_handle){
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

    let mono_font = match load_bdf_to_mono_font(
        match alloc::str::from_utf8(font_bdf.as_slice()) {
            Ok(string) => string,
            Err(_) => return Status::ABORTED
        }
    ) {
        Ok(font) => font,
        // I'm not entirely sure whether I need to do this since err_msg is &'static str so maybe I
        // can just put it in info!(err_msg), but even if that's fine it should be the same outputted
        // binary so it doesn't really matter
        Err(err_msg) => {
            info!("{err_msg}");
            system_table.boot_services().stall(3_000_000);
            return Status::ABORTED;
        }
    };

    info!("Loaded the font\nGrabbing the GOP frame_buffer"); 

    let mut gop_handle = match system_table.boot_services().get_handle_for_protocol
        ::<GraphicsOutput>() {
        Ok(handle) => handle,
        Err(err) => return err.status()
    };

    info!("Loaded the gop handle. Now opening the protocol. The end of messages since open protocol exclusive, in fact, exclusively opens the graphics protocol");

    let mut gop_protocol = match system_table.boot_services().open_protocol_exclusive
        ::<GraphicsOutput>(gop_handle){
        Ok(proto) => proto,
        Err(err) => return err.status()
    };

    let mut mode = match gop_protocol.query_mode(0, system_table.boot_services()) {
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
    
    let (width, height) = mode.info().resolution();
    let stride = mode.info().stride();
    
    let mut cpu_frame_buffer = CPUFrameBuffer {
        width: width, 
        height: height, 
        stride: stride,
        buffer: {
            let mut buf = Vec::with_capacity(stride*height);
            buf.resize(stride*height, 0u32);
            buf.into_boxed_slice()
        }
    };
    
    let mut text_buffer = match TextBuffer::new(
        // Dimensions of text buffer in glyphs, with -2 to account for the padding
        height / mono_font.height - 2,
        width / mono_font.width - 2,
        {
            let mut tmp = String::with_capacity(
                (height / mono_font.height - 2) * (width / mono_font.width - 2)
            );

            for _ in 0..(height / mono_font.height - 2) * (width / mono_font.width - 2) {
                tmp.push(' ');
            }

            tmp.into_boxed_str()
        }
    ) {
        Ok(buffer) => buffer,
        Err(msg) => {
            for x in 0..stride {
                for y in 0..height {
                    unsafe {
                        (frame_buffer.as_mut_ptr() as *mut u32)
                        .add(y * stride + x)
                        .write_volatile(match msg {
                            "The provided string must be ascii" => RED,
                            "The provided string must have length equal to height * width" => BLUE,
                            _ => GREEN
                        })
                    }
                }
            }
            system_table.boot_services().stall(4_000_000);
            return Status::ABORTED;
        }
    };
    
    let kernel_path = match CStr16::from_str_with_buf("kernel.elf", &mut CStr16_buffer) {
        Ok(string) => string,
        Err(_) => {
            print("Couldn't convert rust str to CStr16\n",
                &mut text_buffer,
                &mut cpu_frame_buffer,
                frame_buffer.as_mut_ptr(),
                &mono_font);
            system_table.boot_services().stall(3_000_000);
            return Status::ABORTED
        }
    };

    let kernel_handle = match root_dir.open(
        kernel_path,
        uefi::proto::media::file::FileMode::Read,
        // This only applies to creating a file but is needed since it is an argument
        uefi::proto::media::file::FileAttribute::READ_ONLY
    ) {
        Ok(handle) => handle,
        Err(err) => {
            print("Failed to open a file at /kernel.elf\n",
                &mut text_buffer,
                &mut cpu_frame_buffer,
                frame_buffer.as_mut_ptr(),
                &mono_font);
            system_table.boot_services().stall(3_000_000);
            return err.status() 
        }
    };

    let kernel = match read_file_from_handle(kernel_handle) {
        Ok(kernel) => kernel,
        Err(status) => {
            print("Failed to read the data at the file handle into memory\n",
                &mut text_buffer,
                &mut cpu_frame_buffer,
                frame_buffer.as_mut_ptr(),
                &mono_font);
            system_table.boot_services().stall(3_000_000);
            return status 
        }
    };

    print("Loaded kernel file into memory, now it needs to be loaded as an elf file.",
        &mut text_buffer,
        &mut cpu_frame_buffer,
        frame_buffer.as_mut_ptr(),
        &mono_font);
    text_buffer.dbg_print_cursor();
    system_table.boot_services().stall(3_000_000);
    Status::SUCCESS
}
