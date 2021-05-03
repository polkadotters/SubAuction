#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::benchmarking::vec::Vec;
use crate::Pallet as AUCTIONS;
use pallet_nft::TokenData;
use sp_std::{boxed::Box, vec};

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;

const SEED: u32 = 0;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	let caller: T::AccountId = account(name, index, SEED);
	caller
}

benchmarks! {
	create_auction {
		let caller = create_account::<T>("caller", 0);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = 123;
		let token_data = TokenData { locked:false };
		let class_id = orml_nft::Module::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
		let token_id = orml_nft::Module::<T>::mint(&caller, class_id, class_metadata, token_data).unwrap_or_default();
		let token = (class_id, token_id);
		let auction_info = AuctionInfo {
			name: "Aukce1".as_bytes().to_vec(),
			last_bid: None,
			start: T::BlockNumber::from(1u32),
			end: T::BlockNumber::from(20u32),
			owner: caller.clone(),
			auction_type: AuctionType::English,
			token_id: token,
			minimal_bid: T::CurrencyBalance::from(T::Balance::from(50u32)).into(),
		};
	}: _(RawOrigin::Signed(caller.clone()), auction_info)
	verify {
	}

	bid_value {
		let caller = create_account::<T>("caller", 0);
		let caller2 = create_account::<T>("caller2", 1);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = 123;
		let token_data = TokenData { locked:false };
		let class_id = orml_nft::Module::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
		let token_id = orml_nft::Module::<T>::mint(&caller, class_id, class_metadata, token_data).unwrap_or_default();
		let token = (class_id, token_id);
		let auction_info = AuctionInfo {
			name: "Aukce1".as_bytes().to_vec(),
			last_bid: None,
			start: T::BlockNumber::from(0u32),
			end: T::BlockNumber::from(20u32),
			owner: caller.clone(),
			auction_type: AuctionType::English,
			token_id: token,
			minimal_bid: T::CurrencyBalance::from(T::Balance::from(50u32)).into(),
		};
		let auction_id = AUCTIONS::<T>::new_auction(auction_info).unwrap_or_default();

	}: _(RawOrigin::Signed(caller2.clone()), auction_id, 1_000_000_u32.into())
	verify {
	}

	delete_auction {
		let caller = create_account::<T>("caller", 0);
		let class_metadata = "just a token class".as_bytes().to_vec();
		let class_data = 123;
		let token_data = TokenData { locked:false };
		let class_id = orml_nft::Module::<T>::create_class(&caller, class_metadata.clone(), class_data).unwrap_or_default();
		let token_id = orml_nft::Module::<T>::mint(&caller, class_id, class_metadata, token_data).unwrap_or_default();
		let token = (class_id, token_id);
		let auction_info = AuctionInfo {
			name: "Aukce1".as_bytes().to_vec(),
			last_bid: None,
			start: T::BlockNumber::from(20u32),
			end: T::BlockNumber::from(50u32),
			owner: caller.clone(),
			auction_type: AuctionType::English,
			token_id: token,
			minimal_bid: T::CurrencyBalance::from(T::Balance::from(50u32)).into(),
		};
		let auction_id = AUCTIONS::<T>::new_auction(auction_info).unwrap_or_default();

	}: _(RawOrigin::Signed(caller.clone()), auction_id)
	verify {
	}
}

#[cfg(test)]
mod tests {
	use super::mock::Test;
	use super::*;
	use crate::tests::new_test_ext;
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_create_auction::<Test>());
			assert_ok!(test_benchmark_bid_value::<Test>());
			assert_ok!(test_benchmark_delete_auction::<Test>());
		});
	}
}
