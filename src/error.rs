use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum MarketError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not Rent Exempt
    #[error("Bid Should Be Greater Than Current One")]
    BidMustBeGreater,
    /// Expected Amount Mismatch
    #[error("No Creators Found")]
    NoCreator,
    /// Amount Overflow
    #[error("Value Incorrect")]
    ValueMisMatch,
    #[error("Invalid Id")]
    PdaError,
    #[error("Minimum Cannot be 0")]
    MinPrice,
    #[error("Not In Secondary Market")]
    PrimarySaleFalse,
    #[error("Max Time Limit is 7 days")]
    MaxTimeLimit,
    #[error("Auction Hasn't Ended Yet")]
    AuctionNotEnded,
    #[error("Owner is not allowed to bid")]
    OwnerCannotBid,
    #[error("Owner is not allowed to bid")]
    UnverifiedNFT,
    #[error("Owner Mismatch")]
    WrongOwner,
    #[error("Auction Not Ended")]
    CannotCancel,
    
}

impl From<MarketError> for ProgramError {
    fn from(e: MarketError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

