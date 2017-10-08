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

#![allow(non_upper_case_globals)]
#![cfg_attr(test, feature(test))]
#![cfg_attr(feature = "simd", feature(target_feature))]
#![cfg_attr(feature = "simd", feature(cfg_target_feature))]

#[macro_use] extern crate clap;
#[cfg(feature = "simd")] extern crate stdsimd;

use std::io::{self, Read, Write, BufRead, BufReader, BufWriter};
use std::fs::File;
use std::iter::Iterator;
use clap::App;

fn main() {
    let cli_spec = load_yaml!("cli.yml");
    let cli_args = App::from_yaml(cli_spec).get_matches();

    let mut reader = {
        if let Some(path) = cli_args.value_of("input") {
            Box::new(BufReader::new(match File::open(path) {
                Ok(path) => path,
                _ => {
                    writeln!(io::stderr(), "base💯: no such file: {}", path).expect("base💯: stderr write error");
                    return;
                }
            })) as Box<BufRead>
        } else {
            Box::new(BufReader::new(io::stdin())) as Box<BufRead>
        }
    };

    let mut writer = BufWriter::new(io::stdout());

    if cli_args.is_present("decode") {
        let mut write_buf = [0u8; 0x10000];
        let mut buffer = [0u8; 0x10000];

        while let Ok(num_read) = reader.read(&mut buffer) {
            if num_read == 0 {
                break;
            }

            match writer.write_all(&emoji_to_char(&buffer[..num_read], &mut write_buf)[..num_read / 4]) {
                Ok(_) => (),
                _ => {
                    writeln!(io::stderr(), "base💯: write error").expect("base💯: stderr write error");
                    return;
                }
            }
        }
    } else {
        let mut write_buf = [0u8; 0x40000];
        let mut buffer = [0u8; 0x10000];

        while let Ok(num_read) = reader.read(&mut buffer) {
            if num_read == 0 {
                break;
            }

            match writer.write_all(&char_to_emoji(&buffer[..num_read], &mut write_buf)[..num_read * 4]) {
                Ok(_) => (),
                _ => {
                    writeln!(io::stderr(), "base💯: write error").expect("base💯: stderr write error");
                    return;
                }
            }
        }
    }
    writer.flush().expect("Write error");
}

#[cfg(any(not(feature = "simd"), not(target_arch = "x86_64")))]
fn emoji_to_char<'a, 'b>(buf: &'a[u8], out: &'b mut [u8]) -> &'b[u8] {
    for (i, chunk) in buf.chunks(4).enumerate() {
        out[i] = ((chunk[2].wrapping_sub(143)).wrapping_mul(64)).wrapping_add(chunk[3].wrapping_sub(128)).wrapping_sub(55)
    }
    out
}

#[cfg(feature = "simd")]
#[cfg(target_feature = "avx2")]
pub fn emoji_to_char<'a, 'b>(buf: &'a[u8], out: &'b mut [u8]) -> &'b[u8] {
    use stdsimd::simd::u8x32;
    use stdsimd::vendor::{_mm256_packus_epi32, _mm256_packus_epi16, _mm256_and_si256, _mm256_shuffle_pd};
    let mut i = 0;
    for chunk in buf.chunks(128) {
        if chunk.len() < 128 {
            // Non-SIMD implementation for the final <128 bytes
            for (i, chunk) in chunk.chunks(4).enumerate() {
                out[i] = ((chunk[2].wrapping_sub(143)).wrapping_mul(64)).wrapping_add(chunk[3].wrapping_sub(128)).wrapping_sub(55)
            }
        } else {
            // AVX implementation of decoding algo

            // TODO: Use Get rid of these scalar loads
            let msbs = u8x32::new(chunk[2], chunk[6], chunk[10], chunk[14], chunk[18], chunk[22], chunk[26], chunk[30], chunk[34], chunk[38], chunk[42], chunk[46], chunk[50], chunk[54], chunk[58], chunk[62], chunk[66], chunk[70], chunk[74], chunk[78], chunk[82], chunk[86], chunk[90], chunk[94], chunk[98], chunk[102], chunk[106], chunk[110], chunk[114], chunk[118], chunk[122], chunk[126]);
            let lsbs = u8x32::new(chunk[3], chunk[7], chunk[11], chunk[15], chunk[19], chunk[23], chunk[27], chunk[31], chunk[35], chunk[39], chunk[43], chunk[47], chunk[51], chunk[55], chunk[59], chunk[63], chunk[67], chunk[71], chunk[75], chunk[79], chunk[83], chunk[87], chunk[91], chunk[95], chunk[99], chunk[103], chunk[107], chunk[111], chunk[115], chunk[119], chunk[123], chunk[127]);
            ((((msbs - u8x32::splat(143)) * u8x32::splat(64))
              + lsbs - u8x32::splat(128)) - u8x32::splat(55))
                .store(out, i);
            i += 64;
        }
    }
    out
}

#[cfg(feature = "simd")]
#[cfg(target_feature = "sse2")]
pub fn emoji_to_char<'a, 'b>(buf: &'a[u8], out: &'b mut [u8]) -> &'b[u8] {
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

// #[cfg(any(not(feature = "simd"), not(target_arch = "x86_64")))]
pub fn char_to_emoji<'a, 'b>(buf: &'a[u8], out: &'b mut [u8]) -> &'b [u8] {
    for (i, ch) in buf.iter().enumerate() {
        out[4 * i + 0] = 0xf0;
        out[4 * i + 1] = 0x9f;
        out[4 * i + 2] = (((*ch as u16).wrapping_add(55)) / 64 + 143) as u8;
        out[4 * i + 3] = (ch.wrapping_add(55) % 64).wrapping_add(128);
    }
    out
}
