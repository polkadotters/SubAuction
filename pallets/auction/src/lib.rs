#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]
// Used for encoding/decoding into scale
use frame_support::{traits::{LockableCurrency, LockIdentifier, Currency, WithdrawReasons},
					Parameter, ensure, dispatch::{DispatchResult, DispatchError},
					};
use frame_system::ensure_signed;
use sp_runtime::{Permill, traits::{AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, Zero, CheckedAdd, CheckedSub, StaticLookup}};
use sp_std::{result};
pub use traits::*;
use frame_support::traits::ExistenceRequirement;

pub mod traits;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Identifier for the currency lock on accounts
const AUCTION_LOCK_ID: LockIdentifier = *b"_auction";
/// Set in percent how much next bid has to be raised
const BID_STEP_PERC: u32 = 10;
/// Increase endtime to avoid sniping
const BID_ADD_BLOCKS: u32 = 10;
/// Minimal auction duration
const MIN_AUCTION_DUR: u32 = 10;

/// Define type aliases for better readability
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type NftClassIdOf<T> = pallet_nft::ClassIdOf<T>;
pub type NftTokenIdOf<T> = pallet_nft::TokenIdOf<T>;
pub type AuctionInfoOf<T> = AuctionInfo<<T as frame_system::Config>::AccountId,
																	BalanceOf<T>,
										<T as frame_system::Config>::BlockNumber,
																	NftClassIdOf<T>,
																	NftTokenIdOf<T>,
									>;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nft::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The balance type for bidding
		type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

		/// The auction ID type
		type AuctionId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded + CheckedAdd;

		/// Single type currency (TODO multiple currencies)
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	}

	#[pallet::storage]
	#[pallet::getter(fn auctions)]
	/// Stores on-going and future auctions. Closed auction are removed.
	// TODO: use single Auction storage using double map (auctionId, type)
	pub type Auctions<T: Config> = StorageMap<_, Twox64Concat, T::AuctionId, AuctionInfoOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn auctions_index)]
	/// Track the next auction ID.
	pub type NextAuctionId<T: Config> = StorageValue<_, T::AuctionId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn auction_end_time)]
	/// Index auctions by end time.
	pub type AuctionEndTime<T: Config> = StorageDoubleMap<_, Twox64Concat, T::BlockNumber, Twox64Concat, T::AuctionId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn auction_owner_by_id)]
	/// Auction owner by ID
	pub type AuctionOwnerById<T: Config> = StorageMap<_, Twox64Concat, T::AuctionId, T::AccountId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Auction created
		AuctionCreated(T::AccountId, T::AuctionId),
		/// A bid is placed
		Bid(T::AuctionId, T::AccountId, BalanceOf<T>),
		/// Auction ended
		AuctionConcluded(T::AuctionId),
		/// Auction removed
		AuctionRemoved(T::AuctionId),
	}


	#[pallet::error]
	pub enum Error<T> {
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

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		#[pallet::weight(1000)]
		pub fn create_auction(origin: OriginFor<T>, auction_info: AuctionInfoOf<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let new_auction_id = Self::new_auction(auction_info)?;
			Self::deposit_event(Event::AuctionCreated(sender, new_auction_id));
			Ok(().into())
		}

		#[pallet::weight(1000)]
		pub fn bid_value(origin: OriginFor<T>, id: T::AuctionId, value: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			Self::bid(sender.clone(), id, value)?;
			Self::deposit_event(Event::Bid(id, sender, value));
			Ok(().into())
		}

		#[pallet::weight(1000)]
		pub fn delete_auction(origin: OriginFor<T>, id: T::AuctionId) -> DispatchResultWithPostInfo {
			let _sender = ensure_signed(origin)?;

			Self::remove_auction(id).ok();
			Self::deposit_event(Event::AuctionRemoved(id));
			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_finalize(now: T::BlockNumber) {
			Self::conclude_auction(now);
		}
	}

}

impl<T: Config> Pallet<T> {
	fn conclude_auction(now: T::BlockNumber) {
		for (auction_id, _) in <AuctionEndTime<T>>::drain_prefix(&now) {
			match Self::auctions(auction_id) {
				Some(auction) => {
					pallet_nft::Module::<T>::toggle_lock(&auction.owner, auction.token_id).ok();
					// there is a bid so let's determine a winner and transfer tokens
					if let Some(ref winner) = auction.last_bid {
						let dest = T::Lookup::unlookup(winner.0.clone());
						let source = T::Origin::from(frame_system::RawOrigin::Signed(auction.owner.clone()));
						pallet_nft::Module::<T>::transfer(source, dest, auction.token_id).ok();
						T::Currency::remove_lock(AUCTION_LOCK_ID, &winner.0);
						<T::Currency as Currency<T::AccountId>>::transfer(&winner.0, &auction.owner, winner.1, ExistenceRequirement::KeepAlive).ok();
					}
				}
				None => ()
			}
		}
	}

