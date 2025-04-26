# nftide

[![CI](https://github.com/0xmichalis/nftide/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/0xmichalis/nftide/actions/workflows/ci.yml)

**nftide** is a CLI tool for fetching NFT sales and other market data.

## Installation

1. **Clone the repository:**
   ```sh
   git clone https://github.com/0xmichalis/nftide.git
   cd nftide
   ```

2. **Install Rust (if you don't have it):**
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build the project:**
   ```sh
   cargo build --release
   ```

## Usage

1. **Set your OpenSea API key:**

   Create a `.env` file in the project root with:
   ```
   OPENSEA_API_KEY=your_opensea_api_key_here
   ```

2. **Run the CLI:**
   ```sh
   cargo run -- --collection-slug <slug> [--output-path <dir>]
   ```

   Example:
   ```sh
   cargo run -- --collection-slug ikb-cachet-de-garantie-1 --output-path data
   ```

   This will save the sales data as `data/ikb-cachet-de-garantie-1_sales.json`.


## License

MIT
