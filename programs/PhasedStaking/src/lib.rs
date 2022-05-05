pub mod context;
pub mod state;
use crate::context::*;
use state::*;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{CloseAccount, Mint, Token, TokenAccount}};
use anchor_lang::solana_program::entrypoint::ProgramResult;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");


// Alice Creates a new Deal and defines the parameters for it, initialized the escrow wallet owned by the Safe Transfer program
// 

#[program]
pub mod ido {
    use super::*;
    use anchor_spl::token::Transfer;
    
    pub fn initialize_deal(ctx: Context<InitializeDeal>, application_idx: u64, state_bump: u8, _wallet_bump: u8, amount: u64) -> ProgramResult {

       
        // Set the state attributes
        let state = &mut ctx.accounts.deal_state;
        state.idx = application_idx;
        state.undertaker = ctx.accounts.undertaker.key().clone();
        state.borrower = ctx.accounts.borrower.key().clone();
        state.mint_of_token_being_sent = ctx.accounts.mint_of_token_being_sent.key().clone();
        state.deal_wallet = ctx.accounts.deal_wallet_state.key().clone();
        state.amount_tokens = amount;
        

        msg!("Initialized new Deal instance for {}", amount);

      
        // the Deal program can use Bob's signature to "authenticate" the `transfer()` instruction sent to the token contract.
        let bump_vector = state_bump.to_le_bytes();
        let mint_of_token_being_sent_pk = ctx.accounts.mint_of_token_being_sent.key().clone();
        let application_idx_bytes = application_idx.to_le_bytes();
        let dealStatePDA = vec![
            b"state".as_ref(),
            ctx.accounts.undertaker.key.as_ref(),
            ctx.accounts.borrower.key.as_ref(),
            mint_of_token_being_sent_pk.as_ref(), 
            application_idx_bytes.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![dealStatePDA.as_slice()];

        // Below is the actual instruction that we are going to send to the Token program.
       let transfer_instruction = Transfer{
            from: ctx.accounts.wallet_to_withdraw_from.to_account_info(),
            to: ctx.accounts.deal_wallet_state.to_account_info(),
            authority: ctx.accounts.undertaker.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            // CPI CALL signed with the seed of the Deal_State PDA Account -> dealStatePDA
            outer.as_slice(),
        );

        // The `?` at the end will cause the function to return early in case of an error.
        // This pattern is common in Rust.
        anchor_spl::token::transfer(cpi_ctx, state.amount_tokens)?;

        // Mark stage as deposited.
        state.epoch = Epochs::DealWritten.to_code();
        Ok(())
    }

    // Currently debugging Transaction simulation failed: Error processing Instruction 0: Cross-program invocation with unauthorized signer or writable account

    // #[access_control(CreateMember::accounts(&ctx, nonce))]
    pub fn create_lp(ctx: Context<CreateLP>, idx: u64) -> Result<()> {
        /*let lprovider = &mut ctx.accounts.lprovider;
        lprovider.deal_state = *ctx.accounts.deal_state.to_account_info().key;
        lprovider.owner = *ctx.accounts.owner.key;
        //lprovider.lprovider_signer = *ctx.accounts.lprovider_signer.to_account_info().key.as_ref();
        //lprovider.balance = amount;
        lprovider.idx= idx;
        */
        Ok(())
    }
   /* #[access_control(no_available_rewards(
        &ctx.accounts.reward_event_q,
        &ctx.accounts.member,
        &ctx.accounts.balances,
        &ctx.accounts.balances_locked,
    ))]*/
    pub fn stake(ctx: Context<Stake>, token_amount: u64) -> Result<()> {
       

        // See Serum stake doc to integrate Deposit and stake 
        // Transfer tokens into the Deal Wallet.
        
        {
             // Derives the adresse of lprovider_signer that owns the LP Data Account and ATA 
            let seeds = &[
                // registrar 
                ctx.accounts.deal_state.to_account_info().key.as_ref(),
                //member (LP Account)
                ctx.accounts.lprovider.to_account_info().key.as_ref(),
                //nonce
                &[ctx.accounts.lprovider.idx.try_into().unwrap()],
            ];
            let lprovider_signer = &[&seeds[..]];
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    // TO DO : Convert Owner Pubkey to lprovider_signer ATA  
                    from: ctx.accounts.owner.to_account_info(),
                    // TO DO : Review Access to  dealState ATA wallet 
                    to: ctx.accounts.deal_wallet_state.to_account_info(),
                    authority: ctx.accounts.lprovider_signer.to_account_info(),
                },
                // CPI call signed with the PDA of LP Account .  
                lprovider_signer,
            );
           
            anchor_spl::token::transfer(cpi_ctx, token_amount)?;
        }

        // Mint the CREDIX LP tokens to the staker.
       /*{
            let seeds = &[
                ctx.accounts.registrar.to_account_info().key.as_ref(),
                &[ctx.accounts.registrar.nonce],
            ];
            let registrar_signer = &[&seeds[..]];

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.clone(),
                token::MintTo {
                    mint: ctx.accounts.pool_mint.to_account_info(),
                    to: balances.spt.to_account_info(),
                    authority: ctx.accounts.registrar_signer.to_account_info(),
                },
                registrar_signer,
            );
            token::mint_to(cpi_ctx, spt_amount)?;
        }
        */
        // Update stake timestamp.
        //let member = &mut ctx.accounts.member;
        //member.last_stake_ts = ctx.accounts.clock.unix_timestamp;
       
        Ok(())
    }
    

    

}

