# ckb-contract-guidelines

This repository provides guidelines for writing secure CKB smart contracts. It consists of guidelines, or recommendations kept in this README file, as well as sample C and Rust based smart contracts organized following the guidelines.

Note we still recommend a full code audit for any smart contracts deployed to CKB, even though you might have followed all the guidelines here. The purpose of this repository, is to put together some low hanging fruits that can be done before a security audit, to make sure the code is already in a high quality state, thus minimizing efforts required in a formal audit.

# Guideline Version 1

## Rule 1: 100% test coverage

All smart contracts should have 100% test coverage in terms of both code lines and branches. If you add a line of code, you must make sure there is a test covering the code.

Since coveraging tooling for RISC-V is still in immature phase, several different methodologies can be leveraged. The smart contract author is free to pick any solution.

## Rule 2: Multiple execution environment for tests

Each test case should be executed in all of the following environment:

1. Normal CKB-VM as used in CKB
2. At least 20 runs of CKB-VM running in [chaos mode](https://github.com/nervosnetwork/ckb-vm/pull/118)
3. Native x64 environment for gathering test coverage
4. (For C based smart contract only) Native x64 environment with the latest stable version of [LLVM Undefined Behavior Sanitizer](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html) enabled
5. (For C based smart contract only) Native x64 environment with the latest stable version of [LLVM Address Sanitizer](https://clang.llvm.org/docs/AddressSanitizer.html) enabled

# How To Gather Code Coverage Data

There are several ways to gather code coverage data for smart contracts.

## Native Simulators

Smart contracts can be coded against the [publicly defined set of APIs](https://github.com/nervosnetwork/ckb-c-stdlib/blob/23c85c7588b56f29f15dc7002b2e485d0e6df251/ckb_syscall_apis.h). It's okay to build abstractions on top of the APIs, such as [checked functions in C](https://github.com/nervosnetwork/ckb-c-stdlib/blob/23c85c7588b56f29f15dc7002b2e485d0e6df251/ckb_syscalls.h#L11-L126). Rust might also have its own abstractions. However the defined APIs must be the sole way a smart contract interacts with CKB, no other ways of making syscalls are allowed. For example, a smart contract shall not make calls to [ckb_load_cell_data_as_code](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0009-vm-syscalls/0009-vm-syscalls.md#load-cell-data-as_code), only the wrapped [ckb_dlopen2](https://github.com/nervosnetwork/ckb-c-stdlib/blob/master/ckb_syscall_apis.h#L36-L38) can be used.

As a result, smart contracts following above rules can be compiled to x64 native binary by linking against [ckb-x64-simulator](https://github.com/nervosnetwork/ckb-x64-simulator). The simulator provides native implementations for all the publicly defined APIs above. Hence for each smart contract, we can have 2 binaries:

* A RISC-V version that is runnable in CKB-VM
* A native x64 version for test coverage and sanitizing purposes.

The benefit here, is that all existing toolings on x64 platform can be leverage on the smart contract code. For example, gcov can be used for gathering code coverage data, LLVM sanitizers can be used to discover potential vulnerabilities in C based smart contracts.

In this repository you can find smart contracts tested in this solution.

## Special organization within smart contracts

Smart contracts themselves can be better organized, so test cases can be written on the code directly without CKB-VM environment. [This project](https://github.com/nervosnetwork/force-bridge-eth/blob/2d16aa4ab459ec00d98aa94d110d8ec5791855c8/ckb-contracts/contracts/eth-bridge-typescript/src/main.rs) serves as a decent example in this category.
