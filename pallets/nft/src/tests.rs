use frame_support::assert_ok;

use super::*;
use crate::mock::*;

type NftModule = Module<Test>;

#[test]
fn can_create_token_class() {
	new_test_ext().execute_with(|| {
		assert_ok!(NftModule::create_class(
			Origin::signed(1),
			"token".as_bytes().to_vec(),
			1
		));
		// how to query the orml_nft storage?
	})
}
