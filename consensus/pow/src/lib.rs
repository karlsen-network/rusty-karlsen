// public for benchmarks
#[doc(hidden)]
pub mod matrix;
#[cfg(feature = "wasm32-sdk")]
pub mod wasm;
#[doc(hidden)]
pub mod xoshiro;

use std::cmp::max;
use std::sync::Arc;

use crate::matrix::Matrix;
use karlsen_consensus_core::{constants, hashing, header::Header, BlockLevel};
use karlsen_hashes::{pow_hashers::FishHashContext, PowB3Hash, PowFishHash};
use karlsen_math::Uint256;

/// State is an intermediate data structure with pre-computed values to speed up mining.
pub struct State {
    pub(crate) matrix: Matrix,
    pub(crate) target: Uint256,
    // PRE_POW_HASH || TIME || 32 zero byte padding; without NONCE
    pub(crate) hasher: PowB3Hash,
    pub(crate) header_version: u16,
    pub(crate) fish_context: Arc<FishHashContext>,
}

impl State {
    #[inline]
    pub fn new(header: &Header, fish_context: Arc<FishHashContext>) -> Self {
        let target = Uint256::from_compact_target_bits(header.bits);
        // Zero out the time and nonce.
        let pre_pow_hash = hashing::header::hash_override_nonce_time(header, 0, 0);
        // PRE_POW_HASH || TIME || 32 zero byte padding || NONCE
        //let hasher = PowHash::new(pre_pow_hash, header.timestamp);
        let hasher = PowB3Hash::new(pre_pow_hash, header.timestamp);
        let matrix = Matrix::generate(pre_pow_hash);
        //let fishhasher = PowFishHash::new();
        let header_version = header.version;

        Self { matrix, target, hasher, fish_context, header_version }
    }

    #[inline]
    fn calculate_pow_khashv1(&self, nonce: u64) -> Uint256 {
        // Hasher already contains PRE_POW_HASH || TIME || 32 zero byte padding; so only the NONCE is missing
        let hash = self.hasher.clone().finalize_with_nonce(nonce);
        let hash = self.matrix.heavy_hash(hash);
        Uint256::from_le_bytes(hash.as_bytes())
    }

    #[inline]
    fn calculate_pow_khashv2plus(&self, nonce: u64) -> Uint256 {
        // Hasher already contains PRE_POW_HASH || TIME || 32 zero byte padding; so only the NONCE is missing
        let hash = self.hasher.clone().finalize_with_nonce(nonce);
        //println!("hash-1 : {:?}", hash);
        let hash = PowFishHash::fishhashplus_kernel(&hash, &self.fish_context);
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
        match self.header_version {
            constants::BLOCK_VERSION_KHASHV1 => self.calculate_pow_khashv1(nonce),
            constants::BLOCK_VERSION_KHASHV2 => self.calculate_pow_khashv2plus(nonce),
            _ => unreachable!("wrong block version: {}", self.header_version), // should never happen because this is checked in pre_ghostdag_validation
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

pub fn calc_block_level(header: &Header, max_block_level: BlockLevel, fish_context: Arc<FishHashContext>) -> BlockLevel {
    let (block_level, _) = calc_block_level_check_pow(header, max_block_level, fish_context);
    block_level
}

pub fn calc_block_level_check_pow(
    header: &Header,
    max_block_level: BlockLevel,
    fish_context: Arc<FishHashContext>,
) -> (BlockLevel, bool) {
    if header.parents_by_level.is_empty() {
        return (max_block_level, true); // Genesis has the max block level
    }

    let state = State::new(header, fish_context);
    let (passed, pow) = state.check_pow(header.nonce);
    let block_level = calc_level_from_pow(pow, max_block_level);
    (block_level, passed)
}

pub fn calc_level_from_pow(pow: Uint256, max_block_level: BlockLevel) -> BlockLevel {
    let signed_block_level = max_block_level as i64 - pow.bits() as i64;
    max(signed_block_level, 0) as BlockLevel
}
