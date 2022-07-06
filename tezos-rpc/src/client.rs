use {
    crate::constants, crate::http, crate::models::operation::OperationGroup, crate::protocol_rpc,
    crate::shell_rpc, crate::shell_rpc::injection::block::InjectionBlockPayload,
};

pub struct TezosRPCContext {
    /// A chain identifier. This is either a chain hash in Base58Check notation or a one the predefined aliases: 'main', 'test'.
    pub chain_id: String,
    pub http_client: http::TezosHttp,
}
impl TezosRPCContext {
    /// Changes the rpc endpoint used in RPC requests.
    pub fn change_rpc_endpoint(&mut self, rpc_endpoint: &str) {
        self.http_client
            .change_rpc_endpoint(rpc_endpoint.to_string());
    }
}

pub struct TezosRPC {
    pub context: TezosRPCContext,
}

impl TezosRPC {
    /// Creates a Tezos RPC client that will connect to the specified node RPC.
    ///
    /// ```rust
    /// use tezos_rpc::client::{TezosRPC};
    ///
    /// let client = TezosRPC::new("https://tezos-node.prod.gke.papers.tech");
    /// ```
    pub fn new(rpc_endpoint: &str) -> Self {
        TezosRPC {
            context: TezosRPCContext {
                chain_id: constants::DEFAULT_CHAIN_ALIAS.to_string(),
                http_client: http::TezosHttp::new(rpc_endpoint),
            },
        }
    }

    /// Creates a Tezos RPC client that will connect to the specified node RPC.
    ///
    /// This method allows the user to provide the chain identifier that will be used when
    /// sending requests to the RPC. The default is `main`.
    ///
    /// ```rust
    /// use tezos_rpc::client::{TezosRPC};
    ///
    /// let client = TezosRPC::new_with_chain_id("https://tezos-node.prod.gke.papers.tech", "NetXLH1uAxK7CCh");
    /// ```
    pub fn new_with_chain_id(rpc_endpoint: &str, chain_id: &str) -> Self {
        TezosRPC {
            context: TezosRPCContext {
                chain_id: chain_id.to_string(),
                http_client: http::TezosHttp::new(rpc_endpoint),
            },
        }
    }
}

// Tezos protocol-independent RPCs
// See [RPCs - Reference](https://tezos.gitlab.io/shell/rpc.html) for more details.
impl TezosRPC {
    /// Get the chain unique identifier.
    ///
    /// [`GET /chains/<chain_id>/chain_id`](https://tezos.gitlab.io/shell/rpc.html#get-chains-chain-id-chain-id)
    pub fn get_chain_id(&self) -> shell_rpc::chains::chain::chain_id::RPCRequestBuilder {
        shell_rpc::chains::chain::chain_id::get(&self.context)
    }

    /// Get a list of block hashes from `<chain>`, up to the last checkpoint, swith
    /// decreasing fitness. Without arguments it returns the head of the chain.
    ///
    /// Optional arguments allow to return the list of predecessors of a given block or of a set of blocks.
    ///
    /// [`GET /chains/<chain_id>/blocks`](https://tezos.gitlab.io/shell/rpc.html#get_chains__chain_id__blocks)
    pub fn get_blocks(&self) -> shell_rpc::chains::chain::blocks::RPCRequestBuilder {
        shell_rpc::chains::chain::blocks::get(&self.context)
    }

    /// Get blocks that have been declared invalid along with the errors that led to them being declared invalid.
    ///
    /// [`GET /chains/<chain_id>/invalid_blocks`](https://tezos.gitlab.io/shell/rpc.html#get-chains-chain-id-invalid-blocks)
    pub fn get_invalid_blocks(
        &self,
    ) -> shell_rpc::chains::chain::invalid_blocks::RPCRequestBuilder {
        shell_rpc::chains::chain::invalid_blocks::get(&self.context)
    }

