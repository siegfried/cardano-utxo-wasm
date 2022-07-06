/*!
UTxO helpers for Cardano in WASM

This package wraps UTxO helpers written in Rust into WASM
so that they can be used by Nodejs and the browsers.
*/
use js_sys::{try_iter, Array, Object};
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

    #[wasm_bindgen(method, setter)]
    fn set_hash(this: &JsTransactionID, hash: &str);

    #[wasm_bindgen(method, getter)]
    fn index(this: &JsTransactionID) -> u32;

    #[wasm_bindgen(method, setter)]
    fn set_index(this: &JsTransactionID, index: u32);

    #[wasm_bindgen(typescript_type = "Asset")]
    type JsAsset;

    #[wasm_bindgen(method, getter = policyId)]
    fn policy_id(this: &JsAsset) -> String;

    #[wasm_bindgen(method, setter = policyId)]
    fn set_policy_id(this: &JsAsset, policy_id: &str);

    #[wasm_bindgen(method, getter = assetName)]
    fn asset_name(this: &JsAsset) -> String;

    #[wasm_bindgen(method, setter = assetName)]
    fn set_asset_name(this: &JsAsset, asset_name: &str);

    #[wasm_bindgen(method, getter)]
    fn quantity(this: &JsAsset) -> u64;

    #[wasm_bindgen(method, setter)]
    fn set_quantity(this: &JsAsset, quantity: u64);

    #[wasm_bindgen(typescript_type = "Array<Asset>")]
    type JsAssetArray;

    #[wasm_bindgen(typescript_type = "Output")]
    pub type JsOutput;

    #[wasm_bindgen(method, getter)]
    fn id(this: &JsOutput) -> Option<JsTransactionID>;

    #[wasm_bindgen(method, setter)]
    fn set_id(this: &JsOutput, id: &JsTransactionID);

    #[wasm_bindgen(method, getter)]
    fn lovelace(this: &JsOutput) -> u64;

    #[wasm_bindgen(method, setter)]
    fn set_lovelace(this: &JsOutput, lovelace: u64);

    #[wasm_bindgen(method, getter)]
    fn assets(this: &JsOutput) -> JsAssetArray;

    #[wasm_bindgen(method, setter)]
    fn set_assets(this: &JsOutput, assets: &JsAssetArray);

    #[wasm_bindgen(typescript_type = "Array<Output>")]
    pub type JsOutputArray;

    #[wasm_bindgen(typescript_type = "SelectResult")]
    pub type SelectResult;

    #[wasm_bindgen(method, getter)]
    fn selected(this: &SelectResult) -> JsOutputArray;

    #[wasm_bindgen(method, setter)]
    fn set_selected(this: &SelectResult, selected: &JsOutputArray);

    #[wasm_bindgen(method, getter)]
    fn unselected(this: &SelectResult) -> JsOutputArray;

    #[wasm_bindgen(method, setter)]
    fn set_unselected(this: &SelectResult, unselected: &JsOutputArray);

    #[wasm_bindgen(method, getter)]
    fn excess(this: &SelectResult) -> JsOutput;

    #[wasm_bindgen(method, setter)]
    fn set_excess(this: &SelectResult, excess: &JsOutput);
}

pub type Output<'o> = ExtOutput<&'o JsOutput, (String, String)>;

struct Asset<'a> {
    policy_id: &'a str,
    asset_name: &'a str,
    quantity: u64,
}

impl<'a> Asset<'a> {
    fn new(policy_id: &'a str, asset_name: &'a str, quantity: u64) -> Self {
        Self {
            policy_id,
            asset_name,
            quantity,
        }
    }
}

impl From<Asset<'_>> for JsAsset {
    fn from(value: Asset) -> Self {
        let id: Self = Object::new().unchecked_into();
        id.set_policy_id(value.policy_id);
        id.set_asset_name(value.asset_name);
        id.set_quantity(value.quantity);
        id
    }
}

