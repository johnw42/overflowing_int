use compact_bigint::CowBigInt;
use num_traits::Zero;

fn main() {
    let mut primes = vec![CowBigInt::from(2)];
    let mut i = CowBigInt::from(1);
    let mut i_squared = CowBigInt::from(1);
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
