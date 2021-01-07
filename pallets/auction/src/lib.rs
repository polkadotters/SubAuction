#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused)]
// Used for encoding/decoding into scale
use codec::{Encode, Decode};
use frame_support::{traits::Currency, Parameter, decl_error, decl_event, decl_module, decl_storage, ensure, dispatch::{DispatchResult}};
use frame_system::ensure_signed;
use sp_runtime::{
	traits::{AtLeast32Bit, AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, MaybeDisplay},
	DispatchError, RuntimeDebug,
};
use sp_std::{fmt::Debug, result, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

decl_storage! {
	trait AuctionStore for Module<T: Trait> as AuctionModule {
		/// Stores on-going and future auctions. Closed auction are removed.
		pub Auctions get(fn auctions): double_map hasher(twox_64_concat) T::AuctionId, hasher(twox_64_concat) AuctionType => Option<AuctionInfo<T::AccountId, T::Balance, T::BlockNumber>>;

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
		AuctionNotExist,
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

		#[weight=0]
		fn create_auction(origin, auction_info: AuctionInfo<T::AccountId, T::Balance, T::BlockNumber>) {
			let sender = ensure_signed(origin)?;

			//  match auction_info.auction_type {
			// 	AuctionType::English => {
			// 		let english_auction = EnglishAuction::<T> {default_auction: CommonAuction { t: T as frame_system::Trait}};
			// 		english_auction.new_auction(auction_info)?;
			//
			// 	}
			// 	_ => Error::<T>::NonExistingAuctionType,
			// }
		}
	}
}

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
pub struct CommonAuction<T> {
	t: T
}
pub struct EnglishAuction<T> {
	default_auction: CommonAuction<T>
}
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

	// Currency
	//type Currency: Currency<Self::AccountId>;
}

pub trait Auction<AccountId, BlockNumber> {
	/// The id of an AuctionInfo
	type AuctionId: Default + Copy + Eq + PartialEq + MaybeSerializeDeserialize + Bounded + Debug;
	/// The price to bid.
	type Balance: AtLeast32Bit + Copy + MaybeSerializeDeserialize + Debug + Default;
	// Account Id
	type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default;

	fn new_auction(&self, info: AuctionInfo<AccountId, Self::Balance, BlockNumber>) -> result::Result<Self::AuctionId, DispatchError>;
	/// The auction info of `id`
	fn auction_info(id: Self::AuctionId) -> Option<AuctionInfo<AccountId, Self::Balance, BlockNumber>>;
	/// Update the auction info of `id` with `info`
	fn update_auction(id: Self::AuctionId, info: AuctionInfo<AccountId, Self::Balance, BlockNumber>) -> DispatchResult;
	/// Remove auction by `id`
	fn remove_auction(id: Self::AuctionId);
}

impl<T: Trait> Auction<T::AccountId, T::BlockNumber> for CommonAuction<T> {
	type AuctionId = T::AuctionId;
	type Balance = T::Balance;
	type AccountId = T::AccountId;

	fn new_auction(&self, info: AuctionInfo<Self::AccountId, Self::Balance, T::BlockNumber>) -> result::Result<Self::AuctionId, DispatchError> {
		let current_block_number = frame_system::Module::<T>::block_number();
		ensure!(info.start <= current_block_number, Error::<T>::AuctionStartAlreadyPassed);
		let auction_id = <AuctionsIndex<T>>::try_mutate(|x| -> result::Result<Self::AuctionId, DispatchError> {
			let id = *x;
			ensure!(id != T::AuctionId::max_value(), Error::<T>::NoAvailableAuctionId);
			*x += One::one();
			Ok(id)
		})?;
		<Auctions<T>>::insert(auction_id, info.auction_type, info);
		Ok(auction_id)
	}

	fn auction_info(id: Self::AuctionId) -> Option<AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber>> {
		unimplemented!()
	}

	fn update_auction(id: Self::AuctionId, info: AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber>) -> DispatchResult {
		unimplemented!()
	}

	fn remove_auction(id: Self::AuctionId) {
		unimplemented!()
	}
}

impl<T: Trait> Auction<T::AccountId, T::BlockNumber> for EnglishAuction<T> {
	type AuctionId = T::AuctionId;
	type Balance = T::Balance;
	type AccountId = T::AccountId;

	fn new_auction(&self, info: AuctionInfo<Self::AccountId, Self::Balance, T::BlockNumber>) -> result::Result<Self::AuctionId, DispatchError> {
		self.default_auction.new_auction(info)
	}

	fn auction_info(id: Self::AuctionId) -> Option<AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber>> {
		unimplemented!()
	}

	fn update_auction(id: Self::AuctionId, info: AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber>) -> DispatchResult {
		unimplemented!()
	}

	fn remove_auction(id: Self::AuctionId) {
		unimplemented!()
	}
}
