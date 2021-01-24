#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused)]
// Used for encoding/decoding into scale
use codec::{Encode, Decode};
use frame_support::{traits::{LockableCurrency, LockIdentifier, Currency, WithdrawReason, WithdrawReasons},
					Parameter, decl_error, decl_event, decl_module, decl_storage, ensure, dispatch::{DispatchResult, DispatchError}};
use frame_system::ensure_signed;
use sp_runtime::{Permill, RuntimeDebug, traits::{AtLeast32Bit, AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, MaybeDisplay, Zero, CheckedAdd}};
use sp_std::{fmt::Debug, result, vec::Vec};
pub use traits::*;

pub mod traits;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: pallet_nft::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The balance type for bidding
	type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

	/// The auction ID type
	// why do we need CheckedAdd if kitties don't need it??
	type AuctionId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded + CheckedAdd;

	/// Single type currency (TODO multiple currencies)
	type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
}

/// Identifier for the currency lock on accounts
const AUCTION_LOCK_ID: LockIdentifier = *b"_auction";
/// Set in percent how much next bid has to be raised
const BID_STEP_PERC: u32 = 10;

/// Define type aliases for better readability
pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
pub type NftClassIdOf<T> = pallet_nft::ClassIdOf<T>;
pub type NftTokenIdOf<T> = pallet_nft::TokenIdOf<T>;
pub type AuctionInfoOf<T> = AuctionInfo<<T as frame_system::Trait>::AccountId,
																	BalanceOf<T>,
										<T as frame_system::Trait>::BlockNumber,
																	NftClassIdOf<T>,
																	NftTokenIdOf<T>,
									   >;

decl_storage! {
	trait AuctionStore for Module<T: Trait> as AuctionModule {
		/// Stores on-going and future auctions. Closed auction are removed.
		// TODO: use single Auction storage using double map (auctionId, type)
		pub Auctions get(fn auctions): map hasher(twox_64_concat) T::AuctionId => Option<AuctionInfoOf<T>>;

		/// Track the next auction ID.
		pub NextAuctionId get(fn auctions_index): T::AuctionId;

		/// Index auctions by end time.
		pub AuctionEndTime get(fn auction_end_time): double_map hasher(twox_64_concat) T::BlockNumber, hasher(twox_64_concat) T::AuctionId => Option<()>;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::AuctionId,
		Balance = BalanceOf<T>,
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
		NotATokenOwner,
		AuctionAlreadyConcluded,
		BidOverflow,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight=0]
		fn create_auction(origin, auction_info: AuctionInfoOf<T>) {
			let sender = ensure_signed(origin)?;

			let new_auction_id = Self::new_auction(&sender, auction_info)?;
			Self::deposit_event(RawEvent::AuctionCreated(sender, new_auction_id));
		}

		#[weight=0]
		fn bid_value(origin, id: T::AuctionId, value: BalanceOf<T>) {
			let sender = ensure_signed(origin)?;

			Self::bid(sender, id, value)?;
		}

		#[weight=0]
		fn get_auction_types(origin) {
			let sender = ensure_signed(origin)?;

		}
	}
}

impl<T: Trait> Auction<T::AccountId, T::BlockNumber, NftClassIdOf<T>, NftTokenIdOf<T>> for Module<T>{
	type AuctionId = T::AuctionId;
	type Balance = BalanceOf<T>;
	type AccountId = T::AccountId;

	fn new_auction(owner: &Self::AccountId, info: AuctionInfoOf<T>) -> result::Result<Self::AuctionId, DispatchError> {
		let current_block_number = frame_system::Module::<T>::block_number();
		ensure!(info.start >= current_block_number, Error::<T>::AuctionStartTimeAlreadyPassed);
		ensure!(info.start != Zero::zero() && info.end != Zero::zero() && !info.name.is_empty(), Error::<T>::BadAuctionConfiguration);
		let is_owner = pallet_nft::Module::<T>::is_owner(&owner, info.token_id);
		ensure!(is_owner, Error::<T>::NotATokenOwner);
		let auction_id = <NextAuctionId<T>>::try_mutate(|next_id| -> result::Result<Self::AuctionId, DispatchError> {
			let current_id = *next_id;
			*next_id = next_id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableAuctionId)?;
			Ok(current_id)
		})?;

		<Auctions<T>>::insert(auction_id, info);

		Ok(auction_id)
	}

	fn auction_info(id: Self::AuctionId) -> Option<AuctionInfoOf<T>> {
		Self::auctions(id)
	}

	fn update_auction(id: Self::AuctionId, info: AuctionInfoOf<T>) -> DispatchResult {
		<Auctions<T>>::try_mutate(id, |auction| -> DispatchResult {
			ensure!(auction.is_some(), Error::<T>::AuctionNotExist);
			*auction = Option::Some(info);
			Ok(())
		})
	}

	fn remove_auction(id: Self::AuctionId) -> DispatchResult {
		let current_block_number = frame_system::Module::<T>::block_number();
		if let Some(auction) = Self::auctions(id) {
			ensure!(current_block_number < auction.start, Error::<T>::AuctionAlreadyStarted);
		}
		<Auctions<T>>::remove(id);
		Ok(())
	}

	fn bid(bidder: Self::AccountId, id: Self::AuctionId, value: Self::Balance) -> DispatchResult {
		<Auctions<T>>::try_mutate_exists(id, |auction| -> DispatchResult {
			let mut auction = auction.as_mut().ok_or(Error::<T>::AuctionNotExist)?;
			let block_number = <frame_system::Module<T>>::block_number();
			ensure!(block_number > auction.start, Error::<T>::AuctionNotStarted);
			ensure!(block_number < auction.end, Error::<T>::AuctionAlreadyConcluded);
			ensure!(value >= auction.minimal_bid, Error::<T>::InvalidBidPrice);
			if let Some(ref current_bid) = auction.last_bid {
				ensure!(value > current_bid.1, Error::<T>::InvalidBidPrice);
			} else {
				ensure!(!value.is_zero(), Error::<T>::InvalidBidPrice);
			}
			// first lock or update the bid ??
			T::Currency::set_lock(
				AUCTION_LOCK_ID,
				&bidder,
				value,
				WithdrawReasons::all()
			);
			auction.last_bid = Some((bidder, value));
			let minimal_bid_step = Permill::from_percent(BID_STEP_PERC).mul_floor(value);
			auction.minimal_bid = value.checked_add(&minimal_bid_step).ok_or(Error::<T>::BidOverflow)?;
			Ok(())
		})
	}

	fn conclude_auction(id: Self::AuctionId) -> DispatchResult {
		<Auctions<T>>::try_mutate_exists(id, |auction| -> DispatchResult {
			let mut auction = auction.as_mut().ok_or(Error::<T>::AuctionNotExist)?;
			// let winner = auction.last_bid.ok_or(Error::<T>::BidNotAccepted)?.0;
			// T::Currency::remove_lock(AUCTION_LOCK_ID, &winner);
			Ok(())
		})
	}
}