impl<'o> From<&'o JsOutput> for Output<'o> {
    fn from(value: &'o JsOutput) -> Self {
        let mut output = Self {
            value: value.lovelace(),
            assets: BTreeMap::new(),
            data: Some(value),
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

impl From<Output<'_>> for JsOutput {
    fn from(value: Output) -> Self {
        let js_output: Self = Object::new().unchecked_into();

        js_output.set_lovelace(value.value);

        let assets = Array::new();
        for ((policy_id, asset_name), quantity) in value.assets.into_iter() {
            let asset: JsAsset = Asset::new(&policy_id, &asset_name, quantity).into();
            assets.push(&asset);
        }
        let assets: JsAssetArray = assets.unchecked_into();
        js_output.set_assets(&assets);

        js_output
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
    let js_inputs: Vec<JsOutput> = try_iter(inputs)
        .unwrap()
        .unwrap()
        .into_iter()
        .map(|i| i.unwrap().unchecked_into())
        .collect();
    let mut inputs: Vec<Output> = js_inputs.iter().map(|o| o.into()).collect();
    let js_outputs: Vec<JsOutput> = try_iter(outputs)
        .unwrap()
        .unwrap()
        .into_iter()
        .map(|i| i.unwrap().unchecked_into())
        .collect();
    let outputs: Vec<Output> = js_outputs.iter().map(|o| o.into()).collect();
    let threshold: Output = threshold.into();
    let total_output: Output =
        try_sum(&outputs).ok_or_else(|| JsError::new("Outputs overflowed"))?;

    Ok(
        utxo::select(&mut inputs[..], &total_output, &threshold).and_then(
            |(selected, unselected, excess)| {
                let result: SelectResult = Object::new().unchecked_into();

                let selected: JsOutputArray = {
                    let result = Array::new();

                    for output in selected {
                        result.push(output.data.expect("Unreachable"));
                    }

                    result.unchecked_into()
                };

                let unselected: JsOutputArray = {
                    let result = Array::new();

                    for output in unselected {
                        result.push(output.data.expect("Unreachable"));
                    }

                    result.unchecked_into()
                };

                let excess: JsOutput = excess.into();

                result.set_selected(&selected);
                result.set_unselected(&unselected);
                result.set_excess(&excess);

                Some(result)
            },
        ),
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        select, Asset, JsAsset, JsAssetArray, JsOutput, JsOutputArray, JsTransactionID, Output,
        SelectResult,
    };
    use js_sys::{try_iter, Array, Object};
    use std::collections::BTreeMap;
    use utxo::Select;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    struct TransactionID<'a> {
        hash: &'a str,
        index: u32,
    }

    impl<'a> TransactionID<'a> {
        fn new(hash: &'a str, index: u32) -> Self {
            Self { hash, index }
        }
    }

    impl From<TransactionID<'_>> for JsTransactionID {
        fn from(value: TransactionID) -> Self {
            let id: Self = Object::new().unchecked_into();
            id.set_hash(value.hash);
            id.set_index(value.index);
            id
        }
    }

    #[wasm_bindgen_test]
    fn test_from_js_output_to_output() {
        let js_output: JsOutput = Object::new().unchecked_into();
        js_output.set_id(&TransactionID::new("hash0", 0).into());
        js_output.set_lovelace(1000);
        let js_assets: JsAssetArray = {
            let assets = Array::new();

            let asset: JsAsset = Asset::new("policy1", "aname1", 10000).into();
            assets.push(&asset);

            let asset: JsAsset = Asset::new("policy2", "aname2", 100000).into();
            assets.push(&asset);

            assets.unchecked_into()
        };
        js_output.set_assets(&js_assets);

        assert_eq!(js_output.lovelace(), 1000);
        assert_eq!(js_output.id().unwrap().hash(), "hash0".to_string());
        assert_eq!(js_output.id().unwrap().index(), 0);
        assert_eq!(js_output.assets().unchecked_into::<Array>().length(), 2);

        let output: Output = (&js_output).into();

        assert_eq!(output.value, 1000);
        assert_eq!(output.assets.len(), 2);
        assert_eq!(
            output
                .assets
                .get(&("policy1".into(), "aname1".into()))
                .unwrap(),
            &10000
        );
        assert_eq!(
            output
                .assets
                .get(&("policy2".into(), "aname2".into()))
                .unwrap(),
            &100000
        );
        assert!(output.data.unwrap().loose_eq(&js_output));
    }

    #[wasm_bindgen_test]
    fn test_from_output_to_js_output() {
        let mut output = Output {
            value: 1000,
            assets: BTreeMap::new(),
            data: None,
        };
        output.insert_asset(("policy1".into(), "aname1".into()), 10000);
        output.insert_asset(("policy2".into(), "aname2".into()), 100000);

        let js_output: JsOutput = output.into();

        assert_eq!(js_output.lovelace(), 1000);
        assert_eq!(js_output.assets().unchecked_into::<Array>().length(), 2);
        assert!(js_output.id().is_none());
    }

    #[wasm_bindgen_test]
    fn test_output_select() {
        let outputs: JsOutputArray = {
            let result = Array::new();

            let mut output = Output {
                value: 1000,
                assets: BTreeMap::new(),
                data: None,
            };
            output.insert_asset(("policy1".into(), "aname1".into()), 10000);
            output.insert_asset(("policy2".into(), "aname2".into()), 100000);

            let output: JsOutput = output.into();
            result.push(&output);

            let output = Output {
                value: 5000,
                assets: BTreeMap::new(),
                data: None,
            };

            let output: JsOutput = output.into();
            result.push(&output);

            result.unchecked_into()
        };

        let inputs: JsOutputArray = {
            let result = Array::new();

            let mut output = Output {
                value: 10000,
                assets: BTreeMap::new(),
                data: None,
            };
            output.insert_asset(("policy3".into(), "aname1".into()), 10000);
            output.insert_asset(("policy4".into(), "aname2".into()), 100000);

            let output: JsOutput = output.into();
            output.set_id(&TransactionID::new("hash1", 1).into());
            result.push(&output);

            let mut output = Output {
                value: 200,
                assets: BTreeMap::new(),
                data: None,
            };
            output.insert_asset(("policy1".into(), "aname1".into()), 20000);
            output.insert_asset(("policy2".into(), "aname2".into()), 200000);

            let output: JsOutput = output.into();
            output.set_id(&TransactionID::new("hash2", 2).into());
            result.push(&output);

            let output = Output {
                value: 7000,
                assets: BTreeMap::new(),
                data: None,
            };

            let output: JsOutput = output.into();
            output.set_id(&TransactionID::new("hash3", 3).into());
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
