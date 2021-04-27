use crate as pallet_auction;
use sp_core::H256;
use frame_support::parameter_types;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system as system;

mod auction {
	// Re-export needed for `impl_outer_event!`.
	pub use super::super::*;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Auctions: pallet_auction::{Module, Call, Storage, Event<T>},
		OrmlNft: orml_nft::{Module, Storage},
		Nft: pallet_nft::{Module, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
	}
);

/// Balance of an account.
pub type Balance = u128;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl pallet_nft::Config for Test {
    type Event = Event;
}

impl orml_nft::Config for Test {
	type ClassId = u64;
	type TokenId = u64;
	type ClassData = u32;
	type TokenData = pallet_nft::TokenData;
}

impl pallet_auction::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type AuctionId = u64;
	type Currency = Balances;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

    pallet_balances::GenesisConfig::<Test>{
		balances: vec![(200, 500)],
    }.assimilate_storage(&mut t).unwrap();
    
    let mut t: sp_io::TestExternalities = t.into();

    t.execute_with(|| System::set_block_number(1) );
    t
}