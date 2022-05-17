
# credix-ido

## 

Solava version  1.10.10

## Develpment State
```

	anchor test
	Initialize a deal  OK
	Create LP Member :  OK
	Stake : OK
	Borrow : OK 
	init_Lp_pool / claim_lp_underwriter / claim_lp_staker : In Progress
	redeem: Todo
	Pullback: Todo

``` 
## Accounts

  ![Execution Flow](/assets/flow_chart.png?raw=true "Execution Flow")

## Instructions
``` 
	// Transfer the Underwriter's USDC to a wallet Owned by the current deal
    pub fn initialize_deal(ctx: Context<InitializeDeal>, application_idx: u64, state_bump: u8,
   						  _wallet_bump: u8,mint_bump:u8, amount: u64,senior_tranche:u64){}
	// Initialize a Lp token faucet owned by the current program
    pub fn init_lp_pool(ctx: Context<InitLpPool>, mint_bump: u8, amount : u64 ){}
	// Allow underwriter to withdraw the calculated amount of Lp Tokens 
    pub fn claim_lp_underwriter(ctx: Context<ClaimLpUnderwriter>, application_idx: u64 ,state_bump:u8 ,mint_bump: u8){}
	// Allow staker to withdraw the calculated amount of Lp Tokens 
    pub fn claim_lp_staker(ctx: Context<ClaimLpStaker>, application_idx: u64 ,lp_bump:u8 ,mint_bump: u8){}
	// create a LiqProvider Account assigned to the Pool
    pub fn create_lp(ctx: Context<CreateLP>, application_idx: u64, lp_bump: u8, state_bump:u8 , wallet_bump: u8){}
	// Allow Liqprovider Accounts to deposit USDC into the deal
    pub fn stake(ctx: Context<Stake>, application_idx: u64, lp_bump: u8, state_bump:u8 ,wallet_bump: u8,  amount: u64){}
	// Allow borrower to  withdraw the USDC Amounts of the deal
    pub fn borrow(ctx: Context<Borrow>, application_idx: u64, state_bump:u8 ,_wallet_bump: u8) {}
	// Allow the borrower to deposit the repayments to the deal calculate redeemable amount of LpTokens
    pub fn repay(ctx: Context<Stake>, application_idx: u64, lp_bump: u8, state_bump:u8 ,wallet_bump: u8,  repayment: u64) {}
	// Allow the underwriter to redeem his tokens
	pub fn redeemLP_Underwriter (ctx: Context<Stake>, application_idx: u64, lp_bump: u8, state_bump:u8 ,wallet_bump: u8,  amount: u64) {}
    pub fn redeemLP_Stakers () {}
	// If the the stake_time to complete the senior tranche expires allow the underwriter to withdraw and close the deal after all stakers have withdrawn
    pub fn Pullback (ctx: Context<Stake>, application_idx: u64, lp_bump: u8, state_bump:u8 ,wallet_bump: u8,  amount: u64) {}
```
## Test 

```Bash

	npm install 

	anchor test
	

``` 



