use super::{App, Summary, InOutMem, Device};
use std::collections::BTreeMap;

#[test]
fn app_de() {
    let app_json = r#"{"name":"PiCtory","version":"2.0.6","saveTS":"20220523193431","language":"en","layout":{}}"#;
    let reference = App {
        name: "PiCtory".to_string(),
        version: "2.0.6".to_string(),
        save_ts: "20220523193431".to_string(),
        language: "en".to_string(),
        layout: serde_json::Value::Object(serde_json::Map::<String, serde_json::Value>::new()),
    };
    let app: App = serde_json::from_str(app_json).unwrap();
    assert_eq!(app, reference);
}

#[test]
fn app_ser() {
    let reference = r#"{"name":"PiCtory","version":"2.0.6","saveTS":"20220523193431","language":"en","layout":{}}"#;
    let app = App {
        name: "PiCtory".to_string(),
        version: "2.0.6".to_string(),
        save_ts: "20220523193431".to_string(),
        language: "en".to_string(),
        layout: serde_json::Value::Object(serde_json::Map::<String, serde_json::Value>::new()),
    };
    let app_json = serde_json::to_string(&app).unwrap();
    assert_eq!(app_json, reference);
}

#[test]
fn summary_de() {
    let summary_json = r#"{"inpTotal":96,"outTotal":27}"#;
    let reference = Summary {
        inp_total: 96,
        out_total: 27,
    };
    let summary: Summary = serde_json::from_str(summary_json).unwrap();
    assert_eq!(summary, reference);
}

#[test]
fn summary_ser() {
    let reference = r#"{"inpTotal":96,"outTotal":27}"#;
    let summary = Summary {
        inp_total: 96,
        out_total: 27,
    };
    let summary_json = serde_json::to_string(&summary).unwrap();
    assert_eq!(summary_json, reference);
}

#[test]
fn inoutmem_de_some() {
    let inoutmem_json = r#"["RevPiStatus","8","8","16",true,"0003", "a comment","0"]"#;
    let reference = InOutMem {
        name: "RevPiStatus".to_string(),
        default: 8,
        bit_length: 8,
        offset: 16,
        exported: true,
        sort_pos: 3,
        comment: "a comment".to_string(),
        bit_position: Some(0),
    };
    let inoutmem: InOutMem = serde_json::from_str(inoutmem_json).unwrap();
    assert_eq!(inoutmem, reference);
}

#[test]
fn inoutmem_de_none() {
    let inoutmem_json = r#"["RevPiStatus","8","8","16",true,"0003", "a comment",""]"#;
    let reference = InOutMem {
        name: "RevPiStatus".to_string(),
        default: 8,
        bit_length: 8,
        offset: 16,
        exported: true,
        sort_pos: 3,
        comment: "a comment".to_string(),
        bit_position: None,
    };
    let inoutmem: InOutMem = serde_json::from_str(inoutmem_json).unwrap();
    assert_eq!(inoutmem, reference);
}

#[test]
fn inoutmem_ser_some() {
    let reference = r#"["RevPiStatus","8","8","16",true,"0003","a comment","0"]"#;
    let inoutmem = InOutMem {
        name: "RevPiStatus".to_string(),
        default: 8,
        bit_length: 8,
        offset: 16,
        exported: true,
        sort_pos: 3,
        comment: "a comment".to_string(),
        bit_position: Some(0),
    };
    let inoutmem_json = serde_json::to_string(&inoutmem).unwrap();
    assert_eq!(inoutmem_json, reference);
}

#[test]
fn inoutmem_ser_none() {
    let reference = r#"["RevPiStatus","8","8","16",true,"0003","a comment",""]"#;
    let inoutmem = InOutMem {
        name: "RevPiStatus".to_string(),
        default: 8,
        bit_length: 8,
        offset: 16,
        exported: true,
        sort_pos: 3,
        comment: "a comment".to_string(),
        bit_position: None,
    };
    let inoutmem_json = serde_json::to_string(&inoutmem).unwrap();
    assert_eq!(inoutmem_json, reference);
}

