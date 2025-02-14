#![cfg_attr(not(feature = "std"), no_std)]

pub use constants::*;
pub use types::*;

/// Common types for all runtimes
pub mod types {
    use frame_support::traits::EitherOfDiverse;
    use frame_system::EnsureRoot;

    use sp_runtime::traits::{BlakeTwo256, IdentifyAccount, Verify};
    use sp_std::vec::Vec;
    pub type EnsureRootOr<O> = EitherOfDiverse<EnsureRoot<AccountId>, O>;

    pub use common_types::CurrencyId;

    /// An index to a block.
    pub type BlockNumber = u32;

    /// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
    pub type Signature = sp_runtime::MultiSignature;

    /// Some way of identifying an account on the chain. We intentionally make it equivalent
    /// to the public key of our transaction signing scheme.
    pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

    /// The type for looking up accounts. We don't expect more than 4 billion of them, but you
    /// never know...
    pub type AccountIndex = u32;

    /// IBalance is the signed version of the Balance for orml tokens
    pub type IBalance = i128;

    /// The address format for describing accounts.
    pub type Address = sp_runtime::MultiAddress<AccountId, ()>;

    /// Balance of an account.
    pub type Balance = u128;

    /// Index of a transaction in the chain.
    pub type Index = u32;

    /// A hash of some data used by the chain.
    pub type Hash = sp_core::H256;

    /// Block header type as expected by this runtime.
    pub type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;

    /// Aura consensus authority.
    pub type AuraId = sp_consensus_aura::sr25519::AuthorityId;

    /// Moment type
    pub type Moment = u64;

    // A vector of bytes, conveniently named like it is in Solidity.
    pub type Bytes = Vec<u8>;

    // A 32 bytes fixed-size array.
    pub type Bytes32 = FixedArray<u8, 32>;

    // Fixed-size array of given typed elements.
    pub type FixedArray<T, const S: usize> = [T; S];

    // A cryptographic salt to be combined with a value before hashing.
    pub type Salt = FixedArray<u8, 32>;


}

pub mod currency {
    use super::types::Balance;

    pub const IMBU: Balance = 1_000_000_000_000;
    pub const DOLLARS: Balance = IMBU;
    pub const CENTS: Balance = DOLLARS / 100;
    pub const MILLI_IMBU: Balance = CENTS / 10;
    pub const MICRO_IMBU: Balance = MILLI_IMBU / 1000;

    pub const EXISTENTIAL_DEPOSIT: Balance = MICRO_IMBU;

    /// Minimum vesting amount, in IMBU
    pub const MIN_VESTING: Balance = 10;

    /// Additional fee charged when moving native tokens to target chains (in IMBUs).
    pub const NATIVE_TOKEN_TRANSFER_FEE: Balance = 10 * CENTS;	

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        // map to 1/10 of what the kusama relay chain charges (v9020)
        items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
    }
}

/// Common constants for all runtimes
pub mod constants {
    use super::types::BlockNumber;
    use frame_support::weights::{constants::WEIGHT_PER_SECOND, Weight};
    use sp_runtime::Perbill;

    /// This determines the average expected block time that we are targeting. Blocks will be
    /// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
    /// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
    /// slot_duration()`.
    ///
    /// Change this to adjust the block time.
    pub const MILLISECS_PER_BLOCK: u64 = 12000;
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    // Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;

    /// Milliseconds per day
    pub const MILLISECS_PER_DAY: u64 = 86400000;

    /// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
    /// used to limit the maximal weight of a single extrinsic.
    pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);
    /// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
    /// Operational  extrinsics.
    pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

    /// We allow for 0.5 seconds of compute with a 6 second average block time.
    pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

}

pub mod parachains {

    pub mod kusama {
        pub mod karura {
            pub const ID: u32 = 2000;
            pub const KAR_KEY: &[u8] = &[0, 128];
            pub const AUSD_KEY: &[u8] = &[0, 129];
        }
        pub mod mangata {
            pub const ID: u32 = 2110;
            pub const MGX_KEY: &[u8] = &[0, 0, 0, 0];
        }
        pub mod imbue {
            pub const ID: u32 = 2121;
            pub const IMBU_KEY: &[u8] = &[0, 150];
        }
    }

}

pub mod xcm_fees {
    use super::types::Balance;
    use super::currency::CENTS;
    pub use common_types::{CurrencyId,currency_decimals};
    use smallvec::smallvec;
    use sp_runtime::Perbill;

    use frame_support::{
        weights::{
            constants::{ExtrinsicBaseWeight, WEIGHT_PER_SECOND},
            WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
        },
    };

