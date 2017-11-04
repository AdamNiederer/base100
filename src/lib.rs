// base💯 - Copyright 2017 Adam Niederer

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(test, feature(test))]
#![cfg_attr(feature = "simd", feature(asm))]
#![cfg_attr(feature = "simd", feature(target_feature))]
#![cfg_attr(feature = "simd", feature(cfg_target_feature))]

#[cfg(feature = "simd")] extern crate stdsimd;

#[cfg(any(not(feature = "simd"), not(target_arch = "x86_64")))]
pub fn emoji_to_char<'a, 'b>(buf: &'a [u8], out: &'b mut [u8]) -> &'b [u8] {
    for (i, chunk) in buf.chunks(4).enumerate() {
        out[i] = ((chunk[2].wrapping_sub(143)).wrapping_mul(64)).wrapping_add(chunk[3].wrapping_sub(128)).wrapping_sub(55)
    }
    out
}

#[cfg(feature = "simd")]
#[cfg(target_feature = "avx2")]
pub fn emoji_to_char<'a, 'b>(buf: &'a [u8], out: &'b mut [u8]) -> &'b [u8] {
    use stdsimd::simd::u8x32;
    let mut i = 0;
    for chunk in buf.chunks(128) {
        if chunk.len() < 128 {
            // Non-SIMD implementation for the final <128 bytes
            for chunk in chunk.chunks(4) {
                out[i] = ((chunk[2].wrapping_sub(143)).wrapping_mul(64)).wrapping_add(chunk[3].wrapping_sub(128)).wrapping_sub(55);
                i += 1;
            }
        } else {
            // AVX implementation of decoding algo
            // a, b, c, d contain one garbage word and one useful word
            let a = u8x32::load(chunk, 0);
            let b = u8x32::load(chunk, 32);
            let c = u8x32::load(chunk, 64);
            let d = u8x32::load(chunk, 96);

            // Constant mask for removing low bytes
            let hi_mask = u8x32::new(0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00,
                                     0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00,
                                     0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00,
                                     0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00);

            // Results from the vector magic below
            let hi: u8x32;
            let lo: u8x32;

            unsafe {
                asm! {
                    "vpsrld ymm1, ymm1, 16
                     vpsrld ymm2, ymm2, 16
                     vpsrld ymm3, ymm3, 16
                     vpsrld ymm4, ymm4, 16
                     // ymm1 .. ymm4 now contain 0x00, 0x00, 0xhi, 0xlo ...

                     // Pack ymm* into ymm1 and ymm2 and order them properly
                     vpackusdw ymm1, ymm1, ymm2
                     vpackusdw ymm2, ymm3, ymm4
                     // vpackusdw interleaves ymm[13] and ymm[24]; we want them end-to-end
                     vpermpd ymm1, ymm1, 0xD8 // [191:128] <-> [127:64]
                     vpermpd ymm2, ymm2, 0xD8 // [191:128] <-> [127:64]
                     // ymm1 and ymm2 now contain interleaved high and low bytes

                     // Store high bytes (0xhi 0x00) in ymm3 and ymm4
                     vpand ymm3, ymm1, ymm6
                     vpand ymm4, ymm2, ymm6
                     // Pack hi bytes into ymm4
                     vpackuswb ymm4, ymm3, ymm4

                     // Remove the high bytes and move low bytes into position
                     vpsrlw ymm1, ymm1, 8
                     vpsrlw ymm2, ymm2, 8
                     // Store low bytes in ymm3
                     vpackuswb ymm3, ymm1, ymm2

                     // Fight interleaving again
                     vpermpd ymm4, ymm4, 0xD8 // [191:128] <-> [127:64]
                     vpermpd ymm3, ymm3, 0xD8 // [191:128] <-> [127:64]"
                     : "={ymm3}"(lo), "={ymm4}"(hi)
                     : "{ymm1}"(a), "{ymm2}"(b), "{ymm3}"(c),
                       "{ymm4}"(d), "{ymm6}"(hi_mask)
                     : "cc"
                     : "intel"
                }
            }

            ((((hi - u8x32::splat(143)) * u8x32::splat(64))
              + lo - u8x32::splat(128)) - u8x32::splat(55))
                .store(out, i);
            i += 32;
        }
    }
    out
}

