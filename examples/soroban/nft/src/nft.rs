use loam_sdk::{
    soroban_sdk::{self, contracttype, env, Address, Bytes, Lazy, Map},
    IntoKey,
};
use loam_subcontract_core::Core;

use crate::{
    subcontract::{IsInitable, IsNonFungible},
    Contract,
};

#[contracttype]
#[derive(IntoKey)]
pub struct MyNonFungibleToken {
    admin: Address,
    name: Bytes,
    total_count: u32,
    owners_to_nft_ids: Map<Address, u32>,
    nft_ids_to_owners: Map<u32, Address>,
    nft_ids_to_metadata: Map<u32, Bytes>,
}

impl MyNonFungibleToken {
    #[must_use]
    pub fn new(admin: Address, name: Bytes) -> Self {
        MyNonFungibleToken {
            admin,
            name,
            total_count: 0,
            owners_to_nft_ids: Map::new(env()),
            nft_ids_to_owners: Map::new(env()),
            nft_ids_to_metadata: Map::new(env()),
        }
    }
}

impl Default for MyNonFungibleToken {
    fn default() -> Self {
        MyNonFungibleToken::new(env().current_contract_address(), Bytes::new(env()))
    }
}

impl IsInitable for MyNonFungibleToken {
    fn nft_init(&self, admin: Address, name: Bytes) {
        Contract::admin_get().unwrap().require_auth();
        MyNonFungibleToken::set_lazy(MyNonFungibleToken::new(admin, name));
    }
}

impl IsNonFungible for MyNonFungibleToken {
    // Mint a new NFT with the given owner address and metadata, returning the id
    fn mint(&mut self, owner: Address, metadata: Bytes) -> u32 {
        owner.require_auth();

        let current_count = self.total_count;
        let new_id = current_count + 1;

        //todo: check that the metadata is unique
        self.nft_ids_to_metadata.set(new_id, metadata);
        self.nft_ids_to_owners.set(new_id, owner.clone());
        self.owners_to_nft_ids.set(owner, new_id);
        self.total_count = new_id;

        new_id
    }

    // Transfer the NFT from the current owner to the new owner
    fn transfer(&mut self, id: u32, current_owner: Address, new_owner: Address) {
        current_owner.require_auth();
        self.get_nft(id).expect("NFT does not exist");
        if let Some(owner_id) = self.nft_ids_to_owners.get(id) {
            assert!(
                owner_id == current_owner,
                "You are not the owner of this NFT"
            );
            // remove the current owner
            self.nft_ids_to_owners.remove(id);
            self.owners_to_nft_ids.remove(current_owner);

            // add the new owner
            self.nft_ids_to_owners.set(id, new_owner.clone());
            self.owners_to_nft_ids.set(new_owner, id);
        }
    }

    // Get the NFT from the contract's storage by id
    fn get_nft(&self, id: u32) -> Option<Bytes> {
        self.nft_ids_to_metadata.get(id)
    }

    // Get the NFT from the contract's storage by owner id
    fn get_owner(&self, id: u32) -> Option<Address> {
        self.nft_ids_to_owners.get(id)
    }

    fn get_total_count(&self) -> u32 {
        self.total_count
    }
}
