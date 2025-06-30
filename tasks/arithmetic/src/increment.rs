use utils::{impl_type_identifiable, BidirectionalStack};
use utils::{Executable, TypeIdentifiable};

#[repr(C)]
pub struct Increment {}

impl_type_identifiable!(Increment);

impl Increment {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Increment {
    fn default() -> Self {
        Self::new()
    }
}

impl Executable for Increment {
    fn execute<T: BidirectionalStack>(&mut self, stack: &mut T) -> Vec<Vec<u8>> {
        let result = stack.borrow_front();
        let result = u128::from_be_bytes(result.try_into().unwrap());
        let result = result.saturating_add(1);
        stack.push_front(&result.to_be_bytes()).unwrap();
        Vec::new()
    }

    fn is_finished(&mut self) -> bool {
        true
    }
}
