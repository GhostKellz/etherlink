# GCC Token Economics

## Economic Model Overview

GCC implements a deflationary economic model designed to create sustainable value accrual through utility-driven demand and programmatic supply reduction.

## Supply Mechanics

### Initial Distribution (1 Billion GCC)

| Allocation | Amount | Percentage | Purpose |
|------------|--------|------------|---------|
| Mining/Staking Rewards | 400,000,000 GCC | 40% | Long-term network incentives |
| Development Fund | 300,000,000 GCC | 30% | Protocol development and maintenance |
| Initial Validators | 200,000,000 GCC | 20% | Bootstrap network security |
| Treasury Reserve | 100,000,000 GCC | 10% | Emergency fund and governance |

### Deflationary Mechanisms

#### Transaction Fee Burning
- **50% of all transaction fees are permanently burned**
- **30% distributed to validators as rewards**
- **20% allocated to treasury for protocol development**

#### Dynamic Burn Rate
```rust
pub fn calculate_burn_amount(total_fees: u64, network_activity: f64) -> u64 {
    let base_burn_rate = 0.5; // 50% base rate
    let activity_multiplier = (network_activity / 100.0).min(2.0); // Cap at 2x
    let effective_burn_rate = base_burn_rate * activity_multiplier;

    (total_fees as f64 * effective_burn_rate) as u64
}
```

## Demand Drivers

### 1. Network Utility
- **Transaction Fees**: All blockchain operations require GCC
- **Smart Contract Execution**: Gas costs for contract calls
- **Data Storage**: On-chain storage fees
- **Domain Registration**: CNS domain registration fees

### 2. Network Security
- **Validator Staking**: Minimum 32,000 GCC required
- **Delegation**: Users delegate GCC to validators
- **Slashing Protection**: Higher stakes = better protection

### 3. Economic Incentives
- **Block Rewards**: Validators earn GCC for block production
- **Fee Distribution**: Validators receive 30% of transaction fees
- **Staking Yields**: Annual percentage yields for stakers

## Price Discovery Mechanisms

### Market Forces

#### Supply Pressure (Deflationary)
- **Continuous Burning**: Reduces circulating supply
- **Staking Lock-up**: Reduces liquid supply
- **Treasury Holdings**: Strategic reserves

#### Demand Pressure (Growth)
- **Network Usage**: More transactions = more GCC needed
- **Validator Growth**: New validators need staking amounts
- **Economic Activity**: DeFi, NFTs, domain registrations

### Economic Equilibrium

```rust
pub struct EconomicMetrics {
    pub daily_burn_rate: u64,
    pub daily_issuance: u64,
    pub net_supply_change: i64,
    pub staking_ratio: f64,
    pub velocity: f64,
}

impl EconomicMetrics {
    pub fn calculate_supply_trend(&self) -> SupplyTrend {
        match self.net_supply_change {
            x if x < -1000000 => SupplyTrend::Deflationary,
            x if x > 1000000 => SupplyTrend::Inflationary,
            _ => SupplyTrend::Stable,
        }
    }
}
```

## Fee Structure

### Base Fee Schedule

| Operation Type | Base Fee (GCC) | Gas Limit | Notes |
|----------------|----------------|-----------|-------|
| Simple Transfer | 0.0021 | 21,000 | Standard wallet-to-wallet |
| Domain Registration | 1.0 | 50,000 | One-time registration |
| Smart Contract Deploy | Variable | Variable | Based on contract size |
| Contract Interaction | Variable | Variable | Based on computation |
| Staking Operations | 0.01 | 35,000 | Validator operations |

### Dynamic Fee Adjustment

```rust
pub fn calculate_dynamic_fee(base_fee: u64, network_congestion: f64) -> u64 {
    let congestion_multiplier = match network_congestion {
        x if x < 0.5 => 0.8,  // 20% discount during low usage
        x if x < 0.8 => 1.0,  // Standard rate
        x if x < 0.95 => 1.5, // 50% premium during high usage
        _ => 2.0,             // 100% premium during peak congestion
    };

    (base_fee as f64 * congestion_multiplier) as u64
}
```

## Staking Economics

### Validator Economics

