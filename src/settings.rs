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


#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Settings {
	colors: Option<u16>,
	size:   Option<(u32, u32)>,
	cell:   Option<(u32, u32)>,
}

impl Default for Settings {
	fn default() -> Self {
		Settings::new().build()
	}
}

#[derive(Eq, PartialEq, Default, Debug)]
pub struct Builder {
	colors: Option<u16>,
	size:   Option<(u32, u32)>,
	cell:   Option<(u32, u32)>,
}

impl Settings {
	pub fn new() -> Builder {
		Builder::default()
	}

	pub fn colors(&self) -> Option<u16> {
		self.colors
	}

	pub fn size(&self) -> Option<(u32, u32)> {
		self.size
	}

	pub fn cell(&self) -> Option<(u32, u32)> {
		self.cell
	}
}

impl Builder {
	pub fn colors(&mut self, colors: u16) -> &mut Self {
		self.colors = Some(colors);
		self
	}

	pub fn size(&mut self, width: u32, height: u32) -> &mut Self {
		self.size = Some((width, height));
		self
	}

	pub fn build(&self) -> Settings {
		Settings {
			colors: self.colors.or_else(|| {
				if let Ok(info) = Database::from_env() {
					if info.raw("Ts").is_none() {
						return Some(info.get::<cap::MaxColors>().map(|c| c.0 as u16).unwrap_or(2));
					}
				}

				None
			}),

			size: self.size,

			cell: termsize::get().and_then(|c|
				if c.width.is_some() && c.height.is_some() {
					Some(((c.width.unwrap() / c.cols) as u32, (c.height.unwrap() / c.rows) as u32))
				}
				else {
					None
				}),
		}
	}
}
