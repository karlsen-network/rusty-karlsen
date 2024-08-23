use karlsen_consensus_core::{
    coinbase::*,
    errors::coinbase::{CoinbaseError, CoinbaseResult},
    subnets,
    tx::{ScriptPublicKey, ScriptVec, Transaction, TransactionOutput},
    BlockHashMap, BlockHashSet,
};
use std::{convert::TryInto, mem::size_of};

use crate::{constants, model::stores::ghostdag::GhostdagData};

const LENGTH_OF_BLUE_SCORE: usize = size_of::<u64>();
const LENGTH_OF_SUBSIDY: usize = size_of::<u64>();
const LENGTH_OF_SCRIPT_PUB_KEY_VERSION: usize = size_of::<u16>();
const LENGTH_OF_SCRIPT_PUB_KEY_LENGTH: usize = size_of::<u8>();

const MIN_PAYLOAD_LENGTH: usize = LENGTH_OF_BLUE_SCORE
    + LENGTH_OF_SUBSIDY
    + LENGTH_OF_SCRIPT_PUB_KEY_VERSION
    + LENGTH_OF_SCRIPT_PUB_KEY_LENGTH;

// We define a year as 365.25 days and a month as 365.25 / 12 = 30.4375
// SECONDS_PER_MONTH = 30.4375 * 24 * 60 * 60
const SECONDS_PER_MONTH: u64 = 2629800;

pub const SUBSIDY_BY_MONTH_TABLE_SIZE: usize = 793;
pub type SubsidyByMonthTable = [u64; SUBSIDY_BY_MONTH_TABLE_SIZE];

#[derive(Clone)]
pub struct CoinbaseManager {
    coinbase_payload_script_public_key_max_len: u8,
    max_coinbase_payload_len: usize,
    deflationary_phase_daa_score: u64,
    pre_deflationary_phase_base_subsidy: u64,
    target_time_per_block: u64,

    /// Precomputed number of blocks per month
    blocks_per_month: u64,

    /// Precomputed subsidy by month table
    subsidy_by_month_table: SubsidyByMonthTable,
}

/// Struct used to streamline payload parsing
struct PayloadParser<'a> {
    remaining: &'a [u8], // The unparsed remainder
}

impl<'a> PayloadParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { remaining: data }
    }

    /// Returns a slice with the first `n` bytes of `remaining`, while setting `remaining` to the remaining part
    fn take(&mut self, n: usize) -> &[u8] {
        let (segment, remaining) = self.remaining.split_at(n);
        self.remaining = remaining;
        segment
    }
}

impl CoinbaseManager {
    pub fn new(
        coinbase_payload_script_public_key_max_len: u8,
        max_coinbase_payload_len: usize,
        deflationary_phase_daa_score: u64,
        pre_deflationary_phase_base_subsidy: u64,
        target_time_per_block: u64,
    ) -> Self {
        assert!(1000 % target_time_per_block == 0);
        let bps = 1000 / target_time_per_block;
        let blocks_per_month = SECONDS_PER_MONTH * bps;

        // Precomputed subsidy by month table for the actual block per second rate
        // Here values are rounded up so that we keep the same number of rewarding months as in the original 1 BPS table.
        // In a 10 BPS network, the induced increase in total rewards is 51 KLS (see tests::calc_high_bps_total_rewards_delta())
        let subsidy_by_month_table: SubsidyByMonthTable =
            core::array::from_fn(|i| (SUBSIDY_BY_MONTH_TABLE[i] + bps - 1) / bps);
        Self {
            coinbase_payload_script_public_key_max_len,
            max_coinbase_payload_len,
            deflationary_phase_daa_score,
            pre_deflationary_phase_base_subsidy,
            target_time_per_block,
            blocks_per_month,
            subsidy_by_month_table,
        }
    }

    #[cfg(test)]
    #[inline]
    pub fn bps(&self) -> u64 {
        1000 / self.target_time_per_block
    }

