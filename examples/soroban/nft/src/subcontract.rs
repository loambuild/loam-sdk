use loam_sdk::{soroban_sdk::Lazy, subcontract};

#[subcontract]
pub trait IsNonFungible {
    // Mint a new NFT with the given ID, owner, and metadata
    fn mint(
        &mut self,
        owner: loam_sdk::soroban_sdk::Address,
        metadata: loam_sdk::soroban_sdk::Bytes,
    ) -> u32;

    // Transfer an NFT with the given ID from current_owner to new_owner
    fn transfer(
        &mut self,
        id: u32,
        current_owner: loam_sdk::soroban_sdk::Address,
        new_owner: loam_sdk::soroban_sdk::Address,
    );

    // Get the NFT with the given ID
    fn get_nft(&self, id: u32) -> Option<loam_sdk::soroban_sdk::Bytes>;

    // Find the owner of the NFT with the given ID
    fn get_owner(&self, id: u32) -> Option<loam_sdk::soroban_sdk::Address>;

    // Get the total count of NFTs
    fn get_total_count(&self) -> u32;
}

#[subcontract]
pub trait IsInitable {
    fn nft_init(&self, admin: loam_sdk::soroban_sdk::Address, name: loam_sdk::soroban_sdk::Bytes);
}
