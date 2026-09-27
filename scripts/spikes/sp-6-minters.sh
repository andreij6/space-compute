#!/usr/bin/env bash
set -euo pipefail
CKETH_MINTER=sv3dd-oaaaa-aaaar-qacoa-cai
CKBTC_MINTER=mqygn-kiaaa-aaaar-qaadq-cai
q() { icp canister call "$1" "$2" '()' -n ic --query --identity anonymous; }
eth=$(q "$CKETH_MINTER" get_minter_info)
btc=$(q "$CKBTC_MINTER" get_minter_info)
field() { echo "$1" | grep -oE "$2 = opt \(?[^;]*" | head -1 | sed -E 's/.*= opt \(?//; s/ : nat.*//; s/_//g; s/"//g'; }
min_wei=$(field "$eth" minimum_eth_deposit_amount)
helper=$(field "$eth" deposit_with_subaccount_helper_contract_address)
echo "ckETH: subaccount deposit helper = ${helper:-NONE}"
echo "ckETH: minimum deposit = $min_wei wei ($(python3 -c "print($min_wei/1e18)") ETH)"
echo "ckBTC: $(echo "$btc" | tr -d '\n' | sed -E 's/[[:space:]]+/ /g; s/^\( record \{ //; s/ \}, \)$//')"
[ -n "$helper" ] || { echo "subaccount deposits NOT supported"; exit 1; }