    /// Get the errors that appeared during the block (in)validation.
    ///
    /// [`GET /chains/<chain_id>/invalid_blocks/<block_hash>`](https://tezos.gitlab.io/shell/rpc.html#get-chains-chain-id-invalid-blocks-block-hash)
    pub fn get_invalid_block<'a>(
        &'a self,
        block_hash: &'a String,
    ) -> shell_rpc::chains::chain::invalid_blocks::block::GetRPCRequestBuilder {
        shell_rpc::chains::chain::invalid_blocks::block::get(&self.context, block_hash)
    }

    /// Remove an invalid block for the tezos storage.
    ///
    /// [`DELETE /chains/<chain_id>/invalid_blocks/<block_hash>`](https://tezos.gitlab.io/shell/rpc.html#delete-chains-chain-id-invalid-blocks-block-hash)
    pub fn remove_invalid_block<'a>(
        &'a self,
        block_hash: &'a String,
    ) -> shell_rpc::chains::chain::invalid_blocks::block::DeleteRPCRequestBuilder {
        shell_rpc::chains::chain::invalid_blocks::block::delete(&self.context, block_hash)
    }

    /// Get the bootstrap status of a chain.
    ///
    /// [`GET /chains/<chain_id>/is_bootstrapped`](https://tezos.gitlab.io/shell/rpc.html#get-chains-chain-id-is-bootstrapped)
    pub fn is_bootstrapped(&self) -> shell_rpc::chains::chain::is_bootstrapped::RPCRequestBuilder {
        shell_rpc::chains::chain::is_bootstrapped::get(&self.context)
    }

    /// Get the current caboose for this chain.
    ///
    /// [`GET /chains/<chain_id>/levels/caboose`](https://tezos.gitlab.io/shell/rpc.html#get-chains-chain-id-levels-caboose)
    pub fn get_caboose(&self) -> shell_rpc::chains::chain::levels::caboose::RPCRequestBuilder {
        shell_rpc::chains::chain::levels::caboose::get(&self.context)
    }

    /// Get the current checkpoint for this chain.
    ///
    /// [`GET /chains/<chain_id>/levels/checkpoint`](https://tezos.gitlab.io/shell/rpc.html#get-chains-chain-id-levels-checkpoint)
    pub fn get_checkpoint(
        &self,
    ) -> shell_rpc::chains::chain::levels::checkpoint::RPCRequestBuilder {
        shell_rpc::chains::chain::levels::checkpoint::get(&self.context)
    }

    /// Get the current savepoint for this chain.
    ///
    /// [`GET /chains/<chain_id>/levels/savepoint`](https://tezos.gitlab.io/shell/rpc.html#get-chains-chain-id-levels-savepoint)
    pub fn get_savepoint(&self) -> shell_rpc::chains::chain::levels::savepoint::RPCRequestBuilder {
        shell_rpc::chains::chain::levels::savepoint::get(&self.context)
    }

    /// Inject an operation in node and broadcast it.
    ///
    /// The `signed_operation_contents` should be constructed using contextual RPCs
    /// from the latest block and signed by the client.
    ///
    /// The injection of the operation will apply it on the current mempool context.
    /// This context may change at each operation injection or operation reception from peers.
    ///
    /// By default, the RPC will wait for the operation to be (pre-)validated before returning.
    /// However, if `?async` is true, the function returns immediately.
    /// The optional `?chain` parameter can be used to specify whether to inject on the test chain or the main chain.
    ///
    /// Returns the ID of the operation.
    ///
    /// [`POST /injection/operation?[async]&[chain=<chain_id>]`](https://tezos.gitlab.io/shell/rpc.html#post-injection-operation)
    pub fn inject_operation<'a>(
        &'a self,
        signed_operation_contents: &'a String,
    ) -> shell_rpc::injection::operation::RPCRequestBuilder {
        shell_rpc::injection::operation::post(&self.context, signed_operation_contents)
    }

    /// Inject a block in the node and broadcast it.
    ///
    /// The `operations` might be pre-validated using a contextual RPCs
    /// from the latest block (e.g. `/blocks/head/context/preapply`).
    ///
    /// By default, the RPC will wait for the block to be validated before answering.
    /// If `?async` is true, the function returns immediately. Otherwise, the block will be validated before the result is returned. If ?force is true, it will be injected even on non strictly increasing fitness. An optional ?chain parameter can be used to specify whether to inject on the test chain or the main chain.
    ///
    /// Returns the ID of the block [BlockHash].
    ///
    /// [`POST /injection/block?[async]&[force]&[chain=<chain_id>]]`](https://tezos.gitlab.io/shell/rpc.html#post-injection-block)
    pub fn inject_block<'a>(
        &'a self,
        payload: &'a InjectionBlockPayload,
    ) -> shell_rpc::injection::block::RPCRequestBuilder {
        shell_rpc::injection::block::post(&self.context, payload)
    }
}

// Tezos protocol-dependent RPCs
// See [RPCs - Reference](https://tezos.gitlab.io/active/rpc.html) for more details.
impl TezosRPC {
    /// Get all the information about a block.
    /// The associated metadata may not be present depending on the history mode and block's distance from the head.
    ///
    /// [`GET /chains/<chain_id>/blocks/<block_id>?[force_metadata]&[metadata=<metadata_rpc_arg>]`](https://tezos.gitlab.io/active/rpc.html#get-block-id)
    pub fn get_block(&self) -> protocol_rpc::block::RPCRequestBuilder {
        protocol_rpc::block::get(&self.context)
    }

    /// Access the list of all constants.
    ///
    /// [`GET /chains/<chain_id>/blocks/<block>/context/constants`](https://tezos.gitlab.io/active/rpc.html#get-block-id-context-constants)
    pub fn get_constants(&self) -> protocol_rpc::block::context::constants::RPCRequestBuilder {
        protocol_rpc::block::context::constants::get(&self.context)
    }

