#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::*,
		sp_runtime::{traits::{Hash, Zero}, SaturatedConversion},
		traits::{Currency, ExistenceRequirement, Randomness},
		transactional,
	};
	use scale_info::prelude::string::String;

    use frame_system::pallet_prelude::*;

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum FileType {
		Normal,
		Privileged,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct File<T: Config> {
		pub file_link: [u8; 20],
		pub allow_download: bool,
		pub file_type: FileType,
		pub cost: u64,
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

		#[pallet::constant]
		type MaxFilesUploaded: Get<u32>;

		#[pallet::constant]
		type CostPerByte: Get<u64>;

		#[pallet::constant]
		type FileSizeLimit: Get<u64>;

		// #[pallet::constant]
		// type Accountant: Get<Self::AccountId>;

    }

    #[pallet::error]
    pub enum Error<T> {
		ExceedMaxFileUploaded,
		FileNotAllowedToDownload,
		FileNotFound,
		AlreadyDownloaded,
		ExceedMaxFileDownload,
		InvalidOperation,
		FileCountOverflow,
		FileDownloadCountOverflow,
		InvalidSigner,
		NotEnoughBalance,
		AccountantNotSet,
		FileDoesntExist
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
		Uploaded(T::AccountId, T::Hash),
		Downloaded(T::AccountId, T::Hash, BalanceOf<T>),
		Transfered(T::AccountId, T::AccountId, T::Hash),
    }

	#[pallet::storage]
	#[pallet::getter(fn accountant)]
	pub(super) type Accountant<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn all_files_count)]
	pub(super) type AllFilesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn all_downloads_count)]
	pub(super) type AllDownloadsCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_file_details)]
	pub(super) type Files<T: Config> = StorageMap<_, Twox64Concat, T::Hash, File<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_user_file_details)]
	// I am trying to map accountid to a vector with hash and file. So i can the file with hash as well
	pub(super) type FilesPerUser<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<T::Hash, T::MaxFilesUploaded>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_download_details)]
	// I am trying to map hash to a vector with account id and file. So i can tell which user downloaded which file
	pub(super) type FileDownloads<T: Config> = StorageMap<_, Twox64Concat, T::Hash, BoundedVec<T::AccountId,T::MaxFilesUploaded>, ValueQuery>;


    #[pallet::call]
    impl<T: Config> Pallet<T> {
		// Upload
		
		#[pallet::weight(100)]
		pub fn add_accountant(origin: OriginFor<T>) -> DispatchResult {

			let signer = ensure_signed(origin)?;

			let account = Self::accountant();

			match account {
				Some(acc) => {
					ensure!(acc == signer, <Error<T>>::InvalidSigner);
				},
				None => {
					<Accountant<T>>::put(signer);
				}
			}


			Ok(())
		}

		#[pallet::weight(100)]
		pub fn upload_file(origin: OriginFor<T>, file_link: [u8; 20], allow_download: bool, file_type: FileType, cost: u64, file_size: u64) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let file = File::<T> {
				file_link,
				allow_download,
				file_type,
				cost,
				file_size,
				owner: sender.clone()
			};

			let file_id = T::Hashing::hash_of(&file);
			let new_count = Self::all_files_count().checked_add(1).ok_or(<Error<T>>::FileCountOverflow)?;

			<FilesPerUser<T>>::try_mutate(&sender, |file_vec| file_vec.try_push(file_id))
			.map_err(|_| <Error<T>>::ExceedMaxFileUploaded)?;

			<Files<T>>::insert(file_id, file);
			<AllFilesCount<T>>::put(new_count);

			Self::deposit_event(Event::Uploaded(sender, file_id));

			Ok(())
		}

		// Download 
		#[pallet::weight(100)]
		pub fn download_file(origin: OriginFor<T>, file_id: T::Hash) -> DispatchResult {

			let signer = ensure_signed(origin)?;

			let file = Self::get_file_details(&file_id).ok_or(<Error<T>>::FileNotFound)?;

			ensure!(file.allow_download, <Error<T>>::FileNotAllowedToDownload);

			let downloads = <FileDownloads<T>>::get(&file_id);

			let download_index = downloads.binary_search(&signer);

			match download_index {
				Ok(_) => {
					 Err(<Error<T>>::AlreadyDownloaded)?
				},
				Err(_) => {
					
				},
			}

			match file.file_type {
				FileType::Normal => {
					if file.file_size > 250 {
						let size_difference = file.file_size.checked_sub(T::FileSizeLimit::get()).ok_or_else(|| <Error<T>>::InvalidOperation)?;
						let extra_cost = size_difference.checked_mul(T::CostPerByte::get()).ok_or_else(|| <Error<T>>::InvalidOperation)?;
						let total_cost = extra_cost.checked_add(file.cost).ok_or_else(|| <Error<T>>::InvalidOperation)?;
						let total_cost_in_balance = total_cost.saturated_into::<BalanceOf<T>>(); 

						// Transfer the amount to Dave
						ensure!(T::Currency::free_balance(&signer) >= total_cost_in_balance, <Error<T>>::NotEnoughBalance);
						let accountant = Self::accountant().ok_or_else(|| <Error<T>>::AccountantNotSet)?;
						T::Currency::transfer(&signer, &accountant, total_cost_in_balance, ExistenceRequirement::KeepAlive)?;
						Self::deposit_event(Event::Downloaded(signer.clone(), file_id, total_cost_in_balance));

					}
					else {
						let cost_in_balance = file.cost.saturated_into::<BalanceOf<T>>();
						
						// Transfer the amount to Dave
						ensure!(T::Currency::free_balance(&signer) >= cost_in_balance, <Error<T>>::NotEnoughBalance);
						let accountant = Self::accountant().ok_or_else(|| <Error<T>>::AccountantNotSet)?;
						T::Currency::transfer(&signer, &accountant, cost_in_balance, ExistenceRequirement::KeepAlive)?;
						Self::deposit_event(Event::Downloaded(signer.clone(), file_id, cost_in_balance));
					}
				},
				FileType::Privileged => {
					let cost_in_balance = file.cost.saturated_into::<BalanceOf<T>>();
					
					// Transfer the amount to Dave
					ensure!(T::Currency::free_balance(&signer) >= cost_in_balance, <Error<T>>::NotEnoughBalance);
					let accountant = Self::accountant().ok_or_else(|| <Error<T>>::AccountantNotSet)?;
					T::Currency::transfer(&signer, &accountant, cost_in_balance, ExistenceRequirement::KeepAlive)?;
					Self::deposit_event(Event::Downloaded(signer.clone(), file_id, cost_in_balance));
				}
			}

			let downloads_count = Self::all_downloads_count().checked_add(1).ok_or(<Error<T>>::FileDownloadCountOverflow)?;

			<FileDownloads<T>>::try_mutate(&file_id, |download_vec| download_vec.try_push(signer))
			.map_err(|_| <Error<T>>::ExceedMaxFileDownload)?;

			<AllDownloadsCount<T>>::put(downloads_count);

			Ok(())
		}

		// Transfer
		#[pallet::weight(100)]
		pub fn transfer_file(origin: OriginFor<T>, file_id: T::Hash, new_owner: T::AccountId) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			
			let file = &mut <Files<T>>::get(&file_id).ok_or(<Error<T>>::FileNotFound)?;

			ensure!(file.owner == owner, <Error<T>>::InvalidSigner);

			file.owner = new_owner.clone();

			<FilesPerUser<T>>::try_mutate(&owner, |owned| {
				if let Some(ind) = owned.iter().position(|&id| id == file_id) {
					owned.swap_remove(ind);
					return Ok(())
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::FileDoesntExist)?;

			<Files<T>>::insert(file_id, file);
			<FilesPerUser<T>>::try_mutate(&new_owner, |file_vec| file_vec.try_push(file_id))
			.map_err(|_| <Error<T>>::ExceedMaxFileUploaded)?;	


			Self::deposit_event(Event::Transfered(owner,new_owner, file_id));
			Ok(())
		}
    }


    impl<T: Config> Pallet<T> {
        
    }
}