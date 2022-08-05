use borsh::BorshDeserialize;

use {borsh::BorshSerialize, solana_program::pubkey::Pubkey};
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct AuctionOrder {
    pub is_initialized: bool,
    pub owner_wallet_address: Pubkey,
    pub token_account: Pubkey,
    pub time: u64,
    pub minimum_price: u64,
    pub bidder_wallet_address: Pubkey,
    pub bidder_zion_token_account: Pubkey,
    pub bid: u64,
    pub total_bid_amount: u64,
    pub token_type: Pubkey,
}
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub struct AuctionOrderSol {
    pub is_initialized: bool,
    pub owner_wallet_address: Pubkey,
    pub minimum_price: u64,
    pub time: u64,
    pub token_account: Pubkey,
    pub bidder_wallet_address: Pubkey,
    pub bid: u64,
    pub total_bid_amount: u64,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct RaffleCounter {
    pub raffler_address: Pubkey,
    pub entry_counter: u32,
}
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct RaffleOrder {
    pub is_initialized: bool,
    pub owner_wallet_address: Pubkey,
    pub time: u64,
    pub token_account: Pubkey,
    pub price: u64,
    pub token_type: Pubkey,
    pub ticket_supply: u64,
    pub raffle_entry_record: Vec<RaffleCounter>,
}
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct FeaturedRaffles {
    pub is_initialized: bool,
    pub raffle_account: Pubkey,
    pub is_featured: bool,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct BlackMarketInfo {
    pub is_initialized: bool,
    pub owner_wallet_address: Pubkey,
    pub raffle_fee: u64,
    pub featuring_fee: u64,
}
