use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch};
use frame_support::sp_runtime::DispatchError;
use frame_system::ensure_signed;

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
		pub fn create_token_class(origin, metadata: Vec<u8>, data: T::ClassData) -> Result<T::ClassId, DispatchError> {
			let sender = ensure_signed(origin)?;

			let result: Result<T::ClassId, DispatchError> = orml_nft::Module::<T>::create_class(&sender, metadata, data);

			Self::deposit_event(RawEvent::NFTTokenClassCreated(sender));
			result
		}

		// #[weight = 1000]
		// pub fn mint_tokens(origin, data: <T as orml_nft::Trait>::ClassData) -> dispatch::DispatchResult {
		// 	let sender = ensure_signer(origin)?;

		// 	let result: Result<T::TokenId, DispatchError> = orml_nft::Module::<T>::mint(&sender, data)
		// }
	}
}

decl_event!(
	pub enum Event<T> where
	AccountId = <T as frame_system::Trait>::AccountId {
		NFTTokenClassCreated(AccountId),
		NFTTokenMinted(AccountId),
	}
);
