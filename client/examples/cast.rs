#[repr(C)]
struct MyStruct {
    a: u8,
    b: u8,
    c: u8,
}

fn cast_slice_to_struct(slice: &mut [u8]) -> &mut MyStruct {
    assert_eq!(slice.len(), std::mem::size_of::<MyStruct>());
    unsafe { &mut *(slice.as_mut_ptr() as *mut MyStruct) }
}

fn main() {
    let slice: &mut [u8] = &mut [1, 2, 3];
    let my_struct: &mut MyStruct = cast_slice_to_struct(slice);

    println!("Before: {} {} {}", my_struct.a, my_struct.b, my_struct.c);

    // Modify the struct
    my_struct.a = 10;
    my_struct.b = 20;
    my_struct.c = 30;

    println!("After: {} {} {}", my_struct.a, my_struct.b, my_struct.c);
    println!("Slice: {slice:?}");
}
