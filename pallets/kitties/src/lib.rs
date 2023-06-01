#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

/*
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
*/

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use frame_support::traits::Randomness;
	use sp_io::hashing::blake2_128;

	pub type KittyId = u32;
	#[derive(
		Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, KittyId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_parents)]
	pub type KittyParents<T: Config> =
		StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		KittyCreated {
			who: T::AccountId,
			kitty_id: KittyId,
			kitty: Kitty,
		},
		KittyBreed {
			who: T::AccountId,
			kitty_id: KittyId,
			kitty: Kitty,
		},
		Transfer {
			who: T::AccountId,
			recipient: T::AccountId,
			kitty_id: KittyId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		InvalidKittyId,
		SameKittyId,
		NotOwner,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			let kitty_id = Self::get_next_id()?;
			// let kitty = Kitty(Default::default());
			let kitty = Kitty(Self::random_value(&who));

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);

			// Emit an event.
			Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_id_1: KittyId,
			kitty_id_2: KittyId,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);

			// ensure!(Kitties::<T>::contains_key(kitty_id_1), Error::<T>::InvalidKittyId);
			// ensure!(Kitties::<T>::contains_key(kitty_id_2), Error::<T>::InvalidKittyId);
			let kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

			let kitty_id = Self::get_next_id()?;

			// let kitty = Kitty(Self::random_value(&who));

			let selector = Self::random_value(&who);
			let mut data = [0u8; 16];

			for i in 0..kitty_1.0.len() {
				data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
			}

			let kitty = Kitty(data);

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2));

			// Emit an event.
			Self::deposit_event(Event::KittyBreed { who, kitty_id, kitty });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			kitty_id: KittyId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);

			let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
			ensure!(owner == who, Error::<T>::NotOwner);

			KittyOwner::<T>::insert(kitty_id, &recipient);

			Self::deposit_event(Event::Transfer { who, recipient, kitty_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_next_id() -> Result<KittyId, DispatchError> {
			NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
				let current_id = *next_id;
				*next_id = next_id.checked_add(1).ok_or(Error::<T>::InvalidKittyId)?;
				Ok(current_id)
			})
		}

		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);

			payload.using_encoded(blake2_128)
		}
	}
}
