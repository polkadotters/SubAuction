#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_event, decl_module, decl_storage, dispatch::{DispatchError, DispatchResult}};
use frame_system::ensure_signed;
use sp_runtime::traits::StaticLookup;

pub type CID = sp_std::vec::Vec<u8>;

pub trait Trait: frame_system::Trait + orml_nft::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as NftStore {
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event() = default;

		#[weight = 1000]
		pub fn create_token_class(origin, metadata: CID, data: T::ClassData) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let _result: Result<T::ClassId, DispatchError> = orml_nft::Module::<T>::create_class(&sender, metadata, data);

			Self::deposit_event(RawEvent::NFTTokenClassCreated(sender));
			Ok(())
		}

		#[weight = 1000]
		pub fn mint_tokens(origin, class_id: <T as orml_nft::Trait>::ClassId, metadata: CID, 
                           token_data: <T as orml_nft::Trait>::TokenData) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let _result: Result<T::TokenId, DispatchError> = orml_nft::Module::<T>::mint(&sender, class_id, metadata, token_data);

            Self::deposit_event(RawEvent::NFTTokenMinted(sender));
            Ok(())
		}

		#[weight = 1000]
		pub fn transfer_token(origin, dest: <T::Lookup as StaticLookup>::Source, token_class_id: T::ClassId, token_id: T::TokenId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let to: T::AccountId = T::Lookup::lookup(dest)?;
			let _result = orml_nft::Module::<T>::transfer(&sender, &to, (token_class_id, token_id));
			Self::deposit_event(RawEvent::NFTTokenTransferred(sender, to));
			Ok(())
		}

	}
}

decl_event!(
	pub enum Event<T> where
	AccountId = <T as frame_system::Trait>::AccountId {
		NFTTokenClassCreated(AccountId),
		NFTTokenMinted(AccountId),
		NFTTokenTransferred(AccountId, AccountId),
	}
);