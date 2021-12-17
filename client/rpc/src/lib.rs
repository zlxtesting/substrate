// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Substrate RPC implementation.
//!
//! A core implementation of Substrate RPC interfaces.

#![warn(missing_docs)]

use futures::{
	task::{FutureObj, Spawn, SpawnError},
	FutureExt, Stream, StreamExt,
};
use jsonrpsee::SubscriptionSink;
use sp_core::{testing::TaskExecutor, traits::SpawnNamed};
use sp_runtime::Serialize;
use std::sync::Arc;
use std::sync::atomic::*;

pub use sc_rpc_api::DenyUnsafe;

pub mod author;
pub mod chain;
pub mod offchain;
pub mod state;
pub mod system;

#[cfg(any(test, feature = "test-helpers"))]
pub mod testing;

/// Task executor that is being used by RPC subscriptions.
#[derive(Clone)]
pub struct SubscriptionTaskExecutor(Arc<dyn SpawnNamed>);

impl SubscriptionTaskExecutor {
	/// Create a new `Self` with the given spawner.
	pub fn new(spawn: impl SpawnNamed + 'static) -> Self {
		Self(Arc::new(spawn))
	}
}

impl Spawn for SubscriptionTaskExecutor {
	fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
		self.0
			.spawn("substrate-rpc-subscription", Some("rpc"), future.map(drop).boxed());
		Ok(())
	}
}

impl Default for SubscriptionTaskExecutor {
	fn default() -> Self {
		let spawn = TaskExecutor::default();
		Self::new(spawn)
	}
}

/// Helper for polling a subscription and sending out responses.
pub async fn handle_subscription_stream<S, T>(
	mut stream: S,
	mut sink: SubscriptionSink,
	method: &str,
) where
	S: Stream<Item = T> + Unpin,
	T: Serialize,
{
	log::debug!("starting subscription `{}´", method);
	loop {
		let timeout = tokio::time::sleep(std::time::Duration::from_secs(60));
		tokio::pin!(timeout);

		tokio::select! {
			Some(item) = stream.next() => {
				if let Err(e) = sink.send(&item) {
					log::debug!("Could not send data to '{}' subscriber: {:?}", method, e);
					break;
				}
			},
			_ = &mut timeout => {
				if sink.is_closed() {
					log::debug!("subscription `{}' timeout", method);
					break;
				}
			}
			else => break,
		};
	}
	log::debug!("closing subscription `{}´", method);
}
