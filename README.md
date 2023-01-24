# UTxO utils of Cardano in WASM

[![.github/workflows/wasm.yml](https://github.com/siegfried/cardano-utxo-wasm/actions/workflows/wasm.yml/badge.svg)](https://github.com/siegfried/cardano-utxo-wasm/actions/workflows/wasm.yml)

This package wraps UTxO helpers from Rust crates such as [utxo](https://github.com/siegfried/utxo) into WASM so that they can be used by Nodejs and the browsers.

## UTxO Selection Example

In real use cases, `policyId` and `assetName` are usually hex string fetched from query API such as GraphQL. `select` is expected to be used before you feed inputs to transaction builder.

```typescript
import type { Output } from 'cardano-utxo-wasm'
import init, { select, sum } from 'cardano-utxo-wasm'

// The output required to spend.
const output: Output = {
  lovelace: BigInt('10000'),
  assets: [
    { policyId: 'policy1', assetName: 'asset1', quantity: BigInt('1000') },
    { policyId: 'policy2', assetName: 'asset2', quantity: BigInt('1000') }
  ]
}

// A UTxO should not be selected because there are 2 assets not needed.
const input0: Output = {
  data: { hash: "tx0", index: 0 },
  lovelace: BigInt('100000'),
  assets: [
    { policyId: 'policy1', assetName: 'asset1', quantity: BigInt('2000') },
    { policyId: 'policy2', assetName: 'asset2', quantity: BigInt('2000') },
    { policyId: 'policy3', assetName: 'asset3', quantity: BigInt('1000') },
    { policyId: 'policy4', assetName: 'asset4', quantity: BigInt('1000') }
  ]
}

// A UTxO should be selected because there are 2 assets needed.
const input1: Output = {
  data: { hash: "tx1", index: 1 },
  lovelace: BigInt('1000'),
  assets: [
    { policyId: 'policy1', assetName: 'asset1', quantity: BigInt('2000') },
    { policyId: 'policy2', assetName: 'asset2', quantity: BigInt('1000') }
  ]
}

// A UTxO should be selected because the lovelace was not enough in `input1`.
const input2: Output = {
  data: { hash: "tx2", index: 2 },
  lovelace: BigInt('10000'),
  assets: []
}

// A UTxO should not be selected because `input1` can cover all the assets needed.
const input3: Output = {
  data: { hash: "tx3", index: 3 },
  lovelace: BigInt('10000'),
  assets: [
    { policyId: 'policy2', assetName: 'asset2', quantity: BigInt('1000') }
  ]
}

init().then(() => {
  // Select the UTxOs for the output
  const result = select(
    [input0, input1, input2, input3],
    [output],
    { lovelace: BigInt('0'), assets: [] }
  )

  // [input1, input2]
  console.log(result?.selected)

  // [input0, input3]
  console.log(result?.unselected)

  console.log(result?.excess)

  // Sum the UTxOs to one
  const total_output = sum([input0, input1, input2, input3])

  console.log(total_output)
})
```

## Make a donation

ADA: addr1qyekuuu2szr9t525k7pve467lhuy6cdrwjfjrhjswatvgyc5kkvr22hlffqdj63vk8nf8rje5np37v4fwlpvj4c4qryqydr67v
