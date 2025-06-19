#!/usr/bin/env cargo
/*
[dependencies]
clap = "4.0"
*/

//! Comprehensive test script for tool-meister
//! Run with: cargo run --bin test_runner

use std::process::{exit, Command};

fn main() {
    println!("🧪 Running Tool Meister Comprehensive Test Suite");
    println!("================================================");

    let mut all_passed = true;

    // Run unit tests
    print!("📋 Running unit tests... ");
    let unit_result = Command::new("cargo")
        .args(["test", "--lib", "--quiet"])
        .status()
        .expect("Failed to run unit tests");

    if unit_result.success() {
        println!("✅ PASSED");
    } else {
        println!("❌ FAILED");
        all_passed = false;
    }

    // Run integration tests
    print!("🔗 Running integration tests... ");
    let integration_result = Command::new("cargo")
        .args(["test", "--test", "integration_tests", "--quiet"])
        .status()
        .expect("Failed to run integration tests");

    if integration_result.success() {
        println!("✅ PASSED");
    } else {
        println!("❌ FAILED");
        all_passed = false;
    }

    // Run all tests together
    print!("🎯 Running complete test suite... ");
    let all_result = Command::new("cargo")
        .args(["test", "--quiet"])
        .status()
        .expect("Failed to run all tests");

    if all_result.success() {
        println!("✅ PASSED");
    } else {
        println!("❌ FAILED");
        all_passed = false;
    }

    // Build the project to ensure everything compiles
    print!("🔨 Building project... ");
    let build_result = Command::new("cargo")
        .args(["build", "--quiet"])
        .status()
        .expect("Failed to build project");

    if build_result.success() {
        println!("✅ PASSED");
    } else {
        println!("❌ FAILED");
        all_passed = false;
    }

    println!();
    if all_passed {
        println!("🎉 All tests passed! Your implementation is working correctly.");
        println!("🚀 Next time, just run 'cargo test' to validate your changes quickly.");
        exit(0);
    } else {
        println!("💥 Some tests failed. Please check the output above for details.");
        exit(1);
    }
}
