use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{Mint, TokenAccount,Token},
};
use spl_token::{self, solana_program::system_program};
use crate::*;
use state::*;
#[derive(Accounts)]
#[instruction(application_idx: u64, state_bump: u8, wallet_bump: u8)]
pub struct InitializeDeal<'info> {

     // Derived PDAs
     #[account(
        init,
        payer = underwriter,
        seeds=[b"state".as_ref(), underwriter.key().as_ref(), borrower.key.as_ref(), mint_of_token_being_sent.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        bump,
        space=  8 + DealState::MAX_SIZE // 
    )]
    pub deal_state: Account<'info, DealState>, // puppet
    #[account(
        init,
        payer = underwriter,
        seeds=[b"wallet".as_ref(), underwriter.key().as_ref(),  borrower.key.as_ref(), mint_of_token_being_sent.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        bump, //= wallet_bump,
        token::mint=mint_of_token_being_sent,
        token::authority=deal_state,
    )]
    pub deal_wallet_state: Account<'info, TokenAccount>,

    // Pda account of Alice 
    #[account(mut)]
    pub underwriter: Signer<'info>,                     // Alice
    // CHECK: no checks needed functionnal first secure after 
    // pub lprovider: AccountInfo<'info>,                        // Charlie
    /// CHECK: no checks needed functionnal first secure after 
    pub borrower: AccountInfo<'info>,                      // Bob
    pub mint_of_token_being_sent: Account<'info, Mint>,  // USDC

    // Alice's USDC wallet that has already approved the escrow wallet
    #[account(
        mut,
        constraint=wallet_to_withdraw_from.owner == underwriter.key(),
        constraint=wallet_to_withdraw_from.mint == mint_of_token_being_sent.key()
    )]
    pub wallet_to_withdraw_from: Account<'info, TokenAccount>,

    // Application level accounts
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(application_idx: u64, lp_bump: u8, wallet_bump: u8)]
pub struct CreateLP<'info> {
    // Stake instance.
    #[account(
        mut, 
        seeds=[b"state".as_ref(), deal_underwriter.key().as_ref(), deal_borrower.key.as_ref(), deal_mint.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        bump,
        // find the write scope to  enforce 
        //has_one = underwriter,
        //has_one = borrower,
        //has_one = mint_of_token_being_sent,
    )]
    pub deal_state: Account<'info, DealState>,

    
    #[account(
        mut,
        seeds=[b"wallet".as_ref(), deal_underwriter.key().as_ref(), deal_borrower.key.as_ref(), deal_mint.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        bump = wallet_bump,
    )]
    pub deal_wallet: Account<'info, TokenAccount>,

    // Member.
    #[account(
        init, 
        payer= staker,
        seeds=[b"lprovider".as_ref(),deal_state.to_account_info().key.as_ref(),staker.key.as_ref(), application_idx.to_le_bytes().as_ref()],
        bump,
        space= 8+ (4*32) +8 +8 +1 
    )]
    pub lprovider: Account<'info, LP>,

    // Charlie PDA
    #[account(mut)] // Transformed from owner to
    pub staker: Signer<'info>,
    /// CHECK. Only used to derive de dealstate
    pub deal_underwriter: AccountInfo<'info>,
    /// CHECK: Only used to derive dealstate
    pub deal_borrower:AccountInfo<'info>,
    /// CHECK: Only used to derive dealstate
    pub deal_mint:Account<'info,Mint>,

    pub system_program: Program<'info, System>,
    /// CHECK: nsafe for some reason
    /// PDA of the LP ACCOUNT to move the funds owned by the LP account
    //pub lprovider_state: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
#[instruction(application_idx: u64, state_bump: u8, wallet_bump: u8)]
pub struct Stake<'info> {
        // Stake instance.
        #[account(
            mut, 
            seeds=[b"state".as_ref(), deal_underwriter.key().as_ref(), deal_borrower.key.as_ref(), deal_mint.key().as_ref(), application_idx.to_le_bytes().as_ref()],
            bump,
            // find the write scope to  enforce 
            //has_one = underwriter,
            //has_one = borrower,
            //has_one = mint_of_token_being_sent,
        )]
        pub deal_state: Account<'info, DealState>,
    
        
        #[account(
            mut,
            seeds=[b"wallet".as_ref(), deal_underwriter.key().as_ref(), deal_borrower.key.as_ref(), deal_borrower.key().as_ref(), application_idx.to_le_bytes().as_ref()],
            bump = wallet_bump,
        )]
        pub deal_wallet: Account<'info, TokenAccount>,
    
        // Member.
        #[account(
            init, 
            payer= staker,
            seeds=[b"lprovider".as_ref(),deal_state.to_account_info().key.as_ref(),lprovider.key.as_ref(), application_idx.to_le_bytes().as_ref()],
            bump,
            space= 8+ (4*32) +8 +8 +1 
        )]
        pub lprovider: Account<'info, LP>,
    
        // Charlie PDA
        #[account(mut)] // Transformed from owner to
        pub staker: Signer<'info>,
        /// CHECK. Only used to derive de dealstate
        pub deal_underwriter: AccountInfo<'info>,
        /// CHECK: Only used to derive dealstate
        pub deal_borrower:AccountInfo<'info>,
        /// CHECK: Only used to derive dealstate
        pub deal_mint:Account<'info,Mint>,
    
        pub system_program: Program<'info, System>,
        /// CHECK: nsafe for some reason
        /// PDA of the LP ACCOUNT to move the funds owned by the LP account
        pub lprovider_state: Signer<'info>,
        pub token_program: Program<'info, Token>,
        pub rent: Sysvar<'info, Rent>,
}




