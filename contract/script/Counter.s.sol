// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script} from "forge-std/Script.sol";
import {Test1} from "../src/Counter.sol"; // Correct the import path

contract CounterScript is Script {
    Test1 public counter;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        // Deploy Test1 contract
        counter = new Test1();

        vm.stopBroadcast();
    }
}
