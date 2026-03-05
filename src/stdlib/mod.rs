//! OmniLang Standard Library
//! 
//! Core modules written in OmniLang with C FFI bindings.

pub mod io {
    //! I/O operations
    
    /// Print to stdout
    #[no_mangle]
    pub extern "C" fn print_string(s: *const i8) {
        if !s.is_null() {
            unsafe {
                print!("{}", std::str::from_utf8_unchecked(std::slice::from_raw_parts(s as *const u8, strlen(s))));
            }
        }
    }
    
    /// Print integer
    #[no_mangle]
    pub extern "C" fn print_int(n: i64) {
        print!("{}", n);
    }
    
    /// Print float
    #[no_mangle]
    pub extern "C" fn print_float(n: f64) {
        print!("{}", n);
    }
    
    /// Print boolean
    #[no_mangle]
    pub extern "C" fn print_bool(b: bool) {
        print!("{}", b);
    }
    
    /// Print with newline
    #[no_mangle]
    pub extern "C" fn println_string(s: *const i8) {
        if !s.is_null() {
            unsafe {
                println!("{}", std::str::from_utf8_unchecked(std::slice::from_raw_parts(s as *const u8, strlen(s))));
            }
        }
    }
    
    fn strlen(s: *const i8) -> usize {
        unsafe {
            let mut len = 0;
            while *s.add(len) != 0 {
                len += 1;
            }
            len
        }
    }
}

pub mod math {
    //! Mathematical functions
    
    /// Absolute value
    #[no_mangle]
    pub extern "C" fn abs(n: i64) -> i64 {
        n.abs()
    }
    
    /// Square root
    #[no_mangle]
    pub extern "C" fn sqrt(n: f64) -> f64 {
        n.sqrt()
    }
    
    /// Power
    #[no_mangle]
    pub extern "C" fn pow(base: f64, exp: f64) -> f64 {
        base.powf(exp)
    }
    
    /// Sine
    #[no_mangle]
    pub extern "C" fn sin(n: f64) -> f64 {
        n.sin()
    }
    
    /// Cosine
    #[no_mangle]
    pub extern "C" fn cos(n: f64) -> f64 {
        n.cos()
    }
    
    /// Tangent
    #[no_mangle]
    pub extern "C" fn tan(n: f64) -> f64 {
        n.tan()
    }
    
    /// Natural logarithm
    #[no_mangle]
    pub extern "C" fn ln(n: f64) -> f64 {
        n.ln()
    }
    
    /// Logarithm base 10
    #[no_mangle]
    pub extern "C" fn log10(n: f64) -> f64 {
        n.log10()
    }
    
    /// Exponential
    #[no_mangle]
    pub extern "C" fn exp(n: f64) -> f64 {
        n.exp()
    }
    
    /// Maximum of two integers
    #[no_mangle]
    pub extern "C" fn max_int(a: i64, b: i64) -> i64 {
        a.max(b)
    }
    
    /// Minimum of two integers
    #[no_mangle]
    pub extern "C" fn min_int(a: i64, b: i64) -> i64 {
        a.min(b)
    }
    
    /// Maximum of two floats
    #[no_mangle]
    pub extern "C" fn max_float(a: f64, b: f64) -> f64 {
        a.max(b)
    }
    
    /// Minimum of two floats
    #[no_mangle]
    pub extern "C" fn min_float(a: f64, b: f64) -> f64 {
        a.min(b)
    }
}

pub mod collections {
    //! Collection utilities
    
    /// Vector with push operation
    pub struct Vec<T> {
        data: *mut T,
        len: usize,
        capacity: usize,
    }
    
    impl<T> Vec<T> {
        pub fn new() -> Self {
            Self {
                data: std::ptr::null_mut(),
                len: 0,
                capacity: 0,
            }
        }
        
        pub fn push(&mut self, value: T) {
            if self.len >= self.capacity {
                let new_capacity = if self.capacity == 0 { 1 } else { self.capacity * 2 };
                let new_data = unsafe {
                    std::alloc::realloc(
                        self.data as *mut std::ffi::c_void,
                        new_capacity * std::mem::size_of::<T>()
                    ) as *mut T
                };
                self.data = new_data;
                self.capacity = new_capacity;
            }
            unsafe {
                *self.data.add(self.len) = value;
            }
            self.len += 1;
        }
        
        pub fn get(&self, index: usize) -> Option<&T> {
            if index < self.len {
                unsafe {
                    Some(&*self.data.add(index))
                }
            } else {
                None
            }
        }
        
        pub fn len(&self) -> usize {
            self.len
        }
    }
    
    /// HashMap placeholder
    pub struct HashMap<K, V> {
        // Simplified implementation
        _phantom: std::marker::PhantomData<(K, V)>,
    }
    
    impl<K, V> HashMap<K, V> {
        pub fn new() -> Self {
            Self {
                _phantom: std::marker::PhantomData,
            }
        }
    }
}

pub mod string {
    //! String operations
    
    /// String length
    #[no_mangle]
    pub extern "C" fn string_len(s: *const i8) -> usize {
        unsafe {
            let mut len = 0;
            while *s.add(len) != 0 {
                len += 1;
            }
            len
        }
    }
    
    /// String compare
    #[no_mangle]
    pub extern "C" fn string_eq(a: *const i8, b: *const i8) -> bool {
        unsafe {
            let mut i = 0;
            while *a.add(i) != 0 && *b.add(i) != 0 {
                if *a.add(i) != *b.add(i) {
                    return false;
                }
                i += 1;
            }
            *a.add(i) == *b.add(i)
        }
    }
    
    /// String concatenate
    #[no_mangle]
    pub extern "C" fn string_concat(a: *const i8, b: *const i8) -> *mut i8 {
        unsafe {
            let len_a = string_len(a);
            let len_b = string_len(b);
            let total = len_a + len_b + 1;
            let result = std::alloc::alloc(std::alloc::Layout::array::<i8>(total).unwrap()) as *mut i8;
            
            std::ptr::copy_nonoverlapping(a, result, len_a);
            std::ptr::copy_nonoverlapping(b, result.add(len_a), len_b);
            *result.add(total - 1) = 0;
            
            result
        }
    }
}

pub mod time {
    //! Time operations
    
    use std::time::{Duration, Instant};
    
    /// Get current timestamp in milliseconds
    #[no_mangle]
    pub extern "C" fn current_time_millis() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }
    
    /// Get current timestamp in nanoseconds
    #[no_mangle]
    pub extern "C" fn current_time_nanos() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64
    }
}