    pub fn expected_coinbase_transaction<T: AsRef<[u8]>>(
        &self,
        daa_score: u64,
        miner_data: MinerData<T>,
        ghostdag_data: &GhostdagData,
        mergeset_rewards: &BlockHashMap<BlockRewardData>,
        mergeset_non_daa: &BlockHashSet,
    ) -> CoinbaseResult<CoinbaseTransactionTemplate> {
        let mut outputs = Vec::with_capacity(ghostdag_data.mergeset_blues.len() + 1); // + 1 for possible red reward

        // Add an output for each mergeset blue block (∩ DAA window), paying to the script reported by the block.
        // Note that combinatorically it is nearly impossible for a blue block to be non-DAA
        for blue in ghostdag_data
            .mergeset_blues
            .iter()
            .filter(|h| !mergeset_non_daa.contains(h))
        {
            let reward_data = mergeset_rewards.get(blue).unwrap();
            if reward_data.subsidy + reward_data.total_fees > 0 {
                outputs.push(TransactionOutput::new(
                    reward_data.subsidy + reward_data.total_fees,
                    reward_data.script_public_key.clone(),
                ));
            }
        }

        // Collect all rewards from mergeset reds ∩ DAA window and create a
        // single output rewarding all to the current block (the "merging" block)
        let mut red_reward = 0u64;
        for red in ghostdag_data
            .mergeset_reds
            .iter()
            .filter(|h| !mergeset_non_daa.contains(h))
        {
            let reward_data = mergeset_rewards.get(red).unwrap();
            red_reward += reward_data.subsidy + reward_data.total_fees;
        }
        if red_reward > 0 {
            outputs.push(TransactionOutput::new(
                red_reward,
                miner_data.script_public_key.clone(),
            ));
        }

        // Build the current block's payload
        let subsidy = self.calc_block_subsidy(daa_score);
        let payload = self.serialize_coinbase_payload(&CoinbaseData {
            blue_score: ghostdag_data.blue_score,
            subsidy,
            miner_data,
        })?;

        Ok(CoinbaseTransactionTemplate {
            tx: Transaction::new(
                constants::TX_VERSION,
                vec![],
                outputs,
                0,
                subnets::SUBNETWORK_ID_COINBASE,
                0,
                payload,
            ),
            has_red_reward: red_reward > 0,
        })
    }

    pub fn serialize_coinbase_payload<T: AsRef<[u8]>>(
        &self,
        data: &CoinbaseData<T>,
    ) -> CoinbaseResult<Vec<u8>> {
        let script_pub_key_len = data.miner_data.script_public_key.script().len();
        if script_pub_key_len > self.coinbase_payload_script_public_key_max_len as usize {
            return Err(CoinbaseError::PayloadScriptPublicKeyLenAboveMax(
                script_pub_key_len,
                self.coinbase_payload_script_public_key_max_len,
            ));
        }
        let payload: Vec<u8> = data
            .blue_score
            .to_le_bytes()
            .iter()
            .copied() // Blue score                   (u64)
            .chain(data.subsidy.to_le_bytes().iter().copied()) // Subsidy                      (u64)
            .chain(
                data.miner_data
                    .script_public_key
                    .version()
                    .to_le_bytes()
                    .iter()
                    .copied(),
            ) // Script public key version    (u16)
            .chain((script_pub_key_len as u8).to_le_bytes().iter().copied()) // Script public key length     (u8)
            .chain(data.miner_data.script_public_key.script().iter().copied()) // Script public key
            .chain(data.miner_data.extra_data.as_ref().iter().copied()) // Extra data
            .collect();

        Ok(payload)
    }

    pub fn modify_coinbase_payload<T: AsRef<[u8]>>(
        &self,
        mut payload: Vec<u8>,
        miner_data: &MinerData<T>,
    ) -> CoinbaseResult<Vec<u8>> {
        let script_pub_key_len = miner_data.script_public_key.script().len();
        if script_pub_key_len > self.coinbase_payload_script_public_key_max_len as usize {
            return Err(CoinbaseError::PayloadScriptPublicKeyLenAboveMax(
                script_pub_key_len,
                self.coinbase_payload_script_public_key_max_len,
            ));
        }

        // Keep only blue score and subsidy. Note that truncate does not modify capacity, so
        // the usual case where the payloads are the same size will not trigger a reallocation
        payload.truncate(LENGTH_OF_BLUE_SCORE + LENGTH_OF_SUBSIDY);
        payload.extend(
            miner_data
                .script_public_key
                .version()
                .to_le_bytes()
                .iter()
                .copied() // Script public key version (u16)
                .chain((script_pub_key_len as u8).to_le_bytes().iter().copied()) // Script public key length  (u8)
                .chain(miner_data.script_public_key.script().iter().copied()) // Script public key
                .chain(miner_data.extra_data.as_ref().iter().copied()), // Extra data
        );

        Ok(payload)
    }

