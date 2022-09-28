# Descentralized Agreements Network
This system represents a node of the DAN network. A protocol created to ensure compliance with agreements between contractor and contractor.

## Features
### Agreements Pallets

#### Create
Creates a new agreement, defining the parts, value and a hash of the document that describes it
**This function is the only one that charge fees**
#### Sign
Function intended for only the hired described in the agreement, it signs the agreement and prevents its cancellation until it is fulfilled and accepted or unsigned and canceled later

#### Set Review
When the terms of the agreement are concluded by the hired, the hired may request a review of the agreement, which, if accepted by the contractor, sends the agreements funds to the hired.
#### Accept
If the agreement is under review, the contractor can accept it, the agreement funds will be transferred to the hired

#### Unsign
If at any time the hired cannot comply with the agreement, he can use this function to unsign the agreement, allowing the contractor to cancel it.

#### Cancel
Cancels the contract, causing the funds to be sent to the contractor
### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command
(`cargo build --release && ./target/release/node-template --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/node-template --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/node-template purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```