#### Revenue Sources
1. **Block Rewards**: Fixed rewards per block produced
2. **Transaction Fees**: 30% of network fees
3. **MEV (Maximal Extractable Value)**: Validator-specific revenue

#### Costs
1. **Infrastructure**: Server and maintenance costs
2. **Slashing Risk**: Potential stake loss for misbehavior
3. **Opportunity Cost**: Capital locked in staking

### Staking Yields

```rust
pub fn calculate_staking_yield(
    total_staked: u64,
    annual_rewards: u64,
    validator_commission: f64,
) -> f64 {
    let base_yield = annual_rewards as f64 / total_staked as f64;
    let net_yield = base_yield * (1.0 - validator_commission);
    net_yield * 100.0 // Convert to percentage
}
```

### Target Staking Ratio: 60-70%

- **Below 60%**: Increase rewards to incentivize staking
- **Above 70%**: Decrease rewards to maintain liquidity

## Economic Scenarios

### Scenario 1: High Network Activity
- **Effect**: Increased fee generation → More burning → Supply reduction
- **Price Impact**: Upward pressure from reduced supply
- **Validator Impact**: Higher fee revenues

### Scenario 2: Low Network Activity
- **Effect**: Reduced fee generation → Less burning → Slower deflation
- **Price Impact**: Reduced upward pressure
- **Mitigation**: Base rewards maintain validator incentives

### Scenario 3: Mass Adoption
- **Effect**: Exponential fee growth → Aggressive burning → Rapid deflation
- **Price Impact**: Strong upward pressure
- **Consideration**: May need emergency governance for fee adjustments

## Risk Management

### Economic Risks

1. **Death Spiral**: If price falls, less network activity, less burning
2. **Hyperdeflation**: Too much burning could make gas fees prohibitive
3. **Centralization**: High staking requirements could limit validators

### Mitigation Strategies

1. **Governance Adjustments**: Community can modify burn rates
2. **Emergency Reserves**: Treasury can provide liquidity
3. **Fee Flexibility**: Dynamic fee adjustments for accessibility

## Monitoring and Analytics

### Key Metrics to Track

```rust
pub struct TokenomicsMetrics {
    // Supply metrics
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub burned_amount: u64,
    pub staked_amount: u64,

    // Activity metrics
    pub daily_transactions: u64,
    pub daily_fees_collected: u64,
    pub daily_fees_burned: u64,

    // Economic metrics
    pub staking_ratio: f64,
    pub average_gas_price: u64,
    pub network_utilization: f64,

    // Validator metrics
    pub active_validators: u32,
    pub average_stake_size: u64,
    pub validator_yield: f64,
}
```

### Dashboard Integration

```rust
impl ServiceClients {
    pub async fn get_tokenomics_metrics(&self) -> Result<TokenomicsMetrics, EtherlinkError> {
        let supply_info = self.gledger.get_gcc_supply_info().await?;
        let network_stats = self.ghostd.get_network_statistics().await?;
        let staking_info = self.ghostd.get_staking_information().await?;

        Ok(TokenomicsMetrics {
            total_supply: supply_info.total_supply,
            circulating_supply: supply_info.circulating_supply,
            burned_amount: supply_info.burned_amount,
            staked_amount: staking_info.total_staked,
            daily_transactions: network_stats.daily_tx_count,
            daily_fees_collected: network_stats.daily_fees,
            daily_fees_burned: network_stats.daily_burned,
            staking_ratio: staking_info.staking_ratio,
            average_gas_price: network_stats.avg_gas_price,
            network_utilization: network_stats.utilization_rate,
            active_validators: staking_info.validator_count,
            average_stake_size: staking_info.avg_stake_size,
            validator_yield: staking_info.average_yield,
        })
    }
}
```

## Future Considerations

### Potential Upgrades

1. **EIP-1559 Style Fee Market**: More predictable gas pricing
2. **Fee Delegation**: Allow applications to pay user fees
3. **Carbon Credits**: Environmental offset mechanisms
4. **Cross-Chain Integration**: Multi-chain GCC utility

### Research Areas

1. **Optimal Burn Rate**: Economic modeling for ideal deflation
2. **Staking Incentives**: Research on optimal staking rewards
3. **Fee Market Design**: Advanced fee market mechanisms
4. **Economic Security**: Game theory analysis of validator behavior