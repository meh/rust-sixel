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

use std::io::{self, Write, BufWriter};
use std::collections::{HashSet, HashMap};
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;
use picto::buffer;
use picto::color::{Rgba, Hsl};
use picto::processing::prelude::*;
use control;
use control::DEC::SIXEL;

use settings::Settings;

pub struct Encoder {
	settings: Settings,
}

impl Encoder {
	pub fn new(settings: Settings) -> Self {
		Encoder {
			settings: settings,
		}
	}

	pub fn encode<W: Write>(&self, buffer: &buffer::Rgba, output: W) -> io::Result<()> {
		let mut output = BufWriter::new(output);

		let (width, height) = self.settings.size()
			.unwrap_or(buffer.dimensions());

		let buffer = if width < buffer.width() || height < buffer.height() {
			buffer.scale_to::<scaler::Lanczos3>(width, height)
		}
		else {
			buffer.scale_to::<scaler::Linear>(width, height)
		};

		let (width, height) = buffer.dimensions();

		let buffer = match self.settings.colors() {
			Some(8)   => buffer.dither::<ditherer::Palette<ditherer::palette::table::VT340>>(16),
			Some(16)  => buffer.dither::<ditherer::Palette<ditherer::palette::table::VT340>>(16),
			Some(64)  => buffer.dither::<ditherer::NeuQuant>(64),
			Some(256) => buffer.dither::<ditherer::NeuQuant>(256),
			Some(n)   => buffer.dither::<ditherer::Palette<ditherer::palette::table::Gray2>>(n as u32),
			None      => buffer,
		};

		let buffer = buffer.pixels().map(|(_, _, p)| p.get().to_pixel())
			.collect::<Vec<(u8, u8, u8, u8)>>();

		output.write_all(b"\x1BP9q\n")?;

		let mut id       = 0;
		let mut register = HashMap::<(u8, u8, u8, u8), u32, BuildHasherDefault<FnvHasher>>::default();

		for row in 0 .. height / 6 {
			let mut colors = HashSet::<(u8, u8, u8, u8), BuildHasherDefault<FnvHasher>>::default();

			// Find all the colors in the Sixel line.
			for x in 0 .. width {
				for y in row * 6 .. row * 6 + 6 {
					// Remove the alpha component if unavailable.
					if self.settings.colors().is_none() {
						colors.insert(buffer[(y * width + x) as usize]);
					}
					else {
						let (r, g, b, _) = buffer[(y * width + x) as usize];
						colors.insert((r, g, b, 255));
					}
				}
			}

			// Register the colors if needed.
			for &(r, g, b, a) in &colors {
				if !register.contains_key(&(r, g, b, a)) {
					register.insert((r, g, b, a), id);

					// Print the properly colored register.
					if self.settings.colors().is_none() {
						control::format_to(output.by_ref(), &SIXEL::Define(id,
							SIXEL::Color::Rgba(r, g, b, a)), true)?;
					}
					else {
						// Use HSL since it has a bigger color space.
						let hsl = Hsl::<f32>::from(Rgba::new_u8(r, g, b, a));

						control::format_to(output.by_ref(), &SIXEL::Define(id,
							SIXEL::Color::Hsl(
								hsl.hue.to_positive_degrees() as u16,
								(hsl.saturation * 100.0) as u8,
								(hsl.lightness * 100.0) as u8)), true)?;
					}

					id += 1;
				}
			}

			output.write_all(b"\n")?;

			// For each color generate the sixel line.
			for color in &colors {
				control::format_to(output.by_ref(), &SIXEL::Enable(
					*register.get(color).unwrap()), true)?;

				let mut previous = None;
				let mut count    = 0;

				for x in 0 .. width {
					let mut current = SIXEL::Map::default();

					for (i, y) in (row * 6 .. row * 6 + 6).enumerate() {
						if *color == buffer[(y * width + x) as usize] {
							current.set(i as u8, true);
						}
					}

					if let Some(value) = previous {
						if value == current {
							count += 1;
						}
						else {
							control::format_to(output.by_ref(), &if count == 1 {
								SIXEL::Value(value)
							}
							else {
								SIXEL::Repeat(count, value)
							}, true)?;

							previous = Some(current);
							count    = 1;
						}
					}
					else {
						previous = Some(current);
						count    = 1;
					}
				}

				if let Some(value) = previous {
					control::format_to(output.by_ref(), &if count == 1 {
						SIXEL::Value(value)
					}
					else {
						SIXEL::Repeat(count, value)
					}, true)?;
				}

				control::format_to(output.by_ref(), &SIXEL::CarriageReturn, true)?;
				output.write_all(b"\n")?;
			}

			control::format_to(output.by_ref(), &SIXEL::LineFeed, true)?;
			output.write_all(b"\n")?;
		}

		output.write_all(b"\x1B\\\n")?;

		Ok(())
	}
}
