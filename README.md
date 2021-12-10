# Airdrop tokens
Program to provide users airdrop of selected token

## Initialize airdrop account
Initializer creates setups airdrop with given settings, like token and withdraw amount.
Initial funds are sent to the airdrop token pot account.
Token pot account can be funded anytime by sending tokens to token account address.

## Get airdrop
User calls get airdrop method to get airdrop token.
Amount is limited by withdraw_amount setting.
If user's associated token account does not exist it will be created automatically.

##Cancel airdrop
Initializer of airdrop can cancel airdrop by calling cancel airdrop method.
It will refund all remaining tokens to initializer associated token account,
and close all program accounts.


Created with :heart: by [schananas](https://github.com/schananas)