#![no_std]
#![feature(const_fn)]

//! Linked List implementation that uses a static array as backing memory for an arbitrary number of linked lists.
//!
//! # Usage
//! To create a linked list you first have to create a [`StaticLinkedListBackingArray`] passing to it an array of `u8`.
//! Then, you can create any number of lists backed by that array using [`new_list()`]. Note that creating arrays of zero-sized types
//! is not possible.
//!
//! Since the created lists share the backing arrays underlying memory the total number of entries across all lists cannot exceed the
//! array's capacity.
//!
//! The list needs some memory in the buffer for its metadata (pointers to the next element). If the buffer used to create a 
//! [`StaticLinkedListBackingArray`] is not big enough to hold `n` entries of data *plus* list metadata, the array will only have a
//! capacity of `n - 1` entries. The remaining bytes will be wasted. The [`capacity_for()`] function was designed to calculate the
//! exact amount of bytes necessary for holding `n` entries of data.
//!
//! [`capacity_for()`]: struct.StaticLinkedListBackingArray.html#method.capacity_for
//! [`StaticLinkedListBackingArray`]: struct.StaticLinkedListBackingArray.html
//! [`new_list()`]: struct.StaticLinkedListBackingArray.html#method.new_list

use core::ptr::null_mut;
use core::mem::size_of;

pub use crate::error::Error;
use crate::error::Error::*;

mod error;

/// Clear the memory of an instance of type T. Types stored in lists must implement this trait. 
/// [`clear()`] is called before a memory block is returned to the backing array's memory pool.
///
/// [`clear()`]: trait.Clear.html#tymethod.clear
pub trait Clear {
	fn clear(&mut self);
}

// One link in the list
struct Linked<T> where T: Clear {
    next: *mut Linked<T>,
    data: T,
}

impl<T> Clear for Linked<T> where T: Clear {
	fn clear(&mut self) {
		self.data.clear();
	}
}

/// A singly-linked list for elements of type T backed by a static array.
pub struct StaticLinkedList<'buf, T> where T: Clear {
	size: usize,

	head: *mut Linked<T>,
	tail: *mut Linked<T>,

	array: *mut StaticLinkedListBackingArray<'buf, T>
}

impl<'buf, T> StaticLinkedList<'buf, T> where T: Clear {
	/// Returns the number of elements stored in the list.
	pub fn size(&self) -> usize {
		self.size
	}

	/// Returns the remaining space in this list's backing array (in units of `T`).
	pub fn free_space(&self) -> usize {
		if self.array.is_null() {
			0
		} else {
			unsafe {
				(*self.array).free_space()	
			}
		}
	}

	/// Appends T to the end of the list.
	pub fn append(&mut self, data: T) -> Result<(), Error> {
		if self.array.is_null() {
			Err(NullPointer)
		} else {
			unsafe {
				if let Some(new) = (*self.array).get_free() {
					(*new).data = data;

					if self.tail.is_null() {
						self.head = new;						
					} else {
						(*self.tail).next = new;
					}

					(*new).next = null_mut();
					self.tail = new;
					self.size += 1;

					Ok(())
				} else {
					Err(OutOfSpace)
				}
			}
		}
	}

	/// Prepends T to the list's head.
	pub fn prepend(&mut self, data: T) -> Result<&mut Self, Error> {
		if self.array.is_null() {
			Err(NullPointer)
		} else {
			unsafe {
				if let Some(new) = (*self.array).get_free() {
					(*new).data = data;

					if self.tail.is_null() {
						self.tail = new;
					}
					(*new).next = self.head;
					self.head = new;
					self.size += 1;

					Ok(self)
				} else {
					Err(OutOfSpace)
				}
			}
		}
	}

	/// Returns a reference to the first element in the list.
	pub fn head(&self) -> Option<&T> {
		if self.head.is_null() {
			None
		} else {
			unsafe {
				Some(&(*self.head).data)
			}
		}
	}

