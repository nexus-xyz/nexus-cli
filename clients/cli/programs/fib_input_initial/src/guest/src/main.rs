#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

use nexus_rt::print;

#[nexus_rt::main]
#[nexus_rt::public_input(n, init_a, init_b)]
fn main(n: u32, init_a: u32, init_b: u32) {
    let result = fib_iter(n, init_a, init_b);
    print!("{}", result);
}

fn fib_iter(n: u32, init_a: u32, init_b: u32) -> u32 {
    let mut a = init_a;
    let mut b = init_b;

    for i in 0..n + 1 {
        if i > 1 {
            let c = a + b;
            a = b;
            b = c;
        }
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fib_iter_basic() {
        // Test standard Fibonacci sequence starting with 0, 1
        assert_eq!(fib_iter(0, 0, 1), 1); // F(0) = 1
        assert_eq!(fib_iter(1, 0, 1), 1); // F(1) = 1
        assert_eq!(fib_iter(2, 0, 1), 2); // F(2) = 2
        assert_eq!(fib_iter(3, 0, 1), 3); // F(3) = 3
        assert_eq!(fib_iter(4, 0, 1), 5); // F(4) = 5
        assert_eq!(fib_iter(5, 0, 1), 8); // F(5) = 8
    }

    #[test]
    fn test_fib_iter_custom_init() {
        // Test with custom initial values
        assert_eq!(fib_iter(0, 2, 3), 3); // Sequence: 2, 3 -> 3
        assert_eq!(fib_iter(1, 2, 3), 3); // Sequence: 2, 3, 5 -> 5
        assert_eq!(fib_iter(2, 2, 3), 5); // Sequence: 2, 3, 5, 8 -> 8
        assert_eq!(fib_iter(3, 2, 3), 8); // Sequence: 2, 3, 5, 8, 13 -> 13
    }

    #[test]
    fn test_fib_iter_edge_cases() {
        // Test edge cases
        assert_eq!(fib_iter(0, 10, 20), 20); // Single value case
        assert_eq!(fib_iter(1, 10, 20), 20); // Two values case
        assert_eq!(fib_iter(2, 10, 20), 30); // First computed value
    }

    #[test]
    fn test_fib_iter_larger_values() {
        // Test with larger values to ensure algorithm works correctly
        assert_eq!(fib_iter(10, 0, 1), 144); // F(10) = 144
        assert_eq!(fib_iter(15, 0, 1), 987); // F(15) = 987
    }
}
