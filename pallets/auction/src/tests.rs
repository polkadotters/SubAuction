use frame_support::{assert_noop, assert_ok};

use super::*;
use crate::{Error, mock::*};
use pallet_nft::TokenData;

pub type AuctionsModule = Module<Test>;
pub type NFT = pallet_nft::Module<Test>;

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
			owner: 100,
			auction_type: AuctionType::English,
			token_id: (0,0),
			minimal_bid: 50,
		};
		assert_noop!(AuctionsModule::create_auction(Origin::signed(100), auction_info.clone()), Error::<Test>::NotATokenOwner);
		create_nft();
		assert_ok!(AuctionsModule::create_auction(Origin::signed(100), auction_info));
	});
}