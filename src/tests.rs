use super::*;

#[derive(Debug, PartialEq, Eq)]
struct U32Clear(pub u32);

impl Clear for U32Clear {
	fn clear(&mut self) {
		self.0 = 0;
	}
}

#[test]
fn test_forbid_zst() {
	struct ZeroSizedType {}

	impl Clear for ZeroSizedType {
		fn clear(&mut self) {}
	}

	let mut buf: [u8; 1] = [0; 1];
	if let Err(error) = StaticLinkedListBackingArray::<ZeroSizedType>::new(&mut buf) {
		assert!(error == ZeroSizedType);
	} else {
		assert!(false);
	}
}

#[test]
fn test_capacity_for() {
	let wrapped_data_size = size_of::<Linked<U32Clear>>();
	const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(10);
	let _buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

	assert_eq!(BUF_SIZE, 10 * wrapped_data_size);
}

#[test]
fn test_capacity() {
	const BUF_1_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(2);
	const BUF_2_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(2) + 1;
	let mut buf_1: [u8;  BUF_1_SIZE] = [0; BUF_1_SIZE];
	let mut buf_2: [u8; BUF_2_SIZE] = [0; BUF_2_SIZE];

	let array_1 = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf_1).unwrap();
	assert_eq!(array_1.capacity(), BUF_1_SIZE / size_of::<Linked<U32Clear>>());

	let array_2 = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf_2).unwrap();
	assert_eq!(array_2.capacity(), BUF_2_SIZE / size_of::<Linked<U32Clear>>());
}

#[test]
fn test_links() {
	let mut buf: [u8; 80] = [0; 80];

	let array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();

	let mut cnt = 0;
	let mut cursor = array.free;

	while !cursor.is_null() {
		cnt += 1;
		unsafe {
			cursor = (*cursor).next;
		}
	}

	assert_eq!(cnt, array.capacity());
}

#[test]
fn test_append() {
	const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(10);
	let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	let mut array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();
	let mut list = array.new_list();

	let mut cnt = 0;

	while !array.is_full() {
		list.append(U32Clear(cnt)).unwrap();
		cnt += 1;
	}

	assert_eq!(cnt as usize, array.capacity());
	assert!(array.free.is_null());
	unsafe {
		assert_eq!((*list.tail).data.0, cnt - 1);
		assert_eq!((*list.head).data.0, 0);
	}

	cnt = 0;
	let mut cursor = list.head;

	while !cursor.is_null() {
		unsafe {
			assert_eq!((*cursor).data.0, cnt);
			cursor = (*cursor).next;
		}
		cnt += 1;
	}
}

#[test]
fn test_prepend() {
	const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(10);
	let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	let mut array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();
	let mut list = array.new_list();

	let mut cnt = 10;

	while !array.is_full() {
		list.prepend(U32Clear(cnt)).unwrap();
		cnt -= 1;
	}

	assert!(array.free.is_null());
	assert_eq!(cnt, 0);
}

#[test]
fn test_remove_all_satisfying_head() {
	const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(3);
	let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	let mut array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();
	let mut list = array.new_list();

	let val_1 = U32Clear(1);
	let val_2 = U32Clear(2);
	let val_3 = U32Clear(3);

	list.append(val_1).unwrap();
	list.append(val_2).unwrap();
	list.append(val_3).unwrap();

	assert!(array.is_full());
	assert!(array.free.is_null());
	assert_eq!(array.free_space, 0);

	list.remove_all_satisfying(|entry| -> bool {
		entry.0 == 1
	}).unwrap();

	assert!(!array.is_full());
	assert_eq!(array.free_space(), 1);

	unsafe {
		assert_eq!((*list.head).data.0, 2);
		assert_eq!((*list.tail).data.0, 3);
		assert_eq!((*list.head).next, list.tail);
	}
}

#[test]
fn test_remove_all_satisfying_tail() {
	const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(3);
	let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	let mut array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();
	let mut list = array.new_list();

	let val_1 = U32Clear(1);
	let val_2 = U32Clear(2);
	let val_3 = U32Clear(3);

	list.append(val_1).unwrap();
	list.append(val_2).unwrap();
	list.append(val_3).unwrap();

	assert!(array.is_full());
	assert!(array.free.is_null());
	assert_eq!(array.free_space, 0);

	list.remove_all_satisfying(|entry| -> bool {
		entry.0 == 3
	}).unwrap();

	assert!(!array.is_full());
	assert_eq!(array.free_space(), 1);

	unsafe {
		assert_eq!((*list.head).data.0, 1);
		assert_eq!((*list.tail).data.0, 2);
		assert_eq!((*list.head).next, list.tail);
	}
}

