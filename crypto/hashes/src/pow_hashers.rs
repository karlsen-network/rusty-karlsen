use crate::Hash;

#[derive(Clone)]
pub struct PowB3Hash{
    pub hasher: blake3::Hasher,
}

#[derive(Clone)]
pub struct PowHash([u64; 25]);

#[derive(Clone)]
pub struct KHeavyHash;


impl PowB3Hash {

    #[inline]
    pub fn new(pre_pow_hash: Hash, timestamp: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&pre_pow_hash.as_bytes());
        hasher.update(&timestamp.to_le_bytes());
        let array: [u8; 32] = [0; 32];
        hasher.update(&array);
        Self { hasher }
    }

    

    

    #[inline(always)]
    pub fn finalize_with_nonce(mut self, nonce: u64) -> Hash {
        
        self.hasher.update(&nonce.to_le_bytes());
        let hash = self.hasher.finalize();
        Hash(*hash.as_bytes())

        //Hash::from_le_u64(self.0[..4].try_into().unwrap())
    }

}

impl PowHash {
    // The initial state of `cSHAKE256("ProofOfWorkHash")`
    // [10] -> 1123092876221303310 ^ 0x04(padding byte) = 1123092876221303306
    // [16] -> 10306167911662716186 ^ 0x8000000000000000(final padding) = 1082795874807940378
    #[rustfmt::skip]
    const INITIAL_STATE: [u64; 25] = [
        1242148031264380989, 3008272977830772284, 2188519011337848018, 1992179434288343456, 8876506674959887717,
        5399642050693751366, 1745875063082670864, 8605242046444978844, 17936695144567157056, 3343109343542796272,
        1123092876221303306, 4963925045340115282, 17037383077651887893, 16629644495023626889, 12833675776649114147,
        3784524041015224902, 1082795874807940378, 13952716920571277634, 13411128033953605860, 15060696040649351053,
        9928834659948351306, 5237849264682708699, 12825353012139217522, 6706187291358897596, 196324915476054915,
    ];
    #[inline]
    pub fn new(pre_pow_hash: Hash, timestamp: u64) -> Self {
        let mut start = Self::INITIAL_STATE;
        for (pre_pow_word, state_word) in pre_pow_hash.iter_le_u64().zip(start.iter_mut()) {
            *state_word ^= pre_pow_word;
        }
        start[4] ^= timestamp;
        Self(start)
    }

    

    

    #[inline(always)]
    pub fn finalize_with_nonce(mut self, nonce: u64) -> Hash {
        
        self.0[9] ^= nonce;


        /*
        let pre_pow_hash = Hash([42; 32]);
        println!("pre_pow_hash : {:?}", pre_pow_hash);
        let mut new_hash = b3::convert_u8_to_u64_array(pre_pow_hash.0);
        println!("new_hash : {:?}", new_hash);
        println!("Hash(new_hash) : {:?}", Hash::from_le_u64(new_hash[..4].try_into().unwrap()));
        //keccak256::f1600(&mut new_hash);
        b3::blake3_hash(&mut new_hash);
        println!("Hash(new_hash) : {:?}", Hash::from_le_u64(new_hash[..4].try_into().unwrap()));
        */

        

        //keccak256::f1600(&mut self.0);
        //b3::blake3_hash_u64_array(&mut self.0);
        //println!("self : {:?}", self.0);
        b3::blake3_hash(&mut self.0);

        Hash::from_le_u64(self.0[..4].try_into().unwrap())
    }

    #[inline(always)]
    pub fn test_hash(state: &mut [u64; 25]) {
        //b3::blake3_hash(state);
        keccak256::f1600(state);
    }

}

impl KHeavyHash {
    // The initial state of `cSHAKE256("HeavyHash")`
    // [4] -> 16654558671554924254 ^ 0x04(padding byte) = 16654558671554924250
    // [16] -> 9793466274154320918 ^ 0x8000000000000000(final padding) = 570094237299545110
    #[rustfmt::skip]
    const INITIAL_STATE: [u64; 25] = [
        4239941492252378377, 8746723911537738262, 8796936657246353646, 1272090201925444760, 16654558671554924250,
        8270816933120786537, 13907396207649043898, 6782861118970774626, 9239690602118867528, 11582319943599406348,
        17596056728278508070, 15212962468105129023, 7812475424661425213, 3370482334374859748, 5690099369266491460,
        8596393687355028144, 570094237299545110, 9119540418498120711, 16901969272480492857, 13372017233735502424,
        14372891883993151831, 5171152063242093102, 10573107899694386186, 6096431547456407061, 1592359455985097269,
    ];
    #[inline]
    pub fn hash(in_hash: Hash) -> Hash {
        let mut state = Self::INITIAL_STATE;
        for (pre_pow_word, state_word) in in_hash.iter_le_u64().zip(state.iter_mut()) {
            *state_word ^= pre_pow_word;
        }
        keccak256::f1600(&mut state);
        //b3::blake3_hash_u64_array(&mut state);
        Hash::from_le_u64(state[..4].try_into().unwrap())
    }
}

mod b3 {

    pub fn convert_u8_to_u64_array(bytes: [u8; 32]) -> [u64; 25] {
        let mut u64_array = [0u64; 25];
    
        // Fill the u64_array using data from bytes
        for (i, chunk) in bytes.chunks_exact(8).enumerate() {
            let mut u64_val = 0u64;
    
            for (j, &b) in chunk.iter().enumerate() {
                u64_val |= (b as u64) << (j * 8);
            }
    
            u64_array[i] = u64_val;
        }
    
        // The remaining elements in the u64_array are already zero-filled
        u64_array
    }

