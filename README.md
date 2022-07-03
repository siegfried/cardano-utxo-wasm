# UTxO utils of Cardano in WASM

[![.github/workflows/wasm.yml](https://github.com/siegfried/cardano-utxo-wasm/actions/workflows/wasm.yml/badge.svg)](https://github.com/siegfried/cardano-utxo-wasm/actions/workflows/wasm.yml)

This package wraps UTxO helpers from Rust crates such as [utxo](https://github.com/siegfried/utxo) into WASM so that they can be used by Nodejs and the browsers.

```typescript
import type { Output } from 'cardano-utxo-wasm'
import init, { select } from 'cardano-utxo-wasm'

const output: Output = {
  lovelace: BigInt('10000'),
  assets: [
    { policyId: 'policy1', assetName: 'asset1', quantity: BigInt('1000') },
    { policyId: 'policy2', assetName: 'asset2', quantity: BigInt('1000') }
  ]
}

const input0: Output = {
  id: { hash: "tx0", index: 0 },
  lovelace: BigInt('100000'),
  assets: [
    { policyId: 'policy1', assetName: 'asset1', quantity: BigInt('1000') },
    { policyId: 'policy3', assetName: 'asset3', quantity: BigInt('1000') },
    { policyId: 'policy4', assetName: 'asset4', quantity: BigInt('1000') }
  ]
}

const input1: Output = {
  id: { hash: "tx1", index: 1 },
  lovelace: BigInt('1000'),
  assets: [
    { policyId: 'policy1', assetName: 'asset1', quantity: BigInt('2000') },
    { policyId: 'policy2', assetName: 'asset2', quantity: BigInt('2000') }
  ]
}

const input2: Output = {
  id: { hash: "tx1", index: 1 },
  lovelace: BigInt('10000'),
  assets: []
}

init().then(() => {
  console.log(select([input0, input1, input2], [output], { lovelace: BigInt('0'), assets: [] }))
})
```
