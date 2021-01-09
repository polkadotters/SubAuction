use codec::{Encode, Decode};
use frame_support::{traits::{Currency}, Parameter, dispatch::{DispatchResult, DispatchError}};
use sp_runtime::{
	traits::{AtLeast32Bit, AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, MaybeDisplay},
	RuntimeDebug
};
use sp_std::{fmt::Debug, result, vec::Vec};

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq)]
pub enum AuctionType {
	English,
	Candle,
	Dutch,
	TopUp,
	FixedSwap,
}

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct AuctionInfo<AccountId, Balance, BlockNumber> {
	// Common fields for every auction
	pub name: Vec<u8>,
	pub owner: AccountId,
	pub last_bid: Option<(AccountId, Balance)>,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub auction_type: AuctionType,
	pub no_identity_allowed: bool,
	pub starting_price: Balance,
	pub private: bool,
	pub max_participants: u32,
}

/// Abstraction over a simple auction system.
pub trait Auction<AccountId, BlockNumber> {
	/// The id of an AuctionInfo
	type AuctionId: Default + Copy + Eq + PartialEq + MaybeSerializeDeserialize + Bounded + Debug;
	/// The price to bid.
	type Balance: AtLeast32Bit + Copy + MaybeSerializeDeserialize + Debug + Default;
	/// Account id
	type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default;

	/// Create new auction with specific startblock and endblock, return the id
	fn new_auction(info: AuctionInfo<AccountId, Self::Balance, BlockNumber>) -> result::Result<Self::AuctionId, DispatchError>;
	/// The auction info of `id`
	fn auction_info(id: Self::AuctionId) -> Option<AuctionInfo<AccountId, Self::Balance, BlockNumber>>;
	/// Update the auction info of `id` with `info`
	fn update_auction(id: Self::AuctionId, info: AuctionInfo<AccountId, Self::Balance, BlockNumber>) -> DispatchResult;
	/// Remove auction by `id`
	fn remove_auction(id: Self::AuctionId);
}