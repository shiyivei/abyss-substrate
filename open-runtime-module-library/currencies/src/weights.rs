//! Autogenerated weights for module_currencies
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-05-04, STEPS: [50, ], REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// /Users/xiliangchen/projects/acala/target/release/acala
// benchmark
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=module_currencies
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./currencies/src/weights.rs
// --template
// ../templates/orml-weight-template.hbs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for module_currencies.
pub trait WeightInfo {
	fn transfer_non_native_currency() -> Weight;
	fn transfer_native_currency() -> Weight;
	fn update_balance_non_native_currency() -> Weight;
	fn update_balance_native_currency_creating() -> Weight;
	fn update_balance_native_currency_killing() -> Weight;
}

/// Default weights.
impl WeightInfo for () {
	fn transfer_non_native_currency() -> Weight {
		Weight::from_ref_time(60_000_000)
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	fn transfer_native_currency() -> Weight {
		Weight::from_ref_time(60_000_000)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	fn update_balance_non_native_currency() -> Weight {
		Weight::from_ref_time(29_000_000)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	fn update_balance_native_currency_creating() -> Weight {
		Weight::from_ref_time(31_000_000)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	fn update_balance_native_currency_killing() -> Weight {
		Weight::from_ref_time(37_000_000)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
}