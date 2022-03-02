#!/bin/bash
pair=${pair:-ETH / USD}
provider=${provider:-amberdata.testnet}
amount=${amount:-0}
accountId=${accountId:-req.martymoriarty.testnet}
min_last_update=${min_last_update:-1646094683}

near call $accountId find_entry "{\"pair\": \"$pair\", \"provider\": \"$provider\"}" --accountId $accountId