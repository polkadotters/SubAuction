use codec::{Encode, Decode};
use frame_support::{traits::{Currency}, Parameter, dispatch::{DispatchResult, DispatchError}};
use sp_runtime::{
	traits::{AtLeast32Bit, AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, MaybeDisplay},
	RuntimeDebug
};
use sp_std::{fmt::{Display, Debug, Formatter}, result, vec::Vec};

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AuctionType {
	English,
	Candle,
	Dutch,
	TopUp,
	FixedSwap,
}

impl Display for AuctionType {
	fn fmt(&self, f: &mut Formatter) -> sp_std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl Default for AuctionType {
	fn default() -> Self { AuctionType::English }
}

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct AuctionInfo<AccountId, Balance, BlockNumber, NftClassId, NFtTokenId> {
	pub name: Vec<u8>,
	pub last_bid: Option<(AccountId, Balance)>,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub auction_type: AuctionType,
	pub token_id: (NftClassId, NFtTokenId),
	pub minimal_bid: Balance,
}

/// Abstraction over a NFT auction system.
pub trait Auction<AccountId, BlockNumber, NftClassId, NftTokenId> {
	/// The id of an AuctionInfo
	type AuctionId: Default + Copy + Eq + PartialEq + MaybeSerializeDeserialize + Bounded + Debug;
	/// The price to bid.
	type Balance: AtLeast32Bit + Copy + MaybeSerializeDeserialize + Debug + Default;
	/// Account id
	type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default;

	/// Create new auction with specific startblock and endblock, return the id
	fn new_auction(sender: &Self::AccountId, info: AuctionInfo<Self::AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>) -> result::Result<Self::AuctionId, DispatchError>;
	/// The auction info of `id`
	fn auction_info(id: Self::AuctionId) -> Option<AuctionInfo<Self::AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>>;
	/// Update the auction info of `id` with `info`
	fn update_auction(id: Self::AuctionId, info: AuctionInfo<Self::AccountId, Self::Balance, BlockNumber, NftClassId, NftTokenId>) -> DispatchResult;
	/// Remove auction by `id`
	fn remove_auction(id: Self::AuctionId) -> DispatchResult;
	/// Bid
	fn bid(bidder: Self::AccountId, id: Self::AuctionId,  value: Self::Balance) -> DispatchResult;
	/// End auction and select the winner
	fn conclude_auction(id: Self::AuctionId) -> DispatchResult;
}