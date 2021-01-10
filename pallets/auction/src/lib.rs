#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused)]
// Used for encoding/decoding into scale
use codec::{Encode, Decode};
use frame_support::{traits::{LockableCurrency, LockIdentifier, Currency, WithdrawReason, WithdrawReasons},
					Parameter, decl_error, decl_event, decl_module, decl_storage, ensure, dispatch::{DispatchResult, DispatchError}};
use frame_system::ensure_signed;
use sp_runtime::{
	traits::{AtLeast32Bit, AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, MaybeDisplay, Zero},
	RuntimeDebug
};
use sp_std::{fmt::Debug, result, vec::Vec};
pub use traits::*;

pub mod traits;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/*
Parameter - can be used in Dispatchable function (so in the decl_module!)
Member - can be used in the runtime structures
Default - gives trait variables default values (bool = false, int = 0, etc)
Copy - value can be duplicated by simply copying the bits
MaybeSerializeDeserialize - type that implements Serialize, DeserializeOwned and Debug when in std environment.
Bounded - numbers which have upper and lower bounds (so, basically all primitive types???)
 */

pub trait Trait: frame_system::Trait + orml_nft::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The balance type for bidding
	type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

	/// The auction ID type
	type AuctionId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;

	// Currency
	type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

	// Definition of the tokenId token from the Nft pallet
	type NftTokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;

	// Definition of token classId from the Nft pallet
	type NftClassId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
}

pub type AuctionInfoOf<T> = AuctionInfo<<T as frame_system::Trait>::AccountId, <T as Trait>::Balance, <T as frame_system::Trait>::BlockNumber,
	<T as Trait>::NftClassId, <T as Trait>::NftTokenId>;

decl_storage! {
	trait AuctionStore for Module<T: Trait> as AuctionModule {
		/// Stores on-going and future auctions. Closed auction are removed.
		pub EnglishAuctions get(fn english_auctions): map hasher(twox_64_concat) T::AuctionId => Option<AuctionInfoOf<T>>;

		/// Stores on-going and future auctions. Closed auction are removed.
		pub DutchAuctions get(fn dutch_auctions): map hasher(twox_64_concat) T::AuctionId => Option<AuctionInfoOf<T>>;

		/// Track the next auction ID.
		pub AuctionsIndex get(fn auctions_index): T::AuctionId;

		/// Index auctions by end time.
		pub AuctionEndTime get(fn auction_end_time): double_map hasher(twox_64_concat) T::BlockNumber, hasher(twox_64_concat) T::AuctionId => Option<()>;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
		<T as Trait>::AuctionId,
	{
		// Auction created
		AuctionCreated(AccountId, AuctionId),
		/// A bid is placed
		Bid(AuctionId, AccountId, Balance),
		//
		AuctionConcluded(AuctionId),
	}
);

decl_error! {
	/// Error for auction module.
	pub enum Error for Module<T: Trait> {
		AuctionNotExist,
		AuctionNotStarted,
		AuctionAlreadyStarted,
		BidNotAccepted,
		InvalidBidPrice,
		NoAvailableAuctionId,
		AuctionStartTimeAlreadyPassed,
		NonExistingAuctionType,
		BadAuctionConfiguration,
		NotAnAuctionOnwer,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight=0]
		fn create_auction(origin, auction_info: AuctionInfoOf<T>) {
			let sender = ensure_signed(origin)?;
			auction_info.owner = sender;

			<Module<T>>::new_auction(auction_info)?;
		}

		#[weight=0]
		fn bid_value(origin, id: T::AuctionId, value: T::Balance) {
			let sender = ensure_signed(origin)?;
			<Module<T>>::bid(sender, id, value)?;
		}
	}
}

impl<T: Trait> Auction<T::AccountId, T::BlockNumber, T::NftClassId, T::NftTokenId> for Module<T>{
	type AuctionId = T::AuctionId;
	type Balance = T::Balance;
	type AccountId = T::AccountId;

	fn new_auction(info: AuctionInfoOf<T>) -> result::Result<Self::AuctionId, DispatchError> {
		Self::check_new_auction(info.clone());
		let auction_id = <AuctionsIndex<T>>::try_mutate(|x| -> result::Result<Self::AuctionId, DispatchError> {
			let id = *x;
			ensure!(id != T::AuctionId::max_value(), Error::<T>::NoAvailableAuctionId);
			*x += One::one();
			Ok(id)
		})?;

		match info.auction_type {
			AuctionType::English => {
				<EnglishAuctions<T>>::insert(auction_id, info);
			}
			AuctionType::Dutch => {
				<DutchAuctions<T>>::insert(auction_id, info);
			}
			_ => ()
		}
		Ok(auction_id)
	}

	fn auction_info(id: Self::AuctionId) -> Option<AuctionInfoOf<T>> {
		Self::english_auctions(id)
	}

	fn update_auction(id: Self::AuctionId, info: AuctionInfoOf<T>) -> DispatchResult {
		<EnglishAuctions<T>>::try_mutate(id, |auction| -> DispatchResult {
			ensure!(auction.is_some(), Error::<T>::AuctionNotExist);
			*auction = Option::Some(info);
			Ok(())
		})
	}

	fn remove_auction(id: Self::AuctionId) -> DispatchResult {
		let current_block_number = frame_system::Module::<T>::block_number();
		if let Some(auction) = Self::english_auctions(id) {
			ensure!(current_block_number < auction.start, Error::<T>::AuctionAlreadyStarted);
		}
		<EnglishAuctions<T>>::remove(id);
		Ok(())
	}

	fn bid(bidder: Self::AccountId, id: Self::AuctionId, value: Self::Balance) -> DispatchResult {
		<EnglishAuctions<T>>::try_mutate_exists(id, |auction| -> DispatchResult {
			let mut auction = auction.as_mut().ok_or(Error::<T>::AuctionNotExist)?;
			let block_number = <frame_system::Module<T>>::block_number();
			Self::check_bid(auction.clone(), block_number, value);
			auction.last_bid = Some((bidder, value));
			Ok(())
		})
	}
}

impl<T: Trait> Module<T> {

	fn check_new_auction(auction: AuctionInfoOf<T>) -> DispatchResult {
		let current_block_number = frame_system::Module::<T>::block_number();
		ensure!(auction.start <= current_block_number, Error::<T>::AuctionStartTimeAlreadyPassed);
		ensure!(auction.start != Zero::zero() && auction.end != Zero::zero() && !auction.name.is_empty(), Error::<T>::BadAuctionConfiguration);
		let is_owner = orml_nft::Module::<T>::is_owner(&auction.owner, auction.token_id);
		ensure!(is_owner, Error::<T>::NotAnAuctionOnwer);
		Ok(())
	}

	fn check_bid(auction: AuctionInfoOf<T>, block_number: T::BlockNumber, bid: T::Balance) -> DispatchResult {
		ensure!(block_number >= auction.start, Error::<T>::AuctionNotStarted);
		ensure!(bid >= auction.minimal_bid, Error::<T>::InvalidBidPrice);
		if let Some(current_bid) = auction.last_bid {
			ensure!(bid > current_bid.1, Error::<T>::InvalidBidPrice);
		} else {
			ensure!(bid == Zero::zero(), Error::<T>::InvalidBidPrice);
		}
		Ok(())
	}
}
