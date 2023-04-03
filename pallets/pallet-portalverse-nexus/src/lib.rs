#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Codec, Decode, Encode};
    use frame_support::{
        pallet_prelude::*,
        traits::{
            tokens::{ fungibles, DepositConsequence, WithdrawConsequence},
            ReservableCurrency,Currency, UnixTime,ExistenceRequirement::KeepAlive,
        },
        transactional,
    };
	use pallet_dex::traits::Exchange;
	use primitives::{AccountId, Balance, CurrencyId, TradingPair};

	use orml_traits::{
		arithmetic::{Signed, SimpleArithmetic},
		currency::TransferAll,
		BalanceStatus, BasicCurrency, BasicCurrencyExtended, BasicLockableCurrency,
		BasicReservableCurrency, LockIdentifier, MultiCurrency, MultiCurrencyExtended,
		MultiLockableCurrency, MultiReservableCurrency, NamedBasicReservableCurrency,
		NamedMultiReservableCurrency,
	};
	
    // use orml_traits::{ currency::TransferAll, BasicCurrency, BasicCurrencyExtended, BasicLockableCurrency,
	// 	LockIdentifier, MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
	// 	NamedBasicReservableCurrency, NamedMultiReservableCurrency,};
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_runtime::{traits::{
        AtLeast32BitUnsigned, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One,
        StaticLookup, Zero,
    }, SaturatedConversion};
    use sp_runtime::DispatchResult;
    use sp_std::vec::Vec;
    use ethereum_types::{H256, H64};
	use hex_literal::*;

	type BalanceOf<T> =
		<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<
		<T as frame_system::Config>::AccountId,
	>>::CurrencyId;
	pub(crate) type AmountOf<T> = <<T as Config>::Currency as MultiCurrencyExtended<
		<T as frame_system::Config>::AccountId,
	>>::Amount;

	// pub type BalanceOf<T> = <<T as Config>::MyCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    // type CurrencyId = u32;
    // type Balance = u128;

	/// To represent a rendering instance
	#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, TypeInfo, Debug)]
	pub struct Renderer{
		// ipv6 or ipv4(u32)
		pub host: Vec<u8>,
		pub portoffset: u16,
		pub status: u8,
		pub gameid: Vec<u8>,
		pub resolution: Vec<u8>
		// TODO POV staking
	}

    #[pallet::config]
    pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		//type MyCurrency: Currency<Self::AccountId>;
		type TimeProvider: UnixTime;
		// type MultiCurrency: TransferAll<Self::AccountId>
		// 	+ MultiCurrencyExtended<Self::AccountId>
		// 	+ MultiLockableCurrency<Self::AccountId>
		// 	+ MultiReservableCurrency<Self::AccountId>
		// 	+ NamedMultiReservableCurrency<Self::AccountId>;
		type Currency: MultiCurrency<Self::AccountId>
		+ MultiCurrencyExtended<Self::AccountId, CurrencyId = CurrencyId, Balance = Balance>;

		type NativeCurrency: ReservableCurrency<Self::AccountId, Balance = BalanceOf<Self>>;

		type Exchange: Exchange<Self::AccountId, Balance, CurrencyId>;
		#[pallet::constant]
		type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;
		#[pallet::constant]
		type NativeCurrencyBalance: Get<BalanceOf<Self>>;
	}

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		AlreadyRegistered,
    }

	/// Storage of all renderers.
    /// The storage does not guarante that the host:port of a renderer is unique,
    /// some extra constraints must be applied
    #[pallet::storage]
	#[pallet::getter(fn renderers)]
    pub type Renderers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Renderer,
        OptionQuery,
    >;

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
	pub type Ipfslink<T: Config> = StorageValue<
		_,
		Vec<u8>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn pov2uc)]
	pub type Pov2uc<T: Config> = StorageValue<
		_,
		u8,
	>;

	#[pallet::storage]
	#[pallet::getter(fn uc2gametime)]
	pub type Uc2gametime<T: Config> = StorageValue<
		_,
		u8,
	>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A renderer registers on chain
		RendererRegistered(T::AccountId),
		RendererDeregistered(T::AccountId),
		RendererConnected(T::AccountId,T::AccountId,u64),
		RendererDisconnected(T::AccountId,T::AccountId,u64),
		UpdatedIpfsLink(Vec<u8>),
		UpdatedPov2uc(u8),
		UpdatedUc2gametime(u8),
		// Exchangepov2uc(T::AccountId,u64),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        ///
        /// By implementing `fn offchain_worker` you declare a new offchain worker.
        /// This function will be called when the node is fully synced and a new best block is
        /// succesfuly imported.
        /// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
        /// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
        /// so the code should be able to handle that.
        /// You can use `Local Storage` API to coordinate runs of the worker.
        fn offchain_worker(block_number: T::BlockNumber) {
            if !sp_io::offchain::is_validator() {
                return;
            }
            // TODO use VRF to generate challenge group, then submit a transaction
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
		/// Register a renderer. A basic challenge should be initiated immediately by the offchain workers.
		#[pallet::weight(10_000)]
		pub fn register(origin: OriginFor<T>, host: Vec<u8>, portoffset: u16, games: Vec<u8>, resolution: Vec<u8>) -> DispatchResultWithPostInfo {
			let renderer = ensure_signed(origin)?;
			Renderers::<T>::try_mutate(&renderer, |exists| -> DispatchResult {
				ensure!(exists.is_none(), Error::<T>::AlreadyRegistered);
				exists.replace(Renderer {
					host,
					portoffset,
					status: 0,
					gameid: games,
					resolution:resolution
				});
				Ok(())
			})?;
			Self::deposit_event(Event::RendererRegistered(renderer));
			Ok(().into())
		}


		/// deregister a renderer. 
		#[pallet::weight(10_000)]
		pub fn deregister(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let renderer = ensure_signed(origin)?;
			Renderers::<T>::remove(&renderer);
			Self::deposit_event(Event::RendererDeregistered(renderer));
			Ok(().into())
		}


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
					host:exists.as_ref().unwrap().host.clone(),
					portoffset:exists.as_ref().unwrap().portoffset,
					status: 1,
					gameid: exists.as_ref().unwrap().gameid.clone(),
					resolution:exists.as_ref().unwrap().resolution.clone()
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

			Self::deposit_event(Event::RendererConnected(renderer,player,starttime));
			Ok(())
		}


		#[pallet::weight(10_000)]
		pub fn disconnect(origin: OriginFor<T>, renderer: T::AccountId) -> DispatchResult {
			let player = ensure_signed(origin)?;
			Renderers::<T>::try_mutate(&renderer, |exists| -> DispatchResult {
				ensure!(!exists.is_none(), Error::<T>::AlreadyRegistered);
				exists.replace(Renderer {
					host:exists.as_ref().unwrap().host.clone(),
					portoffset:exists.as_ref().unwrap().portoffset,
					status: 0,
					gameid: exists.as_ref().unwrap().gameid.clone(),
					resolution:exists.as_ref().unwrap().resolution.clone()
				});
				Ok(())
			})?;
			// TODO pay for the connection	
			let starttime: u64 = Orders::<T>::get(&renderer, &player).unwrap();
			let endtime: u64 = T::TimeProvider::now().as_secs();
			let duration: u64 = endtime - starttime;
			let price: u64 = 10000000000000;
			let amount: u64 = duration * price;
			let dest = renderer.clone();
			//T::MyCurrency::transfer(&player, &dest, amount.saturated_into::<BalanceOf<T>>(), KeepAlive)?;
			Orders::<T>::remove(&renderer, &player);
			Self::deposit_event(Event::RendererDisconnected(renderer,player,endtime));
			Ok(())
		}

		/// Set link for ipfsjson file.
		#[pallet::weight(10_000)]
		pub fn setipfslink(origin: OriginFor<T>, link:Vec<u8>) -> DispatchResultWithPostInfo {
			//let _me = ensure_root(origin)?;
			Ipfslink::<T>::put(&link);
			Self::deposit_event(Event::UpdatedIpfsLink(link));
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn setpov2uc(origin: OriginFor<T>, rate:u8) -> DispatchResultWithPostInfo {
			//let _me = ensure_root(origin)?;
			Pov2uc::<T>::put(&rate);
			Self::deposit_event(Event::UpdatedPov2uc(rate));
			Ok(().into())
		}
		
		#[pallet::weight(10_000)]
		pub fn setuc2gametime(origin: OriginFor<T>, rate:u8) -> DispatchResultWithPostInfo {
			//let _me = ensure_root(origin)?;
			Uc2gametime::<T>::put(&rate);
			Self::deposit_event(Event::UpdatedUc2gametime(rate));
			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn exchangepov2uc(origin: OriginFor<T>, pov:u64) -> DispatchResultWithPostInfo {
			let player = ensure_signed(origin)?;
			//T::MyCurrency::burn_from(player,pov);
			//let bytes = s.as_bytes();
			//let dest = T::AccountId::try_from("5DJk1gegyQJk6BNs7LceZ1akt5e9fpm4gUYGzcfpKaLG9Mmb").unwrap();
			// let a = get_account_id_from_seed::<sr25519::Public>("Alice");
			//T::MyCurrency::transfer(&player, &dest, pov.saturated_into::<BalanceOf<T>>(), KeepAlive)?;
			//T::CountryCurrency::deposit(currency_id, &who, total_supply)?;
			//Self::deposit_event(Event::Exchangepov2uc(player,pov));
			Ok(().into())
		}


    }

	// impl<T: Config> RendererManager<T::AccountId, Renderer> for Pallet<T> {

	// 	fn get_renderer(id: T::AccountId) -> Renderer{
	// 		Renderers::<T>::get(&id).unwrap()
	// 	}
	// }


}