    /// Access the complete status of a contract.
    ///
    /// * `address` : A contract identifier encoded in b58check. e.g. `KT1HxgqnVjGy7KsSUTEsQ6LgpD5iKSGu7QpA`
    ///
    /// Optional query arguments :
    /// * `normalize_types` : Whether types should be normalized (annotations removed, combs flattened) or kept as they appeared in the original script.
    ///
    /// [`GET ../<block_id>/context/contracts/<contract_id>?[normalize_types]`](https://tezos.gitlab.io/jakarta/rpc.html#get-block-id-context-contracts-contract-id)
    pub fn get_contract<'a>(
        &'a self,
        address: &'a String,
    ) -> protocol_rpc::block::context::contract::RPCRequestBuilder<'a> {
        protocol_rpc::block::context::contract::get(&self.context, address)
    }

    /// Access the balance of a contract.
    ///
    /// [`GET /chains/<chain_id>/blocks/<block>/context/contracts/<contract_id>/balance`](https://tezos.gitlab.io/active/rpc.html#get-block-id-context-contracts-contract-id-balance)
    pub fn get_balance<'a>(
        &'a self,
        address: &'a String,
    ) -> protocol_rpc::block::context::contract::balance::RPCRequestBuilder {
        protocol_rpc::block::context::contract::balance::get(&self.context, address)
    }

    /// Access the counter of a contract.
    ///
    /// [`GET /chains/<chain_id>/blocks/<block>/context/contracts/<contract_id>/counter`](https://tezos.gitlab.io/active/rpc.html#get-block-id-context-contracts-contract-id-counter)
    pub fn get_counter<'a>(
        &'a self,
        address: &'a String,
    ) -> protocol_rpc::block::context::contract::counter::RPCRequestBuilder {
        protocol_rpc::block::context::contract::counter::get(&self.context, address)
    }

    /// Access the manager public key of a contract.
    ///
    /// [`GET /chains/<chain_id>/blocks/<block_id>/context/contracts/<contract_id>/manager_key`](https://tezos.gitlab.io/active/rpc.html#get-block-id-context-contracts-contract-id-manager-key)
    pub fn get_manager_key<'a>(
        &'a self,
        address: &'a String,
    ) -> protocol_rpc::block::context::contract::manager_key::RPCRequestBuilder {
        protocol_rpc::block::context::contract::manager_key::get(&self.context, address)
    }

    /// Get the (optionally paginated) list of values in a big map. Order of values is unspecified, but is guaranteed to be consistent.
    ///
    /// Optional query arguments:
    ///
    /// * `offset` : Skip the first [offset] values. Useful in combination with [length] for pagination.
    /// * `length` : Only retrieve [length] values. Useful in combination with [offset] for pagination.
    ///
    /// [`GET /chains/<chain_id>/blocks/<block_id>/context/big_maps/<big_map_id>?[offset=<uint>]&[length=<uint>]`](https://tezos.gitlab.io/active/rpc.html#get-block-id-context-big-maps-big-map-id)
    pub fn get_big_map<'a>(
        &'a self,
        id: &'a u32,
    ) -> protocol_rpc::block::context::big_maps::big_map::RPCRequestBuilder {
        protocol_rpc::block::context::big_maps::big_map::get(&self.context, id)
    }

    /// Access the value associated with a key in a big map.
    ///
    /// * `script_expr` - The Blake2b hash of the map key packed (Base58Check-encoded)
    /// e.g. `expru3MJA26WX3kQ9WCPBPhCqsXE33BBtXnTQpYmQwtbJyHSu3ME9E`
    ///
    /// [`GET /chains/<chain_id>/blocks/<block_id>/context/big_maps/<big_map_id>/<script_expr>`](https://tezos.gitlab.io/active/rpc.html#get-block-id-context-big-maps-big-map-id-script-expr)
    pub fn get_big_map_value<'a>(
        &'a self,
        big_map_id: &'a u32,
        script_expr: &'a String,
    ) -> protocol_rpc::block::context::big_maps::big_map::script_expr::RPCRequestBuilder<'a> {
        protocol_rpc::block::context::big_maps::big_map::script_expr::get(
            &self.context,
            big_map_id,
            script_expr,
        )
    }

    /// Simulate the application of the operations with the context of the given block and return the result of each operation application.
    ///
    /// [`POST /chains/<chain_id>/blocks/<block_id>/helpers/preapply/operations`](https://tezos.gitlab.io/active/rpc.html#post-block-id-helpers-preapply-operations)
    pub fn preapply_operations<'a>(
        &'a self,
        operations: &'a Vec<&OperationGroup>,
    ) -> protocol_rpc::block::helpers::preapply::operations::RPCRequestBuilder<'a> {
        protocol_rpc::block::helpers::preapply::operations::post(&self.context, operations)
    }

    /// Run an operation without signature checks.
    ///
    /// [`POST /chains/<chain_id>/blocks/<block_id>/helpers/scripts/run_operation`](https://tezos.gitlab.io/api/rpc.html#post-block-id-helpers-scripts-run-operation)
    pub fn run_operation<'a>(
        &'a self,
        operation: &'a OperationGroup,
    ) -> protocol_rpc::block::helpers::scripts::run_operation::RPCRequestBuilder<'a> {
        protocol_rpc::block::helpers::scripts::run_operation::post(&self.context, operation)
    }
}
