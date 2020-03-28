#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_std::collections::{btree_set::BTreeSet};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure};
use system::ensure_signed;

// todo: writing tests
#[cfg(test)]
mod tests;

// The pallet's configuration trait.
pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

pub type ScopeId = u64;

// This pallet's storage items.
decl_storage! {
	pub trait Store for Module<T: Trait> as SocialBan {
		pub IsAccountBlockedInScope get(is_account_blocked_by_scope): map hasher(blake2_256) (ScopeId, T::AccountId) => bool = false;
		pub BlockedAccountsByScope get(blocked_accounts_by_scope): map hasher(blake2_256) ScopeId => BTreeSet<T::AccountId>;
	}
}

// The pallet's events
decl_event! {
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		AccountBlocked(AccountId, ScopeId, AccountId),
		AccountUnblocked(AccountId, ScopeId, AccountId),
	}
}

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Unpossible to block youré own account
		UnpossibleToBlockOwnAccount,
		/// Unpossible to unblock youré own account
		UnpossibleToUnblockOwnAccount,
		/// Unable to found Account with our ID to be blocked
		AccountNotFound,
		/// Unable to found Scope with our ID
		ScopeNotFound,
		/// Attempt to block account that already blocked by Scope
		AccountAlreadyBlockedInScope,
		/// Account not blocked by this Scope yet
		AccountNotBlockedByScope,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		pub fn block_account(origin, scope_id: ScopeId, subject_acc: T::AccountId) {

			let _owner = ensure_signed(origin)?;

			ensure!(
				_owner == subject_acc,
				Error::<T>::UnpossibleToBlockOwnAccount
			);

			let mut _already_blocked_accounts_by_scope = Self::blocked_accounts_by_scope(scope_id);
			let _subject = subject_acc.clone();

			ensure!(
				_already_blocked_accounts_by_scope.contains(&_subject),
				Error::<T>::AccountAlreadyBlockedInScope
			);

			_already_blocked_accounts_by_scope.insert(subject_acc.clone());
			<BlockedAccountsByScope<T>>::insert(scope_id, _already_blocked_accounts_by_scope);
			<IsAccountBlockedInScope<T>>::insert((scope_id, subject_acc.clone()), true);

			Self::deposit_event(RawEvent::AccountBlocked(_owner, scope_id, subject_acc));
		}

	    pub fn unblock_account(origin, scope_id: ScopeId, subject_acc: T::AccountId) {

			let _owner = ensure_signed(origin)?;

			ensure!(
				_owner == subject_acc,
				Error::<T>::UnpossibleToUnblockOwnAccount
			);

			let mut _already_blocked_accounts_by_scope = Self::blocked_accounts_by_scope(scope_id);
			let _subject = subject_acc.clone();

			ensure!(
				!_already_blocked_accounts_by_scope.contains(&_subject),
				Error::<T>::AccountNotBlockedByScope
			);

			let _subject = subject_acc.clone();

			_already_blocked_accounts_by_scope.remove(&_subject);
			<BlockedAccountsByScope<T>>::insert(scope_id, _already_blocked_accounts_by_scope);
			<IsAccountBlockedInScope<T>>::remove((scope_id, subject_acc.clone()));

			Self::deposit_event(RawEvent::AccountUnblocked(_owner, scope_id, subject_acc));
		}
	}
}
