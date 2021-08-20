use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},

};
use sp_runtime::{
	traits::{AtLeast32Bit, Bounded, MaybeSerializeDeserialize},
	RuntimeDebug,
};
use sp_std::{
	fmt::Debug,
	result,
	vec::Vec,
};

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct EnglishAuctionInfo<AccountId, Balance, BlockNumber, NftClassId, NFtTokenId> {
	// Common fields for all auction types
	pub name: Vec<u8>,
	pub last_bid: Option<(AccountId, Balance)>,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub owner: AccountId,
	pub token_id: (NftClassId, NFtTokenId),
	pub minimal_bid: Balance,
	// Custom fields for English auction
}

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct CandleAuctionInfo<AccountId, Balance, BlockNumber, NftClassId, NFtTokenId> {
	// Common fields for all auction types
	pub name: Vec<u8>,
	pub last_bid: Option<(AccountId, Balance)>,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub owner: AccountId,
	pub token_id: (NftClassId, NFtTokenId),
	pub minimal_bid: Balance,
	// Custom fields for Candle auction
}

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct TopUpAuctionInfo<AccountId, Balance, BlockNumber, NftClassId, NFtTokenId> {
	pub name: Vec<u8>,
	pub last_bid: Option<(AccountId, Balance)>,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub owner: AccountId,
	pub token_id: (NftClassId, NFtTokenId),
	pub minimal_bid: Balance,
	// Custom fields for TopUp auction
}

/// Abstraction over the English auction
pub trait EnglishAuction<AccountId, BlockNumber, NftClassId, NftTokenId> {
	/// The id of an AuctionInfo
	type AuctionId: Default + Copy + Eq + PartialEq + MaybeSerializeDeserialize + Bounded + Debug;
	/// The price to bid.
	type Balance: AtLeast32Bit + Copy + MaybeSerializeDeserialize + Debug + Default;

	/// Create new auction with specific startblock and endblock, return the id
	fn new_auction(
		info: EnglishAuctionInfo<AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>,
	) -> result::Result<Self::AuctionId, DispatchError>;
	/// Update the auction info of `id` with `info`
	fn update_auction(
		id: Self::AuctionId,
		info: EnglishAuctionInfo<AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>,
	) -> DispatchResult;
	/// Remove auction by `id`
	fn remove_auction(id: Self::AuctionId) -> DispatchResult;
	/// Bid
	fn bid(bidder: AccountId, id: Self::AuctionId, value: Self::Balance) -> DispatchResult;
	/// Conclude
	fn conclude_auction(now: BlockNumber);
	/// Check if newly created auction is valid
	fn check_new_auction(info: EnglishAuctionInfo<AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>) -> DispatchResult;
}

/// Abstraction over the Candle auction
pub trait CandleAuction<AccountId, BlockNumber, NftClassId, NftTokenId> {
	/// The id of an AuctionInfo
	type AuctionId: Default + Copy + Eq + PartialEq + MaybeSerializeDeserialize + Bounded + Debug;
	/// The price to bid.
	type Balance: AtLeast32Bit + Copy + MaybeSerializeDeserialize + Debug + Default;

	/// Create new auction with specific startblock and endblock, return the id
	fn new_auction(
		info: CandleAuctionInfo<AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>,
	) -> result::Result<Self::AuctionId, DispatchError>;
	/// Update the auction info of `id` with `info`
	fn update_auction(
		id: Self::AuctionId,
		info: CandleAuctionInfo<AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>,
	) -> DispatchResult;
	/// Remove auction by `id`
	fn remove_auction(id: Self::AuctionId) -> DispatchResult;
	/// Bid
	fn bid(bidder: AccountId, id: Self::AuctionId, value: Self::Balance) -> DispatchResult;
	/// Conclude
	fn conclude_auction(now: BlockNumber);
	/// Check if newly created auction is valid
	fn check_new_auction(info: CandleAuctionInfo<AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>) -> DispatchResult;
}

/// Abstraction over the TopUp auction
pub trait TopUpAuction<AccountId, BlockNumber, NftClassId, NftTokenId> {
	/// The id of an AuctionInfo
	type AuctionId: Default + Copy + Eq + PartialEq + MaybeSerializeDeserialize + Bounded + Debug;
	/// The price to bid.
	type Balance: AtLeast32Bit + Copy + MaybeSerializeDeserialize + Debug + Default;

	/// Create new auction with specific startblock and endblock, return the id
	fn new_auction(
		info: TopUpAuctionInfo<AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>,
	) -> result::Result<Self::AuctionId, DispatchError>;
	/// Update the auction info of `id` with `info`
	fn update_auction(
		id: Self::AuctionId,
		info: TopUpAuctionInfo<AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>,
	) -> DispatchResult;
	/// Remove auction by `id`
	fn remove_auction(id: Self::AuctionId) -> DispatchResult;
	/// Bid
	fn bid(bidder: AccountId, id: Self::AuctionId, value: Self::Balance) -> DispatchResult;
	/// Conclude
	fn conclude_auction(now: BlockNumber);
	/// Check
	fn check_new_auction(info: TopUpAuctionInfo<AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>) -> DispatchResult;
}
