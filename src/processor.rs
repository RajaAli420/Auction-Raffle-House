use std::str::FromStr;

use mpl_token_metadata::state::Metadata;
use solana_program::system_instruction::transfer;

use crate::state::{AuctionOrderSol, FeaturedRaffles};

use {
    crate::state::{RaffleCounter, RaffleOrder},
    crate::{error::MarketError, instruction::MarketplaceInstruction, state::AuctionOrder},
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        borsh::try_from_slice_unchecked,
        entrypoint::ProgramResult,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        sysvar::{clock::Clock, Sysvar},
    },
    spl_token::instruction as SPLIX,
    spl_token::state as SPLS,
};
pub struct Processor {}
impl Processor {
    pub fn start_process(
        program_id: Pubkey,
        account_info: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let run = MarketplaceInstruction::unpack(instruction_data)?;
        match run {
            MarketplaceInstruction::AuctionStart {
                minimum_price,
                time,
            } => Self::auction_order(program_id, account_info, minimum_price, time),
            MarketplaceInstruction::PlaceBid { new_bid } => {
                Self::place_bid(program_id, account_info, new_bid)
            }
            MarketplaceInstruction::CompleteAuction => {
                Self::complete_auction_order(program_id, account_info)
            }
            MarketplaceInstruction::CompleteAuctionAnyTime => {
                Self::complete_auction_order_any_time(program_id, account_info)
            }
            MarketplaceInstruction::CanceAuction => Self::cancel_auction(program_id, account_info),
            MarketplaceInstruction::RaffleStart {
                time,
                price,
                total_ticket,
            } => Self::raffle_start(program_id, account_info, time, price, total_ticket),
            MarketplaceInstruction::EndRaffle => Self::end_raffle(program_id, account_info),
            MarketplaceInstruction::MakeRaffleEntry { amount, quantity } => {
                Self::make_raffle_entry(program_id, account_info, amount, quantity)
            }
            MarketplaceInstruction::AuctionStartSol {
                minimum_price,
                time,
            } => Self::auction_order_sol(program_id, account_info, minimum_price, time),
            MarketplaceInstruction::PlaceBidSol { new_bid } => {
                Self::place_bid_sol(program_id, account_info, new_bid)
            }
            MarketplaceInstruction::CompleteAuctionSol => {
                Self::complete_auction_order_sol(program_id, account_info)
            }
            MarketplaceInstruction::CanceAuctionSol => {
                Self::cancel_auction_sol(program_id, account_info)
            }
            MarketplaceInstruction::CompleteAuctionAnyTimeSol => {
                Self::complete_auction_order_sol_anytime(program_id, account_info)
            }
            MarketplaceInstruction::CompleteAuctionUserZion => {
                Self::complete_auction_order_user(program_id, account_info)
            }
            MarketplaceInstruction::CompleteAuctionUserSol => {
                Self::complete_auction_order_sol_user(program_id, account_info)
            }
            MarketplaceInstruction::HandleNonTransfer => {
                Self::handle_raffle_non_transfers(program_id, account_info)
            }
        }
    }
    fn auction_order(
        program_id: Pubkey,
        account_info: &[AccountInfo],
        minimum_price: u64,
        time: u64,
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let holder_info = next_account_info(accounts)?; //cat king wallet
        let token_account_info = next_account_info(accounts)?; // NFT for auction Wallet
        let auction_order_account_info = next_account_info(accounts)?; // auction data account
        let token_program = next_account_info(accounts)?; // token program
        let metadata_account = next_account_info(accounts)?;
        let token_type_info = next_account_info(accounts)?;
        let metadata = Metadata::from_account_info(metadata_account)?;
        let mut found = 0;
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        if metadata.collection.is_some() || metadata.data.creators.is_some() {
            if let Some(collection) = metadata.collection {
                if collection.verified == true {
                    found += 1;
                }
            }
            if let Some(creators) = metadata.data.creators {
                if creators[0].verified == true {
                    found += 1;
                }
            }
        } else {
            return Err(MarketError::InvalidInstruction.into());
        }
        if found == 0 {
            return Err(MarketError::UnverifiedNFT.into());
        }

        let mut auction_order_struct: AuctionOrder =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        if auction_order_struct.is_initialized == true {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        auction_order_struct.is_initialized = true;
        auction_order_struct.owner_wallet_address = *holder_info.key;
        auction_order_struct.token_account = *token_account_info.key;
        if (time - Clock::get()?.unix_timestamp as u64) < 604800 {
            auction_order_struct.time = time;
        } else {
            return Err(MarketError::MaxTimeLimit.into());
        }
        auction_order_struct.bid = 0;
        if SPLS::Account::unpack_unchecked(&mut token_account_info.data.borrow())?.amount != 1 {
            return Err(ProgramError::InsufficientFunds);
        }
        if Clock::get()?.unix_timestamp as u64 > time {
            return Err(MarketError::InvalidInstruction.into());
        }
        if minimum_price as f64 / 1000000000.00 == 0.00 || holder_info.is_signer != true {
            return Err(MarketError::MinPrice.into());
        }
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on"], &program_id);
        if let Err(error) = invoke(
            &SPLIX::set_authority(
                token_program.key,
                token_account_info.key,
                Some(&pda),
                SPLIX::AuthorityType::AccountOwner,
                holder_info.key,
                &[holder_info.key],
            )?,
            &[
                token_program.clone(),
                token_account_info.clone(),
                holder_info.clone(),
            ],
        ) {
            return Err(error);
        }
        auction_order_struct.minimum_price = minimum_price;
        auction_order_struct.total_bid_amount = 0;
        auction_order_struct.token_type = *token_type_info.key;
        auction_order_struct
            .serialize(&mut &mut auction_order_account_info.data.borrow_mut()[..])?;
        Ok(())
    }
    fn place_bid(program_id: Pubkey, account_info: &[AccountInfo], bid: u64) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let bidder_account_info = next_account_info(accounts)?; // bidder wallet
        let biddder_zion_token_account_info = next_account_info(accounts)?; // zion to be transferred from
        let zion_mint_account_info = next_account_info(accounts)?;
        let auction_order_account_info = next_account_info(accounts)?; // auction data to be updated account
        let previous_bidder = next_account_info(accounts)?; // previous bidder info
        let previous_bidder_zion_token_account_info = next_account_info(accounts)?; //
        let token_program = next_account_info(accounts)?;
        let pda_zion_token_account_info = next_account_info(accounts)?; // would be created once
        let pda_account_info = next_account_info(accounts)?;
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let mut auction_order_struct: AuctionOrder =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on"], &program_id);
        //initial bid
        if auction_order_struct.owner_wallet_address == *bidder_account_info.key {
            return Err(MarketError::OwnerCannotBid.into());
        }
        if *previous_bidder_zion_token_account_info.key
            != auction_order_struct.bidder_zion_token_account
        {
            return Err(MarketError::ValueMisMatch.into());
        }
        if &pda != pda_account_info.key {
            return Err(MarketError::PdaError.into());
        }
        if SPLS::Account::unpack_unchecked(&mut pda_zion_token_account_info.data.borrow())?.owner
            != *pda_account_info.key
        {
            return Err(MarketError::WrongOwner.into());
        }
        let bid_increment = ((auction_order_struct.total_bid_amount as f64 * 5.00) / 100.00) as u64;
        if bid < bid_increment + auction_order_struct.bid {
            return Err(MarketError::BidMustBeGreater.into());
        }
        if auction_order_struct.time - Clock::get()?.unix_timestamp as u64 <= 120 {
            auction_order_struct.time += 120;
        }
        if bid > auction_order_struct.minimum_price
            && auction_order_struct.bid == 0
            && auction_order_struct.time > Clock::get()?.unix_timestamp as u64
        {
            if let Err(error) = invoke(
                &SPLIX::transfer(
                    token_program.key,
                    biddder_zion_token_account_info.key,
                    pda_zion_token_account_info.key,
                    bidder_account_info.key,
                    &[bidder_account_info.key],
                    bid,
                )?,
                &[
                    biddder_zion_token_account_info.clone(),
                    zion_mint_account_info.clone(),
                    pda_zion_token_account_info.clone(),
                    bidder_account_info.clone(),
                ],
            ) {
                return Err(error);
            }
            auction_order_struct.bidder_wallet_address = *bidder_account_info.key;
            auction_order_struct.bidder_zion_token_account = *biddder_zion_token_account_info.key;
            auction_order_struct.bid = bid;
            auction_order_struct.total_bid_amount = bid;
        } else if bid > auction_order_struct.bid //bigger bid
            && auction_order_struct.time >Clock::get()?.unix_timestamp as u64
            && auction_order_struct.minimum_price < bid
        {
            //setting new bid
            if let Err(error) = invoke(
                &SPLIX::transfer(
                    token_program.key,
                    biddder_zion_token_account_info.key,
                    pda_zion_token_account_info.key,
                    bidder_account_info.key,
                    &[bidder_account_info.key],
                    bid,
                )?,
                &[
                    biddder_zion_token_account_info.clone(),
                    zion_mint_account_info.clone(),
                    pda_zion_token_account_info.clone(),
                    bidder_account_info.clone(),
                ],
            ) {
                return Err(error);
            }
            if *previous_bidder.key != auction_order_struct.bidder_wallet_address
                || *previous_bidder.key
                    != SPLS::Account::unpack_unchecked(
                        &mut previous_bidder_zion_token_account_info.data.borrow(),
                    )?
                    .owner
            {
                return Err(ProgramError::IllegalOwner);
            }
            //refunding previous bidder
            if let Err(error) = invoke_signed(
                &SPLIX::transfer(
                    token_program.key,
                    pda_zion_token_account_info.key,
                    previous_bidder_zion_token_account_info.key,
                    &pda,
                    &[&pda],
                    auction_order_struct.bid,
                )?,
                &[
                    pda_zion_token_account_info.clone(),
                    zion_mint_account_info.clone(),
                    previous_bidder_zion_token_account_info.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            auction_order_struct.bidder_wallet_address = *bidder_account_info.key;
            auction_order_struct.bidder_zion_token_account = *biddder_zion_token_account_info.key;
            auction_order_struct.bid = bid;
            auction_order_struct.total_bid_amount = auction_order_struct.total_bid_amount + bid;
        } else {
            return Err(MarketError::BidMustBeGreater.into());
        }

        auction_order_struct
            .serialize(&mut &mut auction_order_account_info.data.borrow_mut()[..])?;
        Ok(())
    }
    //admin 100
    fn complete_auction_order(program_id: Pubkey, account_info: &[AccountInfo]) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let cat_king = next_account_info(accounts)?; //holder info
        let cat_king_zion_token_account = next_account_info(accounts)?; //can be a static account value for now to transfer the bid amount
        let bidder_info = next_account_info(accounts)?; //get from sellerOrder Account in web3 claiming k liyey
        let auction_order_account_info = next_account_info(accounts)?; //fetch data and matach with web3
        let auction_nft_token_account_info = next_account_info(accounts)?; //from auciton order Account in web3
        let pda_account_info = next_account_info(accounts)?; //which holder the authority for NFT on Auction
        let auction_nft_new_token_account = next_account_info(accounts)?; // new token account of user to send nft to
        let auction_nft_mint = next_account_info(accounts)?; // auction TOken mint account for transfer function
        let token_program = next_account_info(accounts)?;
        let pda_zion_token_account_info = next_account_info(accounts)?; // to transfer to cat king zion token acount
        let zion_mint_account_info = next_account_info(accounts)?; // zoin mint static value
        let auction_order_struct: AuctionOrder =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        if bidder_info.is_signer != true
            || *cat_king.key != auction_order_struct.owner_wallet_address
            || *auction_nft_token_account_info.key != auction_order_struct.token_account
            || *bidder_info.key != auction_order_struct.bidder_wallet_address
        {
            return Err(MarketError::ValueMisMatch.into());
        }
        if SPLS::Account::unpack_unchecked(&mut auction_nft_new_token_account.data.borrow())?.owner
            != *bidder_info.key
        {
            return Err(MarketError::WrongOwner.into());
        }
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on"], &program_id);
        if *pda_account_info.key != pda {
            return Err(MarketError::PdaError.into());
        }
        if SPLS::Account::unpack_unchecked(&mut cat_king_zion_token_account.data.borrow())?.owner
            != auction_order_struct.owner_wallet_address
        {
            return Err(MarketError::WrongOwner.into());
        }

        //transferring ZIon to cat king
        if Clock::get()?.unix_timestamp as u64 > auction_order_struct.time {
            if let Err(error) = invoke_signed(
                &SPLIX::transfer(
                    token_program.key,
                    pda_zion_token_account_info.key,
                    cat_king_zion_token_account.key,
                    &pda,
                    &[&pda],
                    auction_order_struct.bid,
                )?,
                &[
                    pda_zion_token_account_info.clone(),
                    zion_mint_account_info.clone(),
                    cat_king_zion_token_account.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            // transfering NFT to bidder when they claim
            if let Err(error) = invoke_signed(
                &SPLIX::transfer_checked(
                    token_program.key,
                    auction_nft_token_account_info.key,
                    auction_nft_mint.key,
                    auction_nft_new_token_account.key,
                    &pda,
                    &[&pda],
                    1,
                    0,
                )?,
                &[
                    auction_nft_token_account_info.clone(),
                    auction_nft_mint.clone(),
                    auction_nft_new_token_account.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if let Err(error) = invoke_signed(
                &SPLIX::close_account(
                    token_program.key,
                    auction_nft_token_account_info.key,
                    cat_king.key,
                    &pda,
                    &[&pda],
                )?,
                &[
                    auction_nft_token_account_info.clone(),
                    cat_king.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            //closing the auction order account
            **cat_king.try_borrow_mut_lamports()? = cat_king
                .lamports()
                .checked_add(auction_order_account_info.lamports())
                .ok_or(ProgramError::InsufficientFunds)?;
            **auction_order_account_info.try_borrow_mut_lamports()? = 0;
            *auction_order_account_info.try_borrow_mut_data()? = &mut [];
        } else {
            return Err(MarketError::AuctionNotEnded.into());
        }
        Ok(())
    }
    //admin 2
    fn complete_auction_order_any_time(
        program_id: Pubkey,
        account_info: &[AccountInfo],
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let holder_info = next_account_info(accounts)?; //holder info
        let holder_zion_token_account = next_account_info(accounts)?; //can be a static account value for now to transfer the bid amount
        let bidder_info = next_account_info(accounts)?; //get from sellerOrder Account in web3 claiming k liyey
        let auction_order_account_info = next_account_info(accounts)?; //fetch data and matach with web3
        let auction_nft_token_account_info = next_account_info(accounts)?; //from auciton order Account in web3
        let pda_account_info = next_account_info(accounts)?; //which holder the authority for NFT on Auction
        let auction_nft_new_token_account = next_account_info(accounts)?; // new token account of user to send nft to
        let auction_nft_mint = next_account_info(accounts)?; // auction TOken mint account for transfer function
        let token_program = next_account_info(accounts)?;
        let pda_zion_token_account_info = next_account_info(accounts)?; // to transfer to cat king zion token acount
        let zion_mint_account_info = next_account_info(accounts)?; // zoin mint static value
        let client_zion_token_account_info = next_account_info(accounts)?;
        let auction_order_struct: AuctionOrder =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        if holder_info.is_signer != true
            || *holder_info.key != auction_order_struct.owner_wallet_address
            || *auction_nft_token_account_info.key != auction_order_struct.token_account
            || *bidder_info.key != auction_order_struct.bidder_wallet_address
        {
            return Err(MarketError::ValueMisMatch.into());
        }
        if SPLS::Account::unpack_unchecked(&mut client_zion_token_account_info.data.borrow())?.owner
            != Pubkey::from_str("5XJKsYXoLUSPh5KwdhecAyACLZujGKJ7z6ovQBznWKtq").unwrap()
        {
            return Err(MarketError::WrongOwner.into());
        }
        if SPLS::Account::unpack_unchecked(&mut auction_nft_new_token_account.data.borrow())?.owner
            != *bidder_info.key
        {
            return Err(MarketError::WrongOwner.into());
        }

        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on"], &program_id);
        if *pda_account_info.key != pda {
            return Err(MarketError::PdaError.into());
        }
        //transferring ZIon to cat king
        if auction_order_struct.bid == 0 {
            return Err(MarketError::InvalidInstruction.into());
        }
        if let Err(error) = invoke_signed(
            &SPLIX::transfer(
                token_program.key,
                pda_zion_token_account_info.key,
                holder_zion_token_account.key,
                &pda,
                &[&pda],
                (auction_order_struct.bid as f64 * 97.5 / 100.00) as u64,
            )?,
            &[
                pda_zion_token_account_info.clone(),
                zion_mint_account_info.clone(),
                holder_zion_token_account.clone(),
                pda_account_info.clone(),
            ],
            &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
        ) {
            return Err(error);
        }
        if let Err(error) = invoke_signed(
            &SPLIX::transfer(
                token_program.key,
                pda_zion_token_account_info.key,
                client_zion_token_account_info.key,
                &pda,
                &[&pda],
                (auction_order_struct.bid as f64 * 2.5 / 100.00) as u64,
            )?,
            &[
                pda_zion_token_account_info.clone(),
                zion_mint_account_info.clone(),
                client_zion_token_account_info.clone(),
                pda_account_info.clone(),
            ],
            &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
        ) {
            return Err(error);
        }
        // transfering NFT to bidder when they claim
        if let Err(error) = invoke_signed(
            &SPLIX::transfer_checked(
                token_program.key,
                auction_nft_token_account_info.key,
                auction_nft_mint.key,
                auction_nft_new_token_account.key,
                &pda,
                &[&pda],
                1,
                0,
            )?,
            &[
                auction_nft_token_account_info.clone(),
                auction_nft_mint.clone(),
                auction_nft_new_token_account.clone(),
                pda_account_info.clone(),
            ],
            &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
        ) {
            return Err(error);
        }
        if let Err(error) = invoke_signed(
            &SPLIX::close_account(
                token_program.key,
                auction_nft_token_account_info.key,
                holder_info.key,
                &pda,
                &[&pda],
            )?,
            &[
                auction_nft_token_account_info.clone(),
                holder_info.clone(),
                pda_account_info.clone(),
            ],
            &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
        ) {
            return Err(error);
        }
        //closing the auction order account
        **holder_info.try_borrow_mut_lamports()? = holder_info
            .lamports()
            .checked_add(auction_order_account_info.lamports())
            .ok_or(ProgramError::InsufficientFunds)?;
        **auction_order_account_info.try_borrow_mut_lamports()? = 0;
        *auction_order_account_info.try_borrow_mut_data()? = &mut [];
        Ok(())
    }
    fn complete_auction_order_user(
        program_id: Pubkey,
        account_info: &[AccountInfo],
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let holder_info = next_account_info(accounts)?; //holder info
        let holder_zion_token_account = next_account_info(accounts)?; //can be a static account value for now to transfer the bid amount
        let bidder_info = next_account_info(accounts)?; //get from sellerOrder Account in web3 claiming k liyey
        let auction_order_account_info = next_account_info(accounts)?; //fetch data and matach with web3
        let auction_nft_token_account_info = next_account_info(accounts)?; //from auciton order Account in web3
        let pda_account_info = next_account_info(accounts)?; //which holder the authority for NFT on Auction
        let auction_nft_new_token_account = next_account_info(accounts)?; // new token account of user to send nft to
        let auction_nft_mint = next_account_info(accounts)?; // auction TOken mint account for transfer function
        let token_program = next_account_info(accounts)?;
        let pda_zion_token_account_info = next_account_info(accounts)?; // to transfer to cat king zion token acount
        let zion_mint_account_info = next_account_info(accounts)?; // zoin mint static value
        let client_zion_token_account_info = next_account_info(accounts)?;
        let auction_order_struct: AuctionOrder =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        if bidder_info.is_signer != true
            || *holder_info.key != auction_order_struct.owner_wallet_address
            || *auction_nft_token_account_info.key != auction_order_struct.token_account
            || *bidder_info.key != auction_order_struct.bidder_wallet_address
        {
            return Err(MarketError::ValueMisMatch.into());
        }
        if SPLS::Account::unpack_unchecked(&mut client_zion_token_account_info.data.borrow())?.owner
            != Pubkey::from_str("5XJKsYXoLUSPh5KwdhecAyACLZujGKJ7z6ovQBznWKtq").unwrap()
        {
            return Err(MarketError::WrongOwner.into());
        }

        if SPLS::Account::unpack_unchecked(&mut auction_nft_new_token_account.data.borrow())?.owner
            != *bidder_info.key
        {
            return Err(MarketError::WrongOwner.into());
        }
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on"], &program_id);
        if *pda_account_info.key != pda {
            return Err(MarketError::PdaError.into());
        }
        //transferring ZIon to cat king
        if Clock::get()?.unix_timestamp as u64 > auction_order_struct.time {
            if let Err(error) = invoke_signed(
                &SPLIX::transfer(
                    token_program.key,
                    pda_zion_token_account_info.key,
                    holder_zion_token_account.key,
                    &pda,
                    &[&pda],
                    (auction_order_struct.bid as f64 * 97.5 / 100.00) as u64,
                )?,
                &[
                    pda_zion_token_account_info.clone(),
                    zion_mint_account_info.clone(),
                    holder_zion_token_account.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if let Err(error) = invoke_signed(
                &SPLIX::transfer(
                    token_program.key,
                    pda_zion_token_account_info.key,
                    client_zion_token_account_info.key,
                    &pda,
                    &[&pda],
                    (auction_order_struct.bid as f64 * 2.5 / 100.00) as u64,
                )?,
                &[
                    pda_zion_token_account_info.clone(),
                    zion_mint_account_info.clone(),
                    client_zion_token_account_info.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            // transfering NFT to bidder when they claim
            if let Err(error) = invoke_signed(
                &SPLIX::transfer_checked(
                    token_program.key,
                    auction_nft_token_account_info.key,
                    auction_nft_mint.key,
                    auction_nft_new_token_account.key,
                    &pda,
                    &[&pda],
                    1,
                    0,
                )?,
                &[
                    auction_nft_token_account_info.clone(),
                    auction_nft_mint.clone(),
                    auction_nft_new_token_account.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if let Err(error) = invoke_signed(
                &SPLIX::close_account(
                    token_program.key,
                    auction_nft_token_account_info.key,
                    holder_info.key,
                    &pda,
                    &[&pda],
                )?,
                &[
                    auction_nft_token_account_info.clone(),
                    holder_info.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            //closing the auction order account
            **holder_info.try_borrow_mut_lamports()? = holder_info
                .lamports()
                .checked_add(auction_order_account_info.lamports())
                .ok_or(ProgramError::InsufficientFunds)?;
            **auction_order_account_info.try_borrow_mut_lamports()? = 0;
            *auction_order_account_info.try_borrow_mut_data()? = &mut [];
        } else {
            return Err(MarketError::AuctionNotEnded.into());
        }
        Ok(())
    }
    fn cancel_auction(program_id: Pubkey, account_info: &[AccountInfo]) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let payer_info = next_account_info(accounts)?;
        let token_account_info = next_account_info(accounts)?;
        let auction_order_account_info = next_account_info(accounts)?;
        let token_program = next_account_info(accounts)?;
        let pda_account_info = next_account_info(accounts)?;
        let bidder_info = next_account_info(accounts)?;
        let previous_bidder_zion_token_account_info = next_account_info(accounts)?;
        let zion_mint_account_info = next_account_info(accounts)?;
        let pda_zion_token_account_info = next_account_info(accounts)?;
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let auction_order_struct: AuctionOrder =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        if payer_info.is_signer != true
            || *payer_info.key != auction_order_struct.owner_wallet_address
            || *token_account_info.key != auction_order_struct.token_account
            || *bidder_info.key != auction_order_struct.bidder_wallet_address
        {
            return Err(ProgramError::IllegalOwner.into());
        }
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on"], &program_id);
        if *pda_account_info.key != pda {
            return Err(MarketError::PdaError.into());
        }
        if Clock::get()?.unix_timestamp as u64 > auction_order_struct.time {
            if let Err(error) = invoke_signed(
                &SPLIX::set_authority(
                    token_program.key,
                    token_account_info.key,
                    Some(&payer_info.key),
                    SPLIX::AuthorityType::AccountOwner,
                    &pda,
                    &[&pda],
                )?,
                &[
                    token_program.clone(),
                    token_account_info.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if auction_order_struct.bid != 0 {
                if let Err(error) = invoke_signed(
                    &SPLIX::transfer(
                        token_program.key,
                        pda_zion_token_account_info.key,
                        previous_bidder_zion_token_account_info.key,
                        &pda,
                        &[&pda],
                        auction_order_struct.bid,
                    )?,
                    &[
                        pda_zion_token_account_info.clone(),
                        zion_mint_account_info.clone(),
                        previous_bidder_zion_token_account_info.clone(),
                        pda_account_info.clone(),
                    ],
                    &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
                ) {
                    return Err(error);
                }
            }
            **payer_info.try_borrow_mut_lamports()? = payer_info
                .lamports()
                .checked_add(auction_order_account_info.lamports())
                .ok_or(ProgramError::InsufficientFunds)?;
            **auction_order_account_info.try_borrow_mut_lamports()? = 0;
            *auction_order_account_info.try_borrow_mut_data()? = &mut [];
        } else {
            return Err(MarketError::CannotCancel.into());
        }
        Ok(())
    }

    //sol auctions
    fn auction_order_sol(
        program_id: Pubkey,
        account_info: &[AccountInfo],
        minimum_price: u64,
        time: u64,
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let holder_info = next_account_info(accounts)?;
        let token_account_info = next_account_info(accounts)?;
        let auction_order_account_info = next_account_info(accounts)?;
        let token_program = next_account_info(accounts)?;
        let metadata_account = next_account_info(accounts)?;
        let metadata = Metadata::from_account_info(metadata_account)?;
        let mut found = 0;
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        if metadata.collection.is_some() || metadata.data.creators.is_some() {
            if let Some(collection) = metadata.collection {
                if collection.verified == true {
                    found += 1;
                }
            }
            if let Some(creators) = metadata.data.creators {
                if creators[0].verified == true {
                    found += 1;
                }
            }
        } else {
            return Err(MarketError::InvalidInstruction.into());
        }
        if found == 0 {
            return Err(MarketError::UnverifiedNFT.into());
        }
        let mut auction_order_struct: AuctionOrderSol =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        if auction_order_struct.is_initialized == true {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        auction_order_struct.owner_wallet_address = *holder_info.key;
        auction_order_struct.token_account = *token_account_info.key;
        if Clock::get()?.unix_timestamp as u64 > time {
            return Err(MarketError::InvalidInstruction.into());
        }
        if (time - Clock::get()?.unix_timestamp as u64) < 604800 {
            auction_order_struct.time = time;
        } else {
            return Err(MarketError::MaxTimeLimit.into());
        }
        auction_order_struct.bid = 0;
        if SPLS::Account::unpack_unchecked(&mut token_account_info.data.borrow())?.amount != 1 {
            return Err(ProgramError::InsufficientFunds);
        }
        if minimum_price as f64 / 1000000000.00 <= 0.00 || holder_info.is_signer != true {
            return Err(MarketError::MinPrice.into());
        }
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on$0!"], &program_id);
        if let Err(error) = invoke(
            &SPLIX::set_authority(
                token_program.key,
                token_account_info.key,
                Some(&pda),
                SPLIX::AuthorityType::AccountOwner,
                holder_info.key,
                &[holder_info.key],
            )?,
            &[
                token_program.clone(),
                token_account_info.clone(),
                holder_info.clone(),
            ],
        ) {
            return Err(error);
        }
        auction_order_struct.minimum_price = minimum_price;
        auction_order_struct.total_bid_amount = 0;
        auction_order_struct
            .serialize(&mut &mut auction_order_account_info.data.borrow_mut()[..])?;
        Ok(())
    }
    fn place_bid_sol(program_id: Pubkey, account_info: &[AccountInfo], bid: u64) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let bidder_account_info = next_account_info(accounts)?;
        let auction_order_account_info = next_account_info(accounts)?;
        let previous_bidder = next_account_info(accounts)?;
        let sys_program_info = next_account_info(accounts)?;
        let pda_account_info = next_account_info(accounts)?;
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let mut auction_order_struct: AuctionOrderSol =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        if auction_order_struct.owner_wallet_address == *bidder_account_info.key {
            return Err(MarketError::OwnerCannotBid.into());
        }
        let bid_increment = ((auction_order_struct.total_bid_amount as f64 * 5.00) / 100.00) as u64;
        if bid < bid_increment + auction_order_struct.bid {
            return Err(MarketError::BidMustBeGreater.into());
        }
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on$0!"], &program_id);
        if auction_order_struct.time - Clock::get()?.unix_timestamp as u64 <= 120 {
            auction_order_struct.time += 120;
        }
        if &pda != pda_account_info.key {
            return Err(MarketError::PdaError.into());
        }
        if bid as f64 > auction_order_struct.minimum_price as f64
            && auction_order_struct.bid == 0
            && (Clock::get()?.unix_timestamp as u64) < auction_order_struct.time
        {
            if let Err(error) = invoke(
                &transfer(bidder_account_info.key, &pda, (bid as f64) as u64),
                &[
                    bidder_account_info.clone(),
                    pda_account_info.clone(),
                    sys_program_info.clone(),
                ],
            ) {
                return Err(error);
            }
            auction_order_struct.bidder_wallet_address = *bidder_account_info.key;
            auction_order_struct.bid = bid;
            auction_order_struct.total_bid_amount = bid;
        } else if bid > auction_order_struct.bid
            && (Clock::get()?.unix_timestamp as u64) < auction_order_struct.time
            && auction_order_struct.minimum_price < bid
        {
            if *previous_bidder.key != auction_order_struct.bidder_wallet_address {
                return Err(ProgramError::IllegalOwner);
            }

            if let Err(error) = invoke(
                &transfer(bidder_account_info.key, &pda, bid),
                &[
                    bidder_account_info.clone(),
                    pda_account_info.clone(),
                    sys_program_info.clone(),
                ],
            ) {
                return Err(error);
            }
            //refunding previous bidder
            if let Err(error) = invoke_signed(
                &transfer(&pda, previous_bidder.key, auction_order_struct.bid),
                &[
                    pda_account_info.clone(),
                    previous_bidder.clone(),
                    sys_program_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
            ) {
                return Err(error);
            }

            auction_order_struct.bidder_wallet_address = *bidder_account_info.key;
            auction_order_struct.bid = bid;
            auction_order_struct.total_bid_amount = auction_order_struct.total_bid_amount + bid;
        } else {
            return Err(MarketError::BidMustBeGreater.into());
        }

        auction_order_struct
            .serialize(&mut &mut auction_order_account_info.data.borrow_mut()[..])?;
        Ok(())
    }
    fn complete_auction_order_sol(
        program_id: Pubkey,
        account_info: &[AccountInfo],
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let holder_info = next_account_info(accounts)?; //cat_king
        let bidder_info = next_account_info(accounts)?; //get from sellerOrder Account in web3 will claim themselves
        let auction_order_account_info = next_account_info(accounts)?;
        let sell_token_account_info = next_account_info(accounts)?; //from sell order Account in web3
        let sell_mint_account_info = next_account_info(accounts)?; //from sell order Account in web3
        let sell_token_new_account_info = next_account_info(accounts)?; //from sell order Account in web3
        let pda_account_info = next_account_info(accounts)?;
        let sys_program_info = next_account_info(accounts)?;
        let token_program = next_account_info(accounts)?;
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let auction_order_struct: AuctionOrderSol =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        if *holder_info.key != auction_order_struct.owner_wallet_address
            || *sell_token_account_info.key != auction_order_struct.token_account
            || *bidder_info.key != auction_order_struct.bidder_wallet_address
            || bidder_info.is_signer != true
            || auction_order_struct.bid == 0
        {
            return Err(MarketError::ValueMisMatch.into());
        }

        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on$0!"], &program_id);
        if SPLS::Account::unpack_unchecked(&mut sell_token_new_account_info.data.borrow())?.owner
            != *bidder_info.key
        {
            return Err(MarketError::WrongOwner.into());
        }
        if *pda_account_info.key != pda {
            return Err(MarketError::PdaError.into());
        }
        if (Clock::get()?.unix_timestamp as u64) > auction_order_struct.time
            && auction_order_struct.bid != 0
        {
            if let Err(error) = invoke_signed(
                &transfer(&pda, holder_info.key, auction_order_struct.bid),
                &[
                    sys_program_info.clone(),
                    pda_account_info.clone(),
                    holder_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if let Err(error) = invoke_signed(
                &SPLIX::transfer_checked(
                    token_program.key,
                    sell_token_account_info.key,
                    sell_mint_account_info.key,
                    sell_token_new_account_info.key,
                    &pda,
                    &[&pda],
                    1,
                    0,
                )?,
                &[
                    sell_token_account_info.clone(),
                    sell_mint_account_info.clone(),
                    sell_token_new_account_info.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if let Err(error) = invoke_signed(
                &SPLIX::close_account(
                    token_program.key,
                    sell_token_account_info.key,
                    holder_info.key,
                    &pda,
                    &[&pda],
                )?,
                &[
                    sell_token_account_info.clone(),
                    holder_info.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            **holder_info.try_borrow_mut_lamports()? = holder_info
                .lamports()
                .checked_add(auction_order_account_info.lamports())
                .ok_or(ProgramError::InsufficientFunds)?;
            **auction_order_account_info.try_borrow_mut_lamports()? = 0;
            *auction_order_account_info.try_borrow_mut_data()? = &mut [];
        } else {
            return Err(MarketError::AuctionNotEnded.into());
        }
        Ok(())
    }
    fn complete_auction_order_sol_anytime(
        program_id: Pubkey,
        account_info: &[AccountInfo],
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let holder_info = next_account_info(accounts)?;
        let bidder_info = next_account_info(accounts)?; //get from sellerOrder Account in web3
        let auction_order_account_info = next_account_info(accounts)?;
        let sell_token_account_info = next_account_info(accounts)?; //from sell order Account in web3
        let sell_mint_account_info = next_account_info(accounts)?; //from sell order Account in web3
        let sell_token_new_account_info = next_account_info(accounts)?; //from sell order Account in web3
        let pda_account_info = next_account_info(accounts)?;
        let sys_program_info = next_account_info(accounts)?;
        let token_program = next_account_info(accounts)?;
        let cat_king = next_account_info(accounts)?; //2 percentages
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let auction_order_struct: AuctionOrderSol =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        if *holder_info.key != auction_order_struct.owner_wallet_address
            || *sell_token_account_info.key != auction_order_struct.token_account
            || *bidder_info.key != auction_order_struct.bidder_wallet_address
            || holder_info.is_signer != true
            || auction_order_struct.bid == 0
        {
            return Err(MarketError::ValueMisMatch.into());
        }
        if cat_king.key
            != &Pubkey::from_str("5XJKsYXoLUSPh5KwdhecAyACLZujGKJ7z6ovQBznWKtq").unwrap()
        {
            return Err(MarketError::WrongOwner.into());
        }
        if SPLS::Account::unpack_unchecked(&mut sell_token_new_account_info.data.borrow())?.owner
            != *bidder_info.key
        {
            return Err(MarketError::WrongOwner.into());
        }
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on$0!"], &program_id);

        if *pda_account_info.key != pda {
            return Err(MarketError::PdaError.into());
        }
        if auction_order_struct.bid == 0 {
            return Err(MarketError::InvalidInstruction.into());
        }
        if let Err(error) = invoke_signed(
            &transfer(
                &pda,
                holder_info.key,
                ((auction_order_struct.bid as f64 * 97.5) / 100.00) as u64,
            ),
            &[
                sys_program_info.clone(),
                pda_account_info.clone(),
                holder_info.clone(),
            ],
            &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
        ) {
            return Err(error);
        }
        if let Err(error) = invoke_signed(
            &transfer(
                &pda,
                cat_king.key,
                ((auction_order_struct.bid as f64 * 2.5) / 100.00) as u64,
            ),
            &[
                sys_program_info.clone(),
                pda_account_info.clone(),
                cat_king.clone(),
            ],
            &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
        ) {
            return Err(error);
        }
        if let Err(error) = invoke_signed(
            &SPLIX::transfer_checked(
                token_program.key,
                sell_token_account_info.key,
                sell_mint_account_info.key,
                sell_token_new_account_info.key,
                &pda,
                &[&pda],
                1,
                0,
            )?,
            &[
                sell_token_account_info.clone(),
                sell_mint_account_info.clone(),
                sell_token_new_account_info.clone(),
                pda_account_info.clone(),
            ],
            &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
        ) {
            return Err(error);
        }
        if let Err(error) = invoke_signed(
            &SPLIX::close_account(
                token_program.key,
                sell_token_account_info.key,
                holder_info.key,
                &pda,
                &[&pda],
            )?,
            &[
                sell_token_account_info.clone(),
                holder_info.clone(),
                pda_account_info.clone(),
            ],
            &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
        ) {
            return Err(error);
        }
        **holder_info.try_borrow_mut_lamports()? = holder_info
            .lamports()
            .checked_add(auction_order_account_info.lamports())
            .ok_or(ProgramError::InsufficientFunds)?;
        **auction_order_account_info.try_borrow_mut_lamports()? = 0;
        *auction_order_account_info.try_borrow_mut_data()? = &mut [];
        Ok(())
    }
    fn complete_auction_order_sol_user(
        program_id: Pubkey,
        account_info: &[AccountInfo],
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let holder_info = next_account_info(accounts)?;
        let bidder_info = next_account_info(accounts)?; //get from sellerOrder Account in web3
        let auction_order_account_info = next_account_info(accounts)?;
        let sell_token_account_info = next_account_info(accounts)?; //from sell order Account in web3
        let sell_mint_account_info = next_account_info(accounts)?; //from sell order Account in web3
        let sell_token_new_account_info = next_account_info(accounts)?; //from sell order Account in web3
        let pda_account_info = next_account_info(accounts)?;
        let sys_program_info = next_account_info(accounts)?;
        let token_program = next_account_info(accounts)?;
        let cat_king = next_account_info(accounts)?; //2 percentages
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let auction_order_struct: AuctionOrderSol =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        if *holder_info.key != auction_order_struct.owner_wallet_address
            || *sell_token_account_info.key != auction_order_struct.token_account
            || *bidder_info.key != auction_order_struct.bidder_wallet_address
            || bidder_info.is_signer != true
            || auction_order_struct.bid == 0
        {
            return Err(MarketError::ValueMisMatch.into());
        }
        if cat_king.key
            != &Pubkey::from_str("5XJKsYXoLUSPh5KwdhecAyACLZujGKJ7z6ovQBznWKtq").unwrap()
        {
            return Err(MarketError::WrongOwner.into());
        }
        if SPLS::Account::unpack_unchecked(&mut sell_token_new_account_info.data.borrow())?.owner
            != *bidder_info.key
        {
            return Err(MarketError::WrongOwner.into());
        }

        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on$0!"], &program_id);

        if *pda_account_info.key != pda {
            return Err(MarketError::PdaError.into());
        }
        if (Clock::get()?.unix_timestamp as u64) > auction_order_struct.time
            && auction_order_struct.bid != 0
        {
            if let Err(error) = invoke_signed(
                &transfer(
                    &pda,
                    holder_info.key,
                    ((auction_order_struct.bid as f64 * 97.5) / 100.00) as u64,
                ),
                &[
                    sys_program_info.clone(),
                    pda_account_info.clone(),
                    holder_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if let Err(error) = invoke_signed(
                &transfer(
                    &pda,
                    cat_king.key,
                    ((auction_order_struct.bid as f64 * 2.5) / 100.00) as u64,
                ),
                &[
                    sys_program_info.clone(),
                    pda_account_info.clone(),
                    cat_king.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if let Err(error) = invoke_signed(
                &SPLIX::transfer_checked(
                    token_program.key,
                    sell_token_account_info.key,
                    sell_mint_account_info.key,
                    sell_token_new_account_info.key,
                    &pda,
                    &[&pda],
                    1,
                    0,
                )?,
                &[
                    sell_token_account_info.clone(),
                    sell_mint_account_info.clone(),
                    sell_token_new_account_info.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if let Err(error) = invoke_signed(
                &SPLIX::close_account(
                    token_program.key,
                    sell_token_account_info.key,
                    holder_info.key,
                    &pda,
                    &[&pda],
                )?,
                &[
                    sell_token_account_info.clone(),
                    holder_info.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            **holder_info.try_borrow_mut_lamports()? = holder_info
                .lamports()
                .checked_add(auction_order_account_info.lamports())
                .ok_or(ProgramError::InsufficientFunds)?;
            **auction_order_account_info.try_borrow_mut_lamports()? = 0;
            *auction_order_account_info.try_borrow_mut_data()? = &mut [];
        } else {
            return Err(MarketError::AuctionNotEnded.into());
        }
        Ok(())
    }
    fn cancel_auction_sol(program_id: Pubkey, account_info: &[AccountInfo]) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let payer_info = next_account_info(accounts)?;
        let token_account_info = next_account_info(accounts)?;
        let auction_order_account_info = next_account_info(accounts)?;
        let token_program = next_account_info(accounts)?;
        let pda_account_info = next_account_info(accounts)?;
        let sys_program_info = next_account_info(accounts)?;
        let previous_bidder = next_account_info(accounts)?;
        if *auction_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let auction_order_struct: AuctionOrderSol =
            BorshDeserialize::try_from_slice(&mut auction_order_account_info.data.borrow())?;
        if *payer_info.key != auction_order_struct.owner_wallet_address
            || *token_account_info.key != auction_order_struct.token_account
            || *previous_bidder.key != auction_order_struct.bidder_wallet_address
            || payer_info.is_signer != true
        {
            return Err(ProgramError::IllegalOwner.into());
        }
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on$0!"], &program_id);

        if *pda_account_info.key != pda {
            return Err(MarketError::PdaError.into());
        }
        // 16                            15
        if Clock::get()?.unix_timestamp as u64 > auction_order_struct.time {
            if let Err(error) = invoke_signed(
                &SPLIX::set_authority(
                    token_program.key,
                    token_account_info.key,
                    Some(&payer_info.key),
                    SPLIX::AuthorityType::AccountOwner,
                    &pda,
                    &[&pda],
                )?,
                &[
                    token_program.clone(),
                    token_account_info.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if auction_order_struct.bid != 0 {
                //     if let Err(error) = invoke_signed(
                //         &transfer(&pda, bidder_info.key, auction_order_struct.bid),
                //         &[
                //             pda_account_info.clone(),
                //             bidder_info.clone(),
                //             sys_program_info.clone(),
                //         ],
                //         &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
                //     ) {
                //         return Err(error);
                //     }
                // }
                if let Err(error) = invoke_signed(
                    &transfer(&pda, previous_bidder.key, auction_order_struct.bid),
                    &[
                        pda_account_info.clone(),
                        previous_bidder.clone(),
                        sys_program_info.clone(),
                    ],
                    &[&[&b"C@tC@rte!R@ffle$&Auct!on$0!"[..], &[_nonce]]],
                ) {
                    return Err(error);
                }
                **payer_info.try_borrow_mut_lamports()? = payer_info
                    .lamports()
                    .checked_add(auction_order_account_info.lamports())
                    .ok_or(ProgramError::InsufficientFunds)?;
                **auction_order_account_info.try_borrow_mut_lamports()? = 0;
                *auction_order_account_info.try_borrow_mut_data()? = &mut [];
            }
        } else {
            return Err(MarketError::CannotCancel.into());
        }

        Ok(())
    }
    //RAFFLES
    fn raffle_start(
        program_id: Pubkey,
        account_info: &[AccountInfo],
        time: u64,
        price: u64,
        total_ticket: u64,
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let holder_info = next_account_info(accounts)?;
        let token_account_info = next_account_info(accounts)?;
        let raffle_order_account_info = next_account_info(accounts)?;
        let token_program = next_account_info(accounts)?; // token program
        let token_type = next_account_info(accounts)?; //
        let mut raffle_order_struct: RaffleOrder =
            try_from_slice_unchecked(&mut raffle_order_account_info.data.borrow())?;
        if raffle_order_struct.is_initialized == true {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        if *raffle_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        if account_info.len() == 6 {
            let feature_raffle_account = next_account_info(accounts)?; //
            if *feature_raffle_account.owner != program_id {
                return Err(ProgramError::IncorrectProgramId);
            }
            let mut feature_account_data: FeaturedRaffles =
                BorshDeserialize::try_from_slice(&mut feature_raffle_account.data.borrow())?;
            if feature_account_data.is_initialized == true {
                return Err(ProgramError::AccountAlreadyInitialized);
            }
            feature_account_data.is_initialized = true;
            feature_account_data.is_featured = true;
            feature_account_data.raffle_account = *raffle_order_account_info.key;
            feature_account_data
                .serialize(&mut &mut feature_raffle_account.data.borrow_mut()[..])?;
        }
        if SPLS::Account::unpack_unchecked(&mut token_account_info.data.borrow())?.amount != 1 {
            return Err(ProgramError::InsufficientFunds);
        }
        if price == 0 || holder_info.is_signer != true {
            return Err(MarketError::MinPrice.into());
        }
        raffle_order_struct.is_initialized = true;
        raffle_order_struct.owner_wallet_address = *holder_info.key;
        raffle_order_struct.token_account = *token_account_info.key;
        raffle_order_struct.time = time;
        raffle_order_struct.price = price;
        raffle_order_struct.token_type = *token_type.key;
        raffle_order_struct.ticket_supply = total_ticket;
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on"], &program_id);
        if let Err(error) = invoke(
            &SPLIX::set_authority(
                token_program.key,
                token_account_info.key,
                Some(&pda),
                SPLIX::AuthorityType::AccountOwner,
                holder_info.key,
                &[holder_info.key],
            )?,
            &[
                token_program.clone(),
                token_account_info.clone(),
                holder_info.clone(),
            ],
        ) {
            return Err(error);
        }
        raffle_order_struct.serialize(&mut &mut raffle_order_account_info.data.borrow_mut()[..])?;
        Ok(())
    }
    fn make_raffle_entry(
        program_id: Pubkey,
        account_info: &[AccountInfo],
        amount: u64,
        quantity: u8,
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let raffler_info = next_account_info(accounts)?; //cat king wallet
        let raffle_order_account_info = next_account_info(accounts)?; // auction data account
        let mut raffle_struct: RaffleOrder =
            try_from_slice_unchecked(&mut raffle_order_account_info.data.borrow())?;
        let mut exist = false;
        if *raffler_info.key == raffle_struct.owner_wallet_address {
            return Err(MarketError::OwnerCannotBid.into());
        }
        if *raffle_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        if raffle_struct.raffle_entry_record.is_empty() == true {
            if quantity as u64 <= raffle_struct.ticket_supply {
                raffle_struct.raffle_entry_record.push(RaffleCounter {
                    raffler_address: *raffler_info.key,
                    entry_counter: quantity as u32,
                });
                exist = true;
            } else {
                return Err(MarketError::ValueMisMatch.into());
            }
        } else {
            for i in 0..raffle_struct.raffle_entry_record.len() {
                if *raffler_info.key == raffle_struct.raffle_entry_record[i].raffler_address {
                    exist = true;
                    raffle_struct.raffle_entry_record[i].entry_counter += quantity as u32;
                    break;
                }
            }
        }
        if exist == false {
            raffle_struct.raffle_entry_record.push(RaffleCounter {
                raffler_address: *raffler_info.key,
                entry_counter: quantity as u32,
            });
        }
        let mut total = 0;
        for i in 0..raffle_struct.raffle_entry_record.len() {
            let temp = raffle_struct.raffle_entry_record[i].entry_counter;
            total = total + temp;
        }
        if total as u64 > raffle_struct.ticket_supply {
            return Err(MarketError::ValueMisMatch.into());
        }
        if amount != raffle_struct.price * quantity as u64 {
            return Err(MarketError::ValueMisMatch.into());
        }
        if raffle_struct.time > Clock::get()?.unix_timestamp as u64
            && raffle_struct.token_type
                != Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap()
        {
            let raffler_zion_token_account_info = next_account_info(accounts)?;
            let zion_mint_account_info = next_account_info(accounts)?;
            let cat_king_zion_token_account = next_account_info(accounts)?;
            let token_program = next_account_info(accounts)?; // token program
            let client_zion_token_account_info = next_account_info(accounts)?; //ppublic owner

            let spl_accounts = &[
                raffler_info.clone(),
                raffler_zion_token_account_info.clone(),
                zion_mint_account_info.clone(),
                cat_king_zion_token_account.clone(),
                token_program.clone(),
                client_zion_token_account_info.clone(),
            ];
            if let Err(error) =
                Self::handle_spl_tokens(spl_accounts, raffle_struct.owner_wallet_address, amount)
            {
                return Err(error);
            }
        } else if raffle_struct.token_type
            == Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap()
        {
            let cat_king_wallet_account_info = next_account_info(accounts)?;
            let sys_program_info = next_account_info(accounts)?;
            let rafflee_info = next_account_info(accounts)?;
            let spl_accounts = &[
                raffler_info.clone(),
                cat_king_wallet_account_info.clone(),
                sys_program_info.clone(),
                rafflee_info.clone(),
            ];

            if let Err(error) =
                Self::handle_sol(spl_accounts, raffle_struct.owner_wallet_address, amount)
            {
                return Err(error);
            }
        } else {
            return Err(MarketError::ValueMisMatch.into());
        }
        raffle_struct.serialize(&mut &mut raffle_order_account_info.data.borrow_mut()[..])?;
        Ok(())
    }
    fn end_raffle(program_id: Pubkey, account_info: &[AccountInfo]) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let cat_king = next_account_info(accounts)?;
        let raffler_info = next_account_info(accounts)?; //get from sellerOrder Account in web3 claiming k liyey
        let raffle_order_account_info = next_account_info(accounts)?; //fetch data and matach with web3
        let raffle_nft_token_account_info = next_account_info(accounts)?; //from auciton order Account in web3
        let _raffle_nft_mint = next_account_info(accounts)?;
        let _pda_account_info = next_account_info(accounts)?; //which holder the authority for NFT on Auction
        let _raffle_nft_new_token_account = next_account_info(accounts)?; // new token account of user to send nft to
        let _token_program = next_account_info(accounts)?;
        if *raffle_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let raffle_struct: RaffleOrder =
            try_from_slice_unchecked(&mut raffle_order_account_info.data.borrow())?;

        let mut exist = false;
        for i in 0..raffle_struct.raffle_entry_record.len() {
            if *raffler_info.key == raffle_struct.raffle_entry_record[i].raffler_address {
                exist = true;
                break;
            }
        }
        if account_info.len() == 10 || account_info.len() == 11 {
            let admin = next_account_info(accounts)?;
            let _system_account = next_account_info(accounts)?;
            if *admin.key
                == Pubkey::from_str("5XJKsYXoLUSPh5KwdhecAyACLZujGKJ7z6ovQBznWKtq").unwrap()
                && admin.is_signer == true
            {
                if *cat_king.key == raffle_struct.owner_wallet_address
                    && *raffle_nft_token_account_info.key == raffle_struct.token_account
                // && cat_king.is_signer == true
                {
                    if let Err(error) =
                        Self::transfer_to_winner_raffle(program_id, account_info, exist)
                    {
                        return Err(error);
                    }
                } else {
                    return Err(MarketError::ValueMisMatch.into());
                }
            } else {
                return Err(MarketError::WrongOwner.into());
            }
            if account_info.len() == 11 {
                let feature_account = next_account_info(accounts)?;
                **cat_king.try_borrow_mut_lamports()? = cat_king
                .lamports()
                .checked_add(feature_account.lamports())
                .ok_or(ProgramError::InsufficientFunds)?;
            **feature_account.try_borrow_mut_lamports()? = 0;
            *feature_account.try_borrow_mut_data()? = &mut [];
            }
            
        } else {
            if *cat_king.key == raffle_struct.owner_wallet_address
                && *raffle_nft_token_account_info.key == raffle_struct.token_account
                && cat_king.is_signer == true
            {
                if let Err(error) = Self::transfer_to_winner_raffle(program_id, account_info, exist)
                {
                    return Err(error);
                }
            } else {
                return Err(MarketError::ValueMisMatch.into());
            }
            if account_info.len() == 9 {
                let feature_account = next_account_info(accounts)?;
                **cat_king.try_borrow_mut_lamports()? = cat_king
                .lamports()
                .checked_add(feature_account.lamports())
                .ok_or(ProgramError::InsufficientFunds)?;
            **feature_account.try_borrow_mut_lamports()? = 0;
            *feature_account.try_borrow_mut_data()? = &mut [];
            }    
        }
        

        Ok(())
    }
    fn transfer_to_winner_raffle(
        program_id: Pubkey,
        account_info: &[AccountInfo],
        exist: bool,
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let cat_king = next_account_info(accounts)?;
        let _raffler_info = next_account_info(accounts)?; //get from sellerOrder Account in web3 claiming k liyey
        let raffle_order_account_info = next_account_info(accounts)?; //fetch data and matach with web3
        let raffle_nft_token_account_info = next_account_info(accounts)?; //from auciton order Account in web3
        let raffle_nft_mint = next_account_info(accounts)?;
        let pda_account_info = next_account_info(accounts)?; //which holder the authority for NFT on Auction
        let raffle_nft_new_token_account = next_account_info(accounts)?; // new token account of user to send nft to
        let token_program = next_account_info(accounts)?;
        if *raffle_order_account_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on"], &program_id);
        let raffle_struct: RaffleOrder =
            try_from_slice_unchecked(&mut raffle_order_account_info.data.borrow())?;
        if Clock::get()?.unix_timestamp as u64 > raffle_struct.time && exist == true {
            if let Err(error) = invoke_signed(
                &SPLIX::transfer_checked(
                    token_program.key,
                    raffle_nft_token_account_info.key,
                    raffle_nft_mint.key,
                    raffle_nft_new_token_account.key,
                    &pda,
                    &[&pda],
                    1,
                    0,
                )?,
                &[
                    raffle_nft_token_account_info.clone(),
                    raffle_nft_mint.clone(),
                    raffle_nft_new_token_account.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if let Err(error) = invoke_signed(
                &SPLIX::close_account(
                    token_program.key,
                    raffle_nft_token_account_info.key,
                    cat_king.key,
                    &pda,
                    &[&pda],
                )?,
                &[
                    raffle_nft_token_account_info.clone(),
                    cat_king.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            **cat_king.try_borrow_mut_lamports()? = cat_king
                .lamports()
                .checked_add(raffle_order_account_info.lamports())
                .ok_or(ProgramError::InsufficientFunds)?;
            **raffle_order_account_info.try_borrow_mut_lamports()? = 0;
            *raffle_order_account_info.try_borrow_mut_data()? = &mut [];
        } else if Clock::get()?.unix_timestamp as u64 > raffle_struct.time
            && raffle_struct.raffle_entry_record.len() == 0
        {
            let (pda, _nonce) =
                Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on"], &program_id);
            if let Err(error) = invoke_signed(
                &SPLIX::set_authority(
                    token_program.key,
                    raffle_nft_token_account_info.key,
                    Some(&cat_king.key),
                    SPLIX::AuthorityType::AccountOwner,
                    &pda,
                    &[&pda],
                )?,
                &[
                    token_program.clone(),
                    raffle_nft_token_account_info.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            **cat_king.try_borrow_mut_lamports()? = cat_king
                .lamports()
                .checked_add(raffle_order_account_info.lamports())
                .ok_or(ProgramError::InsufficientFunds)?;
            **raffle_order_account_info.try_borrow_mut_lamports()? = 0;
            *raffle_order_account_info.try_borrow_mut_data()? = &mut [];
        } else {
            return Err(MarketError::ValueMisMatch.into());
        }

        Ok(())
    }
    fn handle_sol(
        account_info: &[AccountInfo],
        owner_wallet_address: Pubkey,
        amount: u64,
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let raffler_info = next_account_info(accounts)?; //cat king wallet
        let cat_king_wallet_account_info = next_account_info(accounts)?;
        let sys_program_info = next_account_info(accounts)?;
        let rafflee_info = next_account_info(accounts)?;

        if owner_wallet_address
            == Pubkey::from_str("5XJKsYXoLUSPh5KwdhecAyACLZujGKJ7z6ovQBznWKtq").unwrap()
        {
            if let Err(error) = invoke(
                &transfer(&raffler_info.key, cat_king_wallet_account_info.key, amount),
                &[
                    sys_program_info.clone(),
                    raffler_info.clone(),
                    cat_king_wallet_account_info.clone(),
                ],
            ) {
                return Err(error);
            }
        } else {
            if *cat_king_wallet_account_info.key
                != Pubkey::from_str("5XJKsYXoLUSPh5KwdhecAyACLZujGKJ7z6ovQBznWKtq").unwrap()
            {
                return Err(MarketError::WrongOwner.into());
            }
            if let Err(error) = invoke(
                &transfer(
                    &raffler_info.key,
                    rafflee_info.key,
                    (amount as f64 * 97.5 / 100.00) as u64,
                ),
                &[
                    sys_program_info.clone(),
                    raffler_info.clone(),
                    rafflee_info.clone(),
                ],
            ) {
                return Err(error);
            }

            if let Err(error) = invoke(
                &transfer(
                    &raffler_info.key,
                    cat_king_wallet_account_info.key,
                    (amount as f64 * 2.5 / 100.00) as u64,
                ),
                &[
                    sys_program_info.clone(),
                    raffler_info.clone(),
                    cat_king_wallet_account_info.clone(),
                ],
            ) {
                return Err(error);
            }
        }
        Ok(())
    }

    fn handle_spl_tokens(
        account_info: &[AccountInfo],
        owner_wallet_address: Pubkey,
        amount: u64,
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let raffler_info = next_account_info(accounts)?; //cat king wallet
        let raffler_zion_token_account_info = next_account_info(accounts)?;
        let zion_mint_account_info = next_account_info(accounts)?;
        let cat_king_zion_token_account = next_account_info(accounts)?;
        let token_program = next_account_info(accounts)?; // token program
        let client_zion_token_account_info = next_account_info(accounts)?; //ppublic owner
        if SPLS::Account::unpack_unchecked(&mut cat_king_zion_token_account.data.borrow())?.owner
            != Pubkey::from_str("5XJKsYXoLUSPh5KwdhecAyACLZujGKJ7z6ovQBznWKtq").unwrap()
        {
            return Err(MarketError::WrongOwner.into());
        }
        if owner_wallet_address
            == Pubkey::from_str("5XJKsYXoLUSPh5KwdhecAyACLZujGKJ7z6ovQBznWKtq").unwrap()
        {
            if let Err(error) = invoke(
                &SPLIX::transfer(
                    token_program.key,
                    raffler_zion_token_account_info.key,
                    cat_king_zion_token_account.key,
                    raffler_info.key,
                    &[raffler_info.key],
                    amount,
                )?,
                &[
                    raffler_zion_token_account_info.clone(),
                    zion_mint_account_info.clone(),
                    cat_king_zion_token_account.clone(),
                    raffler_info.clone(),
                ],
            ) {
                return Err(error);
            }
        } else {
            if SPLS::Account::unpack_unchecked(&mut client_zion_token_account_info.data.borrow())?
                .owner
                != owner_wallet_address
            {
                return Err(MarketError::WrongOwner.into());
            }
            if let Err(error) = invoke(
                &SPLIX::transfer(
                    token_program.key,
                    raffler_zion_token_account_info.key,
                    client_zion_token_account_info.key,
                    raffler_info.key,
                    &[raffler_info.key],
                    (amount as f64 * 97.5 / 100.00) as u64,
                )?,
                &[
                    raffler_zion_token_account_info.clone(),
                    zion_mint_account_info.clone(),
                    client_zion_token_account_info.clone(),
                    raffler_info.clone(),
                ],
            ) {
                return Err(error);
            }
            if let Err(error) = invoke(
                &SPLIX::transfer(
                    token_program.key,
                    raffler_zion_token_account_info.key,
                    cat_king_zion_token_account.key,
                    raffler_info.key,
                    &[raffler_info.key],
                    (amount as f64 * 2.5 / 100.00) as u64,
                )?,
                &[
                    raffler_zion_token_account_info.clone(),
                    zion_mint_account_info.clone(),
                    cat_king_zion_token_account.clone(),
                    raffler_info.clone(),
                ],
            ) {
                return Err(error);
            }
        }
        Ok(())
    }
    fn handle_raffle_non_transfers(
        program_id: Pubkey,
        account_info: &[AccountInfo],
    ) -> ProgramResult {
        let accounts = &mut account_info.iter();
        let cat_king = next_account_info(accounts)?; // signer
        let raffler_info = next_account_info(accounts)?; //get from sellerOrder Account in web3 claiming k liyey
        let raffle_order_account_info = next_account_info(accounts)?; //fetch data and matach with web3
        let raffle_nft_token_account_info = next_account_info(accounts)?; //from auciton order Account in web3
        let raffle_nft_mint = next_account_info(accounts)?;
        let pda_account_info = next_account_info(accounts)?; //which holder the authority for NFT on Auction
        let raffle_nft_new_token_account = next_account_info(accounts)?; // new token account of user to send nft to
        let token_program = next_account_info(accounts)?;
        let (pda, _nonce) =
            Pubkey::find_program_address(&[b"C@tC@rte!R@ffle$&Auct!on"], &program_id);
        let raffle_struct: RaffleOrder =
            try_from_slice_unchecked(&mut raffle_order_account_info.data.borrow())?;
        if cat_king.is_signer != true {
            return Err(MarketError::WrongOwner.into());
        }
        if cat_king.key
            != &Pubkey::from_str("5XJKsYXoLUSPh5KwdhecAyACLZujGKJ7z6ovQBznWKtq").unwrap()
        {
            return Err(MarketError::WrongOwner.into());
        }
        // if *cat_king.key != raffle_struct.owner_wallet_address
        if *raffle_nft_token_account_info.key != raffle_struct.token_account {
            return Err(MarketError::ValueMisMatch.into());
        }
        let mut exist = false;

        for i in 0..raffle_struct.raffle_entry_record.len() {
            if *raffler_info.key == raffle_struct.raffle_entry_record[i].raffler_address {
                exist = true;
                break;
            }
        }
        if exist == true
            && SPLS::Account::unpack_unchecked(&mut raffle_nft_new_token_account.data.borrow())?
                .owner
                == *raffler_info.key
            && Clock::get()?.unix_timestamp as u64 > raffle_struct.time
        {
            if let Err(error) = invoke_signed(
                &SPLIX::transfer_checked(
                    token_program.key,
                    raffle_nft_token_account_info.key,
                    raffle_nft_mint.key,
                    raffle_nft_new_token_account.key,
                    &pda,
                    &[&pda],
                    1,
                    0,
                )?,
                &[
                    raffle_nft_token_account_info.clone(),
                    raffle_nft_mint.clone(),
                    raffle_nft_new_token_account.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            if let Err(error) = invoke_signed(
                &SPLIX::close_account(
                    token_program.key,
                    raffle_nft_token_account_info.key,
                    cat_king.key,
                    &pda,
                    &[&pda],
                )?,
                &[
                    raffle_nft_token_account_info.clone(),
                    cat_king.clone(),
                    pda_account_info.clone(),
                ],
                &[&[&b"C@tC@rte!R@ffle$&Auct!on"[..], &[_nonce]]],
            ) {
                return Err(error);
            }
            **cat_king.try_borrow_mut_lamports()? = cat_king
                .lamports()
                .checked_add(raffle_order_account_info.lamports())
                .ok_or(ProgramError::InsufficientFunds)?;
            **raffle_order_account_info.try_borrow_mut_lamports()? = 0;
            *raffle_order_account_info.try_borrow_mut_data()? = &mut [];
        }
        Ok(())
    }
    //sell functions for
}





    



