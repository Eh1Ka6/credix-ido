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
    
    pub fn initialize_deal(ctx: Context<InitializeDeal>, application_idx: u64, state_bump: u8, _wallet_bump: u8, amount: u64,senior_tranche:u64) -> ProgramResult {

       
        // Set the state attributes
        let state = &mut ctx.accounts.deal_state;
        state.idx = application_idx;
        state.underwriter= ctx.accounts.underwriter.key().clone();
        state.borrower = ctx.accounts.borrower.key().clone();
        state.mint_of_token_being_sent = ctx.accounts.mint_of_token_being_sent.key().clone();
        state.deal_wallet = ctx.accounts.deal_wallet.key().clone();
        state.amount_underwritten = amount;
        // Senior tranche is provided by Alice 
        state.senior_tranche = senior_tranche;
        state.amount_to_repay = amount + senior_tranche;

        msg!("Initialized new Deal instance for {}", amount);

      
        // the Deal program can use Bob's signature to "authenticate" the `transfer()` instruction sent to the token contract.
        let bump_vector = state_bump.to_le_bytes();
        let mint_of_token_being_sent_pk = ctx.accounts.mint_of_token_being_sent.key().clone();
        let application_idx_bytes = application_idx.to_le_bytes();
        let dealStatePDA = vec![
            b"state".as_ref(),
            ctx.accounts.underwriter.key.as_ref(),
            ctx.accounts.borrower.key.as_ref(),
            mint_of_token_being_sent_pk.as_ref(), 
            application_idx_bytes.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![dealStatePDA.as_slice()];

        // Below is the actual instruction that we are going to send to the Token program.
       let transfer_instruction = Transfer{
            from: ctx.accounts.wallet_to_withdraw_from.to_account_info(),
            to: ctx.accounts.deal_wallet.to_account_info(),
            authority: ctx.accounts.underwriter.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            // CPI CALL signed with the seed of the Deal_State PDA Account -> dealStatePDA
            outer.as_slice(),
        );

        // The `?` at the end will cause the function to return early in case of an error.
        // This pattern is common in Rust.
        anchor_spl::token::transfer(cpi_ctx, state.amount_underwritten)?;

        // Mark stage as deposited.
        state.epoch = Epochs::DealWritten.to_code();
        Ok(())
    }

    // Currently debugging: Transaction simulation failed: Error processing Instruction 0: Cross-program invocation with unauthorized signer or writable account
    pub fn create_lp(ctx: Context<CreateLP>, application_idx: u64, lp_bump: u8, state_bump:u8 , wallet_bump: u8) -> ProgramResult {
        

         //?? need to verify that the sent dealstate context correspond to the one we have ?? // Or already done in  in the context.rs
        if Epochs::from(ctx.accounts.deal_state.epoch)? != Epochs::DealWritten {
            msg!("Stage is invalid, state stage is {}", ctx.accounts.deal_state.epoch);
            return Err(ProgramError::Custom(100));
        }
        msg!("Initialized new Member instance for {}", ctx.accounts.staker.key().clone());

         // Set the state attributes
         let lpstate = &mut ctx.accounts.lprovider;
         lpstate.idx= application_idx ;
         lpstate.deal_mint = ctx.accounts.mint_of_token_being_sent.key().clone();
         lpstate.deal_wallet = ctx.accounts.deal_wallet.key().clone();
         lpstate.deal_state = ctx.accounts.deal_state.key().clone();
         lpstate.staker = ctx.accounts.staker.key().clone();
         
         
         
         //lpstate.lprovider_state = ctx.accounts.lprovider_state.key().clone();
         //lpstate.balance = amount;
        
        /*let lprovider = &mut ctx.accounts.lprovider;
        lprovider.deal_state = *ctx.accounts.deal_state.to_account_info().key;
        lprovider.owner = *ctx.accounts.owner.key;
        //lprovider.lprovider_signer = *ctx.accounts.lprovider_signer.to_account_info().key.as_ref();
        //lprovider.balance = amount;
        lprovider.idx= idx;
        */
        Ok(())
    }
    
    pub fn stake(ctx: Context<Stake>, application_idx: u64, lp_bump: u8, state_bump:u8 ,wallet_bump: u8,  amount: u64) -> ProgramResult {
       
        //?? need to verify that the sent dealstate context correspond to the one we have ?? // Or already done in  in the context.rs
        if Epochs::from(ctx.accounts.deal_state.epoch)? != Epochs::DealWritten {
            msg!("Stage is invalid, state stage is {}", ctx.accounts.deal_state.epoch);
            return Err(ProgramError::Custom(100));
        }

        if  (ctx.accounts.deal_state.amount_staked + amount) > ctx.accounts.deal_state.senior_tranche  {
            msg!("Amount is superior to senior tranche  {}", ctx.accounts.deal_state.amount_staked + amount);
            return Err(ProgramError::Custom(100));
        }

     
            // See Serum stake doc to integrate Deposit and stake 
            // Transfer tokens into the Deal Wallet.
                let lpstate = &mut ctx.accounts.lprovider;
                
                let bump_vector = lp_bump.to_le_bytes();
                let application_idx_bytes = application_idx.to_le_bytes();
                let lpPDA = vec![
                    b"lprovider".as_ref(),
                    ctx.accounts.deal_state.to_account_info().key.as_ref(),
                    ctx.accounts.staker.key.as_ref(), 
                    application_idx_bytes.as_ref(),
                    bump_vector.as_ref(),
                ];

            
                let outer = vec![lpPDA.as_slice()];
                
                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        // TO DO : Convert Owner Pubkey to lprovider_signer ATA  
                        from: ctx.accounts.staker_wallet.to_account_info(),
                        // TO DO : Review Access to  dealState ATA wallet 
                        to: ctx.accounts.deal_wallet.to_account_info(),
                        authority: ctx.accounts.staker.to_account_info(),
                    },
                    // CPI call signed with the PDA of LP Account .  
                    outer.as_slice(),
                );
            
                anchor_spl::token::transfer(cpi_ctx, amount)?; 
                // update state
                let dealstate  = &mut ctx.accounts.deal_state;

                // TO DO : Double check for overflows
                // TEST if balance are not updated if transfer fails 
                dealstate.amount_staked += amount;  
                lpstate.balance += amount;
                //msg!("Amount is superior to senior tranche  {}", dealstate.amount_staked);
                //msg!("Amount is superior to senior tranche  {}", dealstate.senior_tranche);
                if dealstate.amount_staked >= dealstate.senior_tranche {

                    // Change the phase 
                    dealstate.epoch = Epochs::LPCompleted.to_code();
                    msg!("Set new epoch {}", dealstate.epoch);
                }
                    // Mint the CREDIX LP tokens to the staker.
                        //How to mint 
                        // GET Current LP Token Price 
            Ok(())
    }
 

    pub fn borrow(ctx: Context<Borrow>, application_idx: u64, state_bump:u8 ,_wallet_bump: u8) -> ProgramResult {
           msg!("Borrrow funds being executed");
            if Epochs::from(ctx.accounts.deal_state.epoch)? != Epochs::LPCompleted {
                msg!("Stage is invalid, state stage is {}", ctx.accounts.deal_state.epoch);
                return Err(ProgramError::Custom(100));
            }
                // Allow the borrower to withdraw the funds
                
            // Nothing interesting here! just boilerplate to compute our signer seeds for
            // signing on behalf of our PDA.
            let bump_vector = state_bump.to_le_bytes();
            let application_idx_bytes = application_idx.to_le_bytes();
            let mint_of_token_being_sent_pk = ctx.accounts.mint_of_token_being_sent.key().clone();
            let inner = vec![
                b"state".as_ref(),
                ctx.accounts.underwriter.to_account_info().key.as_ref(),
                ctx.accounts.borrower.to_account_info().key.as_ref(),
                mint_of_token_being_sent_pk.as_ref(),
                application_idx_bytes.as_ref(),
                bump_vector.as_ref(),
            ];
            let outer = vec![inner.as_slice()];

            // Perform the actual transfer
            let transfer_instruction = Transfer{
                from: ctx.accounts.deal_wallet.to_account_info(),
                to:  ctx.accounts.wallet_to_deposit_to.to_account_info(),
                authority:  ctx.accounts.deal_state.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                transfer_instruction,
                outer.as_slice(),
            );
            anchor_spl::token::transfer(cpi_ctx, ctx.accounts.deal_state.amount_staked + ctx.accounts.deal_state.amount_underwritten)?;


            
            let state = &mut ctx.accounts.deal_state;
            state.epoch = Epochs::Repaymentphase.to_code();
         

            Ok(())
     }
