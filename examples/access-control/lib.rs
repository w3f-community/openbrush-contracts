#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
#[ink::contract]
pub mod my_access_control {
    use erc721::{
        traits::{ IErc721, Erc721Error, Id, IErc721Mint },
        impls::{ Erc721Storage, Erc721Internal, Erc721, Erc721Mint }
    };
    use access_control::{
        traits::{ IAccessControl, AccessControlError, RoleType },
        impls::{ AccessControlStorage, AccessControl, RoleData }
    };
    use utils::{
        traits::{ InkStorage },
        iml_getters,
    };
    use ink_storage::{
        collections::{
            HashMap as StorageHashMap,
        },
    };
    use ink_lang::{ Env, EmitEvent };
    use ink_prelude::{ vec::Vec, string::ToString };

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: Id,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        id: Id,
    }

    /// Event emitted when an operator is enabled or disabled for an owner.
    /// The operator can manage all NFTs of the owner.
    #[ink(event)]
    pub struct ApprovalForAll {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        operator: AccountId,
        approved: bool,
    }

    #[derive(Default)]
    #[ink(storage)]
    pub struct Erc721Struct {
        // Fields of Erc721Storage
        /// Mapping from token to owner.
        token_owner: StorageHashMap<Id, AccountId>,
        /// Mapping from token to approvals users.
        token_approvals: StorageHashMap<Id, AccountId>,
        /// Mapping from owner to number of owned token.
        owned_tokens_count: StorageHashMap<AccountId, u32>,
        /// Mapping from owner to operator approvals.
        operator_approvals: StorageHashMap<(AccountId, AccountId), bool>,

        // Fields of AccessControlStorage
        /// Mapping from role type to role data(the list of members and admin role).
        roles: StorageHashMap<RoleType, RoleData>,
    }

    // ::ink_lang_ir::Selector::new("MINTER".as_ref()).as_bytes()
    const MINTER: RoleType = 0xfd9ab216;

    impl Erc721Struct {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::_empty()
        }

        #[inline]
        fn only_minter(&self) -> Result<(), Erc721Error> {
            if !self.has_role(MINTER, self.env().caller()) {
                return Err(Erc721Error::Unknown("Caller is not minter".to_string()));
            }

            Ok(())
        }
    }
    // We override _empty method and use it in the constructor.
    // _empty is a base constructor which can create an empty struct.
    // Some implementations require initialization of some variables, you can do it in _empty function.
    // In this case, all your constructors which are using _empty function will be initialized properly.
    impl InkStorage for Erc721Struct {
        fn _empty() -> Self {
            let mut instance = Self::default();
            let caller = Self::env().caller();
            instance._init_with_admin(caller);
            // We grant minter role to caller in constructor, so he can mint/burn tokens
            instance._grant_role(MINTER, caller).expect("Can't provide Minter to caller");
            instance
        }
    }
    impl Erc721Storage for Erc721Struct {
        iml_getters!(token_owner, _token_owner, _token_owner_mut, StorageHashMap<Id, AccountId>);
        iml_getters!(token_approvals, _token_approvals, _token_approvals_mut, StorageHashMap<Id, AccountId>);
        iml_getters!(owned_tokens_count, _owned_tokens_count, _owned_tokens_count_mut, StorageHashMap<AccountId, u32>);
        iml_getters!(operator_approvals, _operator_approvals, _operator_approvals_mut, StorageHashMap<(AccountId, AccountId), bool>);
    }
    impl AccessControlStorage for Erc721Struct {
        iml_getters!(roles, _roles, _roles_mut, StorageHashMap<RoleType, RoleData>);
    }

    // Erc721 has additional trait Erc721Internal which contains internal methods which is used for implementation of Erc721 trait.
    // You also can override them. Methods which emit events is not defined in Erc721Internal, so you MUST define them here by self.
    impl Erc721Internal for Erc721Struct {
        fn _emit_transfer_event(&self, _from: AccountId, _to: AccountId, _id: Id) {
            self.env().emit_event(Transfer {
                from: Some(_from),
                to: Some(_to),
                id: _id,
            });
        }

        fn _emit_approval_event(&self, _from: AccountId, _to: AccountId, _id: Id) {
            self.env().emit_event(Approval {
                from: _from,
                to: _to,
                id: _id,
            });
        }

        fn _emit_approval_for_all_event(&self, _owner: AccountId, _operator: AccountId, _approved: bool) {
            self.env().emit_event(ApprovalForAll {
                owner: _owner,
                operator: _operator,
                approved: _approved,
            });
        }
    }
    impl Erc721 for Erc721Struct {}
    impl AccessControl for Erc721Struct {}

    impl IErc721 for Erc721Struct {
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u32 {
            self._balance_of(owner)
        }

        #[ink(message)]
        fn owner_of(&self, id: Id) -> Option<AccountId> {
            self._owner_of(&id)
        }

        #[ink(message)]
        fn get_approved(&self, id: Id) -> Option<AccountId> {
            self._get_approved(id)
        }

        #[ink(message)]
        fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            self._is_approved_for_all(owner, operator)
        }

        #[ink(message)]
        fn set_approval_for_all(&mut self, to: AccountId, approved: bool) -> Result<(), Erc721Error> {
            panic_on_error(self._set_approval_for_all(to, approved))
        }

        #[ink(message)]
        fn approve(&mut self, to: AccountId, id: Id) -> Result<(), Erc721Error> {
            panic_on_error(self._approve(to, id))
        }

        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            id: Id,
        ) -> Result<(), Erc721Error> {
            panic_on_error(self._transfer_from(from, to, id))
        }

        #[ink(message)]
        fn safe_transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            id: Id,
            data: Vec<u8>,
        ) -> Result<(), Erc721Error> {
            panic_on_error(self._safe_transfer_from(from, to, id, data))
        }
    }

    impl IAccessControl for Erc721Struct {
        #[ink(message)]
        fn has_role(&self, role: RoleType, address: AccountId) -> bool {
            self._has_role(&role, &address)
        }

        #[ink(message)]
        fn get_role_admin(&self, role: RoleType) -> RoleType {
            self._get_role_admin(&role)
        }

        #[ink(message)]
        fn grant_role(&mut self, role: RoleType, address: AccountId) -> Result<(), AccessControlError> {
            panic_on_error(self._grant_role(role, address))
        }

        #[ink(message)]
        fn revoke_role(&mut self, role: RoleType, address: AccountId) -> Result<(), AccessControlError> {
            panic_on_error(self._revoke_role(role, address))
        }

        #[ink(message)]
        fn renounce_role(&mut self, role: RoleType, address: AccountId) -> Result<(), AccessControlError> {
            panic_on_error(self._renounce_role(role, address))
        }
    }

    impl Erc721Mint for Erc721Struct {
        fn _mint(&mut self, id: Id) -> Result<(), Erc721Error> {
            self.only_minter()?;
            Erc721Mint::_mint(self, id)
        }

        fn _burn(&mut self, id: Id) -> Result<(), Erc721Error> {
            self.only_minter()?;
            Erc721Mint::_burn(self, id)
        }
    }
    impl IErc721Mint for Erc721Struct {
        #[ink(message)]
        fn mint(&mut self, id: Id) -> Result<(), Erc721Error> {
            panic_on_error(self._mint(id))
        }

        #[ink(message)]
        fn burn(&mut self, id: Id) -> Result<(), Erc721Error> {
            panic_on_error(self._burn(id))
        }
    }

    // TODO: ink! doesn't revert transactions if you returned error from the public method,
    // so let's do it manually for now. https://github.com/paritytech/ink/issues/641
    fn panic_on_error<T, E>(result: Result<T, E>) -> Result<T, E> {
        match result {
            Err(_) => panic!("Got error during execution"),
            Ok(ok) => Ok(ok),
        }
    }
}
