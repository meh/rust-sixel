//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.

extern crate clap;
use clap::{App, Arg};

extern crate sixel;
use sixel::{Settings, Encoder};

extern crate picto;
use picto::read;

use std::io;

fn main() {
	let matches = App::new("draw")
		.version(env!("CARGO_PKG_VERSION"))
		.about("Draw an image in your terminal.")
		.arg(Arg::with_name("INPUT")
			.index(1)
			.required(true)
			.help("The path to the image to draw."))
		.arg(Arg::with_name("colors")
			.short("c")
			.long("colors")
			.takes_value(true)
			.help("Amount of colors to use."))
		.arg(Arg::with_name("width")
				.short("W")
				.long("width")
				.takes_value(true)
				.help("Width of the image."))
		.arg(Arg::with_name("height")
				.short("H")
				.long("height")
				.takes_value(true)
				.help("Height of the image."))
		.get_matches();

	let mut settings = Settings::new();

	if let Some(colors) = matches.value_of("colors") {
		settings.colors(colors.parse().unwrap());
	}

	match (matches.value_of("width"), matches.value_of("height")) {
		(Some(width), Some(height)) => {
			settings.size(width.parse().unwrap(), height.parse().unwrap());
		}

		(Some(width), None) => {
			settings.size(width.parse().unwrap(), width.parse().unwrap());
		}

		(None, Some(height)) => {
			settings.size(height.parse().unwrap(), height.parse().unwrap());
		}

		(None, None) => ()
	}

	let encoder = Encoder::new(settings.build());
	let image   = read::from_path(matches.value_of("INPUT").unwrap()).unwrap();

	encoder.encode(&image, io::stdout()).unwrap();
}
