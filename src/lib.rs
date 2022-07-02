/*!
UTxO helpers for Cardano in WASM

This package wraps UTxO helpers written in Rust into WASM
so that they can be used by Nodejs and the browsers.
*/
use js_sys::{try_iter, Array, BigInt, Object, Reflect};
use std::collections::BTreeMap;
use utxo::{try_sum, ExtOutput};
use wasm_bindgen::{prelude::*, JsCast};

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type TransactionID = {
  hash: string
  index: number
}

export type Asset = {
  policyId: string
  assetName: string
  quantity: bigint
}

export type Output = {
  id?: TransactionID
  lovelace: bigint
  assets: Array<Asset>
}

export type SelectResult = {
  selected: Array<Output>
  unselected: Array<Output>
  excess: Output
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "TransactionID")]
    type JsTransactionID;

    #[wasm_bindgen(method, getter)]
    fn hash(this: &JsTransactionID) -> String;

    #[wasm_bindgen(method, getter)]
    fn index(this: &JsTransactionID) -> u32;

    #[wasm_bindgen(typescript_type = "Asset")]
    type JsAsset;

    #[wasm_bindgen(method, getter = policyId)]
    fn policy_id(this: &JsAsset) -> String;

    #[wasm_bindgen(method, getter = assetName)]
    fn asset_name(this: &JsAsset) -> String;

    #[wasm_bindgen(method, getter)]
    fn quantity(this: &JsAsset) -> u64;

    #[wasm_bindgen(typescript_type = "Array<Asset>")]
    type JsAssetArray;

    #[derive(Clone)]
    #[wasm_bindgen(typescript_type = "Output")]
    pub type JsOutput;

    #[wasm_bindgen(method, getter)]
    fn id(this: &JsOutput) -> Option<JsTransactionID>;

    #[wasm_bindgen(method, getter)]
    fn lovelace(this: &JsOutput) -> u64;

    #[wasm_bindgen(method, getter)]
    fn assets(this: &JsOutput) -> JsAssetArray;

    #[wasm_bindgen(typescript_type = "Array<Output>")]
    pub type JsOutputArray;

    #[wasm_bindgen(typescript_type = "SelectResult")]
    pub type SelectResult;

    #[wasm_bindgen(method, getter)]
    fn selected(this: &SelectResult) -> JsOutputArray;

    #[wasm_bindgen(method, getter)]
    fn unselected(this: &SelectResult) -> JsOutputArray;

    #[wasm_bindgen(method, getter)]
    fn excess(this: &SelectResult) -> JsOutput;
}

type TransactionID = (String, u32);
type AssetID = (String, String);
pub type Output = ExtOutput<TransactionID, AssetID>;

impl From<JsTransactionID> for TransactionID {
    fn from(value: JsTransactionID) -> Self {
        (value.hash(), value.index())
    }
}

impl From<TransactionID> for JsTransactionID {
    fn from(value: TransactionID) -> Self {
        let object = Object::new();

        Reflect::set(&object, &"hash".into(), &value.0.into()).unwrap();
        Reflect::set(&object, &"index".into(), &value.1.into()).unwrap();

        object.unchecked_into()
    }
}

impl From<JsOutput> for Output {
    fn from(value: JsOutput) -> Self {
        let mut output = Self {
            id: value.id().and_then(|i| Some(i.into())),
            value: value.lovelace(),
            assets: BTreeMap::new(),
        };

        if let Some(assets) = try_iter(&value.assets()).unwrap() {
            for result in assets {
                let asset: JsAsset = result.unwrap().unchecked_into();
                output.insert_asset((asset.policy_id(), asset.asset_name()), asset.quantity());
            }
        }

        output
    }
}

impl From<Output> for JsOutput {
    fn from(value: Output) -> Self {
        let object = Object::new();

        if let Some(id) = value.id {
            let id: JsTransactionID = id.into();
            Reflect::set(&object, &"id".into(), &id).unwrap();
        }

        Reflect::set(
            &object,
            &JsValue::from_str(&"lovelace"),
            &value.value.into(),
        )
        .unwrap();

        let assets = Array::new();
        for ((policy_id, asset_name), quantity) in value.assets.into_iter() {
            let object = Object::new();

            Reflect::set(&object, &"policyId".into(), &policy_id.into()).unwrap();
            Reflect::set(&object, &"assetName".into(), &asset_name.into()).unwrap();
            Reflect::set(&object, &"quantity".into(), &BigInt::from(quantity)).unwrap();

            assets.push(&object);
        }
        Reflect::set(&object, &"assets".into(), &assets).unwrap();

        object.unchecked_into()
    }
}

