use polkadot_signal_runtime::{
	self, opaque::Block, AccountId, Balance, BlockNumber, Hash, VERSION,
};
use sc_client_api::{BlockBackend, HeaderBackend};
use sc_consensus_aura::{ImportQueueParams, SlotProportion, StartAuraParams};
use sc_executor::WasmExecutor;
use sc_network::NetworkBackend;
pub use sc_service::{
	config::KeystoreConfig, error::Error as ServiceError, BasePath, ChainSpec, Configuration,
	TaskManager,
};
use sc_service::{NetworkStarter, PartialComponents, TFullBackend, TFullClient, WarpSync};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::Ss58Codec, sr25519};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use sp_timestamp::InherentDataProvider as TimestampInherent;
use std::{sync::Arc, time::Duration};

pub type FullClient = TFullClient<Block, polkadot_signal_runtime::RuntimeApi, WasmExecutor>;
type FullBackend = TFullBackend<Block>;

pub fn development_config(id: &str) -> Result<ChainSpec, String> {
	let mut properties = serde_json::map::Map::new();
	properties.insert("tokenSymbol".into(), "SIGNAL".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), 42.into());

	Ok(ChainSpec::from_genesis(
		id,
		"dev",
		ChainSpec::DEV_CHAIN_ID,
		|| {
			let alice = get_account_id_from_seed("Alice");
			let bob = get_account_id_from_seed("Bob");
			
			polkadot_signal_runtime::genesis_config(
				vec![sr25519::Public::from_string(&alice.to_ss58check()).unwrap().into()],
				vec![
					(
						alice.clone(),
						get_account_id_from_seed("Alice//stash"),
					),
					(
						bob.clone(),
						get_account_id_from_seed("Bob//stash"),
					),
				],
				true,
			)
		},
		vec![],
		None,
		None,
		None,
		None,
		Some(properties),
		VERSION.spec_version,
	))
}

fn get_account_id_from_seed(seed: &str) -> AccountId {
	sr25519::Public::from_string(&format!("//{}", seed))
		.expect("valid seed")
		.into()
}

pub fn new_partial(
	config: &Configuration,
) -> Result<PartialComponents<FullClient, FullBackend, (), sc_consensus::DefaultImportQueue<Block, FullClient>, sc_transaction_pool::FullPool<Block, FullClient>, ()>, ServiceError> {
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = sc_executor::WasmExecutor::builder()
		.with_execution_method(config.wasm_method)
		.with_onchain_heap_allocations(config.default_heap_pages)
		.with_offchain_heap_allocations(config.max_runtime_instances)
		.with_runtime_cache_size(config.runtime_cache_size)
		.build();

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, polkadot_signal_runtime::RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let import_queue = sc_consensus_aura::import_queue::<AuraId, _, _, _, _, _>(
		ImportQueueParams {
			block_import: client.clone(),
			justification_import: None,
			client: client.clone(),
			create_inherent_data_providers: |_, _| async move {
				Ok((TimestampInherent::from_now(),))
			},
			spawner: &task_manager.spawn_essential_handle(),
			registry: config.prometheus_registry(),
			check_for_equivocation: Default::default(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			compatibility_mode: Default::default(),
		},
	)?;

	Ok(PartialComponents {
		client,
		backend,
		task_manager,
		keystore_container,
		select_chain: (),
		import_queue,
		transaction_pool,
		other: (telemetry,),
	})
}

pub fn new_full<Network: NetworkBackend<Block, <Block as BlockT>::Hash>>(
	mut config: Configuration,
) -> Result<TaskManager, ServiceError> {
	let PartialComponents {
		client,
		backend,
		mut task_manager,
		keystore_container,
		select_chain: _,
		import_queue,
		transaction_pool,
		other: (mut telemetry,),
	} = new_partial(&config)?;

	let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
		&client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"),
		&config.chain_spec,
	);

	config
		.network
		.extra_sets
		.push(sc_consensus_grandpa::grandpa_peers_set_config(grandpa_protocol_name.clone()));

	let warp_sync = Arc::new(WarpSync::new(backend.clone(), (), Vec::new()));

	let (network, system_rpc_tx, tx_handler_controller, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: Some(sc_service::WarpSyncParams {
				warp_sync,
				target_chunk_size: None,
			}),
		})?;

	if config.offchain_worker.enabled {
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-worker",
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				is_validator: config.role.is_authority(),
				keystore: Some(keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(OffchainTransactionPoolFactory::new(
					transaction_pool.clone(),
				)),
				network_provider: network.clone(),
				enable_http_requests: true,
				custom_extensions: |_| vec![],
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let backoff_authoring_blocks: Option<()> = None;
	let prometheus_registry = config.prometheus_registry().cloned();

	let aura = sc_consensus_aura::start_aura::<AuraId, _, _, _, _, _, _, _, _, _, _>(
		StartAuraParams {
			slot_duration: sc_consensus_aura::slot_duration(&*client)?,
			client: client.clone(),
			select_chain: (),
			block_import: client.clone(),
			proposer: sc_basic_authorship::ProposerFactory::new(
				task_manager.spawn_handle(),
				client.clone(),
				transaction_pool.clone(),
				prometheus_registry.as_ref(),
				telemetry.as_ref().map(|x| x.handle()),
			),
			create_inherent_data_providers: |_, ()| async move {
				Ok((TimestampInherent::from_now(),))
			},
			force_authoring,
			backoff_authoring_blocks,
			keystore: keystore_container.keystore(),
			sync_oracle: network.clone(),
			justification_sync_link: network.clone(),
			block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
			max_block_proposal_slot_portion: None,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			compatibility_mode: Default::default(),
		},
	)?;

	let overseer_handle = task_manager.spawn_handle();

	task_manager.spawn_essential_handle().spawn_blocking(
		"aura",
		Some("block-authoring"),
		aura,
	);

	let grandpa_config = sc_consensus_grandpa::Config {
		gossip_duration: Duration::from_millis(333),
		justification_period: 512,
		name: None,
		observer_enabled: false,
		keystore: Some(keystore_container.keystore()),
		local_role: role,
		telemetry: telemetry.as_ref().map(|x| x.handle()),
		protocol_name: grandpa_protocol_name,
	};

	if role.is_authority() {
		let grandpa = sc_consensus_grandpa::GrandpaBlockImport::new(
			grandpa_config.clone(),
			backend.clone(),
			client.clone(),
			network.clone(),
			network.clone(),
			offchain_tx_pool_factory.clone(),
			telemetry.as_ref().map(|x| x.handle()),
		)?;

		let voting_rule_builder = sc_consensus_grandpa::VotingRulesBuilder::default();

		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			None,
			sc_consensus_grandpa::run_grandpa_voter(grandpa_config, grandpa, voting_rule_builder),
		);
	}

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				deny_unsafe,
			};
			crate::rpc::create_full(deps).map_err(Into::into)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network,
		client,
		keystore: keystore_container.keystore(),
		task_manager: &mut task_manager,
		transaction_pool,
		rpc_builder: rpc_extensions_builder,
		backend,
		system_rpc_tx,
		tx_handler_controller,
		sync_service: network.clone(),
		warp_sync: None,
		telemetry: telemetry.as_mut(),
	})?;

	network_starter.start_network();
	Ok(task_manager)
}