	/// Returns a reference to the last element in the list.
	pub fn tail(&self) -> Option<&T> {
		if self.tail.is_null() {
			None
		} else {
			unsafe {
				Some(&(*self.tail).data)
			}
		}
	}

	/// Returns a reference to the list element at 'index'.
	pub fn at(&self, index: usize) -> Result<&T, Error> {
		if index >= self.size() {
			Err(IndexOutOfBounds)
		} else {
			let mut i = 0;
			let mut iter = self.into_iter();

			while i != index {
				iter.next();
				i += 1;
			}

			Ok(iter.next().unwrap())
		}
	}

	/// Removes the first element from the list.
	pub fn remove_head(&mut self) -> Result<&mut Self, Error> {
		if !self.head.is_null() {
			unsafe {
				if let Some(p) = self.array.as_mut() {
					(*self.head).data.clear();

					if self.head == self.tail {
						self.tail = null_mut();
					}

					let to_remove = self.head;
					self.head = (*self.head).next;

					p.free(to_remove.as_mut().unwrap());
					self.size -= 1;
					Ok(self)
				} else {
					Err(OutOfSpace)
				}
			}
		} else {
			Err(HeadIsNull)
		}
	}

	/// Removes all elements `e` from the list where `predicate(e) == true`.
	pub fn remove_all_satisfying(&mut self, predicate: fn(&T) -> bool) -> Result<&mut Self, Error> {
		let mut cursor = self.head;
		let mut prev = null_mut();

		unsafe {
			while !cursor.is_null() {
				if predicate(&(*cursor).data) {
					let to_remove = cursor;
					if to_remove == self.head {
						self.head = (*cursor).next;	
					}

					if to_remove == self.tail {
						self.tail = prev;
					}

					if !prev.is_null() {
						(*prev).next = (*cursor).next;
					}
					(*self.array).free(to_remove.as_mut().unwrap());
					self.size -= 1;
				} else {
					prev = cursor;
				}
				cursor = (*cursor).next;
			}
			Ok(self)
		}
	}
}

/// Ensures that the memory occupied by the list is returned to its backing
/// array when it goes out of scope.
impl<'buf, T> Drop for StaticLinkedList<'buf, T> where T: Clear {
	fn drop(&mut self) {
		if self.array.is_null() {
			return;			
		}

		while self.size() > 0 {
			self.remove_head().unwrap();
		}

		unsafe {
			(*self.array).drop_list();
		}
	}
}

/// Iterator over the elements in the list.
pub struct StaticLinkedListIterator<'a, T> where T: Clear {
	cursor: *mut Linked<T>,

	_phantom: &'a core::marker::PhantomData<T>,
}

impl<'a, T> StaticLinkedListIterator<'a, T> where T: Clear {
	fn new(list: &'a StaticLinkedList<T>) -> Self {
		StaticLinkedListIterator {
			cursor: list.head,
			_phantom: &core::marker::PhantomData::<T>,
		}
	}
}

impl<'a, T> Iterator for StaticLinkedListIterator<'a, T> where T: Clear {
	type Item = &'a T;

	fn next(&mut self) -> Option<&'a T> {
		unsafe {
			if self.cursor.is_null() {
				None
			} else {
				let ret = &(*self.cursor).data;
				self.cursor = (*self.cursor).next;
				Some(ret)
			}
		}
	}
}

impl<'l, 'buf, T> IntoIterator for &'l StaticLinkedList<'buf, T> where T: Clear {
	type Item = &'l T;
	type IntoIter = StaticLinkedListIterator<'l, T>;

	fn into_iter(self) -> StaticLinkedListIterator<'l, T> {
		StaticLinkedListIterator::<'l, T>::new(self)
	}
}

/// The backing array for the singly-linked lists. This struct needs to be initialized first
/// before lists can be created.
pub struct StaticLinkedListBackingArray<'buf, T> where T: Clear {
    capacity: usize,
    free_space: usize,
    lists: usize,		// number of lists relying on this array

    buf: &'buf mut [u8], // let the array own the buffer
    free: *mut Linked<T>, // pointer into free linked entries in the buffer
}

