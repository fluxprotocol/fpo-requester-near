#!/bin/bash

# default params
network=${network:-testnet}
accountId=${accountId:-req.martymoriarty.testnet}
master=${master:-martymoriarty.testnet}
initialBalance=${initialBalance:-5}

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare $param="$2"
        # echo $1 $2 // Optional to see the parameter:value result
   fi

  shift
done

NEAR_ENV=$network near delete $accountId $master
NEAR_ENV=$network near create-account $accountId --masterAccount $master --initialBalance $initialBalance