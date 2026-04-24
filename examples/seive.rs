use num_traits::Zero;
use overflowing_int::ArcInt64 as BigInt;

fn main() {
    let mut primes = vec![BigInt::from(2)];
    let mut i = BigInt::from(1);
    let mut i_squared = BigInt::from(1);
    loop {
        i += 2;
        i_squared += 2 * i.clone() + 1; // Update i_squared to (i+1)^2
        if !primes
            .iter()
            .take_while(|&p| p <= &i_squared)
            .any(|p| (&i % p).is_zero())
        {
            if primes.len().is_multiple_of(1000) {
                println!("{} {}", i.bits(), i);
            }
            primes.push(i.clone());
        }
    }
}
