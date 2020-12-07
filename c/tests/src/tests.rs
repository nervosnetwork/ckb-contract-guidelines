use super::*;
use blake2b_ref::Blake2bBuilder;
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

pub fn ckb_hash(data: &[u8]) -> Bytes {
    let mut blake2b = Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build();
    blake2b.update(data);
    let mut hash = vec![0u8; 32];
    blake2b.finalize(&mut hash[..]);
    Bytes::from(hash)
}

pub fn random_32bytes() -> Bytes {
    let mut rng = thread_rng();
    let mut buf = vec![0u8; 32];
    rng.fill(&mut buf[..]);
    Bytes::from(buf)
}

pub fn amount_to_data(amount: u128) -> Bytes {
    let data = amount.to_le_bytes();
    Bytes::from(data[..].to_vec())
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
fn test_sudt_transfer() {
    // deploy contract
    let mut context = Context::default();
    let sudt_bin: Bytes = Loader::default().load_binary("simple_udt.strip");
    let sudt_out_point = context.deploy_cell(sudt_bin);
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
    let sudt_type_script = context
        .build_script(&sudt_out_point, governance_script_hash.raw_data())
        .expect("script");
    let sudt_script_dep = CellDep::new_builder()
        .out_point(sudt_out_point.clone())
        .build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .type_(
                ScriptOpt::new_builder()
                    .set(Some(sudt_type_script.clone()))
                    .build(),
            )
            .build(),
        amount_to_data(100),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![CellOutput::new_builder()
        .capacity(999u64.pack())
        .lock(lock_script2.clone())
        .type_(
            ScriptOpt::new_builder()
                .set(Some(sudt_type_script.clone()))
                .build(),
        )
        .build()];

    let outputs_data = vec![amount_to_data(100)];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(sudt_script_dep)
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
    write_native_setup("sudt_transfer", "simple_udt_sim", &tx, &context, &setup);
}

#[test]
fn test_sudt_transfer_failure() {
    // deploy contract
    let mut context = Context::default();
    let sudt_bin: Bytes = Loader::default().load_binary("simple_udt.strip");
    let sudt_out_point = context.deploy_cell(sudt_bin);
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
    let sudt_type_script = context
        .build_script(&sudt_out_point, governance_script_hash.raw_data())
        .expect("script");
    let sudt_script_dep = CellDep::new_builder()
        .out_point(sudt_out_point.clone())
        .build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .type_(
                ScriptOpt::new_builder()
                    .set(Some(sudt_type_script.clone()))
                    .build(),
            )
            .build(),
        amount_to_data(100),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![CellOutput::new_builder()
        .capacity(999u64.pack())
        .lock(lock_script2.clone())
        .type_(
            ScriptOpt::new_builder()
                .set(Some(sudt_type_script.clone()))
                .build(),
        )
        .build()];

    let outputs_data = vec![amount_to_data(110)];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(sudt_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    context
        .verify_tx(&tx, MAX_CYCLES)
        .expect_err("fail verification");

    // dump raw test tx files
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: false,
        script_index: 0,
        native_binaries: HashMap::default(),
    };
    write_native_setup(
        "sudt_transfer_failure",
        "simple_udt_sim",
        &tx,
        &context,
        &setup,
    );
}

#[test]
fn test_dynamic_linking_ok() {
    // deploy contract
    let mut context = Context::default();
    let sample_bin: Bytes = Loader::default().load_binary("bin_sample.strip");
    let sample_bin_out_point = context.deploy_cell(sample_bin);
    let sample_lib: Bytes = Loader::default().load_binary("lib_sample.strip");
    let sample_lib_data_hash = ckb_hash(&sample_lib);
    let sample_lib_out_point = context.deploy_cell(sample_lib);
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
    let sample_bin_script_dep = CellDep::new_builder()
        .out_point(sample_bin_out_point.clone())
        .build();
    let sample_lib_script_dep = CellDep::new_builder()
        .out_point(sample_lib_out_point.clone())
        .build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::default(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let input_data = {
        let mut hash_data = vec![];
        hash_data.extend(input.as_slice());
        hash_data.extend(&0u64.to_le_bytes());
        let hash = ckb_hash(&hash_data);
        let mut data = vec![0u8; 34];
        data[2..].copy_from_slice(&hash);
        Bytes::from(data)
    };
    let dl_script = context
        .build_script(&sample_bin_out_point, input_data)
        .expect("dl script");
    let output = CellOutput::new_builder()
        .capacity(999u64.pack())
        .lock(lock_script2.clone())
        .type_(
            ScriptOpt::new_builder()
                .set(Some(dl_script.clone()))
                .build(),
        )
        .build();
    let output_data = {
        let mut data = vec![0u8; 33];
        data[..32].copy_from_slice(&sample_lib_data_hash);
        Bytes::from(data)
    };

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .output(output)
        .output_data(output_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(sample_bin_script_dep)
        .cell_dep(sample_lib_script_dep)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);

    // dump raw test tx files
    let mut native_binaries = HashMap::default();
    native_binaries.insert(
        format!("0x{:x}", output_data),
        Loader::default()
            .path("lib_sample_sim.so")
            .to_str()
            .expect("invalid path")
            .to_string(),
    );
    let setup = RunningSetup {
        is_lock_script: false,
        is_output: true,
        script_index: 0,
        native_binaries,
    };
    write_native_setup(
        "dynamic_linking_ok",
        "bin_sample_sim",
        &tx,
        &context,
        &setup,
    );
}
