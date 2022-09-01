#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{sp_runtime::traits::{Hash, Zero},
                        dispatch::{DispatchResultWithPostInfo, DispatchResult}, 
                        traits::{Currency, ExistenceRequirement, Randomness},
                        pallet_prelude::*};
    use frame_system::pallet_prelude::*;

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum FileType {
		Normal,
		Privilege,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct File<T: Config> {
		pub file_link: [u8; 50],
		pub allow_download: bool,
		pub file_type: FileType,
		pub cost: Option<BalanceOf<T>>,
		pub file_size: u64,
		pub owner: AccountOf<T>,
	}

    #[pallet::pallet]
    #[pallet::generate_store(trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: pallet_balances::Config + frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

    }

    #[pallet::error]
    pub enum Error<T> {

    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // TODO Part III
    }

	#[pallet::storage]
	#[pallet::getter(fn all_files_count)]
	pub(super) type AllFilesCount<T: Config> = StorageValue<_, u64, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn get_file_details)]
	pub(super) type Files<T: Config> = StorageMap<_, Twox64Concat, T::Hash, File<T>>;



    #[pallet::call]
    impl<T: Config> Pallet<T> {
		// Upload

		// Download

		// Transfer
    }


    impl<T: Config> Pallet<T> {
        
    }
}