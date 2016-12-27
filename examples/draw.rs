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
use sixel::{Environment, encoder};

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
			.short("C")
			.long("colors")
			.takes_value(true)
			.help("Amount of colors to use."))
		.arg(Arg::with_name("padding")
			.short("p")
			.long("padding")
			.takes_value(true)
			.help("Padding for the image."))
		.arg(Arg::with_name("center")
			.short("c")
			.long("center")
			.help("Center the image."))
		.arg(Arg::with_name("fit")
			.short("f")
			.long("fit")
			.help("Fit the image to the terminal size."))
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
		.arg(Arg::with_name("newline")
			.short("n")
			.long("newline")
			.help("Add a new line at the end."))
		.arg(Arg::with_name("wait")
			.short("w")
			.long("wait")
			.help("Wait for input."))
		.get_matches();

	let     environment = Environment::query().unwrap();
	let mut settings    = encoder::Settings::default();

	if let Some(colors) = matches.value_of("colors") {
		settings.colors(colors.parse().unwrap());
	}

	let size = match (matches.value_of("width"), matches.value_of("height")) {
		(Some(width), Some(height)) =>
			Some((width.parse().unwrap(), height.parse().unwrap())),

		(Some(width), None) =>
			Some((width.parse().unwrap(), width.parse().unwrap())),

		(None, Some(height)) =>
			Some((height.parse().unwrap(), height.parse().unwrap())),

		_ if matches.is_present("fit") =>
			Some(environment.limits()),

		_ if matches.is_present("center") =>
			Some((environment.limits().0, environment.limits().0)),

		_ =>
			None
	};

	if let Some((width, height)) = size {
		settings.size(environment.size(width, height).unwrap());
	}

	if matches.is_present("center") {
		settings.center();
	}

	if let Some(size) = matches.value_of("padding") {
		settings.padding(environment.padding(size.parse().unwrap()).unwrap());
	}

	let image = read::from_path(matches.value_of("INPUT").unwrap()).unwrap();
	encoder::encode(&settings, &image, io::stdout()).unwrap();

	if matches.is_present("newline") {
		println!();
	}

	if matches.is_present("wait") {
		let mut dummy = String::new();
		io::stdin().read_line(&mut dummy).unwrap();
	}
}
