#![cfg_attr(not(feature = "std"), no_std)]

mod functions;

use sp_std::prelude::*;
use sp_std::collections::btree_map::BTreeMap;
use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure};
use sp_runtime::RuntimeDebug;
use system::ensure_signed;
use pallet_timestamp;

pub const MIN_SPACE_OWNERS: u16 = 1;
pub const MAX_SPACE_OWNERS: u16 = u16::max_value();
pub const MAX_TX_NOTES_LEN: u16 = 1024;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct UpdatedAt<T: Trait> {
  block: T::BlockNumber,
  time: T::Moment,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct SpaceOwners<T: Trait> {
  pub updated_at: UpdatedAt<T>,
  pub space_id: SpaceId,
  pub owners: Vec<T::AccountId>,
  pub threshold: u16,

  pub executed_tx_count: u64,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct Transaction<T: Trait> {
  pub updated_at: UpdatedAt<T>,
  pub id: TransactionId,
  pub add_owners: Vec<T::AccountId>,
  pub remove_owners: Vec<T::AccountId>,
  pub new_threshold: Option<u16>,
  pub notes: Vec<u8>,
  pub confirmed_by: Vec<T::AccountId>,
}

type SpaceId = u64;
type TransactionId = u64;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + pallet_timestamp::Trait {
  /// The overarching event type.
  type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_error! {
  pub enum Error for Module<T: Trait> {
    /// Space was not found by id
    SpaceNotFound,
    /// Transaction was not found in a space owners
    TxNotFound,
    /// Space owners already exist on this space
    SpaceOwnersAlreadyExist,

    /// There can not be less owners than allowed
    NotEnoughOwners,
    /// There can not be more owners than allowed
    TooManyOwners,
    /// Account is not a space owner
    NotASpaceOwner,

    /// The threshold can not be less than 1
    ZeroThershold,
    /// The required confirmation count can not be greater than owners count"
    TooBigThreshold,
    /// Transaction notes are too long
    TxNotesOversize,
    /// No space owners will left in result of tx
    NoSpaceOwnersLeft,

    /// Account has already confirmed this transaction
    TxAlreadyConfirmed,
    /// There are not enough confirmations on a transaction
    NotEnoughConfirms,
    /// Transaction is already executed
    TxAlreadyExecuted,
    /// Transaction is not tied to an owed wallet
    TxNotTiedToSpace,
    /// Pending tx already exists
    PendingTxAlreadyExists,
    /// Pendint tx doesn't exist
    PendingTxDoesNotExist,

    /// Overflow in Wallet executed tx counter when executing tx
    OverflowExecutingTx,
    /// Underflow in Wallet pending tx counter when executing tx
    UnderflowExecutingTx,
  }
}

// This pallet's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as TemplateModule {
    MinSpaceOwners get(min_space_owners): u16 = MIN_SPACE_OWNERS;
		MaxSpaceOwners get(max_space_owners): u16 = MAX_SPACE_OWNERS;
		MaxTxNotesLen get(max_tx_notes_len): u16 = MAX_TX_NOTES_LEN;

		SpaceOwnersBySpaceById get(space_by_id): map SpaceId => Option<SpaceOwners<T>>;
		SpaceIdsOwnedByAccountId get(space_ids_owned_by_account_id): map T::AccountId => Vec<SpaceId>;

    NextTxId get(next_tx_id): TransactionId = 1;
		TxById get(tx_by_id): map TransactionId => Option<Transaction<T>>;
		PendingTxIdBySpaceId get(pending_tx_id_by_space_id): map SpaceId => Option<TransactionId>;
		ExecutedTxIdsBySpaceId get(executed_tx_ids_by_space_id): map SpaceId => Vec<TransactionId>;
  }
}

// The pallet's dispatchable functions.
decl_module! {
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    // Initializing events
    fn deposit_event() = default;

		pub fn create_space_owners(
      origin,
      space_id: SpaceId,
      owners: Vec<T::AccountId>,
      threshold: u16
    ) {
			let creator = ensure_signed(origin)?;

			ensure!(Self::space_by_id(space_id).is_none(), Error::<T>::SpaceOwnersAlreadyExist);

			let mut owners_map: BTreeMap<T::AccountId, bool> = BTreeMap::new();
			let mut wallet_owners: Vec<T::AccountId> = Vec::new();

			for owner in owners.iter() {
				if !owners_map.contains_key(&owner) {
					owners_map.insert(owner.clone(), true);
					wallet_owners.push(owner.clone());
				}
			}

			let owners_count = wallet_owners.len() as u16;
			ensure!(owners_count >= Self::min_space_owners(), Error::<T>::NotEnoughOwners);
			ensure!(owners_count <= Self::max_space_owners(), Error::<T>::NotEnoughOwners);

			ensure!(threshold <= owners_count, Error::<T>::TooBigThreshold);
			ensure!(threshold > 0, Error::<T>::ZeroThershold);

			let new_wallet = SpaceOwners {
				updated_at: Self::new_updated_at(),
				space_id: space_id.clone(),
				owners: wallet_owners.clone(),
				threshold,
				executed_tx_count: 0
			};

			<SpaceOwnersBySpaceById<T>>::insert(space_id, new_wallet);

			for owner in wallet_owners.iter() {
				<SpaceIdsOwnedByAccountId<T>>::mutate(owner.clone(), |ids| ids.push(space_id.clone()));
			}

			Self::deposit_event(RawEvent::SpaceOwnersCreated(creator, space_id));
		}

		pub fn propose_change(
      origin,
      space_id: SpaceId,
      add_owners: Vec<T::AccountId>,
      remove_owners: Vec<T::AccountId>,
      new_threshold: Option<u16>,
      notes: Vec<u8>
    ) {
			let sender = ensure_signed(origin)?;

			ensure!(notes.len() <= Self::max_tx_notes_len() as usize, Error::<T>::TxNotesOversize);

			let space = Self::space_by_id(space_id.clone()).ok_or(Error::<T>::SpaceNotFound)?;
			ensure!(Self::pending_tx_id_by_space_id(space_id).is_some(), Error::<T>::PendingTxAlreadyExists);

			let is_space_owner = space.owners.iter().any(|owner| *owner == sender.clone());
      ensure!(is_space_owner, Error::<T>::NotASpaceOwner);

      // ensure!(!Self::transform_new_owners_to_vec(space.owners.clone(), add_owners.clone(), remove_owners.clone()).is_empty(), Error::<T>::NoSpaceOwnersLeft);

			let tx_id = Self::next_tx_id();
			let ref mut new_tx = Transaction {
				updated_at: Self::new_updated_at(),
				id: tx_id,
				add_owners: add_owners,
        remove_owners: remove_owners,
        new_threshold: new_threshold,
				notes,
				confirmed_by: Vec::new()
			};

			new_tx.confirmed_by.push(sender.clone());

			<SpaceOwnersBySpaceById<T>>::insert(space_id.clone(), space);
			<TxById<T>>::insert(tx_id, new_tx);
			PendingTxIdBySpaceId::insert(space_id.clone(), tx_id);
			NextTxId::mutate(|n| { *n += 1; });

			Self::deposit_event(RawEvent::UpdateProposed(sender, space_id, tx_id));
		}

		pub fn confirm_change(origin, space_id: SpaceId, tx_id: TransactionId) {
			let sender = ensure_signed(origin)?;

			let space = Self::space_by_id(space_id.clone()).ok_or(Error::<T>::SpaceNotFound)?;

			let is_space_owner = space.owners.iter().any(|owner| *owner == sender.clone());
			ensure!(is_space_owner, Error::<T>::NotASpaceOwner);

			let mut tx = Self::tx_by_id(tx_id).ok_or(Error::<T>::TxNotFound)?;

			let pending_tx_id = Self::pending_tx_id_by_space_id(space_id.clone()).ok_or(Error::<T>::PendingTxDoesNotExist)?;
			ensure!(pending_tx_id == tx_id, Error::<T>::TxNotTiedToSpace);

			let sender_not_confirmed_yet = !tx.confirmed_by.iter().any(|account| *account == sender.clone());
			ensure!(sender_not_confirmed_yet, Error::<T>::TxAlreadyConfirmed);

			tx.confirmed_by.push(sender.clone());

			if tx.confirmed_by.len() == space.threshold as usize {
				Self::update_space_owners(sender.clone(), space.clone(), tx.clone())?;
			} else {
				<TxById<T>>::insert(tx_id, tx);
			}

			Self::deposit_event(RawEvent::UpdateConfirmed(sender, space_id, tx_id));
		}
	}
}

decl_event!(
  pub enum Event<T> where
    <T as system::Trait>::AccountId,
   {
    SpaceOwnersCreated(AccountId, SpaceId),
		UpdateProposed(AccountId, SpaceId, TransactionId),
		UpdateConfirmed(AccountId, SpaceId, TransactionId),
		SpaceOwnersUpdated(AccountId, SpaceId, TransactionId),
  }
);
