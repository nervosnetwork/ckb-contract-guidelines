# ckb-contract-guidelines

This repository provides guidelines for writing secure CKB smart contracts. It consists of guidelines, or recommendations kept in this README file, as well as sample C and Rust based smart contracts organized following the guidelines.

Note we still recommend a full code audit for any smart contracts deployed to CKB, even though you might have followed all the guidelines here. The purpose of this repository, is to put together some low hanging fruits that can be done before a security audit, to make sure the code is already in a high quality state, thus minimizing efforts required in a formal audit.

# Guideline Version 1

## Rule 1: Use defined APIs

Smart contracts must be coded against the [publicly defined set of APIs](https://github.com/nervosnetwork/ckb-c-stdlib/blob/23c85c7588b56f29f15dc7002b2e485d0e6df251/ckb_syscall_apis.h). It's okay to build abstractions on top of the APIs, such as [checked functions in C](https://github.com/nervosnetwork/ckb-c-stdlib/blob/23c85c7588b56f29f15dc7002b2e485d0e6df251/ckb_syscalls.h#L11-L126). Rust might also have its own abstractions. However the defined APIs must be the sole way a smart contract interacts with CKB, no other ways of making syscalls are allowed. For example, a smart contract shall not make calls to [ckb_load_cell_data_as_code](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0009-vm-syscalls/0009-vm-syscalls.md#load-cell-data-as_code), only the wrapped [ckb_dlopen2](https://github.com/nervosnetwork/ckb-c-stdlib/blob/master/ckb_syscall_apis.h#L36-L38) can be used.

## Rule 2: Multiple binaries for the same source code

A smart contract shall be compiled to 2 different versions of binaries:

* A RISC-V version that is runnable in CKB-VM
* A native x64 version for test coverage and sanitizing purposes. This version should be compiled against [ckb-x64-simulator](https://github.com/nervosnetwork/ckb-x64-simulator), so implementations for the publicly defined APIs can be provided.

This provides a tradeoff now when RISC-V tooling is not mature enough. One day we might reach a state that tooling on RISC-V is as good as x64, but for now, having a native x64 simulator version enables us with new possibilities to scan the smart contract for vulnerabilities.

## Rule 3: 100% test coverage

All smart contracts should have 100% test coverage in terms of both code lines and branches. If you add a line of code, you must make sure there is a test covering the code.

## Rule 4: Multiple execution environment for tests

Each test case should be executed in all of the following environment:

1. Normal CKB-VM as used in CKB
2. At least 20 runs of CKB-VM running in [chaos mode](https://github.com/nervosnetwork/ckb-vm/pull/118)
3. Native x64 environment for gathering test coverage
4. Native x64 environment with the latest stable version of [LLVM Undefined Behavior Sanitizer](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html) enabled
5. Native x64 environment with the latest stable version of [LLVM Address Sanitizer](https://clang.llvm.org/docs/AddressSanitizer.html) enabled

For environment 1-2, the RISC-V binary shall be used. For environment 3-5, the native x64 version compiled against ckb-x64-simulator shall be used.
