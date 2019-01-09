#[derive(Debug, PartialEq, Eq)]
pub enum Error {
	OutOfSpace,	// The backing array is used completely
	HeadIsNull, // The list's head is a null pointer
	NullPointer, // The list doesn't point to a backing array but to NULL
	ZeroSizedType, // trying to initialize a backing array for a ZST
	IndexOutOfBounds,
}
