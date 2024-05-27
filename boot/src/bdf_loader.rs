use graphics::MonoFont;

use alloc::format;
use alloc::boxed::Box;
use alloc::vec::Vec;

const WHITE: u32 = 0x00ffffff;
const BLACK: u32 = 0;

const MAX_CHARS: usize = 126-32;

/// Converts a bdf file to a MonoFont, failing if the format is invalid or it's not a monospaced
/// font. Only looks at the bare minimum information needed to parse it into a MonoFont
pub fn load_bdf_to_mono_font(bdf_file: &str) -> Result<MonoFont, &'static str> {
    let mut lines = bdf_file.lines();

    let mut glyphs: [Box<[u32]>; MAX_CHARS] = core::array::from_fn(|_| Box::from([]));

    let mut ind = 0usize;

    let mut font_width = -1i64;
    let mut font_height = -1i64;

    for _ in 0..MAX_CHARS {
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
        
        let mut current = Vec::new();

        for line in &mut lines {
            if let Some("ENDCHAR") = line.get(0..7) {
                if current.len() != (font_width * font_height) as usize {
                    return Err("A bitmap was the wrong size");
                }
                glyphs[ind] = current.into_boxed_slice();
                ind += 1;
                break
            }
            
            // else: line must be a line of bitmap bits
            // On the font that I'm using it's ok to use a u16 since the font has a width of 16
            // bits. However this is not generally true. If the font size changes then this section
            // will need to be rewritten. If the font simply becomes a different power of 2
            // supported as a rust int type (8,32,64,128) then subbing that in should work,
            // otherwise the leading zeroes will need to be stripped
            let bits = match u16::from_str_radix(line, 16) {
                Ok(bits) => bits,
                Err(_) => return Err("Failed to parse the bits of this line")
            };

            for offset in 0..16 {

                // I'm a bit unclear whether it's more idiomatic to put the if inside the function
                // argument. It's probably fine since this is such a small example, but I imagine
                // it's better that way for larger blocks.
                if bits & 1 << (15 - offset) != 0 {
                    current.push(WHITE);
                } else {
                    current.push(BLACK);
                }
            }
        }

        if ind == MAX_CHARS - 1 {
            let font = MonoFont {
                characters: glyphs,
                
                // This is technically wrong to do but even if this was on a 32 bit system there is
                // no way that these are exceeding 2**31 anyway. There would be something else very
                // wrong if that actually lost any information
                width: font_width as usize,
                height: font_height as usize
            };

            return Ok(font)
        }
    }

    // At this point we should have returned an error or the font. Therefore we can assume
    // something has gone wrong and it's safe to return an error
    return Err("For some reason, the font failed to be parsed")
}
