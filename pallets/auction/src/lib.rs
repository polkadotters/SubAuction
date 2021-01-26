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
pub type EnglishAuctionInfoOf<T> = EnglishAuction<<T as frame_system::Trait>::AccountId,
																			  BalanceOf<T>,
												  <T as frame_system::Trait>::BlockNumber,
																			NftClassIdOf<T>,
																			NftTokenIdOf<T>,
>;

decl_storage! {
	trait AuctionStore for Module<T: Trait> as AuctionModule {
		/// Stores on-going and future auctions. Closed auction are removed.
		// TODO: use single Auction storage using double map (auctionId, type)
		// TODO: Ask greg
		pub Auctions get(fn auctions): map hasher(twox_64_concat) T::AuctionId => Option<EnglishAuctionInfoOf<T>>;

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
		BidOnOwnAuction,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		// TODO general idea is to have mandatory auction attributes as parameters and optional config as a single struct. Each auction type would then construct itself from the optional params
		#[weight=0]
		fn create_auction(origin, name: Vec<u8>, start: T::BlockNumber, end: T::BlockNumber, token_id: (NftClassIdOf<T>, NftTokenIdOf<T>), auction_type: AuctionType, config: OptionalConfig<T::Balance> ) {
			let sender = ensure_signed(origin)?;

			let new_auction_id = Self::create_auction_by_type(&sender, name, start, end, token_id, auction_type, config);
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

// TODO general purpose code to be re-used across different auction types
impl<T: Trait> Module<T> {

	fn validate_auction(owner: T::AccountId, name: &Vec<u8>, start: T::BlockNumber, end: T::BlockNumber, token_id: (NftClassIdOf<T>, NftTokenIdOf<T>)) -> DispatchResult {
 		let current_block_number = frame_system::Module::<T>::block_number();
		ensure!(start != Zero::zero() && end != Zero::zero() && !name.is_empty(), Error::<T>::BadAuctionConfiguration);
		ensure!(start >= current_block_number, Error::<T>::AuctionStartTimeAlreadyPassed);
		ensure!(start < end, Error::<T>::BadAuctionConfiguration);
 		let is_owner = pallet_nft::Module::<T>::is_owner(&owner, token_id);
 		ensure!(is_owner, Error::<T>::NotATokenOwner);

	}

	// TODO this one would create a specific auction with its configuration and return its id
	fn create_auction_by_type(owner: T::AccountId, name: Vec<u8>, start: T::BlockNumber, end: T::BlockNumber, token_id: (NftClassIdOf<T>, NftTokenIdOf<T>), auction_type: AuctionType,
					  config: OptionalConfig<T::Balance>) -> result::Result<T::AuctionId, DispatchError> {
		Self::validate_auction(&owner, &name, start, end, token_id);
		match auction_type {
			EnglishAuction=> {
				 auction = EnglishAuctionInfoOf::<T> {
					name, last_bid: None, start, end, token_id, auction_type, config,
				 };
				// TODO how to make this work, is a good idea to call the code on self???
				auction.new_auction(owner);
				Zero::zero()
			}
			_ => DispatchError::Module {
				error: 0,
				index: 0,
				message: Some("Non-existing auction type provided"),
			}
		}
	}
}

// TODO the goal is to have different implementations of this interface for different structs representing the auctions
// This one works
// impl<T: Trait> Auction<T::AccountId, T::BlockNumber, NftClassIdOf<T>, NftTokenIdOf<T>> for Module<T>{
// This one not
impl<T: Trait> Auction<T::AccountId, T::BlockNumber, NftClassIdOf<T>, NftTokenIdOf<T>> for EnglishAuctionInfoOf<T>{
	type AuctionId = T::AuctionId;
	type Balance = BalanceOf<T>;
	type AccountId = T::AccountId;

	fn new_auction(self, owner: &Self::AccountId) -> result::Result<Self::AuctionId, DispatchError> {
		let auction_id = <NextAuctionId<T>>::try_mutate(|next_id| -> result::Result<Self::AuctionId, DispatchError> {
			let current_id = *next_id;
			*next_id = next_id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableAuctionId)?;
			Ok(current_id)
		})?;

		<Auctions<T>>::insert(auction_id, self);
		<AuctionOwnerById<T>>::insert(auction_id, owner);

		Ok(auction_id)
	}

	fn auction_info(id: Self::AuctionId) -> Option<EnglishAuctionInfoOf<T>> {
		Self::auctions(id)
	}

	fn update_auction(id: Self::AuctionId, info: EnglishAuctionInfoOf<T>) -> DispatchResult {
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
