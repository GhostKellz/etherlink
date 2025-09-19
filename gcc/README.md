# GCC Token Documentation

GCC (⚡) is the core gas and transaction fee token in the GhostChain ecosystem.

## Overview

GCC serves as the primary utility token for:
- **Transaction Fees**: All blockchain operations require GCC for gas
- **Network Security**: Validators stake GCC for consensus participation
- **Economic Incentives**: Block rewards and network fees distributed in GCC
- **Deflationary Mechanics**: Transaction fees are burned, reducing total supply

## Token Economics

### Supply Dynamics
- **Initial Supply**: 1,000,000,000 GCC
- **Type**: Deflationary (fees burned)
- **Distribution**:
  - 40% - Community mining/staking rewards
  - 30% - Development fund
  - 20% - Initial validators
  - 10% - Treasury reserve

### Fee Structure
- **Base Fee**: 0.001 GCC per transaction
- **Gas Price**: Dynamic based on network congestion
- **Priority Fee**: Optional tip for faster processing
- **Contract Execution**: Variable based on computation complexity

## Usage in Etherlink

### Checking GCC Balance
```rust
use etherlink::{ServiceClients, TokenType, Address};

let address = Address::new("ghost1user123...".to_string());
let balance = services.gledger.get_balance(address, TokenType::GCC).await?;
println!("GCC Balance: {} GCC", balance);
```

### Transferring GCC
```rust
let tx_hash = services.gledger.transfer_tokens(
    from_address,
    to_address,
    TokenType::GCC,
    1000_000 // 1 GCC (6 decimal places)
).await?;
```

### Gas Estimation
```rust
let gas_estimate = services.ghostd.estimate_gas(transaction).await?;
let total_cost = gas_estimate * gas_price;
println!("Transaction will cost approximately {} GCC", total_cost);
```

## Fee Calculation

### Transaction Fees
```rust
pub fn calculate_transaction_fee(gas_used: u64, gas_price: u64) -> u64 {
    gas_used * gas_price
}

// Example: Simple transfer
let gas_used = 21_000; // Standard transfer gas
let gas_price = 100;   // Wei per gas unit
let fee = calculate_transaction_fee(gas_used, gas_price); // 2,100,000 wei = 0.0021 GCC
```

### Smart Contract Fees
```rust
pub fn calculate_contract_fee(
    gas_limit: u64,
    gas_price: u64,
    computation_units: u64
) -> u64 {
    let base_fee = gas_limit * gas_price;
    let computation_fee = computation_units * 10; // 10 wei per computation unit
    base_fee + computation_fee
}
```

## Staking and Rewards

### Validator Staking
```rust
use etherlink::{StakingConfig, ValidatorConfig};

let staking_config = StakingConfig {
    amount: 32_000_000_000, // 32,000 GCC minimum stake
    validator_address: validator_addr,
    commission_rate: 500, // 5% commission
};

let tx_hash = services.ghostd.stake_validator(staking_config).await?;
```

### Delegation
```rust
let delegation_config = DelegationConfig {
    validator: validator_address,
    amount: 1_000_000_000, // 1,000 GCC
    delegator: delegator_address,
};

let tx_hash = services.ghostd.delegate_stake(delegation_config).await?;
```

## Deflationary Mechanics

### Fee Burning
Every transaction burns a portion of the gas fee:
- **50%** of fees are burned (removed from supply)
- **30%** go to validators as rewards
- **20%** go to the treasury for development

### Supply Tracking
```rust
pub struct GCCSupplyInfo {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub burned_amount: u64,
    pub staked_amount: u64,
    pub treasury_amount: u64,
}

let supply_info = services.gledger.get_gcc_supply_info().await?;
println!("Total burned: {} GCC", supply_info.burned_amount / 1_000_000);
```

## Integration Examples

### Payment Processing
```rust
pub async fn process_payment(
    services: &ServiceClients,
    from: Address,
    to: Address,
    amount_gcc: u64
) -> Result<TxHash, EtherlinkError> {
    // Check balance
    let balance = services.gledger.get_balance(from.clone(), TokenType::GCC).await?;

    if balance < amount_gcc {
        return Err(EtherlinkError::Api("Insufficient GCC balance".to_string()));
    }

    // Estimate fees
    let estimated_fee = services.ghostd.estimate_transfer_fee(from.clone(), to.clone(), amount_gcc).await?;

    if balance < amount_gcc + estimated_fee {
        return Err(EtherlinkError::Api("Insufficient GCC for fees".to_string()));
    }

    // Execute transfer
    services.gledger.transfer_tokens(from, to, TokenType::GCC, amount_gcc).await
}
```

### Fee Management
```rust
pub struct FeeManager {
    services: ServiceClients,
    fee_buffer: u64, // Extra GCC to keep for fees
}

impl FeeManager {
    pub async fn ensure_fee_balance(&self, address: Address, required_operations: u32) -> Result<bool, EtherlinkError> {
        let balance = self.services.gledger.get_balance(address, TokenType::GCC).await?;
        let estimated_fees = required_operations as u64 * 2_100_000; // Conservative estimate

        Ok(balance >= estimated_fees + self.fee_buffer)
    }
}
```

## Best Practices

### 1. Fee Estimation
Always estimate fees before transactions to avoid failures:
```rust
let estimated_fee = services.ghostd.estimate_gas(tx).await?;
let gas_price = services.ghostd.get_gas_price().await?;
let total_cost = estimated_fee * gas_price;
```

### 2. Balance Management
Keep a buffer for transaction fees:
```rust
const FEE_BUFFER: u64 = 10_000_000; // 0.01 GCC buffer

if user_balance < transfer_amount + FEE_BUFFER {
    return Err("Insufficient balance including fee buffer");
}
```

### 3. Gas Optimization
Use appropriate gas limits to avoid overpaying:
```rust
// For simple transfers
const TRANSFER_GAS_LIMIT: u64 = 21_000;

// For contract interactions
let gas_limit = services.ghostd.estimate_gas(contract_call).await?;
let optimized_limit = gas_limit * 110 / 100; // Add 10% buffer
```

## Related Documentation

- [SPIRIT Token](../spirit/README.md) - Governance token
- [MANA Token](../mana/README.md) - Utility token
- [GHOST Token](../ghost/README.md) - Brand token
- [Tokenomics](./tokenomics/README.md) - Overall token economics
- [Governance](./governance/README.md) - Token governance mechanisms