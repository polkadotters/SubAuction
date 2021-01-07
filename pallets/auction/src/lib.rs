#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused)]
use codec::{Decode, Encode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, Parameter, ensure, traits::Get};
use frame_system::ensure_signed;
use orml_traits::{Auction, AuctionHandler, OnNewBidResult, Change, MultiCurrency};
use sp_runtime::{
	traits::{
		MaybeSerializeDeserialize, Member, AtLeast32BitUnsigned, Zero
	},
	DispatchError, RuntimeDebug, DispatchResult
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type AuctionId = u32;
pub type Balance = u32;
pub type TokenId = u32;

pub enum AuctionType {
	English,
	Candle,
	Dutch,
	TopUp,
	FixedSwap,
}
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct EnglishAuction<AccountId, BlockNumber> {
	name: Vec<u8>,
	owner: AccountId,
	last_bid: Option<(AccountId, Balance)>,
	start: BlockNumber,
	end: BlockNumber,
	no_identity_allowed: bool,
	minimal_bid: Balance,
	private: bool,
	price_at_start: Balance,
	auction_item: TokenId,
}
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct DutchAuction<AccountId, BlockNumber> {
	dummy: (AccountId, BlockNumber),
}
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct CandleAuction<AccountId, BlockNumber> {
	dummy: (AccountId, BlockNumber),
}

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;
	type Auction: Auction<Self::AccountId, Self::BlockNumber, AuctionId = AuctionId, Balance = Balance>;
	type Currency: MultiCurrency<Self::AccountId, CurrencyId = TokenId, Balance = Balance>;
}

decl_storage! {
	trait Store for Module<T: Trait> as AuctionStorage {
		pub EnglishAuctions get(fn english_auctions): map hasher(twox_64_concat) AuctionId =>
			Option<EnglishAuction<T::AccountId, T::BlockNumber>>;

		/// Mapping from auction id to dutch auction info
		pub DutchAuctions get(fn dutch_auctions): map hasher(twox_64_concat) AuctionId =>
			Option<DutchAuction<T::AccountId, T::BlockNumber>>;

		/// Mapping from auction id to candle auction info
		pub CandleAuctions get(fn candle_auctions): map hasher(twox_64_concat) AuctionId =>
			Option<CandleAuction<T::AccountId, T::BlockNumber>>;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
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
		AuctionNotExists,
		AuctionNotStarted,
		BidNotAccepted,
		InvalidBidPrice,
		NoAvailableAuctionId,
		AuctionStartAlreadyPassed,
		NonExistingAuctionType,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;
		
	}
}

// Default T trait implementation
impl <T: Trait> Module<T> {
	pub fn english_auction_bid_handler(
		now: T::BlockNumber,
		id: AuctionId,
		new_bid: (T::AccountId, Balance),
		last_bid: Option<(T::AccountId, Balance)>,
	) -> sp_std::result::Result<T::BlockNumber, DispatchError> {
		todo!()
	}

	fn check_minimum_increment(
		new_price: Balance,
		last_price: Balance,
	) -> bool {false}

	fn english_auction_end_handler(
		auction_id: AuctionId,
		english_auction: EnglishAuction<T::AccountId, T::BlockNumber>,
		winner: Option<(T::AccountId, Balance)>,
	) {
		todo!()
	}
}

// AuctionHandler implementation for T (bidding)
impl<T: Trait> AuctionHandler<T::AccountId, Balance, T::BlockNumber, AuctionId> for Module<T>  {
	fn on_new_bid(
		now: T::BlockNumber,
		id: AuctionId,
		new_bid: (T::AccountId, Balance),
		last_bid: Option<(T::AccountId, Balance)>,
	) -> OnNewBidResult<T::BlockNumber> {
		let bid_result = 
		if <EnglishAuctions<T>>::contains_key(id) {
			Self::english_auction_bid_handler(now, id, new_bid, last_bid)
		} else {
			Err(Error::<T>::AuctionNotExists.into())
		};

		match bid_result {
			Ok(new_auction_end_time) => OnNewBidResult {
				accept_bid: true,
				auction_end_change: Change::NewValue(Some(new_auction_end_time)),
			},
			Err(_) => OnNewBidResult {
				accept_bid: false,
				auction_end_change: Change::NoChange,
			},
		}
	}

	fn on_auction_ended(id: AuctionId, winner: Option<(T::AccountId, Balance)>) {
		if let Some(english_auction) = Self::english_auctions(id) {
			Self::english_auction_end_handler(id, english_auction, winner.clone());
		}

		if let Some((bidder, _)) = &winner {
			// decrease account ref of winner
			frame_system::Module::<T>::dec_ref(bidder);
		}
	}
} 