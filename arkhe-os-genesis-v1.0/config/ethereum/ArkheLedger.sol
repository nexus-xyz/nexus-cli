// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract ArkheLedger {
    struct NodeRecord {
        string nodeId;
        uint256 coherence; // scaled by 1e18
        uint256 satoshi;
        uint256 timestamp;
    }

    mapping(string => NodeRecord[]) public history;

    event StateRecorded(string indexed nodeId, uint256 coherence, uint256 satoshi, uint256 timestamp);

    function recordState(string memory nodeId, uint256 coherence, uint256 satoshi) public {
        NodeRecord memory rec = NodeRecord(nodeId, coherence, satoshi, block.timestamp);
        history[nodeId].push(rec);
        emit StateRecorded(nodeId, coherence, satoshi, block.timestamp);
    }

    function getLastState(string memory nodeId) public view returns (uint256 coherence, uint256 satoshi, uint256 timestamp) {
        require(history[nodeId].length > 0, "No records");
        NodeRecord memory rec = history[nodeId][history[nodeId].length - 1];
        return (rec.coherence, rec.satoshi, rec.timestamp);
    }
}
