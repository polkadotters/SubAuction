use frame_support::{assert_noop, assert_ok};

use super::*;
use crate::{mock::*, Error};
use pallet_nft::TokenData;

pub type AuctionsModule = Module<Test>;
pub type NFT = pallet_nft::Module<Test>;

fn create_nft() {
	assert_ok!(NFT::create_class(Origin::signed(100), "Class1".as_bytes().to_vec(), 0));
	assert_ok!(NFT::mint(
		Origin::signed(100),
		0,
		"Class1_mint1".as_bytes().to_vec(),
		TokenData { locked: false },
		1
	));
}

#[test]
fn can_create_english_auction() {
	new_test_ext().execute_with(|| {
		let auction_info = EnglishAuctionInfo {
			name: "Aukce1".as_bytes().to_vec(),
			last_bid: None,
			start: 1,
			end: 20,
			owner: 100,
			token_id: (0, 0),
			minimal_bid: 50,
		};
		assert_noop!(
			AuctionsModule::english_create_auction(Origin::signed(100), auction_info.clone()),
			Error::<Test>::NotATokenOwner
		);
		create_nft();
		assert_ok!(AuctionsModule::english_create_auction(Origin::signed(100), auction_info));
	});
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(200, 500)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut t: sp_io::TestExternalities = t.into();

	t.execute_with(|| System::set_block_number(1));
	t
}
