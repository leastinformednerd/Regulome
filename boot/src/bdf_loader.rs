use graphics::MonoFont;

use alloc::format;
use alloc::boxed::Box;
use alloc::vec::Vec;

const WHITE: u32 = 0xffffff00;
const BLACK: u32 = 0;

/// Converts a bdf file to a MonoFont, failing if the format is invalid or it's not a monospaced
/// font
pub fn load_bdf_to_mono_font(bdf_file: &str) -> Result<MonoFont, &'static str> {
    let mut lines = bdf_file.lines();

    let mut glyphs: [Box<[u32]>; 126-32] = core::array::from_fn(|_| Box::from([]));

    let mut ind = 0usize;

    let mut font_width = -1i64;
    let mut font_height = -1i64;

    loop {
        // kind of cursed implementation to parse the file, alternating between the two loops to
        // handle whole file / individual glyph bitmap parsing 
        
        for line in &mut lines {
            if let Some("BBX") = line.get(0..3) {
                let mut atoms = line.split_ascii_whitespace();
                
                let _ = atoms.next();

                let width = atoms.next();
                let height = atoms.next();

                if let (Some(glyph_width), Some(glyph_height)) = (width, height) {
                    let width = match i64::from_str_radix(glyph_width,10) {
                        Ok(num) => num,
                        Err(_) => return Err("Couldn't parse dimension as int")
                    };
                    let height = match i64::from_str_radix(glyph_height, 10) {
                        Ok(num) => num,
                        Err(_) => return Err("Couldn't parse dimension as int")
                    };
        
                    if font_width == -1 {
                        font_width = width;
                        font_height = height;
                    } else if font_width != width || font_height != height {
                        return Err("not monospaced")
                    }
                }
            }

            if let Some("BITMAP") = line.get(0..6) {
                break
            }
        }
        
        for line in &mut lines {

        }
    }
}
