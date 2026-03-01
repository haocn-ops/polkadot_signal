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
		type MaxGroupNameLength: Get<u32>;
		
		#[pallet::constant]
		type MaxGroupMembers: Get<u32>;
		
		#[pallet::constant]
		type MaxKeySize: Get<u32>;
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Group<T: Config> {
		pub group_id: [u8; 32],
		pub name: BoundedVec<u8, T::MaxGroupNameLength>,
		pub admin: T::AccountId,
		pub members: BoundedVec<T::AccountId, T::MaxGroupMembers>,
		pub group_key: BoundedVec<u8, T::MaxKeySize>,
		pub created_at: BlockNumberFor<T>,
		pub updated_at: BlockNumberFor<T>,
		pub version: u32,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct GroupInvite<T: Config> {
		pub group_id: [u8; 32],
		pub inviter: T::AccountId,
		pub invitee: T::AccountId,
		pub created_at: BlockNumberFor<T>,
		pub expires_at: Option<BlockNumberFor<T>>,
	}

	#[pallet::storage]
	#[pallet::getter(fn groups)]
	pub type Groups<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		[u8; 32],
		Group<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn user_groups)]
	pub type UserGroups<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		[u8; 32],
		bool,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn pending_invites)]
	pub type PendingInvites<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		[u8; 32],
		GroupInvite<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn group_counter)]
	pub type GroupCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		GroupCreated { group_id: [u8; 32], admin: T::AccountId },
		GroupUpdated { group_id: [u8; 32] },
		GroupDissolved { group_id: [u8; 32] },
		MemberAdded { group_id: [u8; 32], member: T::AccountId },
		MemberRemoved { group_id: [u8; 32], member: T::AccountId },
		InviteSent { group_id: [u8; 32], invitee: T::AccountId },
		InviteAccepted { group_id: [u8; 32], member: T::AccountId },
		InviteDeclined { group_id: [u8; 32], invitee: T::AccountId },
		AdminTransferred { group_id: [u8; 32], new_admin: T::AccountId },
		GroupKeyUpdated { group_id: [u8; 32], version: u32 },
	}

	#[pallet::error]
	pub enum Error<T> {
		GroupNameTooLong,
		GroupNotFound,
		NotGroupAdmin,
		AlreadyMember,
		NotMember,
		GroupFull,
		InviteNotFound,
		InviteExpired,
		AlreadyInvited,
		CannotRemoveAdmin,
		InvalidGroupKey,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(3))]
		pub fn create_group(
			origin: OriginFor<T>,
			name: Vec<u8>,
			group_key: Vec<u8>,
			initial_members: Vec<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let name: BoundedVec<u8, T::MaxGroupNameLength> = name
				.try_into()
				.map_err(|_| Error::<T>::GroupNameTooLong)?;

			let group_key: BoundedVec<u8, T::MaxKeySize> = group_key
				.try_into()
				.map_err(|_| Error::<T>::InvalidGroupKey)?;

			let group_id = Self::generate_group_id();

			let current_block = frame_system::Pallet::<T>::block_number();

			let mut members: BoundedVec<T::AccountId, T::MaxGroupMembers> = BoundedVec::new();
			members.try_push(who.clone()).map_err(|_| Error::<T>::GroupFull)?;

			for member in initial_members {
				if !members.contains(&member) {
					members.try_push(member).map_err(|_| Error::<T>::GroupFull)?;
				}
			}

			let group = Group {
				group_id,
				name,
				admin: who.clone(),
				members: members.clone(),
				group_key,
				created_at: current_block,
				updated_at: current_block,
				version: 1,
			};

			Groups::<T>::insert(group_id, group);

			for member in members.iter() {
				UserGroups::<T>::insert(member, group_id, true);
			}

			GroupCounter::<T>::mutate(|c| *c = c.saturating_add(1));

			Self::deposit_event(Event::GroupCreated {
				group_id,
				admin: who,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(2))]
		pub fn invite_member(
			origin: OriginFor<T>,
			group_id: [u8; 32],
			invitee: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let group = Groups::<T>::get(group_id).ok_or(Error::<T>::GroupNotFound)?;

			ensure!(group.admin == who, Error::<T>::NotGroupAdmin);
			ensure!(!group.members.contains(&invitee), Error::<T>::AlreadyMember);
			ensure!(
				!PendingInvites::<T>::contains_key(&invitee, group_id),
				Error::<T>::AlreadyInvited
			);

			let current_block = frame_system::Pallet::<T>::block_number();

			let invite = GroupInvite {
				group_id,
				inviter: who.clone(),
				invitee: invitee.clone(),
				created_at: current_block,
				expires_at: None,
			};

			PendingInvites::<T>::insert(&invitee, group_id, invite);

			Self::deposit_event(Event::InviteSent {
				group_id,
				invitee,
			});

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(3))]
		pub fn accept_invite(
			origin: OriginFor<T>,
			group_id: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let invite = PendingInvites::<T>::take(&who, group_id)
				.ok_or(Error::<T>::InviteNotFound)?;

			if let Some(expires) = invite.expires_at {
				let current_block = frame_system::Pallet::<T>::block_number();
				ensure!(current_block < expires, Error::<T>::InviteExpired);
			}

			Groups::<T>::mutate(group_id, |group_opt| -> DispatchResult {
				let group = group_opt.as_mut().ok_or(Error::<T>::GroupNotFound)?;
				
				group.members.try_push(who.clone()).map_err(|_| Error::<T>::GroupFull)?;
				group.updated_at = frame_system::Pallet::<T>::block_number();

				Ok(())
			})?;

			UserGroups::<T>::insert(&who, group_id, true);

			Self::deposit_event(Event::InviteAccepted {
				group_id,
				member: who,
			});

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn decline_invite(
			origin: OriginFor<T>,
			group_id: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			PendingInvites::<T>::take(&who, group_id)
				.ok_or(Error::<T>::InviteNotFound)?;

			Self::deposit_event(Event::InviteDeclined {
				group_id,
				invitee: who,
			});

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(2))]
		pub fn remove_member(
			origin: OriginFor<T>,
			group_id: [u8; 32],
			member: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let group = Groups::<T>::get(group_id).ok_or(Error::<T>::GroupNotFound)?;

			ensure!(group.admin == who, Error::<T>::NotGroupAdmin);
			ensure!(group.members.contains(&member), Error::<T>::NotMember);
			ensure!(group.admin != member, Error::<T>::CannotRemoveAdmin);

			Groups::<T>::mutate(group_id, |group_opt| -> DispatchResult {
				let group = group_opt.as_mut().ok_or(Error::<T>::GroupNotFound)?;
				group.members.retain(|m| m != &member);
				group.updated_at = frame_system::Pallet::<T>::block_number();
				Ok(())
			})?;

			UserGroups::<T>::remove(&member, group_id);

			Self::deposit_event(Event::MemberRemoved {
				group_id,
				member,
			});

			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(2))]
		pub fn leave_group(
			origin: OriginFor<T>,
			group_id: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let group = Groups::<T>::get(group_id).ok_or(Error::<T>::GroupNotFound)?;

			ensure!(group.members.contains(&who), Error::<T>::NotMember);
			ensure!(group.admin != who, Error::<T>::CannotRemoveAdmin);

			Groups::<T>::mutate(group_id, |group_opt| -> DispatchResult {
				let group = group_opt.as_mut().ok_or(Error::<T>::GroupNotFound)?;
				group.members.retain(|m| m != &who);
				group.updated_at = frame_system::Pallet::<T>::block_number();
				Ok(())
			})?;

			UserGroups::<T>::remove(&who, group_id);

			Self::deposit_event(Event::MemberRemoved {
				group_id,
				member: who,
			});

			Ok(())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn transfer_admin(
			origin: OriginFor<T>,
			group_id: [u8; 32],
			new_admin: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Groups::<T>::mutate(group_id, |group_opt| -> DispatchResult {
				let group = group_opt.as_mut().ok_or(Error::<T>::GroupNotFound)?;
				
				ensure!(group.admin == who, Error::<T>::NotGroupAdmin);
				ensure!(group.members.contains(&new_admin), Error::<T>::NotMember);

				group.admin = new_admin.clone();
				group.updated_at = frame_system::Pallet::<T>::block_number();

				Ok(())
			})?;

			Self::deposit_event(Event::AdminTransferred {
				group_id,
				new_admin,
			});

			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn update_group_key(
			origin: OriginFor<T>,
			group_id: [u8; 32],
			new_key: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let new_key: BoundedVec<u8, T::MaxKeySize> = new_key
				.try_into()
				.map_err(|_| Error::<T>::InvalidGroupKey)?;

			Groups::<T>::mutate(group_id, |group_opt| -> DispatchResult {
				let group = group_opt.as_mut().ok_or(Error::<T>::GroupNotFound)?;
				
				ensure!(group.admin == who, Error::<T>::NotGroupAdmin);

				group.group_key = new_key;
				group.version = group.version.saturating_add(1);
				group.updated_at = frame_system::Pallet::<T>::block_number();

				Ok(())
			})?;

			Self::deposit_event(Event::GroupKeyUpdated {
				group_id,
				version: 0,
			});

			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
		pub fn dissolve_group(
			origin: OriginFor<T>,
			group_id: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let group = Groups::<T>::get(group_id).ok_or(Error::<T>::GroupNotFound)?;

			ensure!(group.admin == who, Error::<T>::NotGroupAdmin);

			for member in group.members.iter() {
				UserGroups::<T>::remove(member, group_id);
			}

			Groups::<T>::remove(group_id);
			GroupCounter::<T>::mutate(|c| *c = c.saturating_sub(1));

			Self::deposit_event(Event::GroupDissolved { group_id });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn generate_group_id() -> [u8; 32] {
			use sp_io::hashing::blake2_256;
			
			let counter = GroupCounter::<T>::get();
			let mut input = counter.to_le_bytes().to_vec();
			input.extend_from_slice(&frame_system::Pallet::<T>::block_number().encode());
			
			blake2_256(&input)
		}

		pub fn get_user_groups(account: &T::AccountId) -> Vec<[u8; 32]> {
			UserGroups::<T>::iter_prefix(account)
				.filter_map(|(group_id, is_member)| {
					if is_member { Some(group_id) } else { None }
				})
				.collect()
		}

		pub fn get_group_members(group_id: &[u8; 32]) -> Option<Vec<T::AccountId>> {
			Groups::<T>::get(group_id).map(|g| g.members.into_inner())
		}

		pub fn is_member(group_id: &[u8; 32], account: &T::AccountId) -> bool {
			UserGroups::<T>::get(account, group_id)
		}

		pub fn is_admin(group_id: &[u8; 32], account: &T::AccountId) -> bool {
			Groups::<T>::get(group_id)
				.map(|g| g.admin == *account)
				.unwrap_or(false)
		}
	}
}
