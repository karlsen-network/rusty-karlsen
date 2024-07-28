// public for benchmarks
#[doc(hidden)]
pub mod matrix;
#[cfg(feature = "wasm32-sdk")]
pub mod wasm;
#[doc(hidden)]
pub mod xoshiro;

use std::cmp::max;

use crate::matrix::Matrix;
use karlsen_consensus_core::{constants, hashing, header::Header, BlockLevel};
//use karlsen_consensus_core::errors::block::RuleError;
//use karlsen_hashes::Pow;
use karlsen_hashes::{PowB3Hash, PowFishHash};
use karlsen_math::Uint256;

/// State is an intermediate data structure with pre-computed values to speed up mining.
pub struct State {
    pub(crate) matrix: Matrix,
    pub(crate) target: Uint256,
    // PRE_POW_HASH || TIME || 32 zero byte padding; without NONCE
    pub(crate) hasher: PowB3Hash,
    //pub(crate) fishhasher: PowFishHash,
    pub(crate) header_version: u16,
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
        //let fishhasher = PowFishHash::new();
        let header_version = header.version;

        Self {
            matrix,
            target,
            hasher,
            /*fishhasher,*/ header_version,
        }
    }

    fn calculate_pow_khashv1(&self, nonce: u64) -> Uint256 {
        // Hasher already contains PRE_POW_HASH || TIME || 32 zero byte padding; so only the NONCE is missing
        let hash = self.hasher.clone().finalize_with_nonce(nonce);
        let hash = self.matrix.heavy_hash(hash);
        Uint256::from_le_bytes(hash.as_bytes())
    }

    #[allow(dead_code)]
    fn calculate_pow_khashv2(&self, nonce: u64) -> Uint256 {
        // Hasher already contains PRE_POW_HASH || TIME || 32 zero byte padding; so only the NONCE is missing
        let hash = self.hasher.clone().finalize_with_nonce(nonce);
        //println!("hash-1 : {:?}", hash);
        let hash = PowFishHash::fishhash_kernel(&hash);
        //println!("hash-2 : {:?}", hash);
        //last b3 hash
        let hash = PowB3Hash::hash(hash);
        //println!("hash-3 : {:?}", hash);
        Uint256::from_le_bytes(hash.as_bytes())
    }

    fn calculate_pow_khashv2plus(&self, nonce: u64) -> Uint256 {
        // Hasher already contains PRE_POW_HASH || TIME || 32 zero byte padding; so only the NONCE is missing
        let hash = self.hasher.clone().finalize_with_nonce(nonce);
        //println!("hash-1 : {:?}", hash);
        let hash = PowFishHash::fishhashplus_kernel(&hash);
        //println!("hash-2 : {:?}", hash);
        //last b3 hash
        let hash = PowB3Hash::hash(hash);
        //println!("hash-3 : {:?}", hash);
        Uint256::from_le_bytes(hash.as_bytes())
    }

    #[inline]
    #[must_use]
    /// PRE_POW_HASH || TIME || 32 zero byte padding || NONCE
    pub fn calculate_pow(&self, nonce: u64) -> Uint256 {
        if self.header_version == constants::BLOCK_VERSION_KHASHV1 {
            self.calculate_pow_khashv1(nonce)
        } else if self.header_version == constants::BLOCK_VERSION_KHASHV2 {
            self.calculate_pow_khashv2plus(nonce)
        } else {
            // TODO handle block version error
            //Err(RuleError::WrongBlockVersion(self.header_version));
            self.calculate_pow_khashv1(nonce)
        }
    }

    #[inline]
    #[must_use]
    pub fn check_pow(&self, nonce: u64) -> (bool, Uint256) {
        let pow = self.calculate_pow(nonce);
        // The pow hash must be less or equal than the claimed target.
        (pow <= self.target, pow)
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
