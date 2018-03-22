pub trait ReadableBuffer<T> {
	/// Returns the backing
	fn backing(&self) -> &[T];
	
	/// Returns the buffer's position
	fn pos(&self) -> usize;
	
	/// Returns a mutable reference to the buffer's position
	fn pos_mut(&mut self) -> &mut usize;
	
	/// Returns the processed subslice (`&slice[.. *self.pos()]`)
	fn processed(&self) -> &[T] {
		&self.backing()[.. self.pos()]
	}
	
	/// Returns the remaining subslice (`&slice[*self.pos() ..]`)
	fn remaining(&self) -> &[T] {
		&self.backing()[self.pos() ..]
	}
	
	/// Reads `len`-bytes from the remaining subslice and increments the position accordingly
	fn read(&mut self, len: usize) -> &[T] {
		*self.pos_mut() += len;
		&self.remaining()[.. len]
	}
}

pub trait WriteableBuffer<T>: ReadableBuffer<T> {
	/// Returns the mutable backing
	fn backing_mut(&mut self) -> &mut[T];
	
	/// Returns the processed subslice (`&slice[.. *self.pos()]`)
	fn processed_mut(&mut self) -> &mut [T] {
		let pos = self.pos();
		&mut self.backing_mut()[.. pos]
	}
	
	/// Returns the remaining subslice (`&slice[*self.pos() ..]`)
	fn remaining_mut(&mut self) -> &mut [T] {
		let pos = self.pos();
		&mut self.backing_mut()[pos ..]
	}
	
	/// Appends `slice` and increments the position accordingly
	fn write(&mut self, slice: &[T]) where T: Copy {
		self.remaining_mut()[.. slice.len()].copy_from_slice(slice);
		*self.pos_mut() += slice.len()
	}
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
	fn backing(&self) -> &[T] {
		self.backing
	}
	fn pos(&self) -> usize {
		self.position
	}
	fn pos_mut(&mut self) -> &mut usize {
		&mut self.position
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
	fn backing(&self) -> &[T] {
		self.backing
	}
	fn pos(&self) -> usize {
		self.position
	}
	fn pos_mut(&mut self) -> &mut usize {
		&mut self.position
	}
}
impl<'a, T> WriteableBuffer<T> for MutableBackedBuffer<'a, T> {
	fn backing_mut(&mut self) -> &mut[T] {
		self.backing
	}
}



pub struct OwnedBuffer<T> {
	backing: Vec<T>,
	position: usize
}
impl<T> OwnedBuffer<T> {
	pub fn new(size: usize) -> Self where T: Default + Clone {
		OwnedBuffer{ backing: vec![T::default(); size], position: 0 }
	}
}
impl<T> ReadableBuffer<T> for OwnedBuffer<T> {
	fn backing(&self) -> &[T] {
		&self.backing
	}
	fn pos(&self) -> usize {
		self.position
	}
	fn pos_mut(&mut self) -> &mut usize {
		&mut self.position
	}
}
impl<T> WriteableBuffer<T> for OwnedBuffer<T> {
	fn backing_mut(&mut self) -> &mut[T] {
		&mut self.backing
	}
}
impl<T> From<Vec<T>> for OwnedBuffer<T> {
	fn from(vector: Vec<T>) -> Self {
		OwnedBuffer{ backing: vector, position: 0 }
	}
}
impl<T> Into<Vec<T>> for OwnedBuffer<T> {
	fn into(self) -> Vec<T> {
		self.backing
	}
}