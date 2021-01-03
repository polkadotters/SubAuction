#![cfg_attr(not(feature = "std"), no_std)]

// Used for encoding/decoding into scale
// FullCodec useful for genericcs - A marker trait that tells the compiler that a type encode to the same representation as another type
use codec::{FullCodec, Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, Parameter,};
use frame_system::ensure_signed;
use sp_runtime::{
	traits::{AtLeast32Bit, AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member},
	DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::{result,
			 fmt::Debug};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub struct SubHandler;

/*
Parameter - can be used in Dispatchable function (so in the decl_module!)
Member - can be used in the runtime structures
Default - gives trait variables default values (bool = false, int = 0, etc)
Copy - value can be duplicated by simply copying the bits
MaybeSerializeDeserialize - type that implements Serialize, DeserializeOwned and Debug when in std environment.
Bounded - numbers which have upper and lower bounds (so, basically all primitive types???)
 */

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The balance type for bidding
	type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

	/// The auction ID type
	type AuctionId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;

	/// The `AuctionHandler` that allow custom bidding logic and handles auction
	/// result
	type Handler: AuctionHandler<Self::AccountId, Self::Balance, Self::BlockNumber, Self::AuctionId>;
}

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq)]
pub enum AuctionType {
	English,
	Candle,
	Dutch,
	TopUp,
	FixedSwap,
}

#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, RuntimeDebug)]
pub struct AuctionInfo<AccountId, Balance, BlockNumber> {
	pub bid: Option<(AccountId, Balance)>,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub auction_type: AuctionType,

	// auction configuration
	pub no_identity_allowed: bool,
	pub minimal_bid: Balance,
	pub private: bool,
	pub max_participants: u128,
}

pub trait Auction<AccountId, BlockNumber> {
	/// The id of an AuctionInfo
	type AuctionId: FullCodec + Default + Copy + Eq + PartialEq + MaybeSerializeDeserialize + Bounded + Debug;
	/// The price to bid.
	type Balance: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	/// The auction info of `id`
	fn auction_info(id: Self::AuctionId) -> Option<AuctionInfo<AccountId, Self::Balance, BlockNumber>>;
	/// Update the auction info of `id` with `info`
	fn update_auction(id: Self::AuctionId, info: AuctionInfo<AccountId, Self::Balance, BlockNumber>) -> DispatchResult;
	/// Create new auction with specific startblock and endblock, return the id
	/// of the auction
	fn new_auction(start: BlockNumber, end: Option<BlockNumber>) -> result::Result<Self::AuctionId, DispatchError>;
	/// Remove auction by `id`
	fn remove_auction(id: Self::AuctionId);
}

/// The result of bid handling.
pub struct OnNewBidResult {
	/// Indicates if the bid was accepted
	pub accept_bid: bool,
}

/// Hooks for auction to handle bids.
pub trait AuctionHandler<AccountId, Balance, BlockNumber, AuctionId> {
	/// Called when new bid is received.
	/// The return value determines if the bid should be accepted and update
	/// auction end time. Implementation should reserve money from current
	/// winner and refund previous winner.
	fn on_new_bid(
		now: BlockNumber,
		id: AuctionId,
		new_bid: (AccountId, Balance),
		last_bid: Option<(AccountId, Balance)>,
	) -> OnNewBidResult;
	/// End an auction with `winner`
	fn on_auction_ended(id: AuctionId, winner: Option<(AccountId, Balance)>);
}

decl_storage! {

	trait AuctionStore for Module<T: Trait> as AuctionModule {
		/// Stores on-going and future auctions. Closed auction are removed.
		pub Auctions get(fn auctions): map hasher(twox_64_concat) T::AuctionId => Option<AuctionInfo<T::AccountId, T::Balance, T::BlockNumber>>;

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
		Created(AccountId, AccountId),
		/// A bid is placed
		Bid(AuctionId, AccountId, Balance),
		//
		Concluded(AuctionId),
	}
);

decl_error! {
	/// Error for auction module.
	pub enum Error for Module<T: Trait> {
		AuctionNotExist,
		AuctionNotStarted,
		BidNotAccepted,
		InvalidBidPrice,
		NoAvailableAuctionId,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;
	}
}

impl <AccountId, Balance, BlockNumber, AuctionId> AuctionHandler<AccountId, Balance, BlockNumber, AuctionId> for SubHandler {
	fn on_new_bid(
		now: BlockNumber,
		id: AuctionId,
		new_bid: (AccountId, Balance),
		last_bid: Option<(AccountId, Balance)>,
	) -> OnNewBidResult {
		unimplemented!();
	}

	fn on_auction_ended(id: AuctionId, winner: Option<(AccountId, Balance)>) {
		unimplemented!();
	}
}
