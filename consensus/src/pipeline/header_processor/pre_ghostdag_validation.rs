use super::*;
use crate::constants;
use crate::errors::{BlockProcessResult, RuleError};
use crate::model::services::reachability::ReachabilityService;
use crate::model::stores::statuses::StatusesStoreReader;
use karlsen_consensus_core::blockhash::BlockHashExtensions;
use karlsen_consensus_core::blockstatus::BlockStatus::StatusInvalid;
use karlsen_consensus_core::header::Header;
use karlsen_consensus_core::BlockLevel;
use karlsen_core::time::unix_now;
use karlsen_database::prelude::StoreResultExtensions;
use karlsen_pow::calc_level_from_pow;

impl HeaderProcessor {
    /// Validates the header in isolation including pow check against header declared bits.
    /// Returns the block level as computed from pow state or a rule error if such was encountered
    pub(super) fn validate_header_in_isolation(&self, header: &Header) -> BlockProcessResult<BlockLevel> {
        /*
        println!("header daa_score : {:?}", header.daa_score);
        println!("header blue_score : {:?}", header.blue_score);
        println!("header hash : {:?}", header.hash);
        println!("header hash_merkle_root : {:?}", header.hash_merkle_root);
        */
        self.check_header_version(header)?;
        self.check_block_timestamp_in_isolation(header)?;
        self.check_parents_limit_upper_bound(header)?;
        Self::check_parents_not_origin(header)?;
        self.check_pow_and_calc_block_level(header)
    }

    pub(super) fn validate_parent_relations(&self, header: &Header) -> BlockProcessResult<()> {
        self.check_parents_exist(header)?;
        self.check_parents_incest(header)?;
        Ok(())
    }

    fn check_header_version(&self, header: &Header) -> BlockProcessResult<()> {
        match (self.khashv2_activation.is_active(header.daa_score), header.version) {
            (true, constants::BLOCK_VERSION_KHASHV2) => Ok(()),
            (false, constants::BLOCK_VERSION_KHASHV1) => Ok(()),
            (true, v) => Err(RuleError::WrongBlockVersion(v, constants::BLOCK_VERSION_KHASHV2)),
            (false, v) => Err(RuleError::WrongBlockVersion(v, constants::BLOCK_VERSION_KHASHV1)),
        }
    }

    fn check_block_timestamp_in_isolation(&self, header: &Header) -> BlockProcessResult<()> {
        // Timestamp deviation tolerance is in seconds so we multiply by 1000 to get milliseconds (without BPS dependency)
        let max_block_time = unix_now() + self.timestamp_deviation_tolerance * 1000;
        if header.timestamp > max_block_time {
            return Err(RuleError::TimeTooFarIntoTheFuture(header.timestamp, max_block_time));
        }
        Ok(())
    }

    fn check_parents_limit_upper_bound(&self, header: &Header) -> BlockProcessResult<()> {
        if header.direct_parents().is_empty() {
            return Err(RuleError::NoParents);
        }

        // [Crescendo]: moved the tight parents limit check to pre_pow_validation since it requires selected parent DAA score info
        // which is available only post ghostdag. We keep this upper bound check here since this method is applied to trusted blocks
        // as well.
        if header.direct_parents().len() > self.max_block_parents.upper_bound() as usize {
            return Err(RuleError::TooManyParents(header.direct_parents().len(), self.max_block_parents.upper_bound() as usize));
        }

        Ok(())
    }

    fn check_parents_not_origin(header: &Header) -> BlockProcessResult<()> {
        if header.direct_parents().iter().any(|&parent| parent.is_origin()) {
            return Err(RuleError::OriginParent);
        }

        Ok(())
    }

    fn check_parents_exist(&self, header: &Header) -> BlockProcessResult<()> {
        let mut missing_parents = Vec::new();
        for parent in header.direct_parents() {
            match self.statuses_store.read().get(*parent).unwrap_option() {
                None => missing_parents.push(*parent),
                Some(StatusInvalid) => {
                    return Err(RuleError::InvalidParent(*parent));
                }
                Some(_) => {}
            }
        }
        if !missing_parents.is_empty() {
            return Err(RuleError::MissingParents(missing_parents));
        }
        Ok(())
    }

    fn check_parents_incest(&self, header: &Header) -> BlockProcessResult<()> {
        let parents = header.direct_parents();
        for parent_a in parents.iter() {
            for parent_b in parents.iter() {
                if parent_a == parent_b {
                    continue;
                }

                if self.reachability_service.is_dag_ancestor_of(*parent_a, *parent_b) {
                    return Err(RuleError::InvalidParentsRelation(*parent_a, *parent_b));
                }
            }
        }

        Ok(())
    }

    fn check_pow_and_calc_block_level(&self, header: &Header) -> BlockProcessResult<BlockLevel> {
        let state = karlsen_pow::State::new(header);
        let (passed, pow) = state.check_pow(header.nonce);
        if passed || self.skip_proof_of_work {
            Ok(calc_level_from_pow(pow, self.max_block_level))
        } else {
            Err(RuleError::InvalidPoW)
        }
    }
}
