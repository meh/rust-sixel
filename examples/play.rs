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

#[macro_use(expand)]
extern crate terminfo;
use terminfo::{Database, capability as cap, Expand};

extern crate sixel;
use sixel::{Environment, encoder};

extern crate ffmpeg;

extern crate picto;
use picto::color::Rgba;

use std::io::{self, Write};
use std::thread;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

fn main() {
	ffmpeg::init().unwrap();

	let matches = App::new("draw")
		.version(env!("CARGO_PKG_VERSION"))
		.about("Draw an image in your terminal.")
		.arg(Arg::with_name("INPUT")
			.index(1)
			.required(true)
			.help("The path to the video to play."))
		.arg(Arg::with_name("colors")
			.short("C")
			.long("colors")
			.takes_value(true)
			.help("Amount of colors to use."))
		.arg(Arg::with_name("high-color")
			.short("I")
			.long("high-color")
			.help("Use a larger color space."))
		.arg(Arg::with_name("padding")
			.short("p")
			.long("padding")
			.takes_value(true)
			.help("Padding for the image."))
		.get_matches();

	let     info        = Database::from_env().unwrap();
	let     environment = Environment::query().unwrap();
	let mut settings    = encoder::Settings::default();

	if let Some(colors) = matches.value_of("colors") {
		settings.colors(colors.parse().unwrap());
	}
	else if let Some(colors) = environment.colors() {
		settings.colors(colors);
	}

	if matches.is_present("high-color") {
		settings.high();
	}

	settings.size(environment.size(environment.limits().0, environment.limits().1).unwrap());
	settings.center();
	settings.fast();

	if let Some(size) = matches.value_of("padding") {
		settings.padding(environment.padding(size.parse().unwrap()).unwrap());
	}

	// Open the video.
	let context         = ffmpeg::format::input(&matches.value_of("INPUT").unwrap()).unwrap();
	let (stream, codec) = {
		let stream = context.streams().find(|s| s.codec().medium() == ffmpeg::media::Type::Video).unwrap();
		let codec  = stream.codec().decoder().video().unwrap();

		((stream.index(), stream.time_base().into()), codec)
	};

	// Start the decoding thread.
	let frames = decode(context, codec, stream);
	let quit   = watcher();

	// Make the cursor invisible.
	expand!(io::stdout(),
		info.get::<cap::CursorInvisible>().unwrap()).unwrap();

	// Clear the screen.
	expand!(io::stdout(),
		info.get::<cap::ClearScreen>().unwrap()).unwrap();

	loop {
		let mut frame = None;

		// Skip frames.
		while let Ok(current) = frames.try_recv() {
			frame = Some(current);
		}

		if let Some(frame) = frame {
			// Move the cursor home.
			expand!(io::stdout(),
				info.get::<cap::CursorHome>().unwrap()).unwrap();

			// Encode the image to sixels and print to stdout.
			let image = picto::view::Read::<Rgba, u8>::with_stride(frame.width(), frame.height(), frame.stride(0), frame.data(0)).unwrap();
			encoder::encode(&settings, &image, io::stdout()).unwrap();
		}

		if let Err(TryRecvError::Disconnected) = quit.try_recv() {
			break;
		}
	}

	// Make the cursor visible.
	expand!(io::stdout(),
		info.get::<cap::CursorNormal>().unwrap()).unwrap();

	// Clear the screen.
	expand!(io::stdout(),
		info.get::<cap::ClearScreen>().unwrap()).unwrap();
}

fn decode(mut context: ffmpeg::format::context::Input, mut codec: ffmpeg::decoder::Video, (index, time_base): (usize, f64)) -> Receiver<ffmpeg::frame::Video> {
	let (sender, receiver) = channel();
	let start              = ffmpeg::time::relative() as f64 / 1_000_000.0;

	thread::spawn(move || {
		let mut decoded   = ffmpeg::frame::Video::empty();
		let mut converter = codec.converter(ffmpeg::format::Pixel::RGBA).unwrap();

		for (_, packet) in context.packets().filter(|&(ref s, _)| s.index() == index) {
			if let Ok(true) = codec.decode(&packet, &mut decoded) {
				let mut frame = ffmpeg::frame::Video::empty();
				frame.clone_from(&decoded);
				converter.run(&decoded, &mut frame).unwrap();

				let now = ffmpeg::time::relative() as f64 / 1_000_000.0 - start;
				let pts = frame.timestamp().unwrap_or(0) as f64 * time_base;

				if pts > now {
					ffmpeg::time::sleep(((pts - now) * 1_000_000.0) as u32).unwrap();
				}

				sender.send(frame).unwrap();
			}
		}
	});

	receiver
}

fn watcher() -> Receiver<()> {
	let (sender, receiver) = channel();

	thread::spawn(move || {
		let mut string = String::new();
		io::stdin().read_line(&mut string).unwrap();

		sender.send(()).unwrap();
	});

	receiver
}