	fn check_new_auction(info: &AuctionInfoOf<T>) -> DispatchResult {
		let current_block_number = frame_system::Module::<T>::block_number();
		ensure!(info.start >= current_block_number, Error::<T>::AuctionStartTimeAlreadyPassed);
		ensure!(info.start != Zero::zero() && info.end != Zero::zero() && info.end > info.start + MIN_AUCTION_DUR.into(), Error::<T>::InvalidTimeConfiguration);
		ensure!(!info.name.is_empty(), Error::<T>::EmptyAuctionName);
		let is_owner = pallet_nft::Module::<T>::is_owner(&info.owner, info.token_id);
		ensure!(is_owner, Error::<T>::NotATokenOwner);
		let nft_locked = pallet_nft::Module::<T>::is_locked(info.token_id)?;
		ensure!(nft_locked == false, Error::<T>::TokenLocked);
		Ok(())
	}
}

impl<T: Config> Auction<T::AccountId, T::BlockNumber, NftClassIdOf<T>, NftTokenIdOf<T>> for Pallet<T>{
	type AuctionId = T::AuctionId;
	type Balance = BalanceOf<T>;
	type AccountId = T::AccountId;

	fn new_auction(info: AuctionInfoOf<T>) -> result::Result<Self::AuctionId, DispatchError> {
		// Basic checks before an auction is created
		Self::check_new_auction(&info).ok();
		let auction_id = <NextAuctionId<T>>::try_mutate(|next_id| -> result::Result<Self::AuctionId, DispatchError> {
			let current_id = *next_id;
			*next_id = next_id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableAuctionId)?;
			Ok(current_id)
		})?;

		<Auctions<T>>::insert(auction_id, info.clone());
		<AuctionOwnerById<T>>::insert(auction_id, &info.owner);
		<AuctionEndTime<T>>::insert(info.end, auction_id, ());
		pallet_nft::Module::<T>::toggle_lock(&info.owner, info.token_id).ok();

		Ok(auction_id)
	}

	fn update_auction(id: Self::AuctionId, info: AuctionInfoOf<T>) -> DispatchResult {
		<Auctions<T>>::try_mutate(id, |auction| -> DispatchResult {
			ensure!(auction.is_some(), Error::<T>::AuctionNotExist);
			*auction = Some(info);
			Ok(())
		})
	}

	fn remove_auction(id: Self::AuctionId) -> DispatchResult {
		let auction = <Auctions<T>>::take(id).ok_or(Error::<T>::AuctionNotExist)?;
		let current_block_number = frame_system::Module::<T>::block_number();
		ensure!(current_block_number < auction.start, Error::<T>::AuctionAlreadyStarted);
		pallet_nft::Module::<T>::toggle_lock(&auction.owner, auction.token_id).ok();
		<AuctionOwnerById<T>>::remove(id);
		<Auctions<T>>::remove(id);
		Ok(())
	}

	fn bid(bidder: Self::AccountId, id: Self::AuctionId, value: Self::Balance) -> DispatchResult {
		<Auctions<T>>::try_mutate_exists(id, |auction| -> DispatchResult {
			// Basic checks before a bid can be made
			let mut auction = auction.as_mut().ok_or(Error::<T>::AuctionNotExist)?;
			let block_number = <frame_system::Module<T>>::block_number();
			ensure!(bidder != auction.owner, Error::<T>::BidOnOwnAuction);
			ensure!(block_number > auction.start, Error::<T>::AuctionNotStarted);
			ensure!(block_number < auction.end, Error::<T>::AuctionAlreadyConcluded);
			ensure!(value >= auction.minimal_bid, Error::<T>::InvalidBidPrice);
			if let Some(ref current_bid) = auction.last_bid {
				ensure!(value > current_bid.1, Error::<T>::InvalidBidPrice);
				// Unlock funds from the previous bid
				T::Currency::remove_lock(AUCTION_LOCK_ID, &current_bid.0);
			} else {
				ensure!(!value.is_zero(), Error::<T>::InvalidBidPrice);
			}
			// Lock funds
			T::Currency::set_lock(
				AUCTION_LOCK_ID,
				&bidder,
				value,
				WithdrawReasons::all()
			);
			auction.last_bid = Some((bidder, value));
			// Set next minimal bid
			let minimal_bid_step = Permill::from_percent(BID_STEP_PERC).mul_floor(value);
			auction.minimal_bid = value.checked_add(&minimal_bid_step).ok_or(Error::<T>::BidOverflow)?;
			// Avoid auction sniping
			let time_left = auction.end.checked_sub(&block_number).ok_or(Error::<T>::TimeUnderflow)?;
			if time_left < BID_ADD_BLOCKS.into() {
				auction.end = block_number + BID_ADD_BLOCKS.into();
			}
			Ok(())
		})
	}

}