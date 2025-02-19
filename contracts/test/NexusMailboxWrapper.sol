// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import {NexusMailbox} from "../src/NexusMailbox.sol";
import {INexusVerifierWrapper} from "../src/interfaces/INexusVerifierWrapper.sol";
import {MailboxMessage, VerifierInfo} from "../src/interfaces/INexusMailbox.sol";

contract NexusMailboxWrapper is NexusMailbox {
    function updateSendMessages(uint256 key, bytes32 value) public {
        messages[bytes32(key)] = true;
    }

    function checkVerificationOfEncoding(
        uint256 chainblockNumber,
        MailboxMessage memory receipt,
        bytes32 from,
        bytes32 receiptHash,
        bytes calldata proof
    ) public {
        VerifierInfo memory verifierInfo = verifierWrappers[from];
        verifierInfo.verifier.parseAndVerify(
            chainblockNumber,
            receiptHash,
            proof,
            verifierInfo.mailboxAddress
        );
        verifiedMessages[keccak256(abi.encode(from, receiptHash))] = receipt;
    }

    function searchWrapper(
        bytes32[] memory chainIdTo,
        address[] memory to
    ) public view returns (address) {
        return search(chainIdTo, to);
    }

    function sortWrapper(
        bytes32[] memory chainIdTo,
        address[] memory to,
        int256 left,
        int256 right
    ) public pure returns (bytes32[] memory, address[] memory) {
        quickSort(chainIdTo, to, left, right);
        return (chainIdTo, to);
    }
}
