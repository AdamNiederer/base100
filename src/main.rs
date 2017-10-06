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

#[macro_use] extern crate clap;

use std::io::{self, Read, Write, BufRead, BufReader, BufWriter};
use std::fs::{File};
use std::char;
use std::iter::Iterator;
use clap::App;

const base: u32 = 127_991;

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
    let mut write_buf = [0u8; 0x1_0000];
    let mut buffer = [0u8; 0x1_0000];

    if cli_args.is_present("decode") {
        while let Ok(num_read) = reader.read(&mut buffer) {
            if num_read == 0 {
                break;
            }
            let decoded_str = if cli_args.is_present("fast") {
                unsafe {
                    std::str::from_utf8_unchecked(&buffer)
                }
            } else {
                match std::str::from_utf8(&buffer) {
                    Ok(string) => string,
                    _ => {
                        writeln!(io::stderr(), "base💯: write error").expect("base💯: stderr write error");
                        return;
                    }
                }
            };
            match writer.write(decoded_str.chars()
                               .map(|ch| { (ch as u32 - base) as u8 })
                               .collect::<Vec<u8>>().as_slice()) {
                Ok(_) => (),
                _ => {
                    writeln!(io::stderr(), "base💯: write error").expect("base💯: stderr write error");
                    return;
                }
            }
        }
    } else {
        while let Ok(num_read) = reader.read(&mut buffer) {
            if num_read == 0 {
                break;
            }

            for byte in buffer.iter().take(num_read) {
                let ch: char = char::from_u32(base + u32::from(*byte)).unwrap();
                match writer.write(ch.encode_utf8(&mut write_buf).as_bytes()) {
                    Ok(_) => (),
                    _ => {
                        writeln!(io::stderr(), "base💯: write error").expect("base💯: stderr write error");
                        return;
                    }
                }
            }
        }
    }
    writer.flush().expect("Write error");
}
