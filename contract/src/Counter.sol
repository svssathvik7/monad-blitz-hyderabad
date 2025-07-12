// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract Test1 {
    event TransferTo(
        address indexed sender,
        bytes32 indexed receiver, // use bytes32 for indexed
        bytes32 indexed cid       // use bytes32 for indexed
    );

    function transfer(string calldata receiver, string calldata cid) external {
        // Hash the receiver and CID to bytes32 to emit as indexed params
        bytes32 receiverHash = keccak256(abi.encodePacked(receiver));
        bytes32 cidHash = keccak256(abi.encodePacked(cid));

        emit TransferTo(msg.sender, receiverHash, cidHash);
    }
}
