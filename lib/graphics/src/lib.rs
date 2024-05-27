#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::ops::Deref;

use uefi::proto::console::gop::FrameBuffer;

/// This is a struct to contain the kernel's frame buffer (in the CPU's ram as opposed to the GOP
/// memory mapped frame buffer we get from UEFI which is mapped to VRAM somewhere). This is good
/// practice since it allows much faster reads and (writes when there are multiple updates to the
/// buffer) since we minimise data being moved to and from the GPU
///
/// This shouldn't be directly accessed by user-space processes and instead be written to by the
/// kernel when needed. This interface is particularly barebones since the access should be
/// controlled by a higher level user-space available interface (that may or may not implement
/// windows as a concept [at some point copium]).
pub struct CPUFrameBuffer {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub buffer: Box<[u32]>
}

impl CPUFrameBuffer {
    /// # Safety
    /// This inherits the `uefi::proto::console::gop::FrameBuffer::write_value`'s safety
    /// considerations 
    pub unsafe fn write_to_graphics(&self, frame_buffer: &mut FrameBuffer) {
        unsafe {
            frame_buffer.write_value(0, self.buffer.deref())
        }
    }

    /// Writes a linear array of u32 (GOP pixels) to the buffer as though it was a rectangle with
    /// width and height as given, with the top right corner at (x,y)
    ///
    /// # Arguments
    /// * `pixel_map` - A linear array of pixels in GOP format
    /// * `width` - The width of the rectangle to be drawn
    /// * `height` - The height of the drawn rectangle
    /// * `x` - The x position of the top right pixel of the rectangle
    /// * `y` - The y position of the top right pixel of the rectangle
    pub fn write_rect_pixel_map(&mut self, pixel_map: &[u32], width: usize, height: usize, x: usize, y:usize) -> Result<(), &'static str>{
        if x + width > self.width {
            return Err("The given the x position + pixel map width would go off the screen")
        }

        if y + height > self.height {
            return Err("The given y position + pixel map height would go off the screen")
        }

        if pixel_map.len() != width * height {
            return Err("The pixel map dimensions do not agree with its linear length")
        }
        unsafe {
            let framebuf_base_addr = self.buffer.as_mut_ptr().add(y * self.stride + x) as *mut u32;
            for row_index in 0..height {
                for col_index in 0..width {
                    framebuf_base_addr
                        .add(row_index * self.stride + col_index)
                        .write_volatile(pixel_map[row_index * width + col_index])
                }
            }
        }
        Ok(())
    }

    // This is not particularly performant I think but I am not knowledgable enough to optimize it
    pub fn flush(&self, framebuffer: &mut FrameBuffer) {
        for x in 0..self.stride {
            for y in 0..self.height {
                unsafe {
                    (framebuffer.as_mut_ptr() as *mut u32)
                        .add(y * self.stride + x)
                        .write_volatile(self.buffer[y * self.stride + x])
                }
            }
        }
    }
}

/// An array of pixelmaps that all have the same width and height
/// Has all the ascii printable characters (32 - 126)
pub struct MonoFont {
    pub characters: [
        Box<[u32]>; 126-32
    ],
    pub width: usize,
    pub height: usize
}

pub struct TextBuffer {
    /// Height and width is in characters
    width: usize,
    height: usize,
    
    /// Cursor position is (x,y) where x and y are zero indexed character positions
    cursor: (usize, usize),

    text: Box<str>
}

impl TextBuffer {
    /// Construct a new TextBuffer. Height and width are in characters.
    /// The Box needs to be passed in since this library doesn't know what allocator to use (or if
    /// one even exists) and therefore needs to be given ownership of a part of the heap to have a
    /// size that is unknown at compile time.
    pub fn new(height: usize, width: usize, mut text: Box<str>) -> Result<Self, &'static str>{
        if !text.is_ascii() {
            return Err("The provided string must be ascii")
        }

