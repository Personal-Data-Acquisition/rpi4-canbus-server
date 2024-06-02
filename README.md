## RPI4 Canbus Server
A raspberry pi based sensor module server.

## How to run

To setup the physical hardware for this you'll need a raspberry pi 4 and a dual cheip MCP2515 and SN65HVD230 can hat.



From there you need to run the included setup script to install the necesarry drivers and boot parameters. The setup script will also need to be run after each boot to enable to can hat.

```sh

./setup.sh

# You don't need to actually build first but eh not a bad move to.
cargo build --release

# Then launch the server with
cargo run --release

```
