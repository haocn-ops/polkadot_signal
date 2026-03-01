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
        type MaxAgentNameLength: Get<u32>;
        
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;
        
        #[pallet::constant]
        type MaxCapabilities: Get<u32>;
        
        #[pallet::constant]
        type MaxCapabilityNameLength: Get<u32>;
        
        #[pallet::constant]
        type MaxProtocols: Get<u32>;
        
        #[pallet::constant]
        type MaxEndpointLength: Get<u32>;
        
        #[pallet::constant]
        type MaxTags: Get<u32>;
    }

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub enum AgentType {
        LLM,
        Tool,
        Orchestrator,
        Worker,
        Coordinator,
        Custom(BoundedVec<u8, T::MaxCapabilityNameLength>),
    }

    impl<T: Config> Default for AgentType {
        fn default() -> Self {
            AgentType::Worker
        }
    }

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub enum AgentStatus {
        Active,
        Idle,
        Busy,
        Maintenance,
        Offline,
    }

    impl Default for AgentStatus {
        fn default() -> Self {
            AgentStatus::Active
        }
    }

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Capability<T: Config> {
        pub name: BoundedVec<u8, T::MaxCapabilityNameLength>,
        pub category: BoundedVec<u8, T::MaxCapabilityNameLength>,
        pub description: BoundedVec<u8, T::MaxDescriptionLength>,
        pub version: BoundedVec<u8, T::MaxCapabilityNameLength>,
        pub tags: BoundedVec<BoundedVec<u8, T::MaxCapabilityNameLength>, T::MaxTags>,
        pub cost_units: u64,
        pub cost_amount: u128,
    }

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct AgentProfile<T: Config> {
        pub agent_id: BoundedVec<u8, T::MaxAgentNameLength>,
        pub name: BoundedVec<u8, T::MaxAgentNameLength>,
        pub description: BoundedVec<u8, T::MaxDescriptionLength>,
        pub agent_type: AgentType,
        pub version: BoundedVec<u8, T::MaxCapabilityNameLength>,
        pub capabilities: BoundedVec<Capability<T>, T::MaxCapabilities>,
        pub supported_protocols: BoundedVec<BoundedVec<u8, T::MaxCapabilityNameLength>, T::MaxProtocols>,
        pub endpoint: BoundedVec<u8, T::MaxEndpointLength>,
        pub max_concurrent_tasks: u32,
        pub reliability_score: u32,
        pub total_tasks_completed: u64,
        pub total_tasks_failed: u64,
        pub average_response_time_ms: u64,
        pub status: AgentStatus,
        pub owner: T::AccountId,
        pub created_at: BlockNumberFor<T>,
        pub updated_at: BlockNumberFor<T>,
        pub last_heartbeat: Option<BlockNumberFor<T>>,
    }

    #[pallet::storage]
    #[pallet::getter(fn agents)]
    pub type Agents<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        AgentProfile<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn agents_by_type)]
    pub type AgentsByType<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        AgentType,
        BoundedVec<T::AccountId, ConstU32<1000>>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn agents_by_capability)]
    pub type AgentsByCapability<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxCapabilityNameLength>,
        BoundedVec<T::AccountId, ConstU32<1000>>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn agent_counter)]
    pub type AgentCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn capability_registry)]
    pub type CapabilityRegistry<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxCapabilityNameLength>,
        CapabilityInfo<T>,
        OptionQuery,
    >;

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct CapabilityInfo<T: Config> {
        pub name: BoundedVec<u8, T::MaxCapabilityNameLength>,
        pub category: BoundedVec<u8, T::MaxCapabilityNameLength>,
        pub description: BoundedVec<u8, T::MaxDescriptionLength>,
        pub provider_count: u32,
        pub total_requests: u64,
        pub avg_response_time_ms: u64,
        pub _phantom: PhantomData<T>,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AgentRegistered { account: T::AccountId, agent_id: Vec<u8> },
        AgentUpdated { account: T::AccountId },
        AgentDeregistered { account: T::AccountId },
        AgentStatusChanged { account: T::AccountId, status: AgentStatus },
        HeartbeatReceived { account: T::AccountId },
        CapabilityAdded { name: Vec<u8> },
        StatsUpdated { account: T::AccountId, tasks_completed: u64, tasks_failed: u64 },
    }

    #[pallet::error]
    pub enum Error<T> {
        AgentNameTooLong,
        DescriptionTooLong,
        TooManyCapabilities,
        CapabilityNameTooLong,
        TooManyProtocols,
        EndpointTooLong,
        TooManyTags,
        AgentNotFound,
        AgentAlreadyExists,
        InvalidCapability,
        NotAgentOwner,
        InvalidStatus,
        MaxConcurrentTasksExceeded,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_ref_time(20_000) + T::DbWeight::get().writes(3))]
        pub fn register_agent(
            origin: OriginFor<T>,
            agent_id: Vec<u8>,
            name: Vec<u8>,
            description: Vec<u8>,
            agent_type: AgentType,
            version: Vec<u8>,
            capabilities: Vec<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<Vec<u8>>, u64, u128)>,
            supported_protocols: Vec<Vec<u8>>,
            endpoint: Vec<u8>,
            max_concurrent_tasks: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!Agents::<T>::contains_key(&who), Error::<T>::AgentAlreadyExists);

            let agent_id: BoundedVec<u8, T::MaxAgentNameLength> = agent_id
                .try_into()
                .map_err(|_| Error::<T>::AgentNameTooLong)?;

            let name: BoundedVec<u8, T::MaxAgentNameLength> = name
                .try_into()
                .map_err(|_| Error::<T>::AgentNameTooLong)?;

            let description: BoundedVec<u8, T::MaxDescriptionLength> = description
                .try_into()
                .map_err(|_| Error::<T>::DescriptionTooLong)?;

            let version: BoundedVec<u8, T::MaxCapabilityNameLength> = version
                .try_into()
                .map_err(|_| Error::<T>::CapabilityNameTooLong)?;

            let endpoint: BoundedVec<u8, T::MaxEndpointLength> = endpoint
                .try_into()
                .map_err(|_| Error::<T>::EndpointTooLong)?;

            let mut bounded_capabilities: BoundedVec<Capability<T>, T::MaxCapabilities> = 
                BoundedVec::new();
            
            for (cap_name, category, cap_desc, cap_version, tags, cost_units, cost_amount) in capabilities {
                let bounded_name: BoundedVec<u8, T::MaxCapabilityNameLength> = cap_name
                    .try_into()
                    .map_err(|_| Error::<T>::CapabilityNameTooLong)?;
                
                let bounded_category: BoundedVec<u8, T::MaxCapabilityNameLength> = category
                    .try_into()
                    .map_err(|_| Error::<T>::CapabilityNameTooLong)?;
                
                let bounded_desc: BoundedVec<u8, T::MaxDescriptionLength> = cap_desc
                    .try_into()
                    .map_err(|_| Error::<T>::DescriptionTooLong)?;
                
                let bounded_version: BoundedVec<u8, T::MaxCapabilityNameLength> = cap_version
                    .try_into()
                    .map_err(|_| Error::<T>::CapabilityNameTooLong)?;
                
                let mut bounded_tags: BoundedVec<BoundedVec<u8, T::MaxCapabilityNameLength>, T::MaxTags> = 
                    BoundedVec::new();
                for tag in tags {
                    let bounded_tag: BoundedVec<u8, T::MaxCapabilityNameLength> = tag
                        .try_into()
                        .map_err(|_| Error::<T>::CapabilityNameTooLong)?;
                    bounded_tags.try_push(bounded_tag).map_err(|_| Error::<T>::TooManyTags)?;
                }
                
                let capability = Capability {
                    name: bounded_name.clone(),
                    category: bounded_category,
                    description: bounded_desc,
                    version: bounded_version,
                    tags: bounded_tags,
                    cost_units,
                    cost_amount,
                };
                
                bounded_capabilities.try_push(capability).map_err(|_| Error::<T>::TooManyCapabilities)?;
                
                Self::update_capability_registry(&bounded_name);
            }

            let mut bounded_protocols: BoundedVec<BoundedVec<u8, T::MaxCapabilityNameLength>, T::MaxProtocols> = 
                BoundedVec::new();
            for protocol in supported_protocols {
                let bounded_protocol: BoundedVec<u8, T::MaxCapabilityNameLength> = protocol
                    .try_into()
                    .map_err(|_| Error::<T>::CapabilityNameTooLong)?;
                bounded_protocols.try_push(bounded_protocol).map_err(|_| Error::<T>::TooManyProtocols)?;
            }

            let current_block = frame_system::Pallet::<T>::block_number();

            let profile = AgentProfile {
                agent_id,
                name,
                description,
                agent_type: agent_type.clone(),
                version,
                capabilities: bounded_capabilities,
                supported_protocols: bounded_protocols,
                endpoint,
                max_concurrent_tasks,
                reliability_score: 1000,
                total_tasks_completed: 0,
                total_tasks_failed: 0,
                average_response_time_ms: 0,
                status: AgentStatus::Active,
                owner: who.clone(),
                created_at: current_block,
                updated_at: current_block,
                last_heartbeat: None,
            };

            Agents::<T>::insert(&who, profile);

            AgentsByType::<T>::mutate(&agent_type, |agents| {
                let _ = agents.try_push(who.clone());
            });

            AgentCounter::<T>::mutate(|c| *c = c.saturating_add(1));

            Self::deposit_event(Event::AgentRegistered {
                account: who,
                agent_id: agent_id.into_inner(),
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
        pub fn update_status(
            origin: OriginFor<T>,
            status: AgentStatus,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Agents::<T>::try_mutate(&who, |profile_opt| -> DispatchResult {
                let profile = profile_opt.as_mut().ok_or(Error::<T>::AgentNotFound)?;
                profile.status = status.clone();
                profile.updated_at = frame_system::Pallet::<T>::block_number();
                Ok(())
            })?;

            Self::deposit_event(Event::AgentStatusChanged {
                account: who,
                status,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_ref_time(5_000) + T::DbWeight::get().writes(1))]
        pub fn heartbeat(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Agents::<T>::try_mutate(&who, |profile_opt| -> DispatchResult {
                let profile = profile_opt.as_mut().ok_or(Error::<T>::AgentNotFound)?;
                profile.last_heartbeat = Some(frame_system::Pallet::<T>::block_number());
                profile.status = AgentStatus::Active;
                Ok(())
            })?;

            Self::deposit_event(Event::HeartbeatReceived { account: who });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_ref_time(15_000) + T::DbWeight::get().writes(1))]
        pub fn update_capabilities(
            origin: OriginFor<T>,
            capabilities: Vec<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<Vec<u8>>, u64, u128)>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Agents::<T>::try_mutate(&who, |profile_opt| -> DispatchResult {
                let profile = profile_opt.as_mut().ok_or(Error::<T>::AgentNotFound)?;
                
                let mut bounded_capabilities: BoundedVec<Capability<T>, T::MaxCapabilities> = 
                    BoundedVec::new();
                
                for (cap_name, category, cap_desc, cap_version, tags, cost_units, cost_amount) in capabilities {
                    let bounded_name: BoundedVec<u8, T::MaxCapabilityNameLength> = cap_name
                        .try_into()
                        .map_err(|_| Error::<T>::CapabilityNameTooLong)?;
                    
                    let bounded_category: BoundedVec<u8, T::MaxCapabilityNameLength> = category
                        .try_into()
                        .map_err(|_| Error::<T>::CapabilityNameTooLong)?;
                    
                    let bounded_desc: BoundedVec<u8, T::MaxDescriptionLength> = cap_desc
                        .try_into()
                        .map_err(|_| Error::<T>::DescriptionTooLong)?;
                    
                    let bounded_version: BoundedVec<u8, T::MaxCapabilityNameLength> = cap_version
                        .try_into()
                        .map_err(|_| Error::<T>::CapabilityNameTooLong)?;
                    
                    let mut bounded_tags: BoundedVec<BoundedVec<u8, T::MaxCapabilityNameLength>, T::MaxTags> = 
                        BoundedVec::new();
                    for tag in tags {
                        let bounded_tag: BoundedVec<u8, T::MaxCapabilityNameLength> = tag
                            .try_into()
                            .map_err(|_| Error::<T>::CapabilityNameTooLong)?;
                        bounded_tags.try_push(bounded_tag).map_err(|_| Error::<T>::TooManyTags)?;
                    }
                    
                    let capability = Capability {
                        name: bounded_name.clone(),
                        category: bounded_category,
                        description: bounded_desc,
                        version: bounded_version,
                        tags: bounded_tags,
                        cost_units,
                        cost_amount,
                    };
                    
                    bounded_capabilities.try_push(capability).map_err(|_| Error::<T>::TooManyCapabilities)?;
                    
                    Self::update_capability_registry(&bounded_name);
                }
                
                profile.capabilities = bounded_capabilities;
                profile.updated_at = frame_system::Pallet::<T>::block_number();
                
                Ok(())
            })?;

            Self::deposit_event(Event::AgentUpdated { account: who });

            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(1))]
        pub fn update_stats(
            origin: OriginFor<T>,
            tasks_completed: Option<u64>,
            tasks_failed: Option<u64>,
            response_time_ms: Option<u64>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Agents::<T>::try_mutate(&who, |profile_opt| -> DispatchResult {
                let profile = profile_opt.as_mut().ok_or(Error::<T>::AgentNotFound)?;
                
                if let Some(completed) = tasks_completed {
                    profile.total_tasks_completed = profile.total_tasks_completed.saturating_add(completed);
                }
                
                if let Some(failed) = tasks_failed {
                    profile.total_tasks_failed = profile.total_tasks_failed.saturating_add(failed);
                }
                
                if let Some(time) = response_time_ms {
                    let total = profile.total_tasks_completed.saturating_add(profile.total_tasks_failed);
                    if total > 0 {
                        profile.average_response_time_ms = 
                            (profile.average_response_time_ms * (total - 1) + time) / total;
                    } else {
                        profile.average_response_time_ms = time;
                    }
                }
                
                let total = profile.total_tasks_completed.saturating_add(profile.total_tasks_failed);
                if total > 0 {
                    let success_rate = (profile.total_tasks_completed as u128 * 1000) / total as u128;
                    profile.reliability_score = success_rate.min(1000) as u32;
                }
                
                profile.updated_at = frame_system::Pallet::<T>::block_number();
                
                Ok(())
            })?;

            Self::deposit_event(Event::StatsUpdated {
                account: who,
                tasks_completed: tasks_completed.unwrap_or(0),
                tasks_failed: tasks_failed.unwrap_or(0),
            });

            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(3))]
        pub fn deregister_agent(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let profile = Agents::<T>::take(&who).ok_or(Error::<T>::AgentNotFound)?;

            AgentsByType::<T>::mutate(&profile.agent_type, |agents| {
                agents.retain(|a| a != &who);
            });

            for cap in profile.capabilities.iter() {
                AgentsByCapability::<T>::mutate(&cap.name, |agents| {
                    agents.retain(|a| a != &who);
                });
            }

            AgentCounter::<T>::mutate(|c| *c = c.saturating_sub(1));

            Self::deposit_event(Event::AgentDeregistered { account: who });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn update_capability_registry(cap_name: &BoundedVec<u8, T::MaxCapabilityNameLength>) {
            if !CapabilityRegistry::<T>::contains_key(cap_name) {
                let info = CapabilityInfo {
                    name: cap_name.clone(),
                    category: BoundedVec::default(),
                    description: BoundedVec::default(),
                    provider_count: 1,
                    total_requests: 0,
                    avg_response_time_ms: 0,
                    _phantom: Default::default(),
                };
                CapabilityRegistry::<T>::insert(cap_name, info);
                
                Self::deposit_event(Event::CapabilityAdded {
                    name: cap_name.to_vec(),
                });
            } else {
                CapabilityRegistry::<T>::mutate(cap_name, |info_opt| {
                    if let Some(info) = info_opt {
                        info.provider_count = info.provider_count.saturating_add(1);
                    }
                });
            }
        }

        pub fn get_agent(account: &T::AccountId) -> Option<AgentProfile<T>> {
            Agents::<T>::get(account)
        }

        pub fn get_agents_by_type(agent_type: &AgentType) -> Vec<T::AccountId> {
            AgentsByType::<T>::get(agent_type).into_inner()
        }

        pub fn get_agents_by_capability(cap_name: &[u8]) -> Option<Vec<T::AccountId>> {
            let bounded: BoundedVec<u8, T::MaxCapabilityNameLength> = cap_name.to_vec()
                .try_into()
                .ok()?;
            Some(AgentsByCapability::<T>::get(&bounded).into_inner())
        }

        pub fn find_capable_agents(
            required_capabilities: &[Vec<u8>],
        ) -> Vec<(T::AccountId, u32)> {
            let mut agent_scores: std::collections::BTreeMap<T::AccountId, u32> = 
                std::collections::BTreeMap::new();
            
            for cap in required_capabilities {
                if let Ok(bounded_cap) = cap.clone().try_into() {
                    for agent in AgentsByCapability::<T>::get(&bounded_cap).iter() {
                        *agent_scores.entry(agent.clone()).or_insert(0) += 1;
                    }
                }
            }
            
            agent_scores
                .into_iter()
                .filter(|(_, score)| *score as usize == required_capabilities.len())
                .collect()
        }

        pub fn get_best_agent_for_capability(
            cap_name: &[u8],
        ) -> Option<(T::AccountId, AgentProfile<T>)> {
            let bounded: BoundedVec<u8, T::MaxCapabilityNameLength> = cap_name.to_vec()
                .try_into()
                .ok()?;
            
            let agents = AgentsByCapability::<T>::get(&bounded);
            
            let mut best: Option<(T::AccountId, AgentProfile<T>)> = None;
            
            for account in agents.iter() {
                if let Some(profile) = Agents::<T>::get(account) {
                    if profile.status == AgentStatus::Active || profile.status == AgentStatus::Idle {
                        if best.is_none() || 
                           profile.reliability_score > best.as_ref().unwrap().1.reliability_score {
                            best = Some((account.clone(), profile));
                        }
                    }
                }
            }
            
            best
        }

        pub fn get_total_agents() -> u64 {
            AgentCounter::<T>::get()
        }

        pub fn get_active_agents() -> u32 {
            let mut count = 0u32;
            for profile in Agents::<T>::iter().map(|(_, p)| p) {
                if profile.status == AgentStatus::Active || profile.status == AgentStatus::Idle {
                    count = count.saturating_add(1);
                }
            }
            count
        }
    }
}
