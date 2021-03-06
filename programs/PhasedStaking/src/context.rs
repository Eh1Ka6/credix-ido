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
    pub deal_wallet: Account<'info, TokenAccount>,

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
#[instruction(application_idx: u64, lp_bump: u8, state_bump:u8 ,wallet_bump: u8)]
pub struct CreateLP<'info> {
    // Stake instance.
    #[account(
        mut, 
        seeds=[b"state".as_ref(), underwriter.key().as_ref(), borrower.key.as_ref(), mint_of_token_being_sent.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        bump=state_bump,
        
        has_one = underwriter,
        has_one = borrower,
        has_one = mint_of_token_being_sent,
       
    )]
    pub deal_state: Account<'info, DealState>,

    
    #[account(
        mut,
        seeds=[b"wallet".as_ref(), underwriter.key().as_ref(), borrower.key.as_ref(), mint_of_token_being_sent.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        bump = wallet_bump,
    )]
    pub deal_wallet: Account<'info, TokenAccount>,

    // Member.
    #[account(
        init, 
        payer= staker,
        seeds=[b"lprovider".as_ref(),deal_state.to_account_info().key.as_ref(),staker.key.as_ref(), application_idx.to_le_bytes().as_ref()],
        bump,
        space= 8 + LiqProvider::MAX_SIZE 
    )]
    pub lprovider: Account<'info, LiqProvider>,

 

    // Charlie PDA
    #[account(mut)] // Transformed from owner to
    pub staker: Signer<'info>,
    
    /// CHECK: unsafe but just used to derive
    #[account(mut)]
    pub underwriter: AccountInfo<'info>,
    /// CHECK: unsafe but just used to derive 
    #[account(mut)]
    pub borrower:AccountInfo<'info>,
    pub mint_of_token_being_sent:Account<'info,Mint>,
    
    pub system_program: Program<'info, System>,
    /// CHECK: nsafe for some reason
    /// PDA of the LP ACCOUNT to move the funds owned by the LP account
    //pub lprovider_state: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
#[instruction(application_idx: u64, lp_bump: u8, state_bump:u8 ,wallet_bump: u8,amount:u64)]
pub struct Stake<'info> {
        // Stake instance.
        #[account(
            mut, 
            seeds=[b"state".as_ref(), underwriter.key().as_ref(), borrower.key.as_ref(), mint_of_token_being_sent.key().as_ref(), application_idx.to_le_bytes().as_ref()],
            bump=state_bump,
            // find the write scope to  enforce 
            has_one = underwriter,
            has_one = borrower,
            has_one = mint_of_token_being_sent,
        )]
        pub deal_state: Account<'info, DealState>,
      
        #[account(
            mut,
            seeds=[b"wallet".as_ref(), underwriter.key().as_ref(), borrower.key.as_ref(), mint_of_token_being_sent.key().as_ref(), application_idx.to_le_bytes().as_ref()],
            bump = wallet_bump,
        )]
        pub deal_wallet: Account<'info, TokenAccount>,
    
        // Member.
        #[account(
            mut,
            seeds=[b"lprovider".as_ref(),deal_state.to_account_info().key.as_ref(),staker.key.as_ref(), application_idx.to_le_bytes().as_ref()],
            bump=lp_bump
        )]
        pub lprovider: Account<'info, LiqProvider>,
    
        // Charlie PDA
        #[account(mut)] // Transformed from owner to
        pub staker: Signer<'info>,
        /// CHECK: nsafe for some reason
        #[account(mut)]
        pub underwriter: AccountInfo<'info>,
        /// CHECK: nsafe for some reason
        #[account(mut)]
        pub borrower:AccountInfo<'info>,
       
        pub mint_of_token_being_sent:Account<'info,Mint>,
    
        
        pub system_program: Program<'info, System>,
        #[account(
            mut,
            constraint=staker_wallet.owner == staker.key(),
            constraint=staker_wallet.mint == mint_of_token_being_sent.key()
        )]
        pub staker_wallet: Account<'info, TokenAccount>,

        pub token_program: Program<'info, Token>,
        pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(application_idx: u64, state_bump:u8 ,wallet_bump: u8)]
pub struct Borrow<'info> {
        #[account(
            mut,
            seeds=[b"state".as_ref(), underwriter.key().as_ref(), borrower.key.as_ref(), mint_of_token_being_sent.key().as_ref(), application_idx.to_le_bytes().as_ref()],
            bump = state_bump,
            has_one = underwriter,
            has_one = borrower,
            has_one = mint_of_token_being_sent,
        )]
        pub deal_state: Account<'info, DealState>,
        #[account(
            mut,
            seeds=[b"wallet".as_ref(), underwriter.key().as_ref(), borrower.key.as_ref(), mint_of_token_being_sent.key().as_ref(), application_idx.to_le_bytes().as_ref()],
            bump = wallet_bump,
        )]
        pub deal_wallet: Account<'info, TokenAccount>,
        
        #[account(
            init,
            payer = borrower,
            associated_token::mint = mint_of_token_being_sent,
            associated_token::authority = borrower,
        )]
        pub wallet_to_deposit_to: Account<'info, TokenAccount>,   // Bob's USDC wallet (will be initialized if it did not exist)
    
        /// CHECK: unsafe
        #[account(mut)]
        pub underwriter: AccountInfo<'info>,                     // Alice
        #[account(mut)]
        pub borrower: Signer<'info>,                        // Bob
        pub mint_of_token_being_sent: Account<'info, Mint>,       // USDC
    
        // Application level accounts
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
        pub associated_token_program: Program<'info, AssociatedToken>,
        pub rent: Sysvar<'info, Rent>,
    }


