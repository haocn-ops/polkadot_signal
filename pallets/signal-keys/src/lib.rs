#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		
		#[pallet::constant]
		type MaxOneTimePrekeys: Get<u32>;
		
		#[pallet::constant]
		type MaxKeySize: Get<u32>;
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct IdentityKeyBundle<T: Config> {
		pub identity_key: BoundedVec<u8, T::MaxKeySize>,
		pub signed_prekey: BoundedVec<u8, T::MaxKeySize>,
		pub prekey_signature: BoundedVec<u8, T::MaxKeySize>,
		pub registration_block: BlockNumberFor<T>,
		pub updated_at: BlockNumberFor<T>,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct OneTimePrekey<T: Config> {
		pub key_id: u32,
		pub public_key: BoundedVec<u8, T::MaxKeySize>,
	}

	#[pallet::storage]
	#[pallet::getter(fn identity_keys)]
	pub type IdentityKeys<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		IdentityKeyBundle<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn one_time_prekeys)]
	pub type OneTimePrekeys<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		u32,
		OneTimePrekey<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn prekey_counter)]
	pub type PrekeyCounter<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn key_bundle_count)]
	pub type KeyBundleCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		IdentityRegistered { account: T::AccountId },
		IdentityUpdated { account: T::AccountId },
		OneTimePrekeysAdded { account: T::AccountId, count: u32 },
		OneTimePrekeyUsed { account: T::AccountId, key_id: u32 },
		IdentityRemoved { account: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		IdentityKeyTooLarge,
		SignedPrekeyTooLarge,
		SignatureTooLarge,
		OneTimePrekeyTooLarge,
		TooManyOneTimePrekeys,
		IdentityNotFound,
		OneTimePrekeyNotFound,
		InvalidPrekeySignature,
		NoOneTimePrekeysAvailable,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn register_identity(
			origin: OriginFor<T>,
			identity_key: Vec<u8>,
			signed_prekey: Vec<u8>,
			prekey_signature: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let identity_key: BoundedVec<u8, T::MaxKeySize> = identity_key
				.try_into()
				.map_err(|_| Error::<T>::IdentityKeyTooLarge)?;

			let signed_prekey: BoundedVec<u8, T::MaxKeySize> = signed_prekey
				.try_into()
				.map_err(|_| Error::<T>::SignedPrekeyTooLarge)?;

			let prekey_signature: BoundedVec<u8, T::MaxKeySize> = prekey_signature
				.try_into()
				.map_err(|_| Error::<T>::SignatureTooLarge)?;

			let current_block = frame_system::Pallet::<T>::block_number();

			let is_new = !IdentityKeys::<T>::contains_key(&who);

			let bundle = IdentityKeyBundle {
				identity_key,
				signed_prekey,
				prekey_signature,
				registration_block: current_block,
				updated_at: current_block,
			};

			IdentityKeys::<T>::insert(&who, bundle);

			if is_new {
				KeyBundleCount::<T>::mutate(|c| *c = c.saturating_add(1));
				Self::deposit_event(Event::IdentityRegistered { account: who });
			} else {
				Self::deposit_event(Event::IdentityUpdated { account: who });
			}

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn add_one_time_prekeys(
			origin: OriginFor<T>,
			prekeys: Vec<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				IdentityKeys::<T>::contains_key(&who),
				Error::<T>::IdentityNotFound
			);

			ensure!(
				prekeys.len() as u32 <= T::MaxOneTimePrekeys::get(),
				Error::<T>::TooManyOneTimePrekeys
			);

			let mut counter = PrekeyCounter::<T>::get(&who);
			let mut added_count = 0u32;

			for prekey in prekeys {
				let bounded_prekey: BoundedVec<u8, T::MaxKeySize> = prekey
					.try_into()
					.map_err(|_| Error::<T>::OneTimePrekeyTooLarge)?;

				let one_time_prekey = OneTimePrekey {
					key_id: counter,
					public_key: bounded_prekey,
				};

				OneTimePrekeys::<T>::insert(&who, counter, one_time_prekey);
				counter = counter.saturating_add(1);
				added_count = added_count.saturating_add(1);
			}

			PrekeyCounter::<T>::insert(&who, counter);
			Self::deposit_event(Event::OneTimePrekeysAdded {
				account: who,
				count: added_count,
			});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn remove_identity(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(
				IdentityKeys::<T>::contains_key(&who),
				Error::<T>::IdentityNotFound
			);

			IdentityKeys::<T>::remove(&who);

			let mut key_id = 0u32;
			while let Some(_) = OneTimePrekeys::<T>::take(&who, key_id) {
				key_id = key_id.saturating_add(1);
			}
			PrekeyCounter::<T>::remove(&who);

			KeyBundleCount::<T>::mutate(|c| *c = c.saturating_sub(1));

			Self::deposit_event(Event::IdentityRemoved { account: who });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_identity(account: &T::AccountId) -> Option<IdentityKeyBundle<T>> {
			IdentityKeys::<T>::get(account)
		}

		pub fn get_one_time_prekey(account: &T::AccountId) -> Option<(u32, OneTimePrekey<T>)> {
			let counter = PrekeyCounter::<T>::get(account);
			
			for key_id in 0..counter {
				if let Some(prekey) = OneTimePrekeys::<T>::take(account, key_id) {
					Self::deposit_event(Event::OneTimePrekeyUsed {
						account: account.clone(),
						key_id,
					});
					return Some((key_id, prekey));
				}
			}

			None
		}

		pub fn has_one_time_prekeys(account: &T::AccountId) -> bool {
			let counter = PrekeyCounter::<T>::get(account);
			for key_id in 0..counter {
				if OneTimePrekeys::<T>::contains_key(account, key_id) {
					return true;
				}
			}
			false
		}

		pub fn get_remaining_prekey_count(account: &T::AccountId) -> u32 {
			let counter = PrekeyCounter::<T>::get(account);
			let mut count = 0u32;
			for key_id in 0..counter {
				if OneTimePrekeys::<T>::contains_key(account, key_id) {
					count = count.saturating_add(1);
				}
			}
			count
		}
	}
}
