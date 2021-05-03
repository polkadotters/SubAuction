#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
	ensure,
};
use frame_system::ensure_signed;
use sp_runtime::{
	traits::{StaticLookup, Zero},
	RuntimeDebug,
};
use sp_std::vec::Vec;
use weights::WeightInfo;

mod benchmarking;
pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type ClassData = u32;
pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
pub type GenesisTokenData<AccountId, TokenData> = (
	AccountId, // Token owner
	Vec<u8>,   // Token metadata
	TokenData,
);
pub type GenesisTokens<AccountId, ClassData, TokenData> = (
	AccountId, // Token class owner
	Vec<u8>,   // Token class metadata
	ClassData,
	Vec<GenesisTokenData<AccountId, TokenData>>, // Vector of tokens belonging to this class
);

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
pub struct TokenData {
	pub locked: bool,
}

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + orml_nft::Config<ClassData = ClassData, TokenData = TokenData> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub tokens:
			Vec<GenesisTokens<T::AccountId, <T as orml_nft::Config>::ClassData, <T as orml_nft::Config>::TokenData>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { tokens: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.tokens.iter().for_each(|token_class| {
				let class_id =
					orml_nft::Module::<T>::create_class(&token_class.0, token_class.1.to_vec(), token_class.2)
						.expect("Create class cannot fail while building genesis");
				for (account_id, token_metadata, token_data) in &token_class.3 {
					orml_nft::Module::<T>::mint(&account_id, class_id, token_metadata.to_vec(), token_data.clone())
						.expect("Token mint cannot fail during genesis");
				}
			})
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::create_class())]
		pub fn create_class(origin: OriginFor<T>, metadata: Vec<u8>, data: T::ClassData) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let _result = orml_nft::Module::<T>::create_class(&sender, metadata, data)?;
			Self::deposit_event(Event::NFTTokenClassCreated(sender));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: <T as orml_nft::Config>::ClassId,
			metadata: Vec<u8>,
			token_data: TokenData,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassNotFound)?;
			ensure!(sender == class_info.owner, Error::<T>::NoPermission);
			let mut cloned_data = token_data;
			cloned_data.locked = false;
			let token_id = orml_nft::Module::<T>::mint(&sender, class_id, metadata, cloned_data)?;
			Self::deposit_event(Event::NFTTokenMinted(sender, class_id, token_id));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			token: (T::ClassId, T::TokenId),
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let _class_info = orml_nft::Module::<T>::classes(token.0).ok_or(Error::<T>::ClassNotFound)?;
			let token_info = orml_nft::Module::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
			ensure!(sender == token_info.owner, Error::<T>::NoPermission);
			ensure!(!token_info.data.locked, Error::<T>::TokenLocked);
			let to: T::AccountId = T::Lookup::lookup(dest)?;
			let _result = orml_nft::Module::<T>::transfer(&sender, &to, token);
			Self::deposit_event(Event::NFTTokenTransferred(sender, to, token.0, token.1));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::burn())]
		pub fn burn(origin: OriginFor<T>, token: (T::ClassId, T::TokenId)) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let _class_info = orml_nft::Module::<T>::classes(token.0).ok_or(Error::<T>::ClassNotFound)?;
			let token_info = orml_nft::Module::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
			ensure!(sender == token_info.owner, Error::<T>::NoPermission);
			ensure!(!token_info.data.locked, Error::<T>::TokenLocked);
			let _result = orml_nft::Module::<T>::burn(&sender, token)?;
			Self::deposit_event(Event::NFTTokenBurned(sender, token.1));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::destroy_class())]
		pub fn destroy_class(origin: OriginFor<T>, class_id: T::ClassId) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassNotFound)?;
			ensure!(sender == class_info.owner, Error::<T>::NoPermission);
			ensure!(
				class_info.total_issuance == Zero::zero(),
				Error::<T>::CannotDestroyClass
			);
			let _result = orml_nft::Module::<T>::destroy_class(&sender, class_id)?;
			Self::deposit_event(Event::NFTTokenClassDestroyed(sender, class_id));
			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		NFTTokenClassCreated(T::AccountId),
		NFTTokenMinted(T::AccountId, T::ClassId, T::TokenId),
		NFTTokenMintedLockToggled(T::AccountId, T::ClassId, T::TokenId, bool),
		NFTTokenTransferred(T::AccountId, T::AccountId, T::ClassId, T::TokenId),
		NFTTokenBurned(T::AccountId, T::TokenId),
		NFTTokenClassDestroyed(T::AccountId, T::ClassId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// ClassId not found
		ClassNotFound,
		/// TokenId not found
		TokenNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Can not destroy class. Total issuance is not 0.
		CannotDestroyClass,
		/// Token locked
		TokenLocked,
	}
}

impl<T: Config> Pallet<T> {
	pub fn is_owner(account: &T::AccountId, token: (T::ClassId, T::TokenId)) -> bool {
		orml_nft::Module::<T>::is_owner(account, token)
	}

	pub fn is_locked(token: (T::ClassId, T::TokenId)) -> Result<bool, DispatchError> {
		let token_info = orml_nft::Module::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
		Ok(token_info.data.locked)
	}

	pub fn toggle_lock(account: &T::AccountId, token_id: (T::ClassId, T::TokenId)) -> DispatchResult {
		let _class_info = orml_nft::Module::<T>::classes(token_id.0).ok_or(Error::<T>::ClassNotFound)?;
		orml_nft::Tokens::<T>::mutate_exists(token_id.0, token_id.1, |token| -> DispatchResult {
			if let Some(ref mut token) = token {
				ensure!(*account == token.owner, Error::<T>::NoPermission);
				token.data.locked ^= true; // Toggle
						   // fix clone
				Self::deposit_event(Event::NFTTokenMintedLockToggled(
					account.clone(),
					token_id.0,
					token_id.1,
					token.data.locked,
				));
			}
			Ok(())
		})?;
		Ok(())
	}
}
