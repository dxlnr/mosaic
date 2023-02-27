//! PRNG utilities for the crypto primitives.
//!
//! [sodiumoxide]: https://docs.rs/sodiumoxide/
//! [crypto module]: crate::crypto

use num::{bigint::BigUint, traits::identities::Zero};
use rand::RngCore;
use rand_chacha::ChaCha20Rng;

/// Generates a secure pseudo-random integer.
///
/// Draws from a uniform distribution over the integers between zero (included) and
/// `max_int` (excluded). Employs the `ChaCha20` stream cipher as a PRNG.
pub fn generate_integer(prng: &mut ChaCha20Rng, max_int: &BigUint) -> BigUint {
    if max_int.is_zero() {
        return BigUint::zero();
    }
    let mut bytes = max_int.to_bytes_le();
    let mut rand_int = max_int.clone();
    while &rand_int >= max_int {
        prng.fill_bytes(&mut bytes);
        rand_int = BigUint::from_bytes_le(&bytes);
    }
    rand_int
}
