#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_quadratic_funding.
pub trait WeightInfo {
    fn fund() -> Weight;
    fn create_project() -> Weight;
    fn submit_milestone() -> Weight;
    fn schedule_round(s: u32) -> Weight;
    fn cancel_round() -> Weight;
    fn cancel() -> Weight;
    fn set_withdrawal_expiration() -> Weight;
    fn set_max_proposal_count_per_round(s: u32) -> Weight;
    fn set_is_identity_required() -> Weight;
    fn contribute() -> Weight;
    fn finalize_round() -> Weight;
    fn approve() -> Weight;
    fn withdraw() -> Weight;
    fn refund() -> Weight;
}

/// Weights for pallet_quadratic_funding using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn fund() -> Weight {
        (49_000_000_u64)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn create_project() -> Weight {
        (25_000_000_u64)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn submit_milestone() -> Weight {
        (25_000_000_u64)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn schedule_round(s: u32) -> Weight {
        (33_595_000_u64)
            // Standard Error: 1_000
            .saturating_add((71_000_u64).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn cancel_round() -> Weight {
        (24_000_000_u64)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn cancel() -> Weight {
        (20_000_000_u64)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn set_withdrawal_expiration() -> Weight {
        (1_000_000_u64).saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn set_max_proposal_count_per_round(_s: u32) -> Weight {
        (1_240_000_u64).saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn set_is_identity_required() -> Weight {
        (1_000_000_u64).saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn contribute() -> Weight {
        (55_000_000_u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn finalize_round() -> Weight {
        (23_000_000_u64)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn approve() -> Weight {
        (26_000_000_u64)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    fn withdraw() -> Weight {
        (66_000_000_u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    fn refund() -> Weight {
        (66_000_000_u64)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn fund() -> Weight {
        (49_000_000_u64)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn create_project() -> Weight {
        (25_000_000_u64)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn submit_milestone() -> Weight {
        (25_000_000_u64)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn schedule_round(s: u32) -> Weight {
        (33_595_000_u64)
            // Standard Error: 1_000
            .saturating_add((71_000_u64).saturating_mul(s as Weight))
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn cancel_round() -> Weight {
        (24_000_000_u64)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn cancel() -> Weight {
        (20_000_000_u64)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn set_withdrawal_expiration() -> Weight {
        (1_000_000_u64).saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn set_max_proposal_count_per_round(_s: u32) -> Weight {
        (1_240_000_u64).saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn set_is_identity_required() -> Weight {
        (1_000_000_u64).saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn contribute() -> Weight {
        (55_000_000_u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn finalize_round() -> Weight {
        (23_000_000_u64)
            .saturating_add(RocksDbWeight::get().reads(1_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn approve() -> Weight {
        (26_000_000_u64)
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
    }
    fn withdraw() -> Weight {
        (66_000_000_u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
    fn refund() -> Weight {
        (66_000_000_u64)
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(2_u64))
    }
}
