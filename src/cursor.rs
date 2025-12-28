//! cursor.rs abstracts some common fetch/advance/rewind functionality commonly used e.g. in iterators.

pub struct Cursor<'a, T> {
	input: &'a [T],
	current: usize,
}

impl<'a, T> Cursor<'a, T> {
	pub const fn new(input: &'a [T]) -> Self {
		Self { input, current: 0 }
	}

	pub fn peek(&self) -> Option<&T> {
		self.input.get(self.current)
	}

	pub fn get(&mut self) -> Option<&T> {
		let idx = self.current;
		self.advance();
		self.input.get(idx)
	}

	pub const fn advance(&mut self) {
		self.current += 1;
	}

	pub const fn rewind(&mut self) {
		self.current -= 1;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	pub fn test_cursor_of_ints() {
		let input = vec![1, 2, 3];
		let mut c = Cursor::new(&input);
		assert_eq!(*c.peek().unwrap(), 1);
		assert_eq!(*c.get().unwrap(), 1);
		assert_eq!(*c.get().unwrap(), 2);
		assert_eq!(*c.get().unwrap(), 3);
		assert!(c.peek().is_none());
		assert!(c.get().is_none());
		c.rewind(); // Undo the get above
		c.rewind(); // Back to 3
		assert_eq!(*c.peek().unwrap(), 3);
		c.rewind();
		assert_eq!(*c.get().unwrap(), 2);
	}
}
