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

pub struct SubHandler;

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct AuctionInfo<AccountId, Balance, BlockNumber> {
	pub name: Vec<u8>,
	pub owner: AccountId,
	pub bid: Option<(AccountId, Balance)>,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub auction_type: AuctionType,
	// I suppose token that needs to be auctioned needs to be passed here as well - with some checks afterwards

	// auction configuration
	pub no_identity_allowed: bool,
	pub minimal_bid: Balance,
	pub private: bool,
	pub max_participants: u128,
}

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct EnglishAuctionInfo<AccountId, Balance, BlockNumber> {
	pub name: Vec<u8>,
	pub owner: AccountId,
	pub bid: Option<(AccountId, Balance)>,
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub auction_type: AuctionType,
	// I suppose token that needs to be auctioned needs to be passed here as well - with some checks afterwards

	// auction configuration
	pub no_identity_allowed: bool,
	pub minimal_bid: Balance,
	pub private: bool,
	pub max_participants: u128,
}

pub trait Auction<AccountId, Balance, BlockNumber> {
	type AuctionId: Default + Copy + Eq + PartialEq + MaybeSerializeDeserialize + Bounded + Debug;
	type Balance: AtLeast32Bit + Copy + MaybeSerializeDeserialize + Debug + Default;
	type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default;

	fn new_auction(&self, info: AuctionInfo<AccountId, Self::Balance, BlockNumber>) -> result::Result<Self::AuctionId, DispatchError>; // Ok(1) or Error()
	/// The auction info of `id`
	fn auction_info(id: Self::AuctionId) -> Option<AuctionInfo<AccountId, Self::Balance, BlockNumber>>;
	/// Update the auction info of `id` with `info`
	fn update_auction(id: Self::AuctionId, info: AuctionInfo<AccountId, Self::Balance, BlockNumber>) -> DispatchResult; // special type of Result - Ok(()), DispatchError
	/// Remove auction by `id`
	fn remove_auction(id: Self::AuctionId);
}
