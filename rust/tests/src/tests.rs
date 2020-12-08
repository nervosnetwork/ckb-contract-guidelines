use super::*;
use blake2b_rs::Blake2bBuilder;
use ckb_standalone_debugger::transaction::{
    MockCellDep, MockInfo, MockInput, MockTransaction, ReprMockTransaction,
};
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{
    bytes::Bytes,
    core::{DepType, TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_x64_simulator::RunningSetup;
use rand::{thread_rng, Rng};
use serde_json::to_string_pretty;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn random_32bytes() -> Bytes {
    let mut rng = thread_rng();
    let mut buf = vec![0u8; 32];
    rng.fill(&mut buf[..]);
    Bytes::from(buf)
}

pub fn create_test_folder(name: &str) -> PathBuf {
    let mut path = TX_FOLDER.clone();
    path.push(&name);
    fs::create_dir_all(&path).expect("create folder");
    path
}

pub fn build_mock_transaction(tx: &TransactionView, context: &Context) -> MockTransaction {
    let mock_inputs = tx
        .inputs()
        .into_iter()
        .map(|input| {
            let (output, data) = context
                .get_cell(&input.previous_output())
                .expect("get cell");
            MockInput {
                input,
                output,
                data,
                header: None,
            }
        })
        .collect();
    let mock_cell_deps = tx
        .cell_deps()
        .into_iter()
        .map(|cell_dep| {
            if cell_dep.dep_type() == DepType::DepGroup.into() {
                panic!("Implement dep group support later!");
            }
            let (output, data) = context.get_cell(&cell_dep.out_point()).expect("get cell");
            MockCellDep {
                cell_dep,
                output,
                data,
                header: None,
            }
        })
        .collect();
    let mock_info = MockInfo {
        inputs: mock_inputs,
        cell_deps: mock_cell_deps,
        header_deps: vec![],
    };
    MockTransaction {
        mock_info,
        tx: tx.data(),
    }
}

pub fn write_native_setup(
    test_name: &str,
    binary_name: &str,
    tx: &TransactionView,
    context: &Context,
    setup: &RunningSetup,
) {
    let folder = create_test_folder(test_name);
    let mock_tx = build_mock_transaction(&tx, &context);
    let repr_tx: ReprMockTransaction = mock_tx.into();
    let tx_json = to_string_pretty(&repr_tx).expect("serialize to json");
    fs::write(folder.join("tx.json"), tx_json).expect("write tx to local file");
    let setup_json = to_string_pretty(setup).expect("serialize to json");
    fs::write(folder.join("setup.json"), setup_json).expect("write setup to local file");
    fs::write(
        folder.join("cmd"),
        format!(
            "CKB_TX_FILE=\"{}\" CKB_RUNNING_SETUP=\"{}\" \"{}\"",
            folder.join("tx.json").to_str().expect("utf8"),
            folder.join("setup.json").to_str().expect("utf8"),
            Loader::default().path(binary_name).to_str().expect("utf8")
        ),
    )
    .expect("write cmd to local file");
}

const MAX_CYCLES: u64 = 10_000_000;

#[test]
fn test_nft_transfer() {
    // deploy contract
    let mut context = Context::default();
    let nft_bin: Bytes = Loader::default().load_binary("nft-validator");
    let nft_out_point = context.deploy_cell(nft_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let lock_script = context
        .build_script(&always_success_out_point, random_32bytes())
        .expect("lock script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();
    let lock_script2 = context
        .build_script(&always_success_out_point, random_32bytes())
        .expect("lock script");
    let governance_script = context
        .build_script(&always_success_out_point, random_32bytes())
        .expect("lock script");
    let governance_script_hash = governance_script.calc_script_hash();
    let nft_type_script = context
        .build_script(&nft_out_point, governance_script_hash.raw_data())
        .expect("script");
    let nft_script_dep = CellDep::new_builder()
        .out_point(nft_out_point.clone())
        .build();

    // prepare cells
    let nft_id = random_32bytes();
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .type_(
                ScriptOpt::new_builder()
                    .set(Some(nft_type_script.clone()))
                    .build(),
            )
            .build(),
        nft_id.clone(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![CellOutput::new_builder()
        .capacity(999u64.pack())
        .lock(lock_script2.clone())
        .type_(
            ScriptOpt::new_builder()
                .set(Some(nft_type_script.clone()))
                .build(),
        )
        .build()];

    let outputs_data = vec![nft_id];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(nft_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: false,
        script_index: 0,
        native_binaries: HashMap::default(),
    };
    write_native_setup("nft_transfer", "nft-validator-sim", &tx, &context, &setup);
}

#[test]
fn test_nft_generation() {
    // deploy contract
    let mut context = Context::default();
    let nft_bin: Bytes = Loader::default().load_binary("nft-validator");
    let nft_out_point = context.deploy_cell(nft_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let lock_script = context
        .build_script(&always_success_out_point, random_32bytes())
        .expect("lock script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();
    let governance_script = context
        .build_script(&always_success_out_point, random_32bytes())
        .expect("lock script");
    let governance_script_hash = governance_script.calc_script_hash();
    let nft_type_script = context
        .build_script(&nft_out_point, governance_script_hash.raw_data())
        .expect("script");
    let nft_script_dep = CellDep::new_builder()
        .out_point(nft_out_point.clone())
        .build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10000u64.pack())
            .lock(governance_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let mut blake2b = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    blake2b.update(input.as_slice());
    blake2b.update(&1u64.to_le_bytes());
    let mut hash = vec![0u8; 32];
    blake2b.finalize(&mut hash[..]);
    let nft_id = Bytes::from(hash);

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(9500u64.pack())
            .lock(governance_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(499u64.pack())
            .lock(lock_script.clone())
            .type_(
                ScriptOpt::new_builder()
                    .set(Some(nft_type_script.clone()))
                    .build(),
            )
            .build(),
    ];

    let outputs_data = vec![Bytes::new(), nft_id];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(nft_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 1,
        native_binaries: HashMap::default(),
    };
    write_native_setup("nft_generation", "nft-validator-sim", &tx, &context, &setup);
}

#[test]
fn test_nft_invalid_governance() {
    // deploy contract
    let mut context = Context::default();
    let nft_bin: Bytes = Loader::default().load_binary("nft-validator");
    let nft_out_point = context.deploy_cell(nft_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let lock_script = context
        .build_script(&always_success_out_point, random_32bytes())
        .expect("lock script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();
    let nft_type_script = context
        .build_script(&nft_out_point, random_32bytes())
        .expect("script");
    let nft_script_dep = CellDep::new_builder()
        .out_point(nft_out_point.clone())
        .build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10000u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let mut blake2b = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    blake2b.update(input.as_slice());
    blake2b.update(&1u64.to_le_bytes());
    let mut hash = vec![0u8; 32];
    blake2b.finalize(&mut hash[..]);
    let nft_id = Bytes::from(hash);

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(9500u64.pack())
            .lock(lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(499u64.pack())
            .lock(lock_script.clone())
            .type_(
                ScriptOpt::new_builder()
                    .set(Some(nft_type_script.clone()))
                    .build(),
            )
            .build(),
    ];

    let outputs_data = vec![Bytes::new(), nft_id];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(nft_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    assert!(context.verify_tx(&tx, MAX_CYCLES).is_err());

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 1,
        native_binaries: HashMap::default(),
    };
    write_native_setup("nft_invalid_governance_failure", "nft-validator-sim", &tx, &context, &setup);
}

#[test]
fn test_nft_invalid_nft_data() {
    // deploy contract
    let mut context = Context::default();
    let nft_bin: Bytes = Loader::default().load_binary("nft-validator");
    let nft_out_point = context.deploy_cell(nft_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let lock_script = context
        .build_script(&always_success_out_point, random_32bytes())
        .expect("lock script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();
    let governance_script = context
        .build_script(&always_success_out_point, random_32bytes())
        .expect("lock script");
    let governance_script_hash = governance_script.calc_script_hash();
    let nft_type_script = context
        .build_script(&nft_out_point, governance_script_hash.raw_data())
        .expect("script");
    let nft_script_dep = CellDep::new_builder()
        .out_point(nft_out_point.clone())
        .build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10000u64.pack())
            .lock(governance_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let mut blake2b = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    blake2b.update(input.as_slice());
    blake2b.update(&1u64.to_le_bytes());
    let mut hash = vec![0u8; 32];
    blake2b.finalize(&mut hash[..]);
    let nft_id = Bytes::from(hash);

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(9500u64.pack())
            .lock(governance_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(499u64.pack())
            .lock(lock_script.clone())
            .type_(
                ScriptOpt::new_builder()
                    .set(Some(nft_type_script.clone()))
                    .build(),
            )
            .build(),
    ];

    let outputs_data = vec![Bytes::new(), nft_id.slice(0..16)];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(nft_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    assert!(context.verify_tx(&tx, MAX_CYCLES).is_err());

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 1,
        native_binaries: HashMap::default(),
    };
    write_native_setup("nft_invalid_nft_data_failure", "nft-validator-sim", &tx, &context, &setup);
}

#[test]
fn test_nft_invalid_nft_hash() {
    // deploy contract
    let mut context = Context::default();
    let nft_bin: Bytes = Loader::default().load_binary("nft-validator");
    let nft_out_point = context.deploy_cell(nft_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let lock_script = context
        .build_script(&always_success_out_point, random_32bytes())
        .expect("lock script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point.clone())
        .build();
    let governance_script = context
        .build_script(&always_success_out_point, random_32bytes())
        .expect("lock script");
    let governance_script_hash = governance_script.calc_script_hash();
    let nft_type_script = context
        .build_script(&nft_out_point, governance_script_hash.raw_data())
        .expect("script");
    let nft_script_dep = CellDep::new_builder()
        .out_point(nft_out_point.clone())
        .build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(10000u64.pack())
            .lock(governance_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();

    let mut blake2b = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    blake2b.update(input.as_slice());
    blake2b.update(&1u64.to_le_bytes());
    let mut hash = vec![0u8; 32];
    blake2b.finalize(&mut hash[..]);
    hash[0] += 1;
    let nft_id = Bytes::from(hash);

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(9500u64.pack())
            .lock(governance_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(499u64.pack())
            .lock(lock_script.clone())
            .type_(
                ScriptOpt::new_builder()
                    .set(Some(nft_type_script.clone()))
                    .build(),
            )
            .build(),
    ];

    let outputs_data = vec![Bytes::new(), nft_id];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(nft_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    assert!(context.verify_tx(&tx, MAX_CYCLES).is_err());

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 1,
        native_binaries: HashMap::default(),
    };
    write_native_setup("nft_invalid_nft_hash_failure", "nft-validator-sim", &tx, &context, &setup);
}
