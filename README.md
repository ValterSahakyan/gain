# Token Sale Program

A Solana program built with Anchor for token sales.

## Project Structure

```
├── programs/                 # Anchor programs
│   └── token_sale/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── scripts/                 # Utility scripts
│   ├── calc_pda.js         # Calculate PDAs
│   ├── init_guide.js       # Initialize guide
│   └── init_anchor.js      # Initialize with Anchor
├── tests/                   # Test files
├── app/                     # Client application
├── target/                  # Build artifacts
├── Anchor.toml             # Anchor configuration
├── Cargo.toml              # Workspace configuration
└── package.json            # Node.js dependencies
```

## Quick Start

1. **Install dependencies:**
   ```bash
   npm install
   ```

2. **Build the program:**
   ```bash
   npm run build
   ```

3. **Deploy to devnet:**
   ```bash
   npm run deploy
   ```

4. **Calculate PDAs:**
   ```bash
   npm run pda
   ```

5. **View initialize guide:**
   ```bash
   npm run init
   ```

## Program Details

- **Program ID**: `6ZK4hFGen61b83NHsNTAMq71r3QJCTwknvj4CYfLxdBj`
- **Config PDA**: `CZSqotxPc2UxUCdUauUakLENrkYEtfoxdBPcdaPiqQGj`
- **Mint Authority PDA**: `DTQAP96JiuHi9HmzGUN7HVmNW5ZwCjubWF7zmidKy4wm`

## Features

- Initialize token sale with custom price
- Buy tokens with SOL
- Owner management (pause/unpause, set price)
- Token 2022 support
