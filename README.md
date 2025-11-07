# ZKWASM automated mining game template

## Introduction

The template offers a timer-based mining system that ensures the fair distribution of points (rewards). The entire system operates within the ZKWASM virtual machine, which functions as a trustless REST service (Layer 3), providing mining results along with their cryptographic proofs to the target blockchain (Layer 1 or Layer 2), for example, BNB Chain.

In this system, players manage their miners (robots) by assigning them cards (activities). Each activity lasts a few minutes, and upon completion, it generates different types of resources at the cost of other resources. The player's objective is to strategically manage a team of robots, optimizing their activities to produce the maximum amount of the target resource.

Because the game requires careful, real-time management of miners, it effectively mitigates "Sybil Attacks," as it is challenging for attackers to respond efficiently to the complex and dynamic scenarios necessary to optimize production.

## Hosting the demo game

### Prepare environment

For fresh environment, run `source script/environment_linux.sh` for Linux.

If you see some error messages, need manually install the error module in your OS.

### Required service:

1. Start Redis
```
redis-server
```

2. Mongodb
```
mongod --dbpath $DB_FOLDER
```

3. Start Merkle DB Service:
Since the whole game sever is supposed to be checked in ZKWASM, we need to provide a Merkle backend to generate the witness for it. Here we use the ZKWAM-MINI-ROLLUP at git@github.com:DelphinusLab/zkwasm-mini-rollup.git.

To run the Merkle DB Service, use can clone the zkwasm mini rollup repo:
```
git clone git@github.com:DelphinusLab/zkwasm-mini-rollup.git
```
and then in `./dbservice`, run `bash run.sh`

### Running the game REST server
1. Clone this repo:
```
git clone https://github.com/riddles-are-us/zkwasm-automata
```

2. Install ZKWASM-MINI-ROLLUP reset server

3. Install zkwasm-ts-server
In `./ts`, run:
```
npm install
```

4. Build WASM image:
In './', run:
```
make
```

5. Run WASM service:
```
make run
```

## Provide a FrontEnd
A demo frontend can be find at https://github.com/riddles-are-us/frontend-automata
