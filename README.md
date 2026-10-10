# Oracle Demo

Anchor program showing how to read from common Solana price oracles.

[Source Repository](https://github.com/ChiefWoods/oracle-demo)

## How It Works

Each instruction can be invoked to obtain a price reading from different oracle providers. The following providers are supported:

- Canary
- Pyth Core
- Pyth Lazer
- Chainlink Data Feeds
- Chainlink Data Streams
- Redstone
- Stork
- Switchboard On Demand
- Switchboard Oracle Quote

## Built With

### Languages

- [![Anchor](https://img.shields.io/badge/Anchor-0e0d11?style=for-the-badge)](https://anchor-lang.com/)

## Getting Started

### Prerequisites

1. Update your Solana CLI

```sh
agave-install init 3.1.10
```

2. Update your Anchor

```sh
avm use 1.2.0
```

### Setup

1. Clone the repository

```sh
git clone https://github.com/ChiefWoods/oracle-demo.git
```

2. Resync your program id

```sh
cd programs/oracle-demo
anchor keys sync
```

3. Build the program

```sh
just build
```

#### Testing

Run tests.

```sh
just test pyth-core
```

## Issues

View the [open issues](https://github.com/ChiefWoods/oracle-demo/issues) for a full list of proposed features and known bugs.

## Acknowledgements

### Resources

- [Shields.io](https://shields.io/)

## Contact

[chii.yuen@hotmail.com](mailto:chii.yuen@hotmail.com)