#[cfg(feature = "simd")]
#[cfg(all(not(target_feature = "avx2"), target_feature = "sse2"))]
pub fn emoji_to_char<'a, 'b>(buf: &'a [u8], out: &'b mut [u8]) -> &'b [u8] {
    use stdsimd::simd::u8x16;
    let mut i = 0;
    for chunk in buf.chunks(64) {
        // Chunk data format: 0x00 0x00 0xhi 0xlo
        if chunk.len() < 64 {
            // Non-SIMD implementation for the final <128 bytes
            for (i, chunk) in chunk.chunks(4).enumerate() {
                out[i] = ((chunk[2].wrapping_sub(143)).wrapping_mul(64)).wrapping_add(chunk[3].wrapping_sub(128)).wrapping_sub(55)
            }
        } else {

            // SSE2 implementation of decoding algo

            // TODO: Use Get rid of these scalar loads
            let msbs = u8x16::new(chunk[2], chunk[6], chunk[10], chunk[14], chunk[18], chunk[22], chunk[26], chunk[30], chunk[34], chunk[38], chunk[42], chunk[46], chunk[50], chunk[54], chunk[58], chunk[62]);
            let lsbs = u8x16::new(chunk[3], chunk[7], chunk[11], chunk[15], chunk[19], chunk[23], chunk[27], chunk[31], chunk[35], chunk[39], chunk[43], chunk[47], chunk[51], chunk[55], chunk[59], chunk[63]);
            ((((msbs - u8x16::splat(143)) * u8x16::splat(64))
              + lsbs - u8x16::splat(128)) - u8x16::splat(55))
                .store(out, i);
            i += 16;
        }
    }
    out
}

#[cfg(any(not(feature = "simd"), not(target_arch = "x86_64")))]
pub fn char_to_emoji<'a, 'b>(buf: &'a [u8], out: &'b mut [u8]) -> &'b [u8] {
    for (i, ch) in buf.iter().enumerate() {
        out[4 * i + 0] = 0xf0;
        out[4 * i + 1] = 0x9f;
        // (ch + 55) >> 6 approximates (ch + 55) / 64
        out[4 * i + 2] = ((((*ch as u16).wrapping_add(55)) >> 6) + 143) as u8;
        // (ch + 55) & 0x3f approximates (ch + 55) % 64
        out[4 * i + 3] = (ch.wrapping_add(55) & 0x3f).wrapping_add(128);
    }
    out
}

