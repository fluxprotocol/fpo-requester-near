#!/bin/bash
pair=${pair:-ETH / USD}
provider=${provider:-amberdata.testnet}
amount=${amount:-0}
accountId=${accountId:-req.smartymoriarty.testnet}
oracle=${oracle:-fpo3.franklinwaller2.testnet}
min_last_update=${min_last_update:-1646094683}

near call $oracle get_entry "{\"pair\": \"$pair\", \"provider\": \"$provider\"}" --accountId req.smartymoriarty.testnet