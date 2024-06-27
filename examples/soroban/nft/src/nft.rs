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
    owners_to_nft_ids: Map<Address, u32>,
    nft_ids_to_owners: Map<u32, Address>,
    nft_ids_to_metadata: Map<u32, Bytes>,
}

impl MyNonFungibleToken {
    pub fn new(admin: Address, name: Bytes) -> Self {
        MyNonFungibleToken {
            admin,
            name,
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
    fn nft_init(&mut self, admin: Address, name: Bytes) {
        Contract::admin_get().unwrap().require_auth();
        MyNonFungibleToken::set_lazy(MyNonFungibleToken::new(admin, name));
    }
}

impl IsNonFungible for MyNonFungibleToken {
    // Mint a new NFT with the given ID, owner, and metadata
    fn mint(&mut self, id: u32, owner: Address, metadata: Bytes) {
        owner.require_auth();

        // if the nft id is not already in the contract's storage we can add it
        // todo: handle this more gracefully
        if let Some(_metadata) = self.nft_ids_to_metadata.get(id.clone()) {
            panic!("NFT with this ID already exists");
        }

        self.nft_ids_to_metadata.set(id.clone(), metadata);
        self.nft_ids_to_owners.set(id.clone(), owner.clone());
        self.owners_to_nft_ids.set(owner, id);
    }

    // Transfer the NFT from the current owner to the new owner
    fn transfer(&mut self, id: u32, current_owner: Address, new_owner: Address) {
        if let Some(owner_id) = self.nft_ids_to_owners.get(id.clone()) {
            if owner_id != current_owner {
                panic!("You are not the owner of this NFT");
            }
            // remove the current owner
            self.nft_ids_to_owners.remove(id.clone());
            self.owners_to_nft_ids.remove(current_owner);

            // add the new owner
            self.nft_ids_to_owners.set(id.clone(), new_owner.clone());
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
}
