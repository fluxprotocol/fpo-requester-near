# Flux First Party Oracle Requester Contract on NEAR
Sample contract for data requesters / protocols deployed on NEAR that makes cross contract calls to our Near FPO (First Party Oracle) to receive responses to their data requests.

## To Build, Deploy, & Run
Clone the repo, then cd into the fpo-requester directory.

The scripts folder has all of the scripts you need to build, deploy, and run the fpo requester example contract.

To set everything up in one go, go to the ```rerun.sh``` file in the ```scripts``` folder and replace the two instances of ```YOUR_TESTNET_ACCOUNT_ID``` with your actual testnet account id. 

Save your files, then run the following command:
```
sh scripts/rerun.sh
```

All of the scripts that make up this file are provided individually in the scripts folder as well. Modify the ```get_request.sh``` file with different tokens to get different price entries. you can see what feeds are available [here](https://docs.fluxprotocol.org/docs/live-data-feeds/live-pairs#near-testnet).

Feel free to copy this code into your existing contract in order to plug into the Near FPO. 