/**
Select UTxOs for the outputs

Returns an object contains selected UTxOs, unselected UTxOs and the excess output
to pay the fee and return the change.
The excess output will be larger than or equal to the threshold argument.

Returns nothing if the inputs are not enough for the outputs plus threshold.

Raises errors when the types used are wrong.
*/
#[wasm_bindgen]
pub fn select(
    inputs: &JsOutputArray,
    outputs: &JsOutputArray,
    threshold: &JsOutput,
) -> Result<Option<SelectResult>, JsError> {
    let mut inputs: Vec<Output> = try_iter(inputs)
        .unwrap()
        .unwrap()
        .into_iter()
        .map(|i| i.unwrap().unchecked_into::<JsOutput>().into())
        .collect();
    let outputs: Vec<Output> = try_iter(outputs)
        .unwrap()
        .unwrap()
        .into_iter()
        .map(|i| i.unwrap().unchecked_into::<JsOutput>().into())
        .collect();
    let threshold: Output = threshold.clone().into();
    let total_output: Output =
        try_sum(&outputs).ok_or_else(|| JsError::new(&"Outputs overflowed"))?;

    Ok(
        utxo::select(&mut inputs[..], &total_output, &threshold).and_then(
            |(selected, unselected, excess)| {
                let result = Object::new();

                let selected: JsOutputArray = {
                    let result = Array::new();

                    for output in selected {
                        let js_output: JsOutput = output.clone().into();
                        result.push(&js_output);
                    }

                    result.unchecked_into()
                };

                let unselected: JsOutputArray = {
                    let result = Array::new();

                    for output in unselected {
                        let js_output: JsOutput = output.clone().into();
                        result.push(&js_output);
                    }

                    result.unchecked_into()
                };

                let excess: JsOutput = excess.into();

                Reflect::set(&result, &"selected".into(), &selected.into()).unwrap();
                Reflect::set(&result, &"unselected".into(), &unselected.into()).unwrap();
                Reflect::set(&result, &"excess".into(), &excess.into()).unwrap();

                Some(result.unchecked_into())
            },
        ),
    )
}

#[cfg(test)]
mod tests {
    use crate::{select, JsOutput, JsOutputArray, Output, SelectResult};
    use js_sys::{try_iter, Array};
    use std::collections::BTreeMap;
    use utxo::Select;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_output_converting() {
        let mut output = Output {
            id: Some(("hash1".into(), 1)),
            value: 1000,
            assets: BTreeMap::new(),
        };
        output.insert_asset(("policy1".into(), "aname1".into()), 10000);
        output.insert_asset(("policy2".into(), "aname2".into()), 100000);
        let js_output: JsOutput = output.clone().into();
        let result: Output = js_output.into();

        assert_eq!(output, result);
    }

    #[wasm_bindgen_test]
    fn test_output_select() {
        let outputs: JsOutputArray = {
            let result = Array::new();

            let mut output = Output {
                id: None,
                value: 1000,
                assets: BTreeMap::new(),
            };
            output.insert_asset(("policy1".into(), "aname1".into()), 10000);
            output.insert_asset(("policy2".into(), "aname2".into()), 100000);

            let output: JsOutput = output.into();
            result.push(&output);

            let output = Output {
                id: None,
                value: 5000,
                assets: BTreeMap::new(),
            };

            let output: JsOutput = output.into();
            result.push(&output);

            result.unchecked_into()
        };

        let inputs: JsOutputArray = {
            let result = Array::new();

            let mut output = Output {
                id: Some(("hash1".into(), 1)),
                value: 10000,
                assets: BTreeMap::new(),
            };
            output.insert_asset(("policy3".into(), "aname1".into()), 10000);
            output.insert_asset(("policy4".into(), "aname2".into()), 100000);

            let output: JsOutput = output.into();
            result.push(&output);

            let mut output = Output {
                id: Some(("hash2".into(), 2)),
                value: 200,
                assets: BTreeMap::new(),
            };
            output.insert_asset(("policy1".into(), "aname1".into()), 20000);
            output.insert_asset(("policy2".into(), "aname2".into()), 200000);

            let output: JsOutput = output.into();
            result.push(&output);

            let output = Output {
                id: Some(("hash3".into(), 3)),
                value: 7000,
                assets: BTreeMap::new(),
            };

            let output: JsOutput = output.into();
            result.push(&output);

            result.unchecked_into()
        };

        let threshold: JsOutput = Output::zero().into();

        let select_result = select(&inputs, &outputs, &threshold);
        assert!(select_result.is_ok());

        if let Ok(select_result) = select_result {
            let result: SelectResult = select_result.unwrap();
            let selected: Vec<JsOutput> = {
                let list = result.selected();
                try_iter(&list)
                    .unwrap()
                    .unwrap()
                    .into_iter()
                    .map(|o| o.unwrap().unchecked_into())
                    .collect()
            };
            let unselected: Vec<JsOutput> = {
                let list = result.unselected();
                try_iter(&list)
                    .unwrap()
                    .unwrap()
                    .into_iter()
                    .map(|o| o.unwrap().unchecked_into())
                    .collect()
            };
            let excess: JsOutput = result.excess();

            assert_eq!(selected.len(), 2);
            assert_eq!(selected[0].id().unwrap().hash(), "hash2");
            assert_eq!(selected[0].id().unwrap().index(), 2);
            assert_eq!(selected[0].lovelace(), 200);
            assert_eq!(selected[1].id().unwrap().hash(), "hash3");
            assert_eq!(selected[1].id().unwrap().index(), 3);
            assert_eq!(selected[1].lovelace(), 7000);
            assert_eq!(unselected.len(), 1);
            assert_eq!(excess.lovelace(), 1200);
        }
    }
}
