/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
}

//const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";
const DATA_IMAGE_SVG_NEAR_ICON_BASE64: &str = "<img src='data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/Pgo8IURPQ1RZUEUgc3ZnIFBVQkxJQyAiLS8vVzNDLy9EVEQgU1ZHIDIwMDEwOTA0Ly9FTiIKICJodHRwOi8vd3d3LnczLm9yZy9UUi8yMDAxL1JFQy1TVkctMjAwMTA5MDQvRFREL3N2ZzEwLmR0ZCI+CjxzdmcgdmVyc2lvbj0iMS4wIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciCiB3aWR0aD0iMzAwLjAwMDAwMHB0IiBoZWlnaHQ9IjUwOS4wMDAwMDBwdCIgdmlld0JveD0iMCAwIDMwMC4wMDAwMDAgNTA5LjAwMDAwMCIKIHByZXNlcnZlQXNwZWN0UmF0aW89InhNaWRZTWlkIG1lZXQiPgo8bWV0YWRhdGE+CkNyZWF0ZWQgYnkgcG90cmFjZSAxLjEwLCB3cml0dGVuIGJ5IFBldGVyIFNlbGluZ2VyIDIwMDEtMjAxMQo8L21ldGFkYXRhPgo8ZyB0cmFuc2Zvcm09InRyYW5zbGF0ZSgwLjAwMDAwMCw1MDkuMDAwMDAwKSBzY2FsZSgwLjEwMDAwMCwtMC4xMDAwMDApIgpmaWxsPSIjMDAwMDAwIiBzdHJva2U9Im5vbmUiPgo8cGF0aCBkPSJNMTQ5MCA0NjQ0IGMtNjUyIC03OCAtMTE4NiAtMTQzIC0xMTg3IC0xNDQgLTIgMCAtMyAtOTQ1IC0zIC0yMTAwCmwwIC0yMTAxIDMzIDYgYzE3IDMgNTQzIDY2IDExNjcgMTQwIDYyNCA3NCAxMTUwIDEzNyAxMTY4IDE0MCBsMzIgNiAwIDIwOTkKYzAgMTY3NCAtMyAyMTAwIC0xMiAyMDk5IC03IC0xIC01NDYgLTY2IC0xMTk4IC0xNDV6IG0xMTgwIC0xOTY0IGwwIC0yMDY3Ci0xMDggLTEyIGMtNTkgLTcgLTExOCAtMTQgLTEzMiAtMTcgLTI1IC00IC0xMDcgLTE0IC00NTUgLTU1IC05OSAtMTIgLTE5MQotMjMgLTIwNSAtMjUgLTE0IC0xIC03NCAtOCAtMTM0IC0xNCAtNjAgLTYgLTExMiAtMTIgLTExNSAtMTQgLTMgLTIgLTM3IC03Ci03NSAtMTEgLTM4IC0zIC03MiAtOCAtNzUgLTkgLTMgLTIgLTQ2IC03IC05NSAtMTAgLTQ5IC00IC05MyAtOSAtOTcgLTExIC00Ci0zIC0zNSAtNyAtNzAgLTEwIC0zNSAtMiAtNzQgLTcgLTg2IC05IC0xMiAtMyAtNDggLTcgLTgwIC0xMCAtMzIgLTQgLTc0IC04Ci05MyAtMTEgLTE5IC0yIC01NyAtNyAtODUgLTkgLTExMSAtMTEgLTE2MiAtMTggLTIwMCAtMjYgLTIyIC01IC04NCAtMTIgLTEzNwotMTUgbC05OCAtNyAwIDIwNzAgMCAyMDcwIDYzIDcgYzM0IDQgNjkgOCA3NyAxMCA4IDEgNDcgNiA4NSA5IDM5IDQgODEgOSA5NQoxMSA4NCAxMyAyNjAgMzQgMzM1IDQwIDM5IDQgODQgOSAxMDAgMTEgNTggMTAgMTczIDI0IDQwMCA1MCAyOCAzIDYxIDcgNzUgOQo0MiA3IDE4MiAyNCAyNDUgMzAgMzMgMyA2OSA4IDgwIDEwIDExIDIgNTEgNyA4OCAxMCAzNyA0IDc2IDggODUgMTAgMjIgNCAxMDMKMTQgMTY3IDIwIDI4IDMgNzEgOCA5NSAxMSAyNSAyIDYzIDcgODUgOSAyMiAyIDUxIDYgNjUgOSAxNCAyIDU3IDcgOTUgMTEgMzkKNCA3MiA4IDczIDEwIDIgMiAxMCAxIDE4IC0yIDEyIC00IDE0IC0zMTEgMTQgLTIwNzN6Ii8+CjxwYXRoIGQ9Ik0yNDEwIDQ1OTQgYy01MiAtNyAtMTIyIC0xNSAtMTU1IC0xOCAtMzQgLTQgLTcwIC04IC04MCAtMTAgLTExIC0yCi01MSAtNyAtOTAgLTExIC0zOCAtNCAtNzkgLTkgLTkwIC0xMSAtMTAgLTIgLTQ5IC02IC04NSAtOSAtNjIgLTYgLTkxIC05Ci0xNjIgLTIwIC00MyAtNiAtOTEgLTEyIC0yMTMgLTI1IC0xNTkgLTE3IC0xNzAgLTE4IC0zNzAgLTQ0IC0xMDQgLTE0IC0yMTYKLTI4IC0yNDggLTMwIC0zMSAtMyAtNjcgLTggLTgwIC0xMCAtMTIgLTMgLTU0IC04IC05MyAtMTEgLTM5IC0zIC04MCAtNyAtOTAKLTkgLTExIC0yIC01MCAtNyAtODkgLTExIC0xMDYgLTEyIC0xMDUgLTEwIC0xMDUgLTIyOSAwIC0xMDAgMCAtOTU3IDAgLTE5MDYKbC0xIC0xNzI1IDIyIC0xNyBjMTEgLTEwIDI0IC0xNiAyOCAtMTQgNCAzIDMyIDcgNjIgMTAgMzAgMyA3MCA4IDg5IDEwIDM4IDUKMTEyIDE0IDE3NSAyMiAyMiAyIDUxIDYgNjUgOSAxNCAyIDU2IDcgOTMgMTAgMzcgNCA3NiA4IDg1IDEwIDkgMSA0NiA2IDgyIDEwCjk2IDEwIDE1MiAxNiAxODAgMjAgMzcgNiAxNzggMjMgMjM2IDI5IDI5IDMgMTAzIDEyIDE2NSAyMCA2MyA4IDE0MiAxOCAxNzYKMjEgMzQgMyA3MCA3IDgwIDkgMTAgMiA0MyA3IDczIDEwIDMyNCAzOCA0NDQgNTYgNDU2IDY4IDEyIDEyIDE0IDMwNyAxNCAxOTIzCjAgMTgwNiAtMSAxOTEwIC0xNyAxOTI1IC0xNiAxNCAtMjkgMTUgLTExMyA0eiBtMTAwIC0xOTI5IGMwIC0xNTM3IC0yIC0xOTE1Ci0xMyAtMTkxNSAtMTQgMCAtMTkwOSAtMjI2IC0xOTY5IC0yMzUgbC0zOCAtNSAwIDE5MTMgMCAxOTE0IDI2OCAzMiBjMTQ3IDE4CjU5NiA3MiA5OTcgMTIxIDQwMiA0OCA3MzYgODggNzQzIDg5IDkgMSAxMiAtMzg4IDEyIC0xOTE0eiIvPgo8cGF0aCBkPSJNMTk5OSA0Mzc2IGMtOTkgLTI2IC0xODIgLTk3IC0yMzEgLTE5NSBsLTIzIC00NiAwIC0xNTg1IDAgLTE1ODUgMjIKLTQxIGM1MCAtOTQgMTY0IC0xNTMgMjY3IC0xMzkgNTkgNyAxMjYgMzUgMTczIDcxIDMzIDI1IDM1IDI1IDc0IDEwIDIyIC05IDYwCi0xNiA4NCAtMTYgbDQ1IDAgMCA5OCAtMSA5NyAtMzIgMjAgLTMyIDIwIC0zIDgyMyAtMiA4MjQgLTI4IC02IGMtMTUgLTMgLTcwCi0xMCAtMTIyIC0xNiAtNTIgLTYgLTEwNyAtMTMgLTEyMiAtMTYgbC0yOCAtNiAwIC0xMDMgMCAtMTA0IDM1IDYgMzUgNiAwCi03MjcgYzAgLTcxMCAwIC03MjYgLTIwIC03NDYgLTQzIC00MyAtMTEyIC0zMCAtMTI1IDIzIC0zIDEyIC00IDcxMiAtMyAxNTU1CmwzIDE1MzQgMjggMjQgYzMzIDI5IDc3IDMxIDEwMCA2IDE1IC0xNyAxNyAtNzggMTcgLTY3MCBsMCAtNjUxIDI4IDQgYzM3IDYKMTgxIDI1IDE5MyAyNSA2IDAgOSAyNDAgNyA2NjMgbC0zIDY2MiAtMjQgNDggYy0zMCA2MSAtOTIgMTEzIC0xNTUgMTMyIC01OQoxOCAtOTUgMTggLTE1NyAxeiBtMTc1IC0zOSBjNDYgLTIwIDEwMSAtNzQgMTE4IC0xMTQgMTMgLTMyIDIzIC0xMzE5IDEwCi0xMzI2IC00IC0zIC00MiAtOCAtODQgLTEyIGwtNzcgLTYgLTEgNjI3IGMwIDY3NCAxIDY1MiAtNDkgNjkxIC0zNyAyOCAtMTE0Ci0yIC0xNDMgLTU2IC0xMCAtMTkgLTEzIC0zMjUgLTEzIC0xNTYyIDEgLTE2NTMgLTIgLTE1NzQgNTIgLTE1OTggNDUgLTIxIDgzCi0xMyAxMTkgMjMgbDM0IDM0IDAgNzIzIGMwIDY1OCAtMSA3MjQgLTE3IDc0MSAtOSAxMCAtMjQgMTggLTM1IDE4IC0xNiAwIC0xOAo5IC0xOCA3NyAwIDQyIDIgNzQgNiA3MSAzIC0zIDIyIC0yIDQyIDMgMjAgNSA3MSAxMSAxMTMgMTUgbDc2IDYgMiAtNzkxIGMxCi00NDAgNiAtODAyIDExIC04MTUgNSAtMTMgMjEgLTMzIDM3IC00NCAyNyAtMTkgMjggLTIzIDI1IC05MSBsLTQgLTcxIC0zMyAwCmMtMTkgMCAtNTAgOCAtNzAgMTcgLTM0IDE2IC0zNiAxNiAtNjMgLTUgLTE1NiAtMTI1IC0zNjEgLTk2IC00MjYgNjAgLTE0IDMzCi0xNiAyMDkgLTE2IDE1OTYgMCAxMTY2IDMgMTU2NyAxMiAxNTkzIDMxIDkyIDExMiAxNzIgMjA1IDIwNCA0OSAxNyAxMzggMTMKMTg3IC04eiIvPgo8cGF0aCBkPSJNMTQ2MyA0Mjk3IGwtOTMgLTEwIDAgLTE3ODEgMCAtMTc4MiA5MyAxMiBjNTAgNyAxMDIgMTMgMTE1IDEzIGwyMgoxIDAgMTc4MCAwIDE3ODAgLTIyIC0yIGMtMTMgMCAtNjUgLTUgLTExNSAtMTF6IG0xMDQgLTE3NjkgYzMgLTE2NTUgMiAtMTc0NwotMTUgLTE3NTIgLTkgLTMgLTQ3IC04IC04NCAtMTIgbC02OCAtNiAwIDUzIGMtNCAxMTQzIDIgMzQzNSAxMCAzNDQzIDUgNSAzMwoxMSA2MSAxMiAyOCAyIDU0IDYgNTcgOSAzIDMgMTMgNCAyMiAzIDEzIC0zIDE1IC0xODIgMTcgLTE3NTB6Ii8+CjxwYXRoIGQ9Ik03NTUgNDIxMSBsLTEzMCAtMTYgLTMgLTE3ODAgLTIgLTE3ODEgMTc5IDIyIGMyMDEgMjUgMjQ1IDQxIDMxNgoxMDkgMjQgMjQgNTYgNjcgNzIgOTYgbDI4IDU0IDAgNzM1IDAgNzM2IC0yNiA0OSAtMjUgNTAgMjUgNTUgMjYgNTUgMyA3MDggYzIKNDczIC0xIDcyMSAtOCA3NDcgLTE2IDU5IC02OCAxMjAgLTEyOSAxNTIgLTYzIDMzIC0xMjAgMzUgLTMyNiA5eiBtMzE0IC0zMQpjNDAgLTIwIDk0IC04MSAxMTIgLTEyOSAxMCAtMjUgMTMgLTEzOTIgMyAtMTQ0NSAtMiAtMTYgLTE0IC00NiAtMjQgLTY3IC0yNAotNDUgLTI1IC02OCAtNiAtOTQgMzQgLTQ5IDM2IC05NSAzNiAtODA4IGwwIC03MDkgLTMyIC02MCBjLTU1IC0xMDQgLTE0NAotMTU5IC0yNzkgLTE3MyAtMzUgLTQgLTc1IC05IC04OSAtMTEgLTE0IC0yIC01MSAtNiAtODIgLTEwIGwtNTggLTcgMCAxNzQ2CmMxIDEyNTEgNCAxNzQ4IDEyIDE3NTMgNiA0IDI1IDkgNDIgMTAgMTcgMiA0NSA2IDYxIDggMTcgMyA1OSA4IDk1IDEyIDM2IDMKNjcgOSA3MCAxMSAxMCAxMCA5OCAtOCAxMzkgLTI3eiIvPgo8cGF0aCBkPSJNODg0IDQwNTIgYy0xMiAtMiAtMzAgLTEyIC00MCAtMjMgLTE3IC0xOSAtMTkgLTUzIC0yMSAtNTcyIC0xIC0zMDQKLTMgLTU1NiAtMyAtNTYyIDAgLTUgMSAtODYgNCAtMTc5IDYgLTIwMiA5IC0yMDggOTEgLTE5MyA1NiAxMCA5MiA0NSAxMDQgMTAwCjEzIDU5IDUgMTM2MSAtOCAxMzg1IC0yMCAzNyAtNjkgNTQgLTEyNyA0NHogbTkxIC00NyBsMjUgLTI0IDAgLTY3OCBjMCAtODAxCjEwIC03MzYgLTExNyAtNzU3IGwtMzMgLTUgMCA3MzkgMCA3MzkgMzMgNCBjMTcgMiA0MCA1IDUwIDUgMTAgMSAyOSAtOSA0MgotMjN6Ii8+CjxwYXRoIGQ9Ik04ODMgMjM3MiBjLTIxIC0yIC0zOCAtMTEgLTQ1IC0yNSAtMjAgLTM0IC0xOCAtMTQ2OCAxIC0xNDkxIDEwIC0xMgoyNSAtMTUgNTYgLTEyIDYwIDcgODkgMjMgMTEwIDU5IDE4IDMwIDE5IDY2IDIwIDcwMiAxIDM2OSAtMiA2ODIgLTYgNjk3IC0xNQo1NCAtNTkgNzcgLTEzNiA3MHogbTk2IC00OCBjMjEgLTI2IDIxIC0zNCAyMSAtNzAwIDAgLTUwMSAtMyAtNjgwIC0xMiAtNjk5Ci0xNSAtMzMgLTU2IC01NSAtMTAzIC01NSBsLTM1IDAgMCA3MzQgMCA3MzUgMjMgNCBjNjAgMTIgODYgNyAxMDYgLTE5eiIvPgo8L2c+Cjwvc3ZnPgo='/>";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "NSeven NFT Token for TestNet".to_string(),
                symbol: "N7".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON_BASE64.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

    /// Mint a new token with ID=`token_id` belonging to `receiver_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. `self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
    /// initialization call to `new`.
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        self.tokens.internal_mint(token_id, receiver_id, Some(token_metadata))
    }

    pub fn token_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("NSeven Limited Edition".into()),
            description: Some("Limited Edition original NSeven NEARWarrior".into()),
            media: Some("https://i.ibb.co/1n9fjsd/1589875387-16.png".to_string()),
            media_hash: None,
            copies: Some(1u64),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }        
    }


    #[payable]
    pub fn nft_mint_default(
        &mut self,
        token_id: TokenId,
        receiver_id: AccountId,
    ) -> Token {
        self.tokens.internal_mint(token_id, receiver_id, Some(

            TokenMetadata {
                title: Some("NSeven Limited Edition".into()),
                description: Some("Limited Edition original NSeven NEARWarrior".into()),
                media: Some("https://i.ibb.co/1n9fjsd/1589875387-16.png".to_string()),
                media_hash: None,
                copies: Some(1u64),
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: None,
                reference_hash: None,
            }

        ))
    }    
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use std::collections::HashMap;

    use super::*;

    const MINT_STORAGE_COST: u128 = 5870000000000000000000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn sample_token_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("Olympus Mons".into()),
            description: Some("The tallest mountain in the charted solar system".into()),
            media: None,
            media_hash: None,
            copies: Some(1u64),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.nft_token("1".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "0".to_string();
        let token = contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id, accounts(0).to_string());
        assert_eq!(token.metadata.unwrap(), sample_token_metadata());
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), token_id.clone(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(token_id.clone()) {
            assert_eq!(token.token_id, token_id);
            assert_eq!(token.owner_id, accounts(1).to_string());
            assert_eq!(token.metadata.unwrap(), sample_token_metadata());
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }

    #[test]
    fn test_revoke() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke(token_id.clone(), accounts(1));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
    }

    #[test]
    fn test_revoke_all() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint(token_id.clone(), accounts(0), sample_token_metadata());

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke_all(token_id.clone());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }
}
