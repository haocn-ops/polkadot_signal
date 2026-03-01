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
        type MaxTaskInputSize: Get<u32>;
        
        #[pallet::constant]
        type MaxTaskOutputSize: Get<u32>;
        
        #[pallet::constant]
        type MaxDependencies: Get<u32>;
        
        #[pallet::constant]
        type MaxCapabilities: Get<u32>;
        
        #[pallet::constant]
        type MaxTaskTypeLength: Get<u32>;
        
        #[pallet::constant]
        type MaxActiveTasksPerAgent: Get<u32>;
        
        #[pallet::constant]
        type TaskTimeoutBlocks: Get<BlockNumberFor<Self>>;
    }

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub enum TaskPriority {
        Low,
        Normal,
        High,
        Critical,
    }

    impl Default for TaskPriority {
        fn default() -> Self {
            TaskPriority::Normal
        }
    }

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub enum TaskStatus {
        Pending,
        Queued,
        Assigned,
        Running,
        Completed,
        Failed,
        Cancelled,
        Timeout,
    }

    impl Default for TaskStatus {
        fn default() -> Self {
            TaskStatus::Pending
        }
    }

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Task<T: Config> {
        pub task_id: [u8; 32],
        pub task_type: BoundedVec<u8, T::MaxTaskTypeLength>,
        pub priority: TaskPriority,
        pub input: BoundedVec<u8, T::MaxTaskInputSize>,
        pub output: Option<BoundedVec<u8, T::MaxTaskOutputSize>>,
        pub error: Option<BoundedVec<u8, T::MaxTaskOutputSize>>,
        pub requester: T::AccountId,
        pub assigned_agent: Option<T::AccountId>,
        pub required_capabilities: BoundedVec<BoundedVec<u8, T::MaxTaskTypeLength>, T::MaxCapabilities>,
        pub dependencies: BoundedVec<[u8; 32], T::MaxDependencies>,
        pub status: TaskStatus,
        pub created_at: BlockNumberFor<T>,
        pub started_at: Option<BlockNumberFor<T>>,
        pub completed_at: Option<BlockNumberFor<T>>,
        pub deadline: Option<BlockNumberFor<T>>,
        pub retry_count: u32,
        pub max_retries: u32,
        pub execution_time_ms: Option<u64>,
        pub parent_task_id: Option<[u8; 32]>,
    }

    #[pallet::storage]
    #[pallet::getter(fn tasks)]
    pub type Tasks<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32],
        Task<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn tasks_by_requester)]
    pub type TasksByRequester<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        [u8; 32],
        bool,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn tasks_by_agent)]
    pub type TasksByAgent<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        [u8; 32],
        bool,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn pending_queue)]
    pub type PendingQueue<T: Config> = StorageValue<
        _,
        BoundedVec<[u8; 32], ConstU32<10000>>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn task_counter)]
    pub type TaskCounter<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn agent_active_tasks)]
    pub type AgentActiveTasks<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn task_results)]
    pub type TaskResults<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32],
        TaskResult<T>,
        OptionQuery,
    >;

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct TaskResult<T: Config> {
        pub task_id: [u8; 32],
        pub output: BoundedVec<u8, T::MaxTaskOutputSize>,
        pub success: bool,
        pub execution_time_ms: u64,
        pub completed_at: BlockNumberFor<T>,
        pub agent: T::AccountId,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        TaskCreated { task_id: [u8; 32], requester: T::AccountId },
        TaskAssigned { task_id: [u8; 32], agent: T::AccountId },
        TaskStarted { task_id: [u8; 32] },
        TaskCompleted { task_id: [u8; 32], agent: T::AccountId },
        TaskFailed { task_id: [u8; 32], error: Vec<u8> },
        TaskCancelled { task_id: [u8; 32] },
        TaskRetried { task_id: [u8; 32], retry_count: u32 },
        TaskTimeout { task_id: [u8; 32] },
        QueueProcessed { count: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        TaskInputTooLarge,
        TaskOutputTooLarge,
        TooManyDependencies,
        TooManyCapabilities,
        TaskTypeTooLong,
        TaskNotFound,
        TaskAlreadyExists,
        AgentNotFound,
        AgentBusy,
        AgentNotAssigned,
        NotTaskRequester,
        NotAssignedAgent,
        TaskNotPending,
        TaskNotAssigned,
        TaskNotRunning,
        TaskAlreadyCompleted,
        MaxRetriesExceeded,
        DependencyNotMet,
        DeadlineExceeded,
        QueueFull,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_ref_time(20_000) + T::DbWeight::get().writes(3))]
        pub fn create_task(
            origin: OriginFor<T>,
            task_type: Vec<u8>,
            priority: TaskPriority,
            input: Vec<u8>,
            required_capabilities: Vec<Vec<u8>>,
            dependencies: Vec<[u8; 32]>,
            deadline: Option<BlockNumberFor<T>>,
            max_retries: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let task_type: BoundedVec<u8, T::MaxTaskTypeLength> = task_type
                .try_into()
                .map_err(|_| Error::<T>::TaskTypeTooLong)?;

            let input: BoundedVec<u8, T::MaxTaskInputSize> = input
                .try_into()
                .map_err(|_| Error::<T>::TaskInputTooLarge)?;

            let mut bounded_caps: BoundedVec<BoundedVec<u8, T::MaxTaskTypeLength>, T::MaxCapabilities> = 
                BoundedVec::new();
            for cap in required_capabilities {
                let bounded: BoundedVec<u8, T::MaxTaskTypeLength> = cap
                    .try_into()
                    .map_err(|_| Error::<T>::TaskTypeTooLong)?;
                bounded_caps.try_push(bounded).map_err(|_| Error::<T>::TooManyCapabilities)?;
            }

            let mut bounded_deps: BoundedVec<[u8; 32], T::MaxDependencies> = BoundedVec::new();
            for dep in dependencies {
                if let Some(dep_task) = Tasks::<T>::get(dep) {
                    ensure!(
                        dep_task.status == TaskStatus::Completed,
                        Error::<T>::DependencyNotMet
                    );
                }
                bounded_deps.try_push(dep).map_err(|_| Error::<T>::TooManyDependencies)?;
            }

            let task_id = Self::generate_task_id();
            let current_block = frame_system::Pallet::<T>::block_number();

            let task = Task {
                task_id,
                task_type,
                priority: priority.clone(),
                input,
                output: None,
                error: None,
                requester: who.clone(),
                assigned_agent: None,
                required_capabilities: bounded_caps,
                dependencies: bounded_deps,
                status: TaskStatus::Pending,
                created_at: current_block,
                started_at: None,
                completed_at: None,
                deadline,
                retry_count: 0,
                max_retries,
                execution_time_ms: None,
                parent_task_id: None,
            };

            Tasks::<T>::insert(task_id, task);
            TasksByRequester::<T>::insert(&who, task_id, true);

            PendingQueue::<T>::try_mutate(|queue| -> DispatchResult {
                queue.try_push(task_id).map_err(|_| Error::<T>::QueueFull)
            })?;

            TaskCounter::<T>::mutate(|c| *c = c.saturating_add(1));

            Self::deposit_event(Event::TaskCreated {
                task_id,
                requester: who,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(3))]
        pub fn assign_task(
            origin: OriginFor<T>,
            task_id: [u8; 32],
            agent: T::AccountId,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let active_count = AgentActiveTasks::<T>::get(&agent);
            ensure!(
                active_count < T::MaxActiveTasksPerAgent::get(),
                Error::<T>::AgentBusy
            );

            Tasks::<T>::try_mutate(task_id, |task_opt| -> DispatchResult {
                let task = task_opt.as_mut().ok_or(Error::<T>::TaskNotFound)?;
                
                ensure!(
                    task.status == TaskStatus::Pending || task.status == TaskStatus::Queued,
                    Error::<T>::TaskNotPending
                );

                task.status = TaskStatus::Assigned;
                task.assigned_agent = Some(agent.clone());

                Ok(())
            })?;

            TasksByAgent::<T>::insert(&agent, task_id, true);
            AgentActiveTasks::<T>::insert(&agent, active_count.saturating_add(1));

            PendingQueue::<T>::mutate(|queue| {
                queue.retain(|id| *id != task_id);
            });

            Self::deposit_event(Event::TaskAssigned {
                task_id,
                agent,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_ref_time(5_000) + T::DbWeight::get().writes(1))]
        pub fn start_task(
            origin: OriginFor<T>,
            task_id: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Tasks::<T>::try_mutate(task_id, |task_opt| -> DispatchResult {
                let task = task_opt.as_mut().ok_or(Error::<T>::TaskNotFound)?;
                
                ensure!(
                    task.assigned_agent == Some(who),
                    Error::<T>::NotAssignedAgent
                );
                ensure!(task.status == TaskStatus::Assigned, Error::<T>::TaskNotAssigned);

                task.status = TaskStatus::Running;
                task.started_at = Some(frame_system::Pallet::<T>::block_number());

                Ok(())
            })?;

            Self::deposit_event(Event::TaskStarted { task_id });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(3))]
        pub fn complete_task(
            origin: OriginFor<T>,
            task_id: [u8; 32],
            output: Vec<u8>,
            execution_time_ms: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let output: BoundedVec<u8, T::MaxTaskOutputSize> = output
                .try_into()
                .map_err(|_| Error::<T>::TaskOutputTooLarge)?;

            Tasks::<T>::try_mutate(task_id, |task_opt| -> DispatchResult {
                let task = task_opt.as_mut().ok_or(Error::<T>::TaskNotFound)?;
                
                ensure!(
                    task.assigned_agent == Some(who.clone()),
                    Error::<T>::NotAssignedAgent
                );
                ensure!(task.status == TaskStatus::Running, Error::<T>::TaskNotRunning);

                task.status = TaskStatus::Completed;
                task.output = Some(output.clone());
                task.completed_at = Some(frame_system::Pallet::<T>::block_number());
                task.execution_time_ms = Some(execution_time_ms);

                Ok(())
            })?;

            let result = TaskResult {
                task_id,
                output,
                success: true,
                execution_time_ms,
                completed_at: frame_system::Pallet::<T>::block_number(),
                agent: who.clone(),
            };
            TaskResults::<T>::insert(task_id, result);

            AgentActiveTasks::<T>::mutate(&who, |count| {
                *count = count.saturating_sub(1);
            });

            Self::deposit_event(Event::TaskCompleted {
                task_id,
                agent: who,
            });

            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_ref_time(10_000) + T::DbWeight::get().writes(2))]
        pub fn fail_task(
            origin: OriginFor<T>,
            task_id: [u8; 32],
            error: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let error: BoundedVec<u8, T::MaxTaskOutputSize> = error
                .try_into()
                .map_err(|_| Error::<T>::TaskOutputTooLarge)?;

            Tasks::<T>::try_mutate(task_id, |task_opt| -> DispatchResult {
                let task = task_opt.as_mut().ok_or(Error::<T>::TaskNotFound)?;
                
                ensure!(
                    task.assigned_agent == Some(who.clone()),
                    Error::<T>::NotAssignedAgent
                );
                ensure!(task.status == TaskStatus::Running, Error::<T>::TaskNotRunning);

                if task.retry_count < task.max_retries {
                    task.status = TaskStatus::Pending;
                    task.retry_count = task.retry_count.saturating_add(1);
                    task.assigned_agent = None;
                    task.started_at = None;
                    task.error = Some(error.clone());

                    PendingQueue::<T>::try_mutate(|queue| -> DispatchResult {
                        queue.try_push(task_id).map_err(|_| Error::<T>::QueueFull)
                    })?;

                    AgentActiveTasks::<T>::mutate(&who, |count| {
                        *count = count.saturating_sub(1);
                    });
                    TasksByAgent::<T>::remove(&who, task_id);

                    Self::deposit_event(Event::TaskRetried {
                        task_id,
                        retry_count: task.retry_count,
                    });
                } else {
                    task.status = TaskStatus::Failed;
                    task.error = Some(error.clone());
                    task.completed_at = Some(frame_system::Pallet::<T>::block_number());

                    AgentActiveTasks::<T>::mutate(&who, |count| {
                        *count = count.saturating_sub(1);
                    });

                    Self::deposit_event(Event::TaskFailed {
                        task_id,
                        error: error.into_inner(),
                    });
                }

                Ok(())
            })?;

            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_ref_time(5_000) + T::DbWeight::get().writes(2))]
        pub fn cancel_task(
            origin: OriginFor<T>,
            task_id: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Tasks::<T>::try_mutate(task_id, |task_opt| -> DispatchResult {
                let task = task_opt.as_mut().ok_or(Error::<T>::TaskNotFound)?;
                
                ensure!(task.requester == who, Error::<T>::NotTaskRequester);
                ensure!(
                    task.status != TaskStatus::Completed && 
                    task.status != TaskStatus::Failed &&
                    task.status != TaskStatus::Cancelled,
                    Error::<T>::TaskAlreadyCompleted
                );

                if let Some(agent) = &task.assigned_agent {
                    AgentActiveTasks::<T>::mutate(agent, |count| {
                        *count = count.saturating_sub(1);
                    });
                    TasksByAgent::<T>::remove(agent, task_id);
                }

                task.status = TaskStatus::Cancelled;
                task.completed_at = Some(frame_system::Pallet::<T>::block_number());

                Ok(())
            })?;

            PendingQueue::<T>::mutate(|queue| {
                queue.retain(|id| *id != task_id);
            });

            Self::deposit_event(Event::TaskCancelled { task_id });

            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_ref_time(50_000) + T::DbWeight::get().writes(10))]
        pub fn process_queue(
            origin: OriginFor<T>,
            max_tasks: u32,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut processed = 0u32;
            let current_block = frame_system::Pallet::<T>::block_number();
            let timeout = T::TaskTimeoutBlocks::get();

            let mut to_remove = Vec::new();
            let mut timed_out = Vec::new();

            for task_id in PendingQueue::<T>::get().iter() {
                if processed >= max_tasks {
                    break;
                }

                if let Some(task) = Tasks::<T>::get(task_id) {
                    if let Some(deadline) = task.deadline {
                        if current_block > deadline {
                            timed_out.push(*task_id);
                            to_remove.push(*task_id);
                            continue;
                        }
                    }

                    if let Some(started) = task.started_at {
                        if current_block > started + timeout {
                            timed_out.push(*task_id);
                            to_remove.push(*task_id);
                            continue;
                        }
                    }

                    processed = processed.saturating_add(1);
                }
            }

            for task_id in timed_out {
                Tasks::<T>::mutate(task_id, |task_opt| {
                    if let Some(task) = task_opt {
                        task.status = TaskStatus::Timeout;
                        task.completed_at = Some(current_block);
                        
                        if let Some(agent) = &task.assigned_agent {
                            AgentActiveTasks::<T>::mutate(agent, |count| {
                                *count = count.saturating_sub(1);
                            });
                        }

                        Self::deposit_event(Event::TaskTimeout { task_id });
                    }
                });
            }

            PendingQueue::<T>::mutate(|queue| {
                queue.retain(|id| !to_remove.contains(id));
            });

            Self::deposit_event(Event::QueueProcessed { count: processed });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn generate_task_id() -> [u8; 32] {
            use sp_io::hashing::blake2_256;
            
            let counter = TaskCounter::<T>::get();
            let mut input = counter.to_le_bytes().to_vec();
            input.extend_from_slice(&frame_system::Pallet::<T>::block_number().encode());
            
            blake2_256(&input)
        }

        pub fn get_task(task_id: &[u8; 32]) -> Option<Task<T>> {
            Tasks::<T>::get(task_id)
        }

        pub fn get_tasks_by_requester(requester: &T::AccountId) -> Vec<[u8; 32]> {
            TasksByRequester::<T>::iter_prefix(requester)
                .filter_map(|(task_id, is_active)| if is_active { Some(task_id) } else { None })
                .collect()
        }

        pub fn get_tasks_by_agent(agent: &T::AccountId) -> Vec<[u8; 32]> {
            TasksByAgent::<T>::iter_prefix(agent)
                .filter_map(|(task_id, is_active)| if is_active { Some(task_id) } else { None })
                .collect()
        }

        pub fn get_pending_tasks() -> Vec<[u8; 32]> {
            PendingQueue::<T>::get().into_inner()
        }

        pub fn get_task_result(task_id: &[u8; 32]) -> Option<TaskResult<T>> {
            TaskResults::<T>::get(task_id)
        }

        pub fn get_queue_size() -> u32 {
            PendingQueue::<T>::get().len() as u32
        }

        pub fn get_total_tasks() -> u64 {
            TaskCounter::<T>::get()
        }

        pub fn get_active_task_count(agent: &T::AccountId) -> u32 {
            AgentActiveTasks::<T>::get(agent)
        }

        pub fn find_compatible_tasks(
            capabilities: &[Vec<u8>],
        ) -> Vec<[u8; 32]> {
            let mut compatible = Vec::new();
            
            for task_id in PendingQueue::<T>::get().iter() {
                if let Some(task) = Tasks::<T>::get(task_id) {
                    if task.status == TaskStatus::Pending {
                        let has_all_caps = task.required_capabilities.iter().all(|req_cap| {
                            capabilities.iter().any(|cap| cap.as_slice() == req_cap.as_slice())
                        });
                        
                        if has_all_caps {
                            compatible.push(*task_id);
                        }
                    }
                }
            }
            
            compatible
        }

        pub fn get_next_task_by_priority() -> Option<[u8; 32]> {
            let mut best_task: Option<([u8; 32], TaskPriority)> = None;
            
            for task_id in PendingQueue::<T>::get().iter() {
                if let Some(task) = Tasks::<T>::get(task_id) {
                    if task.status == TaskStatus::Pending {
                        match &best_task {
                            None => best_task = Some((*task_id, task.priority)),
                            Some((_, best_priority)) => {
                                if Self::compare_priority(&task.priority, best_priority) {
                                    best_task = Some((*task_id, task.priority));
                                }
                            }
                        }
                    }
                }
            }
            
            best_task.map(|(id, _)| id)
        }

        fn compare_priority(a: &TaskPriority, b: &TaskPriority) -> bool {
            matches!(
                (a, b),
                (TaskPriority::Critical, TaskPriority::High) |
                (TaskPriority::Critical, TaskPriority::Normal) |
                (TaskPriority::Critical, TaskPriority::Low) |
                (TaskPriority::High, TaskPriority::Normal) |
                (TaskPriority::High, TaskPriority::Low) |
                (TaskPriority::Normal, TaskPriority::Low)
            )
        }
    }
}