#[cfg(feature = "simd")]
#[cfg(target_feature = "avx2")]
pub fn char_to_emoji<'a, 'b>(buf: &'a [u8], out: &'b mut [u8]) -> &'b [u8] {
    use stdsimd::simd::{u8x32, u16x16, i16x16};
    use stdsimd::vendor::{_mm256_unpackhi_epi8, _mm256_unpacklo_epi8, _mm256_unpackhi_epi16, _mm256_unpacklo_epi16};

    let mut i = 0;
    for chunk in buf.chunks(32) {
        if chunk.len() < 32 {
            // Non-SIMD implementation for the final <128 bytes
            for (i, ch) in buf.iter().enumerate() {
                out[4 * i + 0] = 0xf0;
                out[4 * i + 1] = 0x9f;
                // (ch + 55) >> 6 approximates (ch + 55) / 64
                out[4 * i + 2] = ((((*ch as u16).wrapping_add(55)) >> 6) + 143) as u8;
                // (ch + 55) & 0x3f approximates (ch + 55) % 64
                out[4 * i + 3] = (ch.wrapping_add(55) & 0x3f).wrapping_add(128);
            }
        } else {

            let bytes = u8x32::load(chunk, 0);
            let asm_out: u8x32;
            // Constant mask for removing high bytes
            let lo_mask = u8x32::new(0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
                                     0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
                                     0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
                                     0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF);

            // Do the cast-to-u16 addition, shift right, then cast back down to u8.
            unsafe {
                asm! {
                    "// Cast each byte to a word, moving to ymm1 and ymm2
                     vmovddup ymm2, ymm1
                     vandpd ymm2, ymm2, ymm3
                     vpsrlw ymm1, ymm1, 8
                     // ymm1 now contains the high bytes as u16s
                     // ymm2 now contains the low bytes as u16s
                     // Do the 16-bit addition
                     vpaddw ymm1, ymm1, ymm4
                     vpaddw ymm2, ymm1, ymm4
                     // Shift right and convert back to u8s
                     vpsrlw ymm1, ymm1, 6
                     vpsrlw ymm2, ymm2, 6
                     vpackuswb ymm1, ymm1, ymm2
                     // Prevent interleaving
                     vpermpd ymm4, ymm4, 0xD8 // [191:128] <-> [127:64]"
                     : "={ymm1}"(asm_out)
                     : "{ymm1}"(bytes), "{ymm3}"(lo_mask), "{ymm4}"(u16x16::splat(31)) // TODO: Why is this not 55?
                     : "cc ymm2"
                     : "intel"
                }
            }

            let third = asm_out + u8x32::splat(143);
            // let third = unsafe {
            //     TODO: Replace above ASM with this
            //     _mm256_packus_epi16(((i16x16::from(bytes) >> 8) + i16x16::splat(55)) >> 6,
            //                         (i16x16::from(bytes & lo_mask) + i16x16::splat(55)) >> 6)
            //         + u8x32::splat(143)
            // };

            // Here be dragons
            let a = u8x32::splat(0xf0);
            let b = u8x32::splat(0x9f);

            // Our "most significant bytes"
            let mut c = third;

            // Our "least significant byts"
            let mut d = ((bytes + u8x32::splat(55)) & u8x32::splat(0x3f)) + u8x32::splat(128);

            // Permute the bytes ahead of time so packing doesn't screw up the middle lanes of each
            unsafe { asm!{ "vpermpd ymm1, ymm1, 0xD8" : "={ymm1}"(c) : "{ymm1}"(c) : "cc" : "intel" } }
            unsafe { asm!{ "vpermpd ymm1, ymm1, 0xD8" : "={ymm1}"(d) : "{ymm1}"(d) : "cc" : "intel" } }

            let abh = unsafe { _mm256_unpackhi_epi8(a.as_i8x32(), b.as_i8x32()) };
            let abl = unsafe { _mm256_unpacklo_epi8(a.as_i8x32(), b.as_i8x32()) };

            // cdh & cdl now contain packed representations of msb lsb msb lsb...
            let mut cdh = unsafe { _mm256_unpackhi_epi8(c.as_i8x32(), d.as_i8x32()) };
            let mut cdl = unsafe { _mm256_unpacklo_epi8(c.as_i8x32(), d.as_i8x32()) };

            // Permute 'em again to cancel out the 16-bit packing's middle-lane swapping
            // We don't permute abh & abl because they're uniform throughout
            unsafe { asm!{ "vpermpd ymm1, ymm1, 0xD8" : "={ymm1}"(cdh) : "{ymm1}"(cdh) : "cc" : "intel" } }
            unsafe { asm!{ "vpermpd ymm1, ymm1, 0xD8" : "={ymm1}"(cdl) : "{ymm1}"(cdl) : "cc" : "intel" } }

            // Pack everything into its final form data. Now 0xf0 0x9f msb lsb...
            let abcdll = unsafe { _mm256_unpacklo_epi16(i16x16::from(abl), i16x16::from(cdl)) };
            let abcdlh = unsafe { _mm256_unpackhi_epi16(i16x16::from(abl), i16x16::from(cdl)) };
            let abcdhl = unsafe { _mm256_unpacklo_epi16(i16x16::from(abh), i16x16::from(cdh)) };
            let abcdhh = unsafe { _mm256_unpackhi_epi16(i16x16::from(abh), i16x16::from(cdh)) };

            u8x32::from(abcdll).store(out, i);
            u8x32::from(abcdlh).store(out, i + 32);
            u8x32::from(abcdhl).store(out, i + 64);
            u8x32::from(abcdhh).store(out, i + 96);
            i += 128;
        }
    }
    out
}

#[cfg(feature = "simd")]
#[cfg(all(not(target_feature = "avx2"), target_feature = "sse2"))]
pub fn char_to_emoji<'a, 'b>(buf: &'a [u8], out: &'b mut [u8]) -> &'b [u8] {
    // TODO: SSE this up
    for (i, ch) in buf.iter().enumerate() {
        out[4 * i + 0] = 0xf0;
        out[4 * i + 1] = 0x9f;
        // (ch + 55) >> 6 approximates (ch + 55) / 64
        out[4 * i + 2] = ((((*ch as u16).wrapping_add(55)) >> 6) + 143) as u8;
        // (ch + 55) & 0x3f approximates (ch + 55) % 64
        out[4 * i + 3] = (ch.wrapping_add(55) & 0x3f).wrapping_add(128);
    }
    out
}
