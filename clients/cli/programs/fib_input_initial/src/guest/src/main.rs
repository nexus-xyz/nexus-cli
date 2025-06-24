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

    for _i in 0..n + 1 {
        let c = a + b;
        a = b;
        b = c;
    }
    b
}
