


use {
    solana_program::{program_error::ProgramError},
    spl_token::error::TokenError::InvalidInstruction,
    std::convert::TryInto,
    
};
pub enum MarketplaceInstruction {
    AuctionStart {
        minimum_price: u64,
        time: u64,
        
    },
    PlaceBid {
        new_bid: u64,
    },
    CompleteAuction,
    CompleteAuctionUserZion,

    
    CompleteAuctionAnyTime,

    CanceAuction,
    RaffleStart{
        price: u64,
        time: u64,
    },
    MakeRaffleEntry{
        amount:u64,
        quantity:u8
    },
    EndRaffle,
    AuctionStartSol {
        minimum_price: u64,
        time: u64,
    },
    PlaceBidSol {
        new_bid: u64,
    },
    CompleteAuctionSol,
    CompleteAuctionUserSol,
    CompleteAuctionAnyTimeSol,
    CanceAuctionSol,
    
    HandleNonTransfer,
    

}
impl MarketplaceInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            4 => {
                let (minimum_price, rest) = rest.split_at(8);
                let minimum_price = minimum_price
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let (time, _rest) = rest.split_at(8);
                let time = time
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                    // let (_, _rest) = _rest.split_at(4);
                    // let name=str::from_utf8(_rest).unwrap();
                    // msg!("{:?},{:?}",name,name.len());
                    // let name=Self::puffed_out_string(&name.to_string(),4);
                    // msg!(" this is name{:?}",name);
                Self::AuctionStart {
                    minimum_price,
                    time,
                    
                }
            }
            5 => {
                let (new_bid, _rest) = rest.split_at(8);
                let new_bid = new_bid
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                Self::PlaceBid {
                    new_bid
                }
            },
            6=>Self::CompleteAuction,
            
            15=>Self::CompleteAuctionAnyTime,
            7=>Self::CanceAuction,
            9 => {
                let (minimum_price, rest) = rest.split_at(8);
                let price = minimum_price
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let (time, _rest) = rest.split_at(8);
                let time = time
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                Self::RaffleStart {
                    time,
                    price,
                }
            },
            11=>Self::EndRaffle,
            13=>{
                let (new_bid, _rest) = rest.split_at(8);
                let amount = new_bid
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                    let (&quantity, _) = _rest.split_first().ok_or(InvalidInstruction)?;
                Self::MakeRaffleEntry {
                    amount,
                    quantity,
                }
            },
            17 => {
                let (minimum_price, rest) = rest.split_at(8);
                let minimum_price = minimum_price
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                let (time, _rest) = rest.split_at(8);
                let time = time
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                Self::AuctionStartSol {
                    minimum_price,
                    time,
                }
            }
            19 => {
                let (new_bid, _rest) = rest.split_at(8);
                let new_bid = new_bid
                    .try_into()
                    .ok()
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                Self::PlaceBidSol {
                    new_bid
                }
            },
            21=>Self::CompleteAuctionSol,
            23=>Self::CanceAuctionSol,
            
            27=>Self::CompleteAuctionAnyTimeSol,
            29=>Self::CompleteAuctionUserZion,
            31=>Self::CompleteAuctionUserSol,
            32=>Self::HandleNonTransfer,
            
            
           
            _ => return Err(InvalidInstruction.into()),
        })
    }
    //  fn puffed_out_string(s: &String, size: usize) -> String {
    //     let mut array_of_zeroes = vec![];
    //     let puff_amount = size - s.len();
    //     while array_of_zeroes.len() < puff_amount {
    //         array_of_zeroes.push(0u8);
    //     }
    //     s.clone() + std::str::from_utf8(&array_of_zeroes).unwrap()
    // }
}
