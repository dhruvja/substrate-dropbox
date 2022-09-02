#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::*,
		sp_runtime::traits::{Hash, Zero},
		traits::{Currency, ExistenceRequirement, Randomness},
		transactional,
	};

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

	#[pallet::storage]
	#[pallet::getter(fn get_user_file_details)]
	// I am trying to map accountid to a vector with hash and file. So i can the file with hash as well
	pub(super) type FilesPerUser<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<T::Hash, File<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_download_details)]
	// I am trying to map hash to a vector with account id and file. So i can tell which user downloaded which file
	pub(super) type FileDownloads<T: Config> = StorageMap<_, Twox64Concat, T::Hash, BoundedVec<T::AccountId,<File<T>>, ValueQuery>;


    #[pallet::call]
    impl<T: Config> Pallet<T> {
		// Upload

		// Download

		// Transfer
    }


    impl<T: Config> Pallet<T> {
        
    }
}