impl<'buf, T> StaticLinkedListBackingArray<'buf, T> where T: Clear {
	/// Convenience function calculating the bytes required for an array of `n` elements of type `T` 
	/// *plus* the list's metadata (i.e. `next` pointer). **Requires the** `const_fn` **feature.**
	///
	/// # Example:
	///  ```
	///  use static_linkedlist::{Clear, StaticLinkedListBackingArray};
	///
	///  struct U32Clear(pub u32);
	///
	///  impl Clear for U32Clear {
	///  	fn clear(&mut self) {
	///  		self.0 = 0;
	///  	}
	///  }
	///
	///  // Reserve memory for 20 instances of U32Clear plus list metadata. This executes at compile time!
	///  const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(20);
	///  let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	///  let mut array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();
	///  ```
	pub const fn capacity_for(n: usize) -> usize {
		n * size_of::<Linked<T>>()
	}

	/// Creates a new backing array for linked lists from the given `buf`.
	/// The second argument `bytes` *must* be the size of `buf` in bytes!
	/// 
	/// # Buffer size
	/// Note that the linked list needs some space for metadata (pointers to the next element).
	/// Consequently, for `n` elements of size `s`, it does not suffice to allocate `s * n` bytes!
	/// To allocate the exact needed amount of memory, use [`capacity_for()`].
	///
	/// [`capacity_for()`]: ../static_linkedlist/struct.StaticLinkedListBackingArray.html#method.capacity_for
	pub fn new(buf: &'buf mut [u8]) -> Result<Self, Error> {
		if core::mem::size_of::<T>() == 0 {
			Err(ZeroSizedType)
		} else {
			let linkedbuf = buf.as_mut_ptr() as *mut Linked<T>;
			let capacity = buf.len() / size_of::<Linked<T>>();
			
			// Initialize memory:
			// - create linked list of free blocks by setting 'next' pointer to adjacent memory blocks
			// - clear list content calling 'clear()' on every entry
			unsafe {
				let mut cursor = linkedbuf;
				for _i in 0..(capacity - 1) {
					(*cursor).next = cursor.add(1);
					cursor.as_mut().unwrap().clear();
					cursor = (*cursor).next;
				}
				(*cursor).next = null_mut();
			}

			Ok(StaticLinkedListBackingArray {
				capacity: capacity,
				free_space: capacity,
				lists: 0,
				buf: buf,
				free: linkedbuf,
			})
		}
	}

	/// Returns the backing array's capacity.
	pub fn capacity(&self) -> usize {
		self.capacity
	}

	/// Returns the remaining space for element sof type T in the array.
	pub fn free_space(&self) -> usize {
		self.free_space
	}

	unsafe fn get_free(&mut self) -> Option<*mut Linked<T>> {
		if self.free.is_null() {
			None
		} else {
			let to_return = self.free;
			self.free = (*self.free).next;
			self.free_space -= 1;

			Some(to_return)
		}
	}

	fn free(&mut self, link: &mut Linked<T>) {
		// TODO: make sure link points to a link in our buffer.
		link.clear();
		link.next = self.free;
		self.free = link;
		self.free_space += 1;
	}

	/// Returns `true` if the array is full.
	pub fn is_full(&self) -> bool {
		self.free_space == 0
	}

	/// Return the number of lists backed by this array.
	pub fn lists(&self) -> usize {
		self.lists
	}

	fn drop_list(&mut self) {
		self.lists -= 1;
	}

	/// Creates a new [`StaticLinkedList`] backed by the memory of this array.
	///
	/// [`StaticLinkedList`]: struct.StaticLinkedList.html
	pub fn new_list(&mut self) -> StaticLinkedList<'buf, T> {
		self.lists += 1;
		StaticLinkedList {
			size: 0,
			head: core::ptr::null_mut(),
			tail: core::ptr::null_mut(),
			array: self as *mut StaticLinkedListBackingArray<'buf, T>
		}
	}
}

#[cfg(test)]
mod tests;
