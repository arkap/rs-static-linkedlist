# Static-LinkedList

Linked List implementation that uses a static array as backing memory for an arbitrary number of linked lists.

## Usage

To create a linked list you first have to create a `StaticLinkedListBackingArray` passing to it an array of `u8`.
Then, you can create any number of lists backed by that array using `new_list()`. Note that creating arrays of zero-sized types
is not possible.

Since the created lists share the backing arrays underlying memory the total number of entries across all lists cannot exceed the
array's capacity.

The list needs some memory in the buffer for its metadata (pointers to the next element). If the buffer used to create a 
`StaticLinkedListBackingArray` is not big enough to hold `n` entries of data *plus* list metadata, the array will only have a
capacity of `n - 1` entries. The remaining bytes will be wasted. The `capacity_for()` function was designed to calculate the
exact amount of bytes necessary for holding `n` entries of data.