    /*
    pub fn blake3_hash(state: &mut [u64; 25]) {
        // Create a new Blake3 hasher
        let mut hasher = blake3::Hasher::new();

        // Update the hasher with the bytes of the array
        for &num in state {
            hasher.update(&num.to_le_bytes());
        }

        // Finalize the hash to obtain a 32-byte array
        let result = hasher.finalize();

    }
    */

    pub fn blake3_hash(state: &mut [u64; 25]) {
        // Interpret the u64 state array as bytes for hashing
        let state_as_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                state.as_ptr() as *const u8,
                state.len() * std::mem::size_of::<u64>(),
            )
        };
    
        // Compute the BLAKE3 hash of the state bytes
        let hash = blake3::hash(state_as_bytes);
    
        // Convert the hash output (32 bytes) into u64 and store back into the first 4 elements
        let hash_bytes = hash.as_bytes();
        for i in 0..4 {
            state[i] = u64::from_le_bytes([
                hash_bytes[i * 8 + 0],
                hash_bytes[i * 8 + 1],
                hash_bytes[i * 8 + 2],
                hash_bytes[i * 8 + 3],
                hash_bytes[i * 8 + 4],
                hash_bytes[i * 8 + 5],
                hash_bytes[i * 8 + 6],
                hash_bytes[i * 8 + 7],
            ]);
        }
    
        // Optional: Zero out remaining parts of the state, or maintain their original values
        for i in 4..25 {
            state[i] = 0;
        }
    }

    pub(super) fn blake3_hash_u64_array(input: &mut [u64; 25]) -> &mut [u64; 25] {
        // Convert the `[u64; 25]` array to a byte slice
        let byte_slice: &[u8] = unsafe {
            std::slice::from_raw_parts(
                input.as_ptr() as *const u8,
                std::mem::size_of::<[u64; 25]>()
            )
        };
    
        // Compute the Blake3 hash of the input byte slice
        let hash = blake3::hash(byte_slice);
    
        // Copy the hash output back into the original `[u64; 25]` array
        // We will only copy as much data as can fit into the array
        let mut hash_bytes = [0u8; std::mem::size_of::<[u64; 25]>()];
        let hash_slice = hash.as_bytes();
        let copy_length = std::cmp::min(hash_bytes.len(), hash_slice.len());
        hash_bytes[..copy_length].copy_from_slice(&hash_slice[..copy_length]);
    
        // Now convert the bytes back to u64 chunks and modify the original array
        let new_u64_slice: &[u64] = unsafe {
            std::slice::from_raw_parts(
                hash_bytes.as_ptr() as *const u64,
                hash_bytes.len() / std::mem::size_of::<u64>()
            )
        };
    
        for (target, &src) in input.iter_mut().zip(new_u64_slice.iter()) {
            *target = src;
        }
    
        // Return the modified input array
        input
    }

}

mod keccak256 {
    #[cfg(any(not(target_arch = "x86_64"), feature = "no-asm", target_os = "windows"))]
    #[inline(always)]
    pub(super) fn f1600(state: &mut [u64; 25]) {
        keccak::f1600(state);
    }

    #[cfg(all(target_arch = "x86_64", not(feature = "no-asm"), not(target_os = "windows")))]
    #[inline(always)]
    pub(super) fn f1600(state: &mut [u64; 25]) {
        extern "C" {
            fn KeccakF1600(state: &mut [u64; 25]);
        }
        unsafe { KeccakF1600(state) }
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::null;

    use super::{KHeavyHash, PowHash};
    use crate::Hash;
    use sha3::digest::{ExtendableOutput, Update, XofReader};
    use sha3::{CShake256, CShake256Core};

    const PROOF_OF_WORK_DOMAIN: &[u8] = b"ProofOfWorkHash";
    const HEAVY_HASH_DOMAIN: &[u8] = b"HeavyHash";

    #[test]
    fn test_pow_hash() {

        let pre_pow_hash = Hash([42; 32]);

        let timestamp: u64 = 5435345234;
        let nonce: u64 = 432432432;
        let pre_pow_hash = Hash([42; 32]);

        /*
        let initial_bytes = [0xC1, 0xEC, 0xFD, 0xFC]; // Define the starting byte values
        let mut full_array = [0; 32]; // Create an array initialized to zeros
        full_array[..initial_bytes.len()].copy_from_slice(&initial_bytes);
        let pre_pow_hash2 = Hash(full_array);
        */




        let hasher = PowHash::new(pre_pow_hash, timestamp);
        let hash1 = hasher.finalize_with_nonce(nonce);

        let hasher = CShake256::from_core(CShake256Core::new(PROOF_OF_WORK_DOMAIN))
            .chain(pre_pow_hash.0)
            .chain(timestamp.to_le_bytes())
            .chain([0u8; 32])
            .chain(nonce.to_le_bytes());
        let mut hash2 = [0u8; 32];
        hasher.finalize_xof().read(&mut hash2);
        //println!("init : {:?}", initial_bytes);
        //println!("full : {:?}", full_array);
        println!("hash1 : {:?}", hash1);
        println!("Hash(hash2) : {:?}", Hash(hash2));
        //println!("pow : {:?}", PowHash::test_hash(full_array));
        assert_eq!(Hash(hash2), hash1);
        
    }

    #[test]
    fn test_heavy_hash() {
        let val = Hash([42; 32]);
        let hash1 = KHeavyHash::hash(val);

        let hasher = CShake256::from_core(CShake256Core::new(HEAVY_HASH_DOMAIN)).chain(val.0);
        let mut hash2 = [0u8; 32];
        hasher.finalize_xof().read(&mut hash2);
        assert_eq!(Hash(hash2), hash1);
    }
}
