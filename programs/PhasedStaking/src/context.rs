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
        payer = undertaker,
        seeds=[b"state".as_ref(), undertaker.key().as_ref(), borrower.key.as_ref(), mint_of_token_being_sent.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        bump,
        space=  8 + DealState::MAX_SIZE // 
    )]
    pub deal_state: Account<'info, DealState>, // puppet
    #[account(
        init,
        payer = undertaker,
        seeds=[b"wallet".as_ref(), undertaker.key().as_ref(),  borrower.key.as_ref(), mint_of_token_being_sent.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        bump, //= wallet_bump,
        token::mint=mint_of_token_being_sent,
        token::authority=deal_state,
    )]
    pub deal_wallet_state: Account<'info, TokenAccount>,

    // Pda account of Alice 
    #[account(mut)]
    pub undertaker: Signer<'info>,                     // Alice
    // CHECK: no checks needed functionnal first secure after 
   // pub lprovider: AccountInfo<'info>,                        // Charlie
    /// CHECK: no checks needed functionnal first secure after 
    pub borrower: AccountInfo<'info>,                      // Bob
    pub mint_of_token_being_sent: Account<'info, Mint>,  // USDC

    // Alice's USDC wallet that has already approved the escrow wallet
    #[account(
        mut,
        constraint=wallet_to_withdraw_from.owner == undertaker.key(),
        constraint=wallet_to_withdraw_from.mint == mint_of_token_being_sent.key()
    )]
    pub wallet_to_withdraw_from: Account<'info, TokenAccount>,

    // Application level accounts
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(application_idx: u64)]
pub struct CreateLP<'info> {
    // Stake instance.
    pub deal_state: Account<'info, DealState>,
    // Member.
    #[account(
        init, 
        payer= owner,
        seeds=[b"lprovider".as_ref(),deal_state.to_account_info().key.as_ref(),lprovider.key.as_ref(), application_idx.to_le_bytes().as_ref()],
        bump,
        space= 8+ (2*32) +8 +8 +1 
    )]
    pub lprovider: Account<'info, LP>,

    // PDA of the owner  TO DO: see if owner and lprovider_signer are not the same thing
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: nsafe for some reason
    /// PDA of the LP ACCOUNT 
    pub lprovider_signer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateLP<'info> {
    fn accounts(ctx: &Context<CreateLP>, nonce: u8) -> Result<()> {
      /*  let seeds = &[
            ctx.accounts.deal_state.to_account_info().key.as_ref(),
            ctx.accounts.lprovider.to_account_info().key.as_ref(),
            &[nonce],
        ];
        
        let lprovider_signer = Pubkey::create_program_address(seeds, ctx.program_id).map_err(|_| ErrorCode::InstructionMissing)?;
        if &lprovider_signer != ctx.accounts.lprovider_signer.to_account_info().key {
            //return Err(ErrorCode::InvalidMemberSigner.into());
            return Err(ErrorCode::InstructionMissing.into());
        }
*/
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(application_idx: u64, state_bump: u8, wallet_bump: u8)]
pub struct Stake<'info> {
    // Global accounts for the staking instance.
    #[account(has_one = undertaker, has_one = mint_of_token_being_sent)]
    pub deal_state: Account<'info, DealState>,
    #[account(
        mut,
        seeds=[b"wallet".as_ref(), undertaker.key().as_ref(), borrower.key.as_ref(), mint_of_token_being_sent.key().as_ref(), application_idx.to_le_bytes().as_ref()],
        bump = wallet_bump,
    )]
    pub deal_wallet_state: Account<'info, TokenAccount>,


    #[account(mut)]
    /// CHECK: nsafe for some reason
    undertaker: AccountInfo<'info>,                     // Alice
    #[account(mut)]
    borrower: Signer<'info>,                        // Bob
    mint_of_token_being_sent: Account<'info, Mint>,       // USDC
    //deal_wallet_state: Account<'info, TokenAccount>,
    // Member.
    #[account(mut, has_one = owner, has_one = deal_state)]
    pub lprovider: Account<'info, LP>,
    #[account(signer)]
    /// CHECK: nsafe for some reason
    pub owner: AccountInfo<'info>,

    // Program signers.
    #[account(
        seeds = [
            deal_state.to_account_info().key.as_ref(),
            lprovider.to_account_info().key.as_ref(),
            application_idx.to_le_bytes().as_ref(),
        ],
        bump = state_bump
    )]
    /// CHECK: nsafe for some reason
    pub lprovider_signer: AccountInfo<'info>,
    #[account(seeds = [deal_state.to_account_info().key.as_ref(), application_idx.to_le_bytes().as_ref()],bump = state_bump)]
    /// CHECK: nsafe for some reason
    deal_state_signer: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}