    pub fn deserialize_coinbase_payload<'a>(
        &self,
        payload: &'a [u8],
    ) -> CoinbaseResult<CoinbaseData<&'a [u8]>> {
        if payload.len() < MIN_PAYLOAD_LENGTH {
            return Err(CoinbaseError::PayloadLenBelowMin(
                payload.len(),
                MIN_PAYLOAD_LENGTH,
            ));
        }

        if payload.len() > self.max_coinbase_payload_len {
            return Err(CoinbaseError::PayloadLenAboveMax(
                payload.len(),
                self.max_coinbase_payload_len,
            ));
        }

        let mut parser = PayloadParser::new(payload);

        let blue_score = u64::from_le_bytes(parser.take(LENGTH_OF_BLUE_SCORE).try_into().unwrap());
        let subsidy = u64::from_le_bytes(parser.take(LENGTH_OF_SUBSIDY).try_into().unwrap());
        let script_pub_key_version = u16::from_le_bytes(
            parser
                .take(LENGTH_OF_SCRIPT_PUB_KEY_VERSION)
                .try_into()
                .unwrap(),
        );
        let script_pub_key_len = u8::from_le_bytes(
            parser
                .take(LENGTH_OF_SCRIPT_PUB_KEY_LENGTH)
                .try_into()
                .unwrap(),
        );

        if script_pub_key_len > self.coinbase_payload_script_public_key_max_len {
            return Err(CoinbaseError::PayloadScriptPublicKeyLenAboveMax(
                script_pub_key_len as usize,
                self.coinbase_payload_script_public_key_max_len,
            ));
        }

        if parser.remaining.len() < script_pub_key_len as usize {
            return Err(CoinbaseError::PayloadCantContainScriptPublicKey(
                payload.len(),
                MIN_PAYLOAD_LENGTH + script_pub_key_len as usize,
            ));
        }

        let script_public_key = ScriptPublicKey::new(
            script_pub_key_version,
            ScriptVec::from_slice(parser.take(script_pub_key_len as usize)),
        );
        let extra_data = parser.remaining;

        Ok(CoinbaseData {
            blue_score,
            subsidy,
            miner_data: MinerData {
                script_public_key,
                extra_data,
            },
        })
    }

    pub fn calc_block_subsidy(&self, daa_score: u64) -> u64 {
        if daa_score < self.deflationary_phase_daa_score {
            return self.pre_deflationary_phase_base_subsidy;
        }

        let months_since_deflationary_phase_started =
            ((daa_score - self.deflationary_phase_daa_score) / self.blocks_per_month) as usize;
        if months_since_deflationary_phase_started >= self.subsidy_by_month_table.len() {
            *(self.subsidy_by_month_table).last().unwrap()
        } else {
            self.subsidy_by_month_table[months_since_deflationary_phase_started]
        }
    }

    #[cfg(test)]
    pub fn legacy_calc_block_subsidy(&self, daa_score: u64) -> u64 {
        if daa_score < self.deflationary_phase_daa_score {
            return self.pre_deflationary_phase_base_subsidy;
        }

        // Note that this calculation implicitly assumes that block per second = 1 (by assuming daa score diff is in second units).
        let months_since_deflationary_phase_started =
            (daa_score - self.deflationary_phase_daa_score) / SECONDS_PER_MONTH;
        assert!(months_since_deflationary_phase_started <= usize::MAX as u64);
        let months_since_deflationary_phase_started: usize =
            months_since_deflationary_phase_started as usize;
        if months_since_deflationary_phase_started >= SUBSIDY_BY_MONTH_TABLE.len() {
            *SUBSIDY_BY_MONTH_TABLE.last().unwrap()
        } else {
            SUBSIDY_BY_MONTH_TABLE[months_since_deflationary_phase_started]
        }
    }
}

