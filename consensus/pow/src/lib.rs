// public for benchmarks
#[doc(hidden)]
pub mod matrix;
#[cfg(feature = "wasm32-sdk")]
pub mod wasm;
#[doc(hidden)]
pub mod xoshiro;

use std::cmp::max;

use crate::matrix::Matrix;
use kaspa_consensus_core::{hashing, header::Header, BlockLevel};
//use kaspa_hashes::Pow;
use kaspa_hashes::PowB3Hash;
use kaspa_math::Uint256;

/// State is an intermediate data structure with pre-computed values to speed up mining.
pub struct State {
    pub(crate) matrix: Matrix,
    pub(crate) target: Uint256,
    // PRE_POW_HASH || TIME || 32 zero byte padding; without NONCE
    pub(crate) hasher: PowB3Hash,
}

impl State {
    #[inline]
    pub fn new(header: &Header) -> Self {
        let target = Uint256::from_compact_target_bits(header.bits);
        // Zero out the time and nonce.
        let pre_pow_hash = hashing::header::hash_override_nonce_time(header, 0, 0);
        // PRE_POW_HASH || TIME || 32 zero byte padding || NONCE
        //let hasher = PowHash::new(pre_pow_hash, header.timestamp);
        let hasher = PowB3Hash::new(pre_pow_hash, header.timestamp);
        let matrix = Matrix::generate(pre_pow_hash);

        Self { matrix, target, hasher }
    }

    #[inline]
    #[must_use]
    /// PRE_POW_HASH || TIME || 32 zero byte padding || NONCE
    pub fn calculate_pow(&self, nonce: u64) -> Uint256 {
        // Hasher already contains PRE_POW_HASH || TIME || 32 zero byte padding; so only the NONCE is missing
        //println!("nonce:{0}", nonce);
        let hash = self.hasher.clone().finalize_with_nonce(nonce);
        //println!("hash1:{0}", hash);
        let hash = self.matrix.heavy_hash(hash);
        //println!("hash2:{0}", hash);
        Uint256::from_le_bytes(hash.as_bytes())
    }

    #[inline]
    #[must_use]
    pub fn check_pow(&self, nonce: u64) -> (bool, Uint256) {
        let pow = self.calculate_pow(nonce);
        // The pow hash must be less or equal than the claimed target.
        (pow <= self.target, pow)
        //println!("pow:{0}, \n target:{1}, \n diff:{2}", pow, self.target, (pow-self.target));
        //(true, pow)
    }
}

pub fn calc_block_level(header: &Header, max_block_level: BlockLevel) -> BlockLevel {
    if header.parents_by_level.is_empty() {
        return max_block_level; // Genesis has the max block level
    }

    let state = State::new(header);
    let (_, pow) = state.check_pow(header.nonce);
    let signed_block_level = max_block_level as i64 - pow.bits() as i64;
    max(signed_block_level, 0) as BlockLevel
}
