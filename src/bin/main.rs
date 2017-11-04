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

#[macro_use] extern crate clap;
extern crate base100;

use std::io::{self, Read, Write, BufRead, BufReader, BufWriter};
use std::fs::File;
use clap::App;
use base100::{char_to_emoji, emoji_to_char};

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