#[test]
fn test_remove_all_satisfying_inner() {
	const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(3);
	let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	let mut array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();
	let mut list = array.new_list();

	let val_1 = U32Clear(1);
	let val_2 = U32Clear(2);
	let val_3 = U32Clear(3);

	list.append(val_1).unwrap();
	list.append(val_2).unwrap();
	list.append(val_3).unwrap();

	assert!(array.is_full());
	assert!(array.free.is_null());
	assert_eq!(array.free_space, 0);

	list.remove_all_satisfying(|entry| -> bool {
		entry.0 == 2
	}).unwrap();

	assert!(!array.is_full());
	assert_eq!(array.free_space(), 1);

	unsafe {
		assert_eq!((*list.head).data.0, 1);
		assert_eq!((*list.tail).data.0, 3);
		assert_eq!((*list.head).next, list.tail);
	}
}

#[test]
fn test_iterator() {
	const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(20);
	let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	let mut array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();
	let mut list = array.new_list();

	let mut cnt = 0;

	while !array.is_full() {
		list.append(U32Clear(cnt)).unwrap();
		cnt += 1;
	}

	assert!(array.free.is_null());
	assert_eq!(cnt, 20);

	cnt = 0;

	for entry in &list {
		assert_eq!(entry.0, cnt);
		cnt += 1;
	}
}

#[test]
fn test_free() {
	const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(20);
	let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	let mut array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();
	let mut list = array.new_list();

	list.append(U32Clear(1)).unwrap();
	list.append(U32Clear(2)).unwrap();
	list.append(U32Clear(3)).unwrap();

	let free_cnt = array.free_space();

	unsafe {
		let list_head = list.head.as_mut().unwrap();
		array.free(list_head);
	}

	assert_eq!(array.free_space(), free_cnt + 1);
}

#[test]
fn test_list_drop() {
	const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(20);
	let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	let mut array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();
	
	let initial_array_capacity = array.capacity();
	assert_eq!(initial_array_capacity, array.free_space());
	assert_eq!(array.lists(), 0);
	{
		let mut list = array.new_list();
		list.append(U32Clear(1)).unwrap();
		list.append(U32Clear(2)).unwrap();
		list.append(U32Clear(3)).unwrap();

		assert_eq!(array.lists(), 1);
		assert_eq!(list.size(), 3);
		assert_eq!(array.free_space(), initial_array_capacity - 3);
	}

	assert_eq!(array.lists(), 0);
	assert_eq!(initial_array_capacity, array.free_space());
}

#[test]
fn test_at() {
	const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(10);
	let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	let mut array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();
	let mut list = array.new_list();

	for i in 0..10 {
		list.append(U32Clear(i)).unwrap();
	}

	for i in 0..10 {
		assert_eq!(list.at(i).unwrap(), &U32Clear(i as u32));
	}

	let mut error = false;
	if let Err(e) = list.at(10) {	
		error = true;
		assert_eq!(e, IndexOutOfBounds);
	}
	assert!(error);
}

#[test]
fn test_head_and_tail_getters() {
	const BUF_SIZE: usize = StaticLinkedListBackingArray::<U32Clear>::capacity_for(10);
	let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
	let mut array = StaticLinkedListBackingArray::<U32Clear>::new(&mut buf).unwrap();
	
	{
		let mut list = array.new_list();
		list.append(U32Clear(1)).unwrap();
		assert_eq!(list.head().unwrap().0, 1);
		assert_eq!(list.tail().unwrap().0, 1);
	}

	{
		let mut list = array.new_list();
		list.append(U32Clear(1)).unwrap();
		list.append(U32Clear(2)).unwrap();
		assert_eq!(list.head().unwrap().0, 1);
		assert_eq!(list.tail().unwrap().0, 2);	
	}

	{
		let list = array.new_list();
		assert_eq!(list.head(), None);
		assert_eq!(list.tail(), None);
	}
}