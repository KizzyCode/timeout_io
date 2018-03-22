pub trait ReadableBuffer<T> {
	/// Returns a mutable reference to the buffer's position
	fn pos(&mut self) -> &mut usize;
	
	/// Returns the processed subslice (`&slice[.. *self.pos()]`)
	fn processed(&self) -> &[T];
	
	/// Returns the remaining subslice (`&slice[*self.pos() ..]`)
	fn remaining(&self) -> &[T];
	
	/// Reads `len`-bytes from the remaining subslice and increments the position accordingly
	fn read(&mut self, len: usize) -> &[T];
}

pub trait WriteableBuffer<T> {
	/// Returns a mutable reference to the buffer's position
	fn pos(&mut self) -> &mut usize;
	
	/// Returns the processed subslice (`&slice[.. *self.pos()]`)
	fn processed(&mut self) -> &mut [T];
	
	/// Returns the remaining subslice (`&slice[*self.pos() ..]`)
	fn remaining(&mut self) -> &mut [T];
	
	/// Appends `slice` and increments the position accordingly
	fn write(&mut self, slice: &[T]) where T: Copy;
}



pub struct BackedBuffer<'a, T: 'static> {
	backing: &'a[T],
	position: usize
}
impl<'a, T> BackedBuffer<'a, T> {
	pub fn new(backing: &'a[T]) -> Self {
		BackedBuffer{ backing, position: 0 }
	}
}
impl<'a, T> ReadableBuffer<T> for BackedBuffer<'a, T> {
	/// Returns a mutable reference to the buffer's position
	fn pos(&mut self) -> &mut usize {
		&mut self.position
	}
	
	/// Returns the processed subslice (`&slice[.. *self.pos()]`)
	fn processed(&self) -> &[T] {
		&self.backing[.. self.position]
	}
	
	/// Returns the remaining subslice (`&slice[*self.pos() ..]`)
	fn remaining(&self) -> &[T] {
		&self.backing[self.position ..]
	}
	
	/// Reads `len`-bytes from the remaining subslice and increments the position accordingly
	fn read(&mut self, len: usize) -> &[T] {
		self.position += len;
		&self.remaining()[.. len]
	}
}



pub struct MutableBackedBuffer<'a, T: 'static> {
	backing: &'a mut[T],
	position: usize
}
impl<'a, T> MutableBackedBuffer<'a, T> {
	pub fn new(backing: &'a mut[T]) -> Self {
		MutableBackedBuffer{ backing, position: 0 }
	}
}
impl<'a, T> ReadableBuffer<T> for MutableBackedBuffer<'a, T> {
	/// Returns a mutable reference to the buffer's position
	fn pos(&mut self) -> &mut usize {
		&mut self.position
	}
	
	/// Returns the processed subslice (`&slice[.. *self.pos()]`)
	fn processed(&self) -> &[T] {
		&self.backing[.. self.position]
	}
	
	/// Returns the remaining subslice (`&slice[*self.pos() ..]`)
	fn remaining(&self) -> &[T] {
		&self.backing[self.position ..]
	}
	
	/// Reads `len`-bytes from the remaining subslice and increments the position accordingly
	fn read(&mut self, len: usize) -> &[T] {
		self.position += len;
		&self.remaining()[.. len]
	}
}
impl<'a, T> WriteableBuffer<T> for MutableBackedBuffer<'a, T> {
	/// Returns a mutable reference to the buffer's position
	fn pos(&mut self) -> &mut usize {
		&mut self.position
	}
	
	/// Returns the processed subslice (`&slice[.. *self.pos()]`)
	fn processed(&mut self) -> &mut[T] {
		&mut self.backing[.. self.position]
	}
	
	/// Returns the remaining subslice (`&slice[*self.pos() ..]`)
	fn remaining(&mut self) -> &mut[T] {
		&mut self.backing[self.position ..]
	}
	
	/// Appends `slice` and increments the position accordingly
	fn write(&mut self, slice: &[T]) where T: Copy {
		self.remaining()[.. slice.len()].copy_from_slice(slice);
		self.position += slice.len()
	}
}