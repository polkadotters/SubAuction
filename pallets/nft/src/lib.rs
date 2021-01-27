#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{decl_event, decl_module, decl_storage, decl_error, ensure, dispatch::{DispatchResult}};
use frame_system::ensure_signed;
use sp_runtime::{traits::{StaticLookup, Zero}, RuntimeDebug};
use sp_std::vec::Vec;
use serde::{Serialize, Deserialize};

pub type CID = Vec<u8>;

pub trait Trait: orml_nft::Trait<TokenData = TokenData> {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

pub type TokenIdOf<T> = <T as orml_nft::Trait>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Trait>::ClassId;
pub type GenesisTokenData<AccountId, TokenData> = (
	AccountId, // Token owner
	Vec<u8>,                               // Token metadata
	TokenData,
);
pub type GenesisTokens<AccountId, ClassData, TokenData> = (
	AccountId, // Token class owner
	Vec<u8>,                               // Token class metadata
	ClassData,
	Vec<GenesisTokenData<AccountId, TokenData>>, // Vector of tokens belonging to this class
);

// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenData {
	pub locked: bool,
}

decl_storage! {
	trait Store for Module<T: Trait> as Nft {
	}
	add_extra_genesis {
		config(tokens): Vec<GenesisTokens<T::AccountId, <T as orml_nft::Trait>::ClassData, <T as orml_nft::Trait>::TokenData>>;

		build(|config: &GenesisConfig<T>| {
			config.tokens.iter().for_each(|token_class| {
				let class_id = orml_nft::Module::<T>::create_class(&token_class.0, token_class.1.to_vec(), token_class.2.clone())
					.expect("Create class cannot fail while building genesis");
				for (account_id, token_metadata, token_data) in &token_class.3 {
					orml_nft::Module::<T>::mint(&account_id, class_id, token_metadata.to_vec(), token_data.clone())
						.expect("Token mint cannot fail during genesis");
				}
			})
		})
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 1000]
		pub fn create_class(origin, metadata: CID, data: T::ClassData) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let _result = orml_nft::Module::<T>::create_class(&sender, metadata, data)?;
			Self::deposit_event(RawEvent::NFTTokenClassCreated(sender));
			Ok(())
		}

		#[weight = 1000]
		pub fn mint(origin, class_id: <T as orml_nft::Trait>::ClassId, metadata: CID, token_data: TokenData) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassNotFound)?;
			ensure!(sender == class_info.owner, Error::<T>::NoPermission);
			let mut cloned_data = token_data.clone();
			cloned_data.locked = false;
			let token_id = orml_nft::Module::<T>::mint(&sender, class_id, metadata, cloned_data)?;
            Self::deposit_event(RawEvent::NFTTokenMinted(sender, class_id, token_id));
            Ok(())
		}

		#[weight = 1000]
		pub fn transfer(origin, dest: <T::Lookup as StaticLookup>::Source, token: (T::ClassId, T::TokenId)) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let _class_info = orml_nft::Module::<T>::classes(token.0).ok_or(Error::<T>::ClassNotFound)?;
			let token_info = orml_nft::Module::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
			ensure!(sender == token_info.owner, Error::<T>::NoPermission);
			ensure!(token_info.data.locked != true, Error::<T>::TokenLocked);
			let to: T::AccountId = T::Lookup::lookup(dest)?;
			let _result = orml_nft::Module::<T>::transfer(&sender, &to, token);
			Self::deposit_event(RawEvent::NFTTokenTransferred(sender, to, token.0, token.1));
			Ok(())
		}

		#[weight = 1000]
		pub fn burn(origin, token: (T::ClassId, T::TokenId)) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let _class_info = orml_nft::Module::<T>::classes(token.0).ok_or(Error::<T>::ClassNotFound)?;
			let token_info = orml_nft::Module::<T>::tokens(token.0, token.1).ok_or(Error::<T>::TokenNotFound)?;
			ensure!(sender == token_info.owner, Error::<T>::NoPermission);
			ensure!(token_info.data.locked != true, Error::<T>::TokenLocked);
			let _result = orml_nft::Module::<T>::burn(&sender, token)?;
			Self::deposit_event(RawEvent::NFTTokenBurned(sender, token.1));
			Ok(())
		}

		#[weight = 1000]
		pub fn destroy_class(origin, class_id: T::ClassId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let class_info = orml_nft::Module::<T>::classes(class_id).ok_or(Error::<T>::ClassNotFound)?;
			ensure!(sender == class_info.owner, Error::<T>::NoPermission);
			ensure!(class_info.total_issuance == Zero::zero(), Error::<T>::CannotDestroyClass);
			let _result = orml_nft::Module::<T>::destroy_class(&sender, class_id)?;
			Self::deposit_event(RawEvent::NFTTokenClassDestroyed(sender, class_id));
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	pub fn is_owner(account: &T::AccountId, token: (T::ClassId, T::TokenId)) -> bool {
		orml_nft::Module::<T>::is_owner(account, token)
	}

	pub fn toggle_lock(account: &T::AccountId, token_id: (T::ClassId, T::TokenId)) -> DispatchResult {
		let _class_info = orml_nft::Module::<T>::classes(token_id.0).ok_or(Error::<T>::ClassNotFound)?;
		orml_nft::Tokens::<T>::mutate_exists(token_id.0, token_id.1, |token| -> DispatchResult {
			if let Some(ref mut token) = token {
				ensure!(*account == token.owner, Error::<T>::NoPermission);
				token.data.locked ^= true; // Toggle
				// fix clone
				Self::deposit_event(RawEvent::NFTTokenMintedLockToggled(account.clone(), token_id.0, token_id.1, token.data.locked));
			}
			Ok(())
		})?;
		Ok(())
		// waaaaaaaaaaaaaaaat kurva
	}
}

decl_event!(
	pub enum Event<T> where
	AccountId = <T as frame_system::Trait>::AccountId,
	ClassId = <T as orml_nft::Trait>::ClassId,
	TokenId = <T as orml_nft::Trait>::TokenId {
		NFTTokenClassCreated(AccountId),
		NFTTokenMinted(AccountId, ClassId, TokenId),
		NFTTokenMintedLockToggled(AccountId, ClassId, TokenId, bool),
		NFTTokenTransferred(AccountId, AccountId, ClassId, TokenId),
		NFTTokenBurned(AccountId, TokenId), NFTTokenClassDestroyed(AccountId, ClassId),
	}
);

decl_error! {
	/// Error for module-nft module.
	pub enum Error for Module<T: Trait> {
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