/*
        pub fn repay(ctx: Context<Stake>, application_idx: u64, lp_bump: u8, state_bump:u8 ,wallet_bump: u8,  repayment: u64) -> ProgramResult {
           
            
            if Epochs::from(ctx.accounts.deal_state.epoch)? != Epochs::Repaymentphase {
                msg!("Stage is invalid, state stage is {}", ctx.accounts.deal_state.epoch);
                return Err(ErrorCode::StageInvalid.into());
            }
                // Repayment calculation
                    // interest = repayment * Longtermgain / (investment + Longtermgain)  
                let underwriter_share = amount_underwritten / (amount_underwritten + amount_staked);
                let stakers_share = amount_staked / (amount_underwritten + amount_staked);
                let interest_per_repayment = (repayment * (amount_to_repay - amount_underwritten + amount_staked) / amount_to_repay);
                let underwriter_interest = interest_per_repayment * underwritter_share ;
                let stakers_interest =  interest_per_repayment * stakers_share ;
                 
                underwriter_capitalrepayment = repayment - interest_per_repayment * underwriter_share;
                stakers_capitalrepayment = repayment - interest_per_repayment * stakers_share;

                // transfer the interest to alice account
                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        // TO DO : Convert Owner Pubkey to lprovider_signer ATA  
                        from: ctx.accounts.deal_wallet.to_account_info(),
                        // TO DO : Review Access to  dealState ATA wallet 
                        to: ctx.accounts.wallet_to_withdraw_from.to_account_info(),
                        authority: ctx.accounts.deal_state.to_account_info(),
                    },
                    // signed by dealstate program account 
                    outer.as_slice(),
                );
                anchor_spl::token::transfer(cpi_ctx, underwriter_interest)?; 
                // transfer the interest to lpprovider vault 
                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        // TO DO : Convert Owner Pubkey to lprovider_signer ATA  
                        from: ctx.accounts.deal_wallet.to_account_info(),
                        // TO DO : Review Access to  dealState ATA wallet 
                        to: ctx.accounts.lprovider_vault.to_account_info(),
                        authority: ctx.accounts.deal_state.to_account_info(),
                    },
                    // signed by dealstate program account 
                    outer.as_slice(),
                );
                anchor_spl::token::transfer(cpi_ctx, underwriter_interest)?; 
                // transfer the initial capital to the underwriter burning vault 

                // transfer the initial capital to the staker burning vault

                Ok(())
                
        }
            
        

        // allow polling dispatch from the staker vault
        pub fn claim_staker_reward (ctx: Context<Stake>, application_idx: u64, lp_bump: u8, state_bump:u8 ,wallet_bump: u8,  amount: u64)-> ProgramResult {


           // available_reward; 

            Ok(())
        }



        pub fn redeemLPToken () {
            Ok(())

        }

        // state is instatiated during INitialized if the Deal Expire
        pub fn Pullback (ctx: Context<Stake>, application_idx: u64, lp_bump: u8, state_bump:u8 ,wallet_bump: u8,  amount: u64) -> ProgramResult{


            Ok(())
        }
        */
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
       
        //Ok(())


    

    