#[test]
fn device_de() {
    let device_json = r#"{"GUID":"80941337-4242-beed-aaaa-d9df13376969","id":"device_RevPiCore_20220123_4_5_006","type":"BASE","productType":"95","position":"0","name":"RevPi Core/3/3+/S","bmk":"RevPi Core/3/3+/S","inpVariant":0,"outVariant":0,"comment":"This is a RevPiCore Device","offset":42,"inp":{"0":["a","0","8","0",true,"0000","",""],"1":["b","0","8","1",true,"0001","",""]},"out":{},"mem": {},"extend":{}}"#;
    let mut inputs = BTreeMap::new();
    inputs.insert(0, InOutMem {
        name: "a".to_string(),
        default: 0,
        bit_length: 8,
        offset: 0,
        exported: true,
        sort_pos: 0,
        comment: "".to_string(),
        bit_position: None,
    });
    inputs.insert(1, InOutMem {
        name: "b".to_string(),
        default: 0,
        bit_length: 8,
        offset: 1,
        exported: true,
        sort_pos: 1,
        comment: "".to_string(),
        bit_position: None,
    });
    let reference = Device {
        guid: "80941337-4242-beed-aaaa-d9df13376969".to_string(),
        id: "device_RevPiCore_20220123_4_5_006".to_string(),
        dev_type: "BASE".to_string(),
        product_type: 95,
        position: 0,
        name: "RevPi Core/3/3+/S".to_string(),
        bmk: "RevPi Core/3/3+/S".to_string(),
        inp_variant: 0,
        out_variant: 0,
        comment: "This is a RevPiCore Device".to_string(),
        offset: 42,
        inp: inputs,
        out: BTreeMap::new(),
        mem: BTreeMap::new(),
        extend: serde_json::Value::Object(serde_json::Map::<String, serde_json::Value>::new()),
        active: None,
    };
    let device: Device = serde_json::from_str(device_json).unwrap();
    assert_eq!(device, reference);
}

#[test]
fn device_ser() {
    let reference = r#"{"GUID":"80941337-4242-beed-aaaa-d9df13376969","id":"device_RevPiCore_20220123_4_5_006","type":"BASE","productType":"95","position":"0","name":"RevPi Core/3/3+/S","bmk":"RevPi Core/3/3+/S","inpVariant":0,"outVariant":0,"comment":"This is a RevPiCore Device","offset":42,"inp":{"0":["a","0","8","0",true,"0000","",""],"1":["b","0","8","1",true,"0001","",""]},"out":{},"mem": {},"extend":{}}"#;
    let mut inputs = BTreeMap::new();
    inputs.insert(0, InOutMem {
        name: "a".to_string(),
        default: 0,
        bit_length: 8,
        offset: 0,
        exported: true,
        sort_pos: 0,
        comment: "".to_string(),
        bit_position: None,
    });
    inputs.insert(1, InOutMem {
        name: "b".to_string(),
        default: 0,
        bit_length: 8,
        offset: 1,
        exported: true,
        sort_pos: 1,
        comment: "".to_string(),
        bit_position: None,
    });
    let device = Device {
        guid: "80941337-4242-beed-aaaa-d9df13376969".to_string(),
        id: "device_RevPiCore_20220123_4_5_006".to_string(),
        dev_type: "BASE".to_string(),
        product_type: 95,
        position: 0,
        name: "RevPi Core/3/3+/S".to_string(),
        bmk: "RevPi Core/3/3+/S".to_string(),
        inp_variant: 0,
        out_variant: 0,
        comment: "This is a RevPiCore Device".to_string(),
        offset: 42,
        inp: inputs,
        out: BTreeMap::new(),
        mem: BTreeMap::new(),
        extend: serde_json::Value::Object(serde_json::Map::<String, serde_json::Value>::new()),
        active: None,
    };
    let device_json = serde_json::to_string(&device).unwrap();
    assert_eq!(device_json, reference);
}
