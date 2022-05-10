use anchor_lang::prelude::*;

// Store the Data of the  Deal Program
#[error_code]
pub enum ErrorCode {
    #[msg("Wallet to withdraw from is not owned by owner")]
    WalletToWithdrawFromInvalid,
    #[msg("State index is inconsistent")]
    InvalidStateIdx,
    #[msg("Delegate is not set correctly")]
    DelegateNotSetCorrectly,
    #[msg("Stage is invalid")]
    StageInvalid,
    #[msg("Member signer doesn't match the derived address.")]
    InvalidMemberSigner,
    #[msg("The nonce given doesn't derive a valid program address.")]
    InvalidNonce,
}
// 1 State account instance == 1 Safe Pay instance
#[account]
#[derive(Default)]
pub struct DealState {

    // A primary key that allows us to derive other important accounts
    pub idx: u64,
    
    // Alice 
    pub underwriter: Pubkey,

    // Bob
    pub borrower: Pubkey,

    // Charlies are a PDA of the Stakerstats account
    //pub lprovider: Pubkey, // 

    // The Mint of the token that Alice wants to send to Bob USDC
    pub mint_of_token_being_sent: Pubkey,

    // The Deal wallet 
    pub deal_wallet: Pubkey,
    // The amount of tokens Alice wants to place in the deal
    pub amount_tokens: u64,

    // An enumm that is to represent some kind of state machine
    pub epoch: u8,
}
impl DealState {
    pub const MAX_SIZE:  usize = (4 * 32) + 8 + 8 + 1;
}

#[derive(Clone, Copy, PartialEq)]
pub enum Epochs {
    // withdrew funds from Alice and deposited them into the escrow wallet
    DealWritten,

    // Period fr LP to provide liquidity ended 
    LPCompleted,

    // {from FundsDeposited} Bob withdrew the funds 
    DealComplete,

    Repaymentphase,  // up to  65555 repayment phase

    DealClosed,
    // {from FundsDeposited} Alice pulled back the funds
    PullBackComplete,
}


#[account]
#[derive(Default)]
pub struct LiqProvider {
    // The deal the user belong to 
    pub deal_state: Pubkey,
   

    pub deal_wallet: Pubkey,
    pub deal_mint:Pubkey,
   
    // Balance owned by the member
    
    // Charlie Account
    pub staker: Pubkey,
    //pub staker_wallet: Pubkey,
    pub balance:u64,

    pub last_stake_ts: i64,
    /// Signer nonce for PDA derivation 
    pub idx: u64,
    
}
impl LiqProvider {
    pub const MAX_SIZE:  usize = (4 * 32) + 8 + 8 + 8;
}

impl Epochs{
    pub  fn to_code(&self) -> u8 {
        match self {
            Epochs::DealWritten => 1,
            Epochs::LPCompleted => 2,
            Epochs::DealComplete => 3,
            Epochs::Repaymentphase => 4,
            Epochs::DealClosed => 5,
            Epochs::PullBackComplete => 6,
        }
    }

    fn from(val: u8) -> std::result::Result<Epochs, ProgramError> {
        match val {
            1 => Ok(Epochs::DealWritten),
            2 => Ok(Epochs::LPCompleted),
            3 => Ok(Epochs::DealComplete),
            4 => Ok(Epochs::Repaymentphase),
            5 => Ok(Epochs::DealClosed),
            6 => Ok(Epochs::PullBackComplete),
            unknown_value => {
                msg!("Unknown stage: {}", unknown_value);
                return Err(ProgramError::Custom(100));
                
            }
        }
    }
}