    pub struct WeightToFee;
    impl WeightToFeePolynomial for WeightToFee {
        type Balance = Balance;
        fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
            // in Karura, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
            let p = base_tx_in_imbu();
            let q = Balance::from(ExtrinsicBaseWeight::get());
            smallvec![WeightToFeeCoefficient {
                degree: 1,
                negative: false,
                coeff_frac: Perbill::from_rational(p % q, q),
                coeff_integer: p / q,
            }]
        }
    }

    // The fee cost per second for transferring the native token in cents.
    pub fn native_per_second() -> Balance {
        default_per_second()
    }

    pub fn ksm_per_second() -> Balance {
        default_per_second() / 50
    }

    pub fn kar_per_second() -> u128 {
        default_per_second()
    }

    pub fn base_tx_in_imbu() -> Balance {
        CENTS / 10
    }

    pub fn default_per_second() -> Balance {
		let base_weight = Balance::from(ExtrinsicBaseWeight::get());
		let default_per_second = (WEIGHT_PER_SECOND as u128) / base_weight;
		default_per_second * base_tx_in_imbu()
	}
    
}

/// AssetRegistry's AssetProcessor
pub mod asset_registry {
    use super::types::{AccountId, Balance};
	use common_types::{CurrencyId, CustomMetadata};
	use codec::{Decode, Encode};
	use frame_support::{
		dispatch::RawOrigin,
		sp_std::marker::PhantomData,
		traits::{EnsureOrigin, EnsureOriginWithArg},
	};
	use orml_traits::asset_registry::{AssetMetadata, AssetProcessor};
	use scale_info::TypeInfo;
	use sp_runtime::DispatchError;

	#[derive(
		Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo,
	)]
	pub struct CustomAssetProcessor;

	impl AssetProcessor<CurrencyId, AssetMetadata<Balance, CustomMetadata>> for CustomAssetProcessor {
		fn pre_register(
			id: Option<CurrencyId>,
			metadata: AssetMetadata<Balance, CustomMetadata>,
		) -> Result<(CurrencyId, AssetMetadata<Balance, CustomMetadata>), DispatchError> {
			match id {
				Some(id) => Ok((id, metadata)),
				None => Err(DispatchError::Other("asset-registry: AssetId is required")),
			}
		}

		fn post_register(
			_id: CurrencyId,
			_asset_metadata: AssetMetadata<Balance, CustomMetadata>,
		) -> Result<(), DispatchError> {
			Ok(())
		}
	}

	/// The OrmlAssetRegistry::AuthorityOrigin impl
	pub struct AuthorityOrigin<
		// The origin type
		Origin,
		// The default EnsureOrigin impl used to authorize all
		// assets besides tranche tokens.
		DefaultEnsureOrigin,
	>(PhantomData<(Origin, DefaultEnsureOrigin)>);

	impl<
			Origin: Into<Result<RawOrigin<AccountId>, Origin>> + From<RawOrigin<AccountId>>,
			DefaultEnsureOrigin: EnsureOrigin<Origin>,
		> EnsureOriginWithArg<Origin, Option<CurrencyId>> for AuthorityOrigin<Origin, DefaultEnsureOrigin>
	{
		type Success = ();

		fn try_origin(
			origin: Origin,
			asset_id: &Option<CurrencyId>,
		) -> Result<Self::Success, Origin> {
            match asset_id {
				// Any other `asset_id` defaults to EnsureRoot
				_ => DefaultEnsureOrigin::try_origin(origin).map(|_| ()),
			}
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn successful_origin(_asset_id: &Option<CurrencyId>) -> Origin {
			unimplemented!()
		}
	}
}

pub mod common_xcm {
    use super::types::{Balance};
	use common_types::{CurrencyId, CustomMetadata};
	use frame_support::sp_std::marker::PhantomData;
	use sp_runtime::{traits::ConstU32, WeakBoundedVec};
	use xcm::latest::{Junction::GeneralKey, MultiLocation};

	use crate::xcm_fees::default_per_second;

	/// Our FixedConversionRateProvider, used to charge XCM-related fees for tokens registered in
	/// the asset registry that were not already handled by native Trader rules.
	pub struct FixedConversionRateProvider<OrmlAssetRegistry>(PhantomData<OrmlAssetRegistry>);

	impl<
			OrmlAssetRegistry: orml_traits::asset_registry::Inspect<
				AssetId = CurrencyId,
				Balance = Balance,
				CustomMetadata = CustomMetadata,
			>,
		> orml_traits::FixedConversionRateProvider for FixedConversionRateProvider<OrmlAssetRegistry>
	{
		fn get_fee_per_second(location: &MultiLocation) -> Option<u128> {
			let metadata = OrmlAssetRegistry::metadata_by_location(location)?;
			metadata
				.additional
				.xcm
				.fee_per_second
				.or_else(|| Some(default_per_second()))
		}
	}

	pub fn general_key(key: &[u8]) -> xcm::latest::Junction {
		GeneralKey(WeakBoundedVec::<u8, ConstU32<32>>::force_from(
			key.into(),
			None,
		))
	}
}