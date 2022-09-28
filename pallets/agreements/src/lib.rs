#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::{
		traits::Currency,
		sp_runtime::traits::Hash,
		traits::tokens::WithdrawReasons,
		traits::tokens::ExistenceRequirement
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	//*******************
	// Agreements Enums *
	//*******************
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	enum AgreementStatus {
		NotSigned,
		Canceled,
		Signed,
		InReview,
		Complete,
	}
	
	// *******************
	// Agreement structs *
	// *******************
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Agreement<T: Config> {
		contractor: AccountOf<T>,
		hired: AccountOf<T>,
		value: BalanceOf<T>,
		info: T::Hash,
		status: AgreementStatus
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Because this pallet lock and transfer currency, it depends on the runtime definition of a currency
		type Currency: Currency<Self::AccountId>;

		type MaxAgreementsPerAccount: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//********************
	// Agreement Storage *
	//********************
	#[pallet::storage]
	#[pallet::getter(fn agreements)]
	pub type Agreements<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Agreement<T>>;


	#[pallet::storage]
	#[pallet::getter(fn user_agreements)]
	pub type UserAgreements<T: Config> = StorageMap<_, Twox64Concat, AccountOf<T>, BoundedVec<T::Hash, T::MaxAgreementsPerAccount>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		// parameters. [who, agreementId]
		AgreementCreated(T::AccountId, T::Hash),
		// parameters. [agreement_id]
		AgreementCanceled(T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		NotEnoughBalance,
		ZeroValueNotAllowed,
		EqualsAccountsNotAllowed,
		NotFound,
		NotAllowed,
		AgreementAlreadySigned,
		AlreadyExist,
		AgreementNotSigned,
		AgreementNotInReview
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 3))]
		#[frame_support::transactional]
		pub fn create(origin: OriginFor<T>, hired: AccountOf<T>, value: BalanceOf<T>, info: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(sender != hired, <Error<T>>::EqualsAccountsNotAllowed);
			ensure!(value > Self::u32_to_balance(0), <Error<T>>::ZeroValueNotAllowed);
			// check for balance
			ensure!(T::Currency::free_balance(&sender) > value, <Error<T>>::NotEnoughBalance);

			// Remove value from sender
			T::Currency::withdraw(&sender, value, WithdrawReasons::RESERVE, ExistenceRequirement::KeepAlive)?;

			let agreement = Agreement {
				contractor: sender.clone(),
				hired: hired.clone(),
				value: value,
				info: info,
				status: AgreementStatus::NotSigned
			};

			let agid = T::Hashing::hash_of(&(&agreement, frame_system::Pallet::<T>::block_number()));
			ensure!(Self::agreements(&agid).is_none(), <Error<T>>::AlreadyExist);

			<Agreements<T>>::insert(&agid, agreement);

			// create reference to agreement in contractor
			<UserAgreements<T>>::try_mutate(&sender, |contractor_ags| {
				contractor_ags.try_push(agid.clone())
			}).map_err(|_| <Error<T>>::StorageOverflow)?;

			// create reference to agreement in hired
			<UserAgreements<T>>::try_mutate(&hired, |hired_ags| {
				hired_ags.try_push(agid.clone())
			}).map_err(|_| <Error<T>>::StorageOverflow)?;

			Self::deposit_event(Event::AgreementCreated(sender, agid));
			Ok(())
		}
		
		#[pallet::weight((1_000 + T::DbWeight::get().reads_writes(1, 1), DispatchClass::Normal, Pays::No))]
		#[frame_support::transactional]
		pub fn cancel(origin: OriginFor<T>, agg_id: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let check = Self::agreements(&agg_id);
			ensure!(check.is_some(), <Error<T>>::NotFound);
			let mut agreement = check.unwrap();
			ensure!(sender == agreement.contractor, <Error<T>>::NotAllowed);
			ensure!(agreement.status == AgreementStatus::NotSigned, <Error<T>>::AgreementAlreadySigned);

			T::Currency::deposit_into_existing(&sender, agreement.value)?;
			agreement.status = AgreementStatus::Canceled;
			<Agreements<T>>::insert(&agg_id, agreement);

			Self::deposit_event(Event::AgreementCanceled(agg_id));
			Ok(())
		}

		#[pallet::weight((1_000 + T::DbWeight::get().reads_writes(1, 1), DispatchClass::Normal, Pays::No))]
		pub fn unsign(origin: OriginFor<T>, agg_id: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let check = Self::agreements(&agg_id);
			ensure!(check.is_some(), <Error<T>>::NotFound);
			let mut agreement = check.unwrap();
			ensure!(sender == agreement.hired, <Error<T>>::NotAllowed);
			ensure!(agreement.status == AgreementStatus::Signed, <Error<T>>::AgreementNotSigned);

			agreement.status = AgreementStatus::NotSigned;
			<Agreements<T>>::insert(&agg_id, agreement);
			Ok(())
		}

		#[pallet::weight((1_000 + T::DbWeight::get().reads_writes(1, 1), DispatchClass::Normal, Pays::No))]
		pub fn sign(origin: OriginFor<T>, agg_id: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let check = Self::agreements(&agg_id);
			ensure!(check.is_some(), <Error<T>>::NotFound);
			let mut agreement = check.unwrap();
			ensure!(sender == agreement.hired, <Error<T>>::NotAllowed);
			ensure!(agreement.status == AgreementStatus::NotSigned, <Error<T>>::AgreementAlreadySigned);

			agreement.status = AgreementStatus::Signed;
			<Agreements<T>>::insert(&agg_id, agreement);
			Ok(())
		}

		#[pallet::weight((1_000 + T::DbWeight::get().reads_writes(1, 1), DispatchClass::Normal, Pays::No))]
		pub fn set_review(origin: OriginFor<T>, agg_id: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let check = Self::agreements(&agg_id);
			ensure!(check.is_some(), <Error<T>>::NotFound);
			let mut agreement = check.unwrap();
			ensure!(sender == agreement.hired, <Error<T>>::NotAllowed);
			ensure!(agreement.status == AgreementStatus::Signed, <Error<T>>::AgreementNotSigned);

			agreement.status = AgreementStatus::InReview;
			<Agreements<T>>::insert(&agg_id, agreement);
			Ok(())
		}

		#[pallet::weight((1_000 + T::DbWeight::get().reads_writes(1, 1), DispatchClass::Normal, Pays::No))]
		#[frame_support::transactional]
		pub fn accept(origin: OriginFor<T>, agg_id: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let check = Self::agreements(&agg_id);
			ensure!(check.is_some(), <Error<T>>::NotFound);
			let mut agreement = check.unwrap();
			ensure!(sender == agreement.contractor, <Error<T>>::NotAllowed);
			ensure!(agreement.status == AgreementStatus::InReview, <Error<T>>::AgreementNotInReview);

			// charge fees here
			T::Currency::deposit_creating(&agreement.contractor, agreement.value);

			agreement.status = AgreementStatus::Complete;
			<Agreements<T>>::insert(&agg_id, agreement);
			Ok(())
		}

	}

	impl<T: Config> Pallet<T> {
		pub fn u32_to_balance(input: u32) -> BalanceOf<T> {
			input.into()
		}
	}

}