/*
    This table was pre-calculated by calling `calcDeflationaryPeriodBlockSubsidyFloatCalc` (in karlsend-go) for all months until reaching 0 subsidy.
    To regenerate this table, run `TestBuildSubsidyTable` in coinbasemanager_test.go (note the `deflationaryPhaseBaseSubsidy` therein).
    These values apply to 1 block per second.
*/
#[rustfmt::skip]
const SUBSIDY_BY_MONTH_TABLE: [u64; 793] = [
	4400000000, 4278340444, 4160044764, 4045019946, 3933175554, 3824423648, 3718678720, 3615857630, 3515879532, 3418665818, 3324140054, 3232227917, 3142857142, 3055957460, 2971460545, 2889299962, 2809411110, 2731731177, 2656199086, 2582755450, 2511342523, 2441904156, 2374385753, 2308734227, 2244897959,
	2182826757, 2122471818, 2063785687, 2006722221, 1951236555, 1897285061, 1844825321, 1793816087, 1744217254, 1695989823, 1649095876, 1603498542, 1559161969, 1516051298, 1474132633, 1433373015, 1393740396, 1355203615, 1317732372, 1281297205, 1245869467, 1211421302, 1177925626, 1145356101, 1113687121,
	1082893784, 1052951881, 1023837868, 995528854, 968002582, 941237408, 915212289, 889906762, 865300930, 841375447, 818111501, 795490800, 773495560, 752108486, 731312762, 711092039, 691430416, 672312434, 653723064, 635647687, 618072093, 600982462, 584365357, 568207714, 552496829,
	537220347, 522366259, 507922885, 493878868, 480223167, 466945045, 454034062, 441480066, 429273187, 417403827, 405862653, 394640592, 383728819, 373118756, 362802060, 352770620, 343016548, 333532175, 324310044, 315342904, 306623705, 298145590, 289901895, 281886137, 274092014,
	266513397, 259144329, 251979014, 245011820, 238237268, 231650031, 225244931, 219016932, 212961136, 207072782, 201347240, 195780010, 190366712, 185103092, 179985010, 175008443, 170169477, 165464308, 160889237, 156440665, 152115097, 147909130, 143819457, 139842864, 135976223,
	132216494, 128560721, 125006030, 121549626, 118188791, 114920883, 111743332, 108653640, 105649378, 102728184, 99887760, 97125873, 94440353, 91829086, 89290021, 86821161, 84420565, 82086345, 79816666, 77609743, 75463841, 73377274, 71348400, 69375624, 67457395,
	65592204, 63778587, 62015115, 60300403, 58633103, 57011904, 55435531, 53902744, 52412338, 50963142, 49554017, 48183853, 46851574, 45556133, 44296511, 43071717, 41880788, 40722788, 39596807, 38501960, 37437384, 36402244, 35395726, 34417038, 33465410,
	32540095, 31640365, 30765512, 29914848, 29087706, 28283434, 27501400, 26740989, 26001603, 25282661, 24583598, 23903864, 23242925, 22600260, 21975365, 21367749, 20776933, 20202453, 19643857, 19100706, 18572573, 18059044, 17559713, 17074189, 16602089,
	16143043, 15696689, 15262678, 14840666, 14430323, 14031326, 13643361, 13266124, 12899317, 12542652, 12195849, 11858635, 11530745, 11211921, 10901912, 10600476, 10307373, 10022376, 9745258, 9475803, 9213798, 8959037, 8711320, 8470453, 8236246,
	8008515, 7787080, 7571768, 7362409, 7158840, 6960898, 6768430, 6581284, 6399312, 6222372, 6050324, 5883033, 5720368, 5562200, 5408406, 5258864, 5113457, 4972070, 4834593, 4700917, 4570937, 4444551, 4321660, 4202166, 4085977,
	3973000, 3863147, 3756331, 3652469, 3551479, 3453280, 3357798, 3264955, 3174679, 3086900, 3001547, 2918555, 2837857, 2759390, 2683094, 2608906, 2536770, 2466629, 2398427, 2332110, 2267628, 2204928, 2143962, 2084682, 2027040,
	1970993, 1916495, 1863504, 1811979, 1761878, 1713162, 1665793, 1619734, 1574949, 1531401, 1489058, 1447886, 1407852, 1368925, 1331074, 1294270, 1258484, 1223687, 1189852, 1156953, 1124963, 1093858, 1063613, 1034204, 1005608,
	977803, 950767, 924479, 898917, 874062, 849894, 826395, 803545, 781327, 759723, 738717, 718292, 698431, 679119, 660342, 642083, 624330, 607067, 590282, 573961, 558091, 542659, 527655, 513065, 498879,
	485085, 471673, 458631, 445950, 433619, 421630, 409972, 398636, 387614, 376896, 366475, 356342, 346489, 336909, 327593, 318535, 309728, 301164, 292837, 284740, 276867, 269211, 261768, 254530, 247492,
	240649, 233995, 227525, 221234, 215117, 209169, 203385, 197762, 192294, 186977, 181807, 176780, 171892, 167139, 162518, 158024, 153655, 149406, 145275, 141258, 137353, 133555, 129862, 126271, 122780,
	119385, 116084, 112874, 109753, 106719, 103768, 100899, 98109, 95396, 92758, 90194, 87700, 85275, 82917, 80624, 78395, 76227, 74120, 72070, 70078, 68140, 66256, 64424, 62643, 60910,
	59226, 57589, 55996, 54448, 52943, 51479, 50055, 48671, 47325, 46017, 44745, 43507, 42304, 41135, 39997, 38891, 37816, 36770, 35754, 34765, 33804, 32869, 31960, 31077, 30217,
	29382, 28569, 27779, 27011, 26264, 25538, 24832, 24145, 23478, 22829, 22197, 21584, 20987, 20407, 19842, 19294, 18760, 18241, 17737, 17247, 16770, 16306, 15855, 15417, 14990,
	14576, 14173, 13781, 13400, 13029, 12669, 12319, 11978, 11647, 11325, 11012, 10707, 10411, 10123, 9843, 9571, 9307, 9049, 8799, 8556, 8319, 8089, 7865, 7648, 7436,
	7231, 7031, 6836, 6647, 6464, 6285, 6111, 5942, 5778, 5618, 5463, 5312, 5165, 5022, 4883, 4748, 4617, 4489, 4365, 4244, 4127, 4013, 3902, 3794, 3689,
	3587, 3488, 3391, 3298, 3206, 3118, 3031, 2948, 2866, 2787, 2710, 2635, 2562, 2491, 2422, 2355, 2290, 2227, 2165, 2105, 2047, 1990, 1935, 1882, 1830,
	1779, 1730, 1682, 1636, 1590, 1546, 1504, 1462, 1422, 1382, 1344, 1307, 1271, 1236, 1201, 1168, 1136, 1104, 1074, 1044, 1015, 987, 960, 933, 908,
	882, 858, 834, 811, 789, 767, 746, 725, 705, 685, 667, 648, 630, 613, 596, 579, 563, 548, 532, 518, 503, 489, 476, 463, 450,
	438, 425, 414, 402, 391, 380, 370, 359, 349, 340, 330, 321, 312, 304, 295, 287, 279, 271, 264, 257, 249, 243, 236, 229, 223,
	217, 211, 205, 199, 194, 188, 183, 178, 173, 168, 164, 159, 155, 150, 146, 142, 138, 134, 131, 127, 124, 120, 117, 114, 110,
	107, 104, 101, 99, 96, 93, 91, 88, 86, 83, 81, 79, 76, 74, 72, 70, 68, 66, 65, 63, 61, 59, 58, 56, 54,
	53, 52, 50, 49, 47, 46, 45, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 28, 27,
	26, 25, 25, 24, 23, 23, 22, 21, 21, 20, 20, 19, 18, 18, 17, 17, 16, 16, 16, 15, 15, 14, 14, 13, 13,
	13, 12, 12, 12, 11, 11, 11, 10, 10, 10, 9, 9, 9, 9, 8, 8, 8, 8, 7, 7, 7, 7, 7, 6, 6,
	6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3,
	3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1,
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::MAINNET_PARAMS;
    use karlsen_consensus_core::{
        config::params::{Params, TESTNET11_PARAMS},
        constants::SOMPI_PER_KARLSEN,
        network::NetworkId,
        tx::scriptvec,
    };

    #[test]
    fn calc_high_bps_total_rewards_delta() {
        const SECONDS_PER_MONTH: u64 = 2629800;

        let legacy_cbm = create_legacy_manager();
        let pre_deflationary_rewards = legacy_cbm.pre_deflationary_phase_base_subsidy
            * legacy_cbm.deflationary_phase_daa_score;
        let total_rewards: u64 = pre_deflationary_rewards
            + SUBSIDY_BY_MONTH_TABLE
                .iter()
                .map(|x| x * SECONDS_PER_MONTH)
                .sum::<u64>();
        let testnet_11_bps = TESTNET11_PARAMS.bps();
        let total_high_bps_rewards_rounded_up: u64 = pre_deflationary_rewards
            + SUBSIDY_BY_MONTH_TABLE
                .iter()
                .map(|x| {
                    ((x + testnet_11_bps - 1) / testnet_11_bps * testnet_11_bps) * SECONDS_PER_MONTH
                })
                .sum::<u64>();

        let cbm = create_manager(&TESTNET11_PARAMS);
        let total_high_bps_rewards: u64 = pre_deflationary_rewards
            + cbm
                .subsidy_by_month_table
                .iter()
                .map(|x| x * cbm.blocks_per_month)
                .sum::<u64>();
        assert_eq!(
            total_high_bps_rewards_rounded_up, total_high_bps_rewards,
            "subsidy adjusted to bps must be rounded up"
        );

        let delta = total_high_bps_rewards as i64 - total_rewards as i64;

        println!(
            "Total rewards: {} sompi => {} KLS",
            total_rewards,
            total_rewards / SOMPI_PER_KARLSEN
        );
        println!(
            "Total high bps rewards: {} sompi => {} KLS",
            total_high_bps_rewards,
            total_high_bps_rewards / SOMPI_PER_KARLSEN
        );
        println!(
            "Delta: {} sompi => {} KLS",
            delta,
            delta / SOMPI_PER_KARLSEN as i64
        );
    }

    #[test]
    fn subsidy_by_month_table_test() {
        let cbm = create_legacy_manager();
        cbm.subsidy_by_month_table
            .iter()
            .enumerate()
            .for_each(|(i, x)| {
                assert_eq!(
                    SUBSIDY_BY_MONTH_TABLE[i], *x,
                    "for 1 BPS, const table and precomputed values must match"
                );
            });

        for network_id in NetworkId::iter() {
            let cbm = create_manager(&network_id.into());
            cbm.subsidy_by_month_table
                .iter()
                .enumerate()
                .for_each(|(i, x)| {
                    assert_eq!(
                        (SUBSIDY_BY_MONTH_TABLE[i] + cbm.bps() - 1) / cbm.bps(),
                        *x,
                        "{}: locally computed and precomputed values must match",
                        network_id
                    );
                });
        }
    }

    #[test]
    fn subsidy_test() {
        const PRE_DEFLATIONARY_PHASE_BASE_SUBSIDY: u64 = 5000000000;
        const DEFLATIONARY_PHASE_INITIAL_SUBSIDY: u64 = 4400000000;
        const SECONDS_PER_MONTH: u64 = 2629800;
        const SECONDS_PER_HALVING: u64 = SECONDS_PER_MONTH * 12;

        for network_id in NetworkId::iter() {
            let params = &network_id.into();
            let cbm = create_manager(params);

            let pre_deflationary_phase_base_subsidy =
                PRE_DEFLATIONARY_PHASE_BASE_SUBSIDY / params.bps();
            let deflationary_phase_initial_subsidy =
                DEFLATIONARY_PHASE_INITIAL_SUBSIDY / params.bps();
            let blocks_per_halving = SECONDS_PER_HALVING * params.bps();

            struct Test {
                name: &'static str,
                daa_score: u64,
                expected: u64,
            }

            let tests = vec![
                Test {
                    name: "first mined block",
                    daa_score: 1,
                    expected: pre_deflationary_phase_base_subsidy,
                },
                Test {
                    name: "before deflationary phase",
                    daa_score: params.deflationary_phase_daa_score - 1,
                    expected: pre_deflationary_phase_base_subsidy,
                },
                Test {
                    name: "start of deflationary phase",
                    daa_score: params.deflationary_phase_daa_score,
                    expected: deflationary_phase_initial_subsidy,
                },
                Test {
                    name: "after 1 year",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving,
                    expected: (deflationary_phase_initial_subsidy as f64 / 1.4).trunc() as u64
                        + params.bps() / 10,
                },
                Test {
                    name: "after 2 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 2,
                    expected: (deflationary_phase_initial_subsidy as f64 / 1.4_f64.powi(2)).trunc()
                        as u64
                        + params.bps() / 10,
                },
                Test {
                    name: "after 5 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 5,
                    expected: (deflationary_phase_initial_subsidy as f64 / 1.4_f64.powi(5)).trunc()
                        as u64
                        + params.bps() / 10,
                },
                Test {
                    name: "after 32 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 32,
                    expected: (deflationary_phase_initial_subsidy as f64 / 1.4_f64.powi(32)).trunc()
                        as u64
                        + params.bps() / 10,
                },
                Test {
                    name: "after 64 years",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 64,
                    expected: (deflationary_phase_initial_subsidy as f64 / 1.4_f64.powi(64)).trunc()
                        as u64
                        + params.bps() / 10,
                },
                Test {
                    name: "just before subsidy depleted",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 65,
                    expected: 1,
                },
                Test {
                    name: "after subsidy depleted",
                    daa_score: params.deflationary_phase_daa_score + blocks_per_halving * 66,
                    expected: 0,
                },
            ];

            for t in tests {
                assert_eq!(
                    cbm.calc_block_subsidy(t.daa_score),
                    t.expected,
                    "{} test '{}' failed",
                    network_id,
                    t.name
                );
                if params.bps() == 1 {
                    assert_eq!(
                        cbm.legacy_calc_block_subsidy(t.daa_score),
                        t.expected,
                        "{} test '{}' failed",
                        network_id,
                        t.name
                    );
                }
            }
        }
    }

    #[test]
    fn payload_serialization_test() {
        let cbm = create_manager(&MAINNET_PARAMS);

        let script_data = [33u8, 255];
        let extra_data = [2u8, 3];
        let data = CoinbaseData {
            blue_score: 56,
            subsidy: 4400000000,
            miner_data: MinerData {
                script_public_key: ScriptPublicKey::new(0, ScriptVec::from_slice(&script_data)),
                extra_data: &extra_data as &[u8],
            },
        };

        let payload = cbm.serialize_coinbase_payload(&data).unwrap();
        let deserialized_data = cbm.deserialize_coinbase_payload(&payload).unwrap();

        assert_eq!(data, deserialized_data);

        // Test an actual mainnet payload
        let payload_hex =
            "b612c90100000000041a763e07000000000022202b32443ff740012157716d81216d09aebc39e5493c93a7181d92cb756c02c560ac302e31322e382f";
        let mut payload = vec![0u8; payload_hex.len() / 2];
        faster_hex::hex_decode(payload_hex.as_bytes(), &mut payload).unwrap();
        let deserialized_data = cbm.deserialize_coinbase_payload(&payload).unwrap();

        let expected_data = CoinbaseData {
            blue_score: 29954742,
            subsidy: 31112698372,
            miner_data: MinerData {
                script_public_key: ScriptPublicKey::new(
                    0,
                    scriptvec![
                        32, 43, 50, 68, 63, 247, 64, 1, 33, 87, 113, 109, 129, 33, 109, 9, 174,
                        188, 57, 229, 73, 60, 147, 167, 24, 29, 146, 203, 117, 108, 2, 197, 96,
                        172,
                    ],
                ),
                extra_data: &[48u8, 46, 49, 50, 46, 56, 47] as &[u8],
            },
        };
        assert_eq!(expected_data, deserialized_data);
    }

    #[test]
    fn modify_payload_test() {
        let cbm = create_manager(&MAINNET_PARAMS);

        let script_data = [33u8, 255];
        let extra_data = [2u8, 3, 23, 98];
        let data = CoinbaseData {
            blue_score: 56345,
            subsidy: 4400000000,
            miner_data: MinerData {
                script_public_key: ScriptPublicKey::new(0, ScriptVec::from_slice(&script_data)),
                extra_data: &extra_data,
            },
        };

        let data2 = CoinbaseData {
            blue_score: data.blue_score,
            subsidy: data.subsidy,
            miner_data: MinerData {
                // Modify only miner data
                script_public_key: ScriptPublicKey::new(0, ScriptVec::from_slice(&[33u8, 255, 33])),
                extra_data: &[2u8, 3, 23, 98, 34, 34] as &[u8],
            },
        };

        let mut payload = cbm.serialize_coinbase_payload(&data).unwrap();
        payload = cbm
            .modify_coinbase_payload(payload, &data2.miner_data)
            .unwrap(); // Update the payload with the modified miner data
        let deserialized_data = cbm.deserialize_coinbase_payload(&payload).unwrap();

        assert_eq!(data2, deserialized_data);
    }

    fn create_manager(params: &Params) -> CoinbaseManager {
        CoinbaseManager::new(
            params.coinbase_payload_script_public_key_max_len,
            params.max_coinbase_payload_len,
            params.deflationary_phase_daa_score,
            params.pre_deflationary_phase_base_subsidy,
            params.target_time_per_block,
        )
    }

    /// Return a CoinbaseManager with legacy golang 1 BPS properties
    fn create_legacy_manager() -> CoinbaseManager {
        CoinbaseManager::new(150, 204, 15778800 - 259200, 5000000000, 1000)
    }
}
