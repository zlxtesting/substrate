// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Node-specific RPC methods for interaction with contracts.

use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

use serde::Serialize;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::{BlockId, UncheckedExtrinsic},
	traits::{Block as BlockT, SignedExtension},
	DeserializeOwned,
};

pub use pallet_election_provider_multi_phase_rpc_runtime_api::{EpmRuntimeApi, RawSolution};

/// ....
#[rpc]
pub trait EpmRpcApi<AccountId, Address, Call, Extra, Signature, Solution>
where
	AccountId: Codec,
	Address: Codec,
	Extra: SignedExtension,
	Call: Codec,
	Signature: Codec,
	Solution: Codec + Serialize + DeserializeOwned,
{
	#[rpc(name = "epm_submit")]
	fn submit(
		&self,
		origin: AccountId,
		raw_solution: Box<RawSolution<Solution>>,
	) -> Result<UncheckedExtrinsic<Address, Call, Signature, Extra>>;
}

/// EPM specific RPC methods.
pub struct EpmRpc<C, B> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> EpmRpc<C, B> {
	/// Create new EPM RPC endpoint.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}
impl<C, Block, AccountId, Address, Call, Extra, Solution, Signature>
	EpmRpcApi<AccountId, Address, Call, Extra, Signature, Solution> for EpmRpc<C, Block>
where
	AccountId: Codec,
	Address: Codec,
	Extra: SignedExtension,
	Block: BlockT,
	Call: Codec,
	Signature: Codec,
	Solution: Codec + Serialize + DeserializeOwned,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: EpmRuntimeApi<Block, AccountId, Address, Call, Extra, Solution, Signature>,
{
	fn submit(
		&self,
		origin: AccountId,
		raw_solution: Box<RawSolution<Solution>>,
	) -> Result<UncheckedExtrinsic<Address, Call, Signature, Extra>> {
		let api = self.client.runtime_api();
		let at = BlockId::Hash(self.client.info().best_hash);

		Ok(api.submit(&at, origin, raw_solution).expect("todo error handling"))
	}
}