        if text.len() != height * width {
            return Err("The provided string must have length equal to height * width")
        }

        // Now we blank out the text by making it all spaces. This is just an aesthetic choice so
        // any correctly sized inputted string can be used to make an easy to use text buffer
        
        unsafe {
            for byte in text.as_bytes_mut() {
                // 32u8 is ascii space
                *byte = 32u8;
            }
        }

        Ok (
            Self {
                height: height,
                width: width,
                cursor: (0,0),
                text: text
            }
        )
    }

    #[inline]
    /// Moves the text upward one line. This happens when the cursor tries to move beyond the
    /// bottom of the buffer.
    fn shift_up(&mut self) {
        unsafe {
            core::intrinsics::copy(
                // The source, the start of the second line
                self.text.as_ptr().add(self.width),

                // The destination, the start of the buffer
                self.text.as_mut_ptr(),

                // The number of bytes to copy: the width times the number of lines other than the
                // first
                self.width * (self.height - 1)
            );

            // Blank out the final line with spaces
            core::intrinsics::write_bytes(
                // A ptr to the start of the first line
                self.text.as_mut_ptr().add(self.width * (self.height-1)),
                
                // ASCII space
                32u8,
                
                // The number of bytes to write (the width of a line)
                self.width
            )
        }
    }

    #[inline]
    /// Mutates self's cursor position and returns the previous location (that may not be the x,y
    /// value that was stored if the buffer had to be moved up).
    fn next(&mut self) -> (usize, usize){
        let (x,y) = self.cursor;
        
        if x == self.width {
            if y == self.height{
                self.cursor = (0, y);
                self.shift_up();
                return (0, y-1);
            } else {
                self.cursor = (0, y+1);
            }
        } else {
            self.cursor = (x+1, y);
        }

        return (x,y);
    }

    /// Write a character at the current cursor position and then advance the cursor
    pub fn write_char(&mut self, character: char) -> Result<(), &'static str>{
        if !character.is_ascii() || u32::from(character) > 126 || u32::from(character) < 32 {
            return Err("The provided character must be a printable ASCII")
        }
        
        if character == '\n' {
            self.cursor = (0, self.cursor.1 - 1);
            self.shift_up();

            return Ok(())
        }
        
        unsafe {
            let cursor = self.next();
            self.text.as_bytes_mut()[cursor.1 * self.width + cursor.0] =
                u8::try_from(character).or(Err("This error shouldn't be possible"))?
        }

        Ok(())
    }

    /// Write a str into the text buffer
    pub fn write_str(&mut self, string: &str) -> Result<(), &'static str> {
        for character in string.chars() {
            self.write_char(character)?
        }

        Ok(())
    }

    pub fn write_pixels(&self, frame_buffer: &mut CPUFrameBuffer, font: &MonoFont, padding: (usize, usize)) -> Result<(), &'static str> {
        // These two bounds check mean that it is guaranteed to be safe to do all the memory
        // copying I want to do
        if self.width * font.width + padding.0 >= frame_buffer.width {
            return Err("The framebuffer is too small for a line of text")
        }

        if self.height * font.height + padding.1 >= frame_buffer.height { 
            return Err("The framebuffer is too short for the text ")
        }

        for (index, character) in self.text.chars().enumerate() {
            let glyph = font.characters.get(
                // 32 is the start of the ascii printable characters block and can therefore be
                // used to calculate the offset into the glyph array.
                u32::from(character) as usize - 32
            )
            // This error shouldn't be possible if this struct's API is used to interface with it
            // This prevents panics if unsafe code is used to directly modify the struct for
            // whatever reason
            .ok_or("A character was not a printable ascii character")?;
            
            frame_buffer.write_rect_pixel_map(
                glyph, font.width, font.height,
                (index % self.width) * font.width + padding.0,
                index / self.width * font.height + padding.1
            )?
        }

        Ok(())
    }
}
