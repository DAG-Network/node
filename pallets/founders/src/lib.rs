#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::{
		sp_runtime::traits::Hash, traits::tokens::ExistenceRequirement,
		traits::tokens::WithdrawReasons, traits::Currency, sp_runtime::SaturatedConversion
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	// *******************
	// Founders structs *
	// *******************
	// #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	// #[scale_info(skip_type_params(T))]
	// pub struct BucketFee<T: Config> {
	// 	number: T::BlockNumber,
	//     value: BalanceOf<T>
	// }

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Because this pallet lock and transfer currency, it depends on the runtime definition of a currency
		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//********************
	// Founders Storage *
	//********************
	#[pallet::storage]
	#[pallet::getter(fn bucket)]
	pub type Bucket<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tickets)]
	pub type Tickets<T: Config> = StorageMap<_, Blake2_128Concat, AccountOf<T>, u128, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub first_founder: Option<AccountOf<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { first_founder: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			match &self.first_founder {
				Some(acc) => <Tickets<T>>::insert(acc, 100),
				None => {}
			}
		}
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		// parameters. [Total Value Distributed, BlockNumber]
		DistributionPerformed(u128, T::BlockNumber),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		NotEnoughBalance,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T: Config> Pallet<T> {
		pub fn u32_to_balance(input: u32) -> BalanceOf<T> {
			input.into()
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: T::BlockNumber) {
			// get all value in bucket
			let bucket = match <Bucket<T>>::try_get() {
				Ok(val) => val,
				Err(_) => 0,
			};
			if bucket == 0 {
				return;
			}
			// distribuite than
			for (founder, value) in <Tickets<T>>::iter() {
				let to_receive: u128 = (value * bucket) / 100;
				T::Currency::deposit_creating(&founder, Self::u128_to_balance(to_receive));
			}
			Self::deposit_event(Event::DistributionPerformed(bucket, n));
			// clean bucket
			<Bucket<T>>::set(0);
		}
	}

	impl<T: Config> Pallet<T> {
		// *****************
		// Founder Helpers *
		// *****************

		pub fn u128_to_balance(input: u128) -> BalanceOf<T> {
			input.saturated_into()
		}

		pub fn add_to_bucket(value: u128) {
			let mut bucket_value = match <Bucket<T>>::try_get() {
				Ok(val) => val,
				Err(_) => 0,
			};

			bucket_value = bucket_value.saturating_add(value);

			<Bucket<T>>::set(bucket_value);
		}
	}
}
