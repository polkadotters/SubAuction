use super::*;

use std::cell::RefCell;
use pallet_nft::TokenData;
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

mod auction {
	// Re-export needed for `impl_outer_event!`.
	pub use super::super::*;
}

impl_outer_event! {
	pub enum Event for Test {
        frame_system<T>,
        pallet_balances<T>,
		auction<T>,
		pallet_nft<T>,
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

impl Config for Test {
    type Event = Event;
	type Balance = u128;
	type AuctionId = u64;
	type Currency = Balances;
}

type AuctionsModule = Module<Test>;
type System = frame_system::Module<Test>;
type Balances = pallet_balances::Module<Test>;
type NFT = pallet_nft::Module<Test>;

impl frame_system::Config for Test {
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
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl orml_nft::Config for Test {
    type ClassId = u64;
	type TokenId = u64;
	type ClassData = u32;
	type TokenData = pallet_nft::TokenData;
}

impl pallet_nft::Config for Test {
    type Event = Event;
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

fn create_nft(){
	assert_ok!(NFT::create_class(
		Origin::signed(100), "Class1".as_bytes().to_vec(), 0
	));
	assert_ok!(NFT::mint(
		Origin::signed(100), 0, "Class1_mint1".as_bytes().to_vec(), TokenData {locked:false}
	));
}

#[test]
fn can_create_auction() {
	new_test_ext().execute_with(|| {
		let auction_info = AuctionInfo {
			name: "Aukce1".as_bytes().to_vec(),
			last_bid: None,
			start: 1,
			end: 20,
			owner: None,
			auction_type: AuctionType::English,
			token_id: (0,0),
			minimal_bid: 50,
		};
		assert_noop!(AuctionsModule::create_auction(Origin::signed(100), auction_info.clone()), Error::<Test>::NotATokenOwner);
		create_nft();
		assert_ok!(AuctionsModule::create_auction(Origin::signed(100), auction_info));
	});
}