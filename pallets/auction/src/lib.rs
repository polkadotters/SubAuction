#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused)]
// Used for encoding/decoding into scale
use codec::{Encode, Decode};
use frame_support::{traits::{LockableCurrency, LockIdentifier, Currency, WithdrawReason, WithdrawReasons},
					Parameter, decl_error, decl_event, decl_module, decl_storage, ensure, dispatch::{DispatchResult, DispatchError},
					debug};
use frame_system::ensure_signed;
use sp_runtime::{Permill, RuntimeDebug, traits::{AtLeast32Bit, AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, MaybeDisplay, Zero, CheckedAdd, CheckedSub, StaticLookup}, print};
use sp_std::{fmt::Debug, result, vec::Vec};
pub use traits::*;
use frame_support::traits::ExistenceRequirement;

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
	type AuctionId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded + CheckedAdd;

	/// Single type currency (TODO multiple currencies)
	type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
}

/// Identifier for the currency lock on accounts
const AUCTION_LOCK_ID: LockIdentifier = *b"_auction";
/// Set in percent how much next bid has to be raised
const BID_STEP_PERC: u32 = 10;
/// Increase endtime to avoid sniping
const BID_ADD_BLOCKS: u32 = 10;
/// Minimal auction duration
const MIN_AUCTION_DUR: u32 = 10;

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

		/// Auction owner by ID
		pub AuctionOwnerById get(fn auction_owner_by_id): map hasher(twox_64_concat) T::AuctionId => T::AccountId;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::AuctionId,
		Balance = BalanceOf<T>,
	{
		/// Auction created
		AuctionCreated(AccountId, AuctionId),
		/// A bid is placed
		Bid(AuctionId, AccountId, Balance),
		/// Auction ended
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
		InvalidTimeConfiguration,
		NotATokenOwner,
		AuctionAlreadyConcluded,
		BidOverflow,
		BidOnOwnAuction,
		TimeUnderflow,
		TokenLocked,
		EmptyAuctionName,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight=0]
		fn create_auction(origin, auction_info: AuctionInfoOf<T>) {
			let sender = ensure_signed(origin)?;
			let mut auction_clone = auction_info.clone();
			auction_clone.owner = sender.clone();
			let new_auction_id = Self::new_auction(auction_clone)?;
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

impl<T: Trait> Module<T> {
	fn on_finalize(now: T::BlockNumber) {
		debug::warn!("aaaaaaaaaaaaaaaaaaaa");
		for (auction_id, _) in <AuctionEndTime<T>>::drain_prefix(&now) {
			// let auction = Self::auctions(auction_id).ok_or(Error::<T>::AuctionNotExist)?;
			match Self::auctions(auction_id) {
				Some(auction) => {
					pallet_nft::Module::<T>::toggle_lock(&auction.owner, auction.token_id);
					// there is a bid so let's determine a winner and transfer tokens
					if let Some(ref winner) = auction.last_bid {
						// let lookup = <Runtime as frame_system::Trait>::Lookup::unlookup(winner.0);
						let dest = T::Lookup::unlookup(winner.0.clone());
						let source = T::Origin::from(frame_system::RawOrigin::Signed(auction.owner.clone()));
						pallet_nft::Module::<T>::transfer(source, dest, auction.token_id);
						T::Currency::remove_lock(AUCTION_LOCK_ID, &winner.0);
						<T::Currency as Currency<T::AccountId>>::transfer(&winner.0, &auction.owner, winner.1, ExistenceRequirement::KeepAlive);
					}
				}
				None => ()
			}
		}
	}
}

impl<T: Trait> Auction<T::AccountId, T::BlockNumber, NftClassIdOf<T>, NftTokenIdOf<T>> for Module<T>{
	type AuctionId = T::AuctionId;
	type Balance = BalanceOf<T>;
	type AccountId = T::AccountId;

	fn new_auction(info: AuctionInfoOf<T>) -> result::Result<Self::AuctionId, DispatchError> {
		/// Basic checks before an auction is created
		let current_block_number = frame_system::Module::<T>::block_number();
		ensure!(info.start >= current_block_number, Error::<T>::AuctionStartTimeAlreadyPassed);
		ensure!(info.start != Zero::zero() && info.end != Zero::zero() && info.end > info.start + MIN_AUCTION_DUR.into(), Error::<T>::InvalidTimeConfiguration);
		ensure!(!info.name.is_empty(), Error::<T>::EmptyAuctionName);
		let is_owner = pallet_nft::Module::<T>::is_owner(&info.owner, info.token_id);
		ensure!(is_owner, Error::<T>::NotATokenOwner);
		let nft_locked = pallet_nft::Module::<T>::is_locked(info.token_id)?;
		ensure!(nft_locked == false, Error::<T>::TokenLocked);

		let auction_id = <NextAuctionId<T>>::try_mutate(|next_id| -> result::Result<Self::AuctionId, DispatchError> {
			let current_id = *next_id;
			*next_id = next_id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableAuctionId)?;
			Ok(current_id)
		})?;

		// fix clone
		<Auctions<T>>::insert(auction_id, info.clone());
		<AuctionOwnerById<T>>::insert(auction_id, &info.owner);
		pallet_nft::Module::<T>::toggle_lock(&info.owner, info.token_id);

		Ok(auction_id)
	}

	fn auction_info(id: Self::AuctionId) -> Option<AuctionInfoOf<T>> {
		Self::auctions(id)
	}

	fn update_auction(id: Self::AuctionId, info: AuctionInfoOf<T>) -> DispatchResult {
		<Auctions<T>>::try_mutate(id, |auction| -> DispatchResult {
			ensure!(auction.is_some(), Error::<T>::AuctionNotExist);
			*auction = Some(info);
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
			/// Basic checks before a bid can be made
			let mut auction = auction.as_mut().ok_or(Error::<T>::AuctionNotExist)?;
			let block_number = <frame_system::Module<T>>::block_number();
			let owner = Self::auction_owner_by_id(id);
			ensure!(bidder != owner, Error::<T>::BidOnOwnAuction);
			ensure!(block_number > auction.start, Error::<T>::AuctionNotStarted);
			ensure!(block_number < auction.end, Error::<T>::AuctionAlreadyConcluded);
			ensure!(value >= auction.minimal_bid, Error::<T>::InvalidBidPrice);
			if let Some(ref current_bid) = auction.last_bid {
				ensure!(value > current_bid.1, Error::<T>::InvalidBidPrice);
			} else {
				ensure!(!value.is_zero(), Error::<T>::InvalidBidPrice);
			}
			/// Lock funds
			T::Currency::set_lock(
				AUCTION_LOCK_ID,
				&bidder,
				value,
				WithdrawReasons::all()
			);
			auction.last_bid = Some((bidder, value));
			/// Set next minimal bid
			let minimal_bid_step = Permill::from_percent(BID_STEP_PERC).mul_floor(value);
			auction.minimal_bid = value.checked_add(&minimal_bid_step).ok_or(Error::<T>::BidOverflow)?;
			/// Avoid auction sniping
			let time_left = auction.end.checked_sub(&block_number).ok_or(Error::<T>::TimeUnderflow)?;
			if time_left < BID_ADD_BLOCKS.into() {
				auction.end = block_number + BID_ADD_BLOCKS.into();
			}
			Ok(())
		})
	}

}
