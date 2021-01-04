use super::*;

use sp_core::H256;
use frame_support::{
	impl_outer_origin, impl_outer_event, parameter_types, weights::Weight,
	assert_ok, assert_noop,
};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {}
}

mod nfc {
	// Re-export needed for `impl_outer_event!`.
	pub use super::super::*;
}


impl_outer_event! {
	pub enum Event for Test {
		frame_system<T>,
		nfc<T>,
	}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl Trait for Test {
	type Event = Event;
}

type NfcModule = Module<Test>;
type System = frame_system::Module<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t: sp_io::TestExternalities = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
	t.execute_with(|| System::set_block_number(1) );
	t
}

fn last_event() -> Event {
	System::events().last().unwrap().event.clone()
}

#[test]
fn can_create_token_class() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftModule::create_class(Origin::signed(1), "token", ()));
		// how the hell query the orml_nft storage
	})
}
