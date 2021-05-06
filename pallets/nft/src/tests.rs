use frame_support::assert_ok;

use super::*;
use mock::{Event, *};

type NftModule = Module<Test>;

#[test]
fn can_create_token_class() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NftModule::create_class(
			Origin::signed(ALICE),
			"token".as_bytes().to_vec(),
			Default::default()
		));
		let event = Event::pallet_nft(crate::Event::NFTTokenClassCreated(ALICE, 0));
		assert_eq!(last_event(), event);
	})
}
