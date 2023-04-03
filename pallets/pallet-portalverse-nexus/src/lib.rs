#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use pallet_dex::traits::Exchange;
use primitives::Balance;
use primitives::CurrencyId;
use scale_info::prelude::vec::Vec;

pub use primitives::{
	currency::{POV, UC},
	TokenSymbol,
};

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

	use frame_support::traits::{Currency, ExistenceRequirement, ReservableCurrency, UnixTime};
	use frame_system::pallet_prelude::*;
	//use orml_currencies::CurrencyIdOf;

	/// To represent a rendering instance
	#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, TypeInfo, Debug)]
	pub struct Renderer {
		// ipv6 or ipv4(u32)
		pub host: Vec<u8>,
		pub portoffset: u16,
		pub status: u8,
		pub gameid: Vec<u8>,
		pub resolution: Vec<u8>, // TODO POV staking
	}

	// pub(crate) type BalanceOf<T> =
	// 	<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	// pub(crate) type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<
	// 	<T as frame_system::Config>::AccountId,
	// >>::CurrencyId;

	pub type BalanceOf<T> = <<T as Config>::NativeCurrency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<
		<T as frame_system::Config>::AccountId,
	>>::CurrencyId;
	pub(crate) type AmountOf<T> = <<T as Config>::Currency as MultiCurrencyExtended<
		<T as frame_system::Config>::AccountId,
	>>::Amount;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: MultiCurrencyExtended<Self::AccountId, CurrencyId = CurrencyId>;

		type NativeCurrency: ReservableCurrency<Self::AccountId>;

		type Exchange: Exchange<Self::AccountId, Balance, CurrencyId>;

		type TimeProvider: UnixTime;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		NotEnoughBalance,
		NotNativeCurrency,
		AlreadyRegistered,
		InvalidCurrencyId,
	}

	/// Storage of all renderers.
	/// The storage does not guarante that the host:port of a renderer is unique,
	/// some extra constraints must be applied
	#[pallet::storage]
	#[pallet::getter(fn renderers)]
	pub type Renderers<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Renderer, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn orders)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Orders<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		u64, //Starttime,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn ipfslink)]
	pub type Ipfslink<T: Config> = StorageValue<_, Vec<u8>>;

	#[pallet::storage]
	#[pallet::getter(fn pov2uc)]
	pub type Pov2uc<T: Config> = StorageValue<_, u128>;

	#[pallet::storage]
	#[pallet::getter(fn uc2gametime)]
	pub type Uc2gametime<T: Config> = StorageValue<_, u128>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored {
			something: u32,
			who: T::AccountId,
		},
		/// A renderer registers on chain
		RendererRegistered(T::AccountId),
		RendererDeregistered(T::AccountId),
		RendererConnected(T::AccountId, T::AccountId, u64),
		RendererDisconnected(T::AccountId, T::AccountId, u64),
		UpdatedIpfsLink(Vec<u8>),
		UpdatedPov2uc(u128),
		UpdatedUc2gametime(u128),
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn register(
			origin: OriginFor<T>,
			host: Vec<u8>,
			portoffset: u16,
			games: Vec<u8>,
			resolution: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let renderer = ensure_signed(origin)?;
			Renderers::<T>::try_mutate(&renderer, |exists| -> DispatchResult {
				ensure!(exists.is_none(), Error::<T>::AlreadyRegistered);
				exists.replace(Renderer { host, portoffset, status: 0, gameid: games, resolution });
				Ok(())
			})?;
			Self::deposit_event(Event::RendererRegistered(renderer));
			Ok(().into())
		}

		/// deregister a renderer.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn deregister(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let renderer = ensure_signed(origin)?;
			Renderers::<T>::remove(&renderer);
			Self::deposit_event(Event::RendererDeregistered(renderer));
			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn connect(
			origin: OriginFor<T>,
			renderer: T::AccountId,
			// TODO add stablecoin payment
		) -> DispatchResult {
			let player = ensure_signed(origin)?;
			Renderers::<T>::try_mutate(&renderer, |exists| -> DispatchResult {
				ensure!(!exists.is_none(), Error::<T>::AlreadyRegistered);
				exists.replace(Renderer {
					host: exists.as_ref().unwrap().host.clone(),
					portoffset: exists.as_ref().unwrap().portoffset,
					status: 1,
					gameid: exists.as_ref().unwrap().gameid.clone(),
					resolution: exists.as_ref().unwrap().resolution.clone(),
				});
				Ok(())
			})?;
			// TODO pay for the connection
			let starttime = T::TimeProvider::now().as_secs();
			Orders::<T>::try_mutate(&renderer, &player, |exists| -> DispatchResult {
				ensure!(exists.is_none(), Error::<T>::AlreadyRegistered);
				exists.replace(starttime);
				Ok(())
			})?;

			Self::deposit_event(Event::RendererConnected(renderer, player, starttime));
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn disconnect(origin: OriginFor<T>, renderer: T::AccountId) -> DispatchResult {
			let player = ensure_signed(origin)?;
			Renderers::<T>::try_mutate(&renderer, |exists| -> DispatchResult {
				ensure!(!exists.is_none(), Error::<T>::AlreadyRegistered);
				exists.replace(Renderer {
					host: exists.as_ref().unwrap().host.clone(),
					portoffset: exists.as_ref().unwrap().portoffset,
					status: 0,
					gameid: exists.as_ref().unwrap().gameid.clone(),
					resolution: exists.as_ref().unwrap().resolution.clone(),
				});
				Ok(())
			})?;
			// TODO pay for the connection
			let start_time: u64 = Orders::<T>::get(&renderer, &player).unwrap();
			let end_time: u64 = T::TimeProvider::now().as_secs();
			let duration: u64 = end_time - start_time;
			let price: u64 = 10000000000000;
			let amount: u64 = duration * price;
			let dest = renderer.clone();

			let x: Result<Balance,_> = amount.try_into();
			match x {
				Ok(v) => {
					if let Ok(transfer_amount) = v.try_into() {
						T::Currency::transfer(UC, &player, &dest, transfer_amount)?;
					};
				  
				},
				Err(_) => {
					// TODO
				},
			}

			Orders::<T>::remove(&renderer, &player);
			Self::deposit_event(Event::RendererDisconnected(renderer, player, end_time));
			Ok(())
		}

		/// Set link for ipfsjson file.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn setipfslink(origin: OriginFor<T>, link: Vec<u8>) -> DispatchResultWithPostInfo {
			//let _me = ensure_root(origin)?;
			Ipfslink::<T>::put(&link);
			Self::deposit_event(Event::UpdatedIpfsLink(link));
			Ok(().into())
		}
		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]

		pub fn setpov2uc(origin: OriginFor<T>, rate: u128) -> DispatchResultWithPostInfo {
			//let _me = ensure_root(origin)?;
			Pov2uc::<T>::put(&rate);
			Self::deposit_event(Event::UpdatedPov2uc(rate));
			Ok(().into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(10_000)]
		pub fn setuc2gametime(origin: OriginFor<T>, rate: u128) -> DispatchResultWithPostInfo {
			//let _me = ensure_root(origin)?;
			Uc2gametime::<T>::put(&rate);
			Self::deposit_event(Event::UpdatedUc2gametime(rate));
			Ok(().into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(10_000)]
		pub fn exchangepov2uc(
			origin: OriginFor<T>,
			ucid: CurrencyIdOf<T>,
			pov: AmountOf<T>,
		) -> DispatchResultWithPostInfo {
			let player = ensure_signed(origin)?;

			//ensure!(ucid != POV, Error::<T>::InvalidCurrencyId);

			let res: Result<u128, _> = pov.try_into();
			let rate = Pov2uc::<T>::get().unwrap_or(0);

			match res {
				Ok(amount_u128) => {
					let target_balance: Result<BalanceOf<T>, _> = (amount_u128 / rate).try_into();

					match target_balance {
						Ok(slash_amount) => {
							T::NativeCurrency::slash(&player, slash_amount);
							log::info!("amount slashed: {:?}", slash_amount);
							T::Currency::update_balance(UC, &player, pov)?;
						},
						Err(_) => {},
					}
				},

				Err(_) => {},
			}

			Ok(().into())
		}
	}
}
