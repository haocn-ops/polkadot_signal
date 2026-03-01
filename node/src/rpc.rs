use jsonrpsee::RpcModule;
use polkadot_signal_runtime::{opaque::Block, AccountId, Balance, BlockNumber, Hash};
use sc_client_api::{AuxStore, BlockBackend, HeaderBackend, StorageProvider};
use sc_rpc::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderMetadata};
use std::sync::Arc;

pub struct FullDeps<C, P> {
	pub client: Arc<C>,
	pub pool: Arc<P>,
	pub deny_unsafe: DenyUnsafe,
}

pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ BlockBackend<Block>
		+ StorageProvider<Block, <C as HeaderBackend<Block>>::Backend>
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Send
		+ Sync
		+ 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, BlockNumber>
		+ pallet_transaction_payment_rpc::TransactionPaymentApi<Block, Balance>
		+ BlockBuilder<Block>,
	P: TransactionPool + Sync + Send + 'static,
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut module = RpcModule::new(());
	let FullDeps { client, pool, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client).into_rpc())?;

	Ok(module)
}
