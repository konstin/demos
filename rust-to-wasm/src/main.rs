pub struct Counter {
    count: i32,
}

// Methods on the Counter struct.
impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }

    fn increment(&mut self) {
        self.count += 1;
    }

    fn decrement(&mut self) {
        self.count -= 1;
    }

    fn set(&mut self, count: i32) {
        self.count = count;
    }

    fn get(& self) -> i32 {
        self.count
    }
}

#[no_mangle]
pub extern "C" fn counter_create() -> *mut Counter {
    // Allocate a Counter struct on the heap and convert it into a mutable pointer.
    Box::into_raw(Box::new(Counter::new()))
}

#[no_mangle]
pub unsafe extern "C" fn counter_increment(counter: *mut Counter) {
    let counter = &mut *counter;
    counter.increment();
}

#[no_mangle]
pub unsafe extern "C" fn counter_decrement(counter: *mut Counter) {
    let counter = &mut *counter;
    counter.decrement();
}

#[no_mangle]
pub unsafe extern "C" fn counter_set(counter: *mut Counter, count: i32) {
    let counter = &mut *counter;
    counter.set(count);
}

#[no_mangle]
pub unsafe extern "C" fn counter_get(counter: *const Counter) -> i32 {
    let counter = &*counter;
    counter.get()
}

#[no_mangle]
pub unsafe extern "C" fn counter_destroy(counter: *mut Counter) {
    // Convert the mutable pointer back to a box and let it fall out of scope.
    Box::from_raw(counter);
}

#[no_mangle]
pub fn add(a: i32, b: i32, c: i32) -> i32 {
    return a + b + c;
}


fn main() {
    println!("Hello, world!");
}