use sp_runtime::traits::{Convert, Zero};
use sp_std::{marker::PhantomData, prelude::*};

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime, ensure, parameter_types,
    traits::{
        fungibles, Contains, Currency as PalletCurrency, EnsureOriginWithArg, EqualPrivilegeOnly,
        Everything, Get, Imbalance, IsInVec, Nothing, OnUnbalanced, Randomness,
    },
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        ConstantMultiplier, DispatchClass, IdentityFee, Weight,
    },
    PalletId, StorageValue,
};

use orml_asset_registry::{AssetRegistryTrader, FixedRateAssetRegistryTrader};
use orml_traits::{location::AbsoluteReserveProvider, parameter_type_with_key};
use orml_xcm_support::{
    DepositToAlternative, IsNativeConcrete, MultiCurrencyAdapter, MultiNativeAsset,
};

pub use common_runtime::{
    asset_registry::{AuthorityOrigin, CustomAssetProcessor},
    common_xcm::{general_key, FixedConversionRateProvider},
    parachains,
    xcm_fees::{default_per_second, ksm_per_second, native_per_second, WeightToFee},
    EnsureRootOr,
};
pub use common_types::{CurrencyId, currency_decimals, CustomMetadata};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

use pallet_xcm::XcmPassthrough;
use xcm::latest::prelude::*;
use xcm_builder::{
    AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
    AllowTopLevelPaidExecutionFrom, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
    LocationInverter, ParentAsSuperuser, ParentIsPreset, RelayChainAsNative,
    SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
    SignedToAccountId32, SovereignSignedViaLocation, TakeRevenue, TakeWeightCredit,
};
use xcm_executor::XcmExecutor;

use pallet_collective::EnsureProportionAtLeast;
use polkadot_parachain::primitives::Sibling;

parameter_types! {
    // One XCM operation is 100_000_000 weight - almost certainly a conservative estimate.
    pub UnitWeightCost: Weight = 200_000_000;
    pub const MaxInstructions: u32 = 100;
}

