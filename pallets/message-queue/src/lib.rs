#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

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
		type MaxCidLength: Get<u32>;
		
		#[pallet::constant]
		type MaxPendingMessages: Get<u32>;
		
		#[pallet::constant]
		type MessageTtl: Get<BlockNumberFor<Self>>;
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct MessageNotification<T: Config> {
		pub cid: BoundedVec<u8, T::MaxCidLength>,
		pub sender: T::AccountId,
		pub created_at: BlockNumberFor<T>,
		pub expires_at: BlockNumberFor<T>,
		pub read: bool,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct DeliveryReceipt<T: Config> {
		pub message_cid: BoundedVec<u8, T::MaxCidLength>,
		pub recipient: T::AccountId,
		pub delivered_at: BlockNumberFor<T>,
		pub read_at: Option<BlockNumberFor<T>>,
	}

	#[pallet::storage]
	#[pallet::getter(fn pending_messages)]
	pub type PendingMessages<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		u32,
		MessageNotification<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn message_counter)]
	pub type MessageCounter<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn delivery_receipts)]
	pub type DeliveryReceipts<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxCidLength>,
		DeliveryReceipt<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn unread_count)]
	pub type UnreadCount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MessageStored { recipient: T::AccountId, cid: Vec<u8> },
		MessageRetrieved { recipient: T::AccountId, message_index: u32 },
		MessageRead { recipient: T::AccountId, message_index: u32 },
		AllMessagesRead { recipient: T::AccountId },
		MessageExpired { recipient: T::AccountId, message_index: u32 },
		DeliveryConfirmed { sender: T::AccountId, recipient: T::AccountId, cid: Vec<u8> },
		ReadReceipt { sender: T::AccountId, recipient: T::AccountId, cid: Vec<u8> },
	}

	#[pallet::error]
	pub enum Error<T> {
		CidTooLong,
		NoPendingMessages,
		MessageNotFound,
		TooManyPendingMessages,
		InvalidMessageIndex,
		MessageExpired,
		AlreadyDelivered,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(2))]
		pub fn notify_message(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			cid: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let cid: BoundedVec<u8, T::MaxCidLength> = cid
				.try_into()
				.map_err(|_| Error::<T>::CidTooLong)?;

			let current_block = frame_system::Pallet::<T>::block_number();
			let expires_at = current_block + T::MessageTtl::get();

			let mut counter = MessageCounter::<T>::get(&recipient);
			
			ensure!(
				counter < T::MaxPendingMessages::get(),
				Error::<T>::TooManyPendingMessages
			);

			let notification = MessageNotification {
				cid,
				sender: sender.clone(),
				created_at: current_block,
				expires_at,
				read: false,
			};

			PendingMessages::<T>::insert(&recipient, counter, notification);
			MessageCounter::<T>::insert(&recipient, counter + 1);
			UnreadCount::<T>::mutate(&recipient, |c| *c = c.saturating_add(1));

			let cid_vec = PendingMessages::<T>::get(&recipient, counter)
				.map(|n| n.cid.to_vec())
				.unwrap_or_default();

			Self::deposit_event(Event::MessageStored {
				recipient,
				cid: cid_vec,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn mark_read(
			origin: OriginFor<T>,
			message_index: u32,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			PendingMessages::<T>::try_mutate(&who, message_index, |notification_opt| -> DispatchResult {
				let notification = notification_opt.as_mut().ok_or(Error::<T>::MessageNotFound)?;

				ensure!(!notification.read, Error::<T>::AlreadyDelivered);

				notification.read = true;
				
				Ok(())
			})?;

			UnreadCount::<T>::mutate(&who, |c| *c = c.saturating_sub(1));

			Self::deposit_event(Event::MessageRead {
				recipient: who,
				message_index,
			});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn mark_all_read(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let counter = MessageCounter::<T>::get(&who);
			
			for i in 0..counter {
				if let Some(notification) = PendingMessages::<T>::get(&who, i) {
					if !notification.read {
						PendingMessages::<T>::mutate(&who, i, |n| {
							if let Some(n) = n {
								n.read = true;
							}
						});
					}
				}
			}

			UnreadCount::<T>::insert(&who, 0u32);

			Self::deposit_event(Event::AllMessagesRead { recipient: who });

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(2))]
		pub fn confirm_delivery(
			origin: OriginFor<T>,
			sender: T::AccountId,
			cid: Vec<u8>,
		) -> DispatchResult {
			let recipient = ensure_signed(origin)?;

			let cid: BoundedVec<u8, T::MaxCidLength> = cid
				.try_into()
				.map_err(|_| Error::<T>::CidTooLong)?;

			let current_block = frame_system::Pallet::<T>::block_number();

			let receipt = DeliveryReceipt {
				message_cid: cid.clone(),
				recipient: recipient.clone(),
				delivered_at: current_block,
				read_at: None,
			};

			DeliveryReceipts::<T>::insert(&sender, &cid, receipt);

			Self::deposit_event(Event::DeliveryConfirmed {
				sender,
				recipient,
				cid: cid.to_vec(),
			});

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn send_read_receipt(
			origin: OriginFor<T>,
			sender: T::AccountId,
			cid: Vec<u8>,
		) -> DispatchResult {
			let recipient = ensure_signed(origin)?;

			let cid: BoundedVec<u8, T::MaxCidLength> = cid
				.try_into()
				.map_err(|_| Error::<T>::CidTooLong)?;

			DeliveryReceipts::<T>::mutate(&sender, &cid, |receipt_opt| -> DispatchResult {
				let receipt = receipt_opt.as_mut().ok_or(Error::<T>::MessageNotFound)?;

				receipt.read_at = Some(frame_system::Pallet::<T>::block_number());

				Ok(())
			})?;

			Self::deposit_event(Event::ReadReceipt {
				sender,
				recipient,
				cid: cid.to_vec(),
			});

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn clear_expired(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let current_block = frame_system::Pallet::<T>::block_number();
			let counter = MessageCounter::<T>::get(&who);

			for i in 0..counter {
				if let Some(notification) = PendingMessages::<T>::get(&who, i) {
					if notification.expires_at <= current_block {
						PendingMessages::<T>::remove(&who, i);
						
						if !notification.read {
							UnreadCount::<T>::mutate(&who, |c| *c = c.saturating_sub(1));
						}

						Self::deposit_event(Event::MessageExpired {
							recipient: who.clone(),
							message_index: i,
						});
					}
				}
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_pending_count(account: &T::AccountId) -> u32 {
			MessageCounter::<T>::get(account)
		}

		pub fn get_unread_count(account: &T::AccountId) -> u32 {
			UnreadCount::<T>::get(account)
		}

		pub fn get_all_pending(account: &T::AccountId) -> Vec<(u32, MessageNotification<T>)> {
			let counter = MessageCounter::<T>::get(account);
			let mut messages = Vec::new();

			for i in 0..counter {
				if let Some(notification) = PendingMessages::<T>::get(account, i) {
					messages.push((i, notification));
				}
			}

			messages
		}

		pub fn has_pending_messages(account: &T::AccountId) -> bool {
			MessageCounter::<T>::get(account) > 0
		}

		pub fn get_delivery_status(sender: &T::AccountId, cid: &[u8]) -> Option<DeliveryReceipt<T>> {
			let cid: BoundedVec<u8, T::MaxCidLength> = cid.try_into().ok()?;
			DeliveryReceipts::<T>::get(sender, cid)
		}
	}
}
