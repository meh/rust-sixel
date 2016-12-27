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

use std::io;
use termsize;
use terminfo::{Database, capability as cap};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Environment {
	colors: Option<u16>,
	limits: (u32, u32),
	cell:   Option<(u32, u32)>,
}

impl Environment {
	pub fn query() -> io::Result<Self> {
		let info = Database::from_env().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
		let size = termsize::get().ok_or(io::Error::new(io::ErrorKind::Other, "no size"))?;

		Ok(Environment {
			colors: if info.raw("Ts").is_none() {
				Some(info.get::<cap::MaxColors>().map(|c| c.0 as u16).unwrap_or(2))
			}
			else {
				None
			},

			limits: (size.cols as u32, size.rows as u32),

			cell: if size.width.is_some() && size.height.is_some() {
				Some((
					(size.width.unwrap() / size.cols) as u32,
					(size.height.unwrap() / size.rows) as u32))
			}
			else {
				None
			},
		})
	}

	pub fn colors(&self) -> Option<u16> {
		self.colors
	}

	pub fn cell(&self) -> Option<(u32, u32)> {
		self.cell
	}

	pub fn limits(&self) -> (u32, u32) {
		self.limits
	}

	pub fn size(&self, cols: u32, rows: u32) -> Option<(u32, u32)> {
		self.cell.map(|(width, height)|
			(cols * width as u32, rows * height as u32))
	}

	pub fn padding(&self, size: u32) -> Option<(u32, u32)> {
		self.cell.map(|(width, height)|
			(size * width as u32, size * height as u32))
	}
}