use super::{
    AccountId, Balance, Call, CouncilCollective, Currencies, Event, Origin, OrmlAssetRegistry,
    ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, TreasuryAccount, UnknownTokens,
    XcmpQueue,
};

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
    type AssetClaims = PolkadotXcm;
    type AssetTransactor = ImbueAssetTransactor;
    type AssetTrap = PolkadotXcm;
    type Barrier = Barrier;
    type Call = Call;
    type IsReserve = MultiNativeAsset<AbsoluteReserveProvider>;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type OriginConverter = XcmOriginToTransactDispatchOrigin;
    type ResponseHandler = PolkadotXcm;
    type SubscriptionService = PolkadotXcm;
    type Trader = Trader;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type XcmSender = XcmRouter;
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the default `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
    // recognized.
    RelayChainAsNative<RelayChainOrigin, Origin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognized.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    // Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
    // transaction from the Root origin.
    ParentAsSuperuser<Origin>,
    // Native signed account converter; this just converts an `AccountId32` origin into a normal
    // `Origin::Signed` origin of the same 32-byte value.
    SignedAccountId32AsNative<RelayNetwork, Origin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<Origin>,
);

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    // Expected responses are OK.
    AllowKnownQueryResponses<PolkadotXcm>,
    // Subscriptions for version tracking are OK.
    AllowSubscriptionsFrom<Everything>,
);

parameter_types! {
    pub const KsmLocation: MultiLocation = MultiLocation::parent();
    pub const RelayNetwork: NetworkId = NetworkId::Kusama;
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
    pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

pub type ImbueAssetTransactor = MultiCurrencyAdapter<
    Currencies,
    UnknownTokens,
    IsNativeConcrete<CurrencyId, CurrencyIdConvert>,
    AccountId,
    LocationToAccountId,
    CurrencyId,
    CurrencyIdConvert,
    DepositToAlternative<TreasuryAccount, Currencies, CurrencyId, AccountId, Balance>,
>;

pub struct ToTreasury;
impl TakeRevenue for ToTreasury {
    fn take_revenue(revenue: MultiAsset) {
        if let MultiAsset {
            id: Concrete(_location),
            fun: Fungible(_amount),
        } = revenue
        {
            // TODO(sam): implement this
        }
    }
}

parameter_types! {
    pub KsmPerSecond: (AssetId, u128) = (MultiLocation::parent().into(), ksm_per_second());

    pub CanonicalImbuePerSecond: (AssetId, u128) = (
        MultiLocation::new(
            0,
            X1(general_key(parachains::kusama::imbue::IMBU_KEY)),
        ).into(),
        native_per_second(),
    );

    pub ImbuPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::kusama::imbue::ID), general_key(parachains::kusama::imbue::IMBU_KEY))
        ).into(),
        native_per_second(),
    );

    pub MgxPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::kusama::mangata::ID), general_key(parachains::kusama::mangata::MGX_KEY))
        ).into(),
		ksm_per_second() * 50
    );

    pub AUsdPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::kusama::karura::ID), general_key(parachains::kusama::karura::AUSD_KEY))
        ).into(),
		ksm_per_second() * 50
    );

    pub KarPerSecond: (AssetId, u128) = (
        MultiLocation::new(
            1,
            X2(Parachain(parachains::kusama::karura::ID), general_key(parachains::kusama::karura::KAR_KEY))
        ).into(),
		ksm_per_second() * 100
    );
}

pub type Trader = (
    FixedRateOfFungible<CanonicalImbuePerSecond, ToTreasury>,
    FixedRateOfFungible<ImbuPerSecond, ToTreasury>,
    FixedRateOfFungible<KsmPerSecond, ToTreasury>,
    AssetRegistryTrader<
        FixedRateAssetRegistryTrader<FixedConversionRateProvider<OrmlAssetRegistry>>,
        ToTreasury,
    >,
    FixedRateOfFungible<AUsdPerSecond, ToTreasury>,
    FixedRateOfFungible<KarPerSecond, ToTreasury>,
    FixedRateOfFungible<MgxPerSecond, ToTreasury>,
);

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmExecuteFilter = Nothing;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type LocationInverter = LocationInverter<Ancestry>;
    type Origin = Origin;
    type Call = Call;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl orml_xcm::Config for Runtime {
    type Event = Event;
    type SovereignOrigin = MoreThanHalfCouncil;
}

/// CurrencyIdConvert
/// This type implements conversions from our `CurrencyId` type into `MultiLocation` and vice-versa.
/// A currency locally is identified with a `CurrencyId` variant but in the network it is identified
/// in the form of a `MultiLocation`, in this case a pair (Para-Id, Currency-Id).
pub struct CurrencyIdConvert;

/// Convert an incoming `MultiLocation` into a `CurrencyId` if possible.
/// Here we need to know the canonical representation of all the tokens we handle in order to
/// correctly convert their `MultiLocation` representation into our internal `CurrencyId` type.
impl Convert<MultiLocation, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(location: MultiLocation) -> Option<CurrencyId> {
        if location == MultiLocation::parent() {
            return Some(CurrencyId::KSM);
        }

        match location.clone() {
            MultiLocation {
                parents: 0,
                interior: X1(GeneralKey(key)),
            } => match &key[..] {
                parachains::kusama::imbue::IMBU_KEY => Some(CurrencyId::Native),
                _ => OrmlAssetRegistry::location_to_asset_id(location.clone()),
            },
            MultiLocation {
                parents: 1,
                interior: X2(Parachain(para_id), GeneralKey(key)),
            } => match para_id {
                parachains::kusama::karura::ID => match &key[..] {
                    parachains::kusama::karura::AUSD_KEY => Some(CurrencyId::AUSD),
                    parachains::kusama::karura::KAR_KEY => Some(CurrencyId::KAR),
                    parachains::kusama::imbue::IMBU_KEY => Some(CurrencyId::Native),
                    _ => OrmlAssetRegistry::location_to_asset_id(location.clone()),
                },
                parachains::kusama::mangata::ID => match &key[..] {
                    parachains::kusama::mangata::MGX_KEY => Some(CurrencyId::MGX),
                    parachains::kusama::imbue::IMBU_KEY => Some(CurrencyId::Native),
                    _ => OrmlAssetRegistry::location_to_asset_id(location.clone()),
                },

                parachains::kusama::imbue::ID => match &key[..] {
                    parachains::kusama::imbue::IMBU_KEY => Some(CurrencyId::Native),
                    _ => OrmlAssetRegistry::location_to_asset_id(location.clone()),
                },

                id if id == u32::from(ParachainInfo::get()) => match &key[..] {
                    parachains::kusama::imbue::IMBU_KEY => Some(CurrencyId::Native),
                    _ => OrmlAssetRegistry::location_to_asset_id(location.clone()),
                },
                _ => OrmlAssetRegistry::location_to_asset_id(location.clone()),
            },
            _ => OrmlAssetRegistry::location_to_asset_id(location.clone()),
        }
    }
}

impl Convert<MultiAsset, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(asset: MultiAsset) -> Option<CurrencyId> {
        if let MultiAsset {
            id: Concrete(location),
            ..
        } = asset
        {
            Self::convert(location)
        } else {
            None
        }
    }
}

/// Convert our `CurrencyId` type into its `MultiLocation` representation.
/// Other chains need to know how this conversion takes place in order to
/// handle it on their side.
impl Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: CurrencyId) -> Option<MultiLocation> {
        match id {
            CurrencyId::KSM => Some(MultiLocation::parent()),
            CurrencyId::AUSD => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(parachains::kusama::karura::ID),
                    general_key(parachains::kusama::karura::AUSD_KEY),
                ),
            )),
            CurrencyId::KAR => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(parachains::kusama::karura::ID),
                    general_key(parachains::kusama::karura::KAR_KEY),
                ),
            )),
            CurrencyId::MGX => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(parachains::kusama::mangata::ID),
                    general_key(parachains::kusama::mangata::MGX_KEY),
                ),
            )),
            CurrencyId::Native => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(ParachainInfo::get().into()),
                    general_key(parachains::kusama::imbue::IMBU_KEY),
                ),
            )),
            CurrencyId::ForeignAsset(_) => OrmlAssetRegistry::multilocation(&id).ok()?,
        }
    }
}

/// All council members must vote yes to create this origin.
type HalfOfCouncil = EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
/// A majority of the Unit body from Rococo over XCM is our required administration origin.
pub type AdminOrigin = EnsureRootOr<HalfOfCouncil>;
pub type MoreThanHalfCouncil = EnsureRootOr<HalfOfCouncil>;

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

parameter_types! {
    //TODO(Sam): we may need to fine tune this value later on
    pub const BaseXcmWeight: Weight = 100_000_000;
    pub const MaxAssetsForTransfer: usize = 2;
}

impl orml_xtokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type CurrencyIdConvert = CurrencyIdConvert;
    type AccountIdToMultiLocation = AccountIdToMultiLocation;
    type SelfLocation = SelfLocation;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type BaseXcmWeight = BaseXcmWeight;
    type LocationInverter = LocationInverter<Ancestry>;
    type MaxAssetsForTransfer = MaxAssetsForTransfer;
    type MinXcmFee = ParachainMinFee;
    type MultiLocationsFilter = Everything;
    type ReserveProvider = AbsoluteReserveProvider;
}

parameter_types! {
    pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::get().into())));
}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
		None
	};
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
    fn convert(account: AccountId) -> MultiLocation {
        X1(AccountId32 {
            network: NetworkId::Any,
            id: account.into(),
        })
        .into()
    }
}

/// Allow checking in assets that have issuance > 0.
/// This is defined in cumulus but it doesn't seem made available to the world.
pub struct NonZeroIssuance<AccountId, Assets>(PhantomData<(AccountId, Assets)>);
impl<AccountId, Assets> Contains<<Assets as fungibles::Inspect<AccountId>>::AssetId>
    for NonZeroIssuance<AccountId, Assets>
where
    Assets: fungibles::Inspect<AccountId>,
{
    fn contains(id: &<Assets as fungibles::Inspect<AccountId>>::AssetId) -> bool {
        !Assets::total_issuance(*id).is_zero()
    }
}
