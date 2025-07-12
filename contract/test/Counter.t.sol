// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Test1} from "../src/Counter.sol";

contract CounterTest is Test {
    Test1 public counter;
}
