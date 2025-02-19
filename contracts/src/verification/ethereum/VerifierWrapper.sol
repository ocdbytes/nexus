// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import {EthereumVerifier} from "./Verifier.sol";
import {INexusVerifierWrapper} from "../../interfaces/INexusVerifierWrapper.sol";
import {INexusProofManager} from "../../interfaces/INexusProofManager.sol";

contract VerifierWrapper is INexusVerifierWrapper, EthereumVerifier {
    bytes32 immutable nexusAppID;
    INexusProofManager immutable nexus;

    error InvalidEntry();
    error InvalidSlotValue();
    error InvalidAccount();
    struct Proof {
        bytes accountProof;
        address addr;
        bytes storageProof;
        bytes32 storageSlot;
    }

    bytes32 private constant EMPTY_TRIE_ROOT_HASH =
        0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421;

    constructor(
        bytes32 _nexusAppID,
        INexusProofManager _nexus
    ) EthereumVerifier(_nexus) {
        nexusAppID = _nexusAppID;
        nexus = _nexus;
    }

    function parseAndVerify(
        uint256 chainblockNumber,
        bytes32 receipt,
        bytes calldata data,
        address from
    ) external {
        Proof memory proof = abi.decode(data, (Proof));
        if (proof.addr != from) {
            revert InvalidAccount();
        }
        bytes32 state = nexus.getChainState(chainblockNumber, nexusAppID);
        (, , , bytes32 storageRoot) = verifyAccount(
            state,
            proof.accountProof,
            proof.addr
        );
        if (storageRoot == EMPTY_TRIE_ROOT_HASH) {
            revert InvalidEntry();
        }

        bytes32 value = verifyStorage(
            storageRoot,
            proof.storageSlot,
            proof.storageProof
        );

        if (value != receipt) {
            revert InvalidSlotValue();
        }
    }
}
