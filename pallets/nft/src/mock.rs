use super::*;
use crate as pallet_nft;

use frame_support::{parameter_types, weights::Weight};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

mod nfc {
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
		OrmlNft: orml_nft::{Module, Storage},
		Nft: pallet_nft::{Module, Call, Config<T>, Storage, Event<T>},
	}
);

impl pallet_nft::Config for Test {
	type Event = Event;
	type WeightInfo = pallet_nft::weights::SubstrateWeight<Test>;
}

impl orml_nft::Config for Test {
	type ClassId = u64;
	type TokenId = u64;
	type ClassData = u32;
	type TokenData = pallet_nft::TokenData;
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Config for Test {
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into();
	t.execute_with(|| System::set_block_number(1));
	t
}
