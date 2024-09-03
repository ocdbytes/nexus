// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

import "forge-std/test.sol";
import "../src/NexusProofManager.sol";
import "../src/interfaces/INexusProofManager.sol";
import "../src/mock/ERC20.sol";
import "../src/verification/ethereum/Verifier.sol";

contract EthereumVerifierTest is Test  { 
    NexusProofManager proofManager;
    ERC20Token erc20;
    EthereumVerifier verifier;

    bytes32[] dynamicPath;

    uint256 blockNumber = 123;
    bytes32 stateRoot = 0x646298a2ebc208f4ea2e41298eec4a6c6b3a5cb76318681529b28bdcb4867ec0;
    bytes32 blockHash = 0x5f574db327c747d944da576c21506ac2a90dc8f19bbc55791642c5e40d3b100e;
    bytes32 appid = 0x3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d;
    address account = 0xDC11f7E700A4c898AE5CAddB1082cFfa76512aDD;
    bytes32 private constant EMPTY_TRIE_ROOT_HASH =
        0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421;
    bytes32 private constant EMPTY_CODE_HASH =
        0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470;
    function setUp() public { 
        erc20 = new ERC20Token("Avail","Avail");
        proofManager = new NexusProofManager(bytes32(uint256(100)));
        verifier = new EthereumVerifier(INexusProofManager(address(proofManager)));
    }

    function testAccountProof() view public { 
        bytes memory accountTrieProof =
            hex"f9027eb90154f90151a0da14f1d4099ce2d39052e7418db2c4d4ec3daaf91a3a34639d5c6eae9af41450a01689b2a5203afd9ea0a0ca3765e4a538c7176e53eac1f8307a344ffc3c6176558080a0d3193770cae12b76e92e5312157211baddc8c69caa57d35eaef66443fc11e070a0647e5ea7b73aac4f3be6024b685823516bac59d73bfbad3bb972acef207ae11c80a0a1b1028b943f09a20576b8136eef813ded90c04a440ca6f92f864477a9e088c7a07fb37e57079484c1d513705869239229acd96c0a914f6ab8f4a089ad2256a29ba0d0a1bfe5b45d2d863a794f016450a4caca04f3b599e8d1652afca8b752935fd880a0bf9b09e442e044778b354abbadb5ec049d7f5e8b585c3966d476c4fbc9a181d28080a0e1b4c85994464293d3954d1c6cf20c4fafb34e8b34a2620b8e8b03ba1d54219da0ea935f063f75087e291076439b923b42da329d404741c475ced06eedc721aac880b8b3f8b180a0730d76ce58935273dc4d730bed821b9a8e74d27d3b181a682725a16e01d685b980a0e6a168a3122b3aaf71a90a8ed99ed994675d8566aca5c2e5dd36e7bd9e551f1a8080808080a0079579eb8cefec499968deab0c1cadf89c714ffc84dc50e007ba2734ec801aa08080a03e67a8d371ff07b55e77f0e5612754b6883d85dac50d74531ffb569af78ea1e5a0546e44b64fe3c0c67ae98cbb2ea7598a2b2034649b753b927459e0753c229e83808080b870f86ea020dba5d2773a7a778b680a01a6429cdf96c4f181bfd788a57084bd4ddbda0ff2b84bf8490185060db88400a0e5b84f153305fd0aad13a28cf35a501b3a9ee975c6cb99bd7b5cd8afe361fd41a0835e2e3d7d61e7daa9d504aca164722fd3053dad41666c7201bc1c34a9cb128d";
         (, , , bytes32 storageRoot) = verifier.verifyAccount(stateRoot, accountTrieProof, account);
         assert(storageRoot != EMPTY_TRIE_ROOT_HASH);
    }

    function testStorageProof() public {
        proofManager.updateNexusBlock(blockNumber, NexusProofManager.NexusBlock(stateRoot, blockHash));
        bytes memory storageProof =
            hex"f9032cb90214f90211a0fa5e2fd2d6e72c8e43b0ecbeb17a319100e1afbbdb29b63a697bcdbd4a76a2c9a06eb6224a9564b438ad325d0488179f5030a4326eef2ed1ac2d7519d4294174aba061ef152c381b12802824b71a91afc98c04a5b6e51d389fb9f2c05e5abe31b3e5a04e89079af75d215b6bc8b9da9d4004924c10f630376ad15c0a38aca26f72181aa041546186685e7844b9eb3e9aae302edd9e4bea2623126f7b6d1c960eb40c011ca0edd7174916748bfa8745eb7c7ae6d918407122e59528613c45378c4fb2ce754fa0fbcd97095a84dca96a677c8cecc8c896dc30873b04f26c75e9c178d0ca37e6aca0e0c78dc9d094e0862de0a5c9c985baf6559e2c40bec51992680b214bc6bb4f35a057ff5978a35d5ec99f026666883f5f027fde694459680ec6e57fc298e2e98186a0c9d21353484f8b32ef324257ad91a3ec3c38a7b8406e8a4f8ee4df36db357af9a0e6d922b0ba239623bb6e760dca22188040f4cc5a0ec67956f7a150caf572c960a0e8d15b5addd823945cf6fc4bfbbad157509404495f1837252b0ee02218968373a0d3296997569db72f2d654f637d919044d60780c9dd0b0991cfa7643d342a3923a0829e3a88a6375481ab9af6f12a2aacf1ce623d271db7aee004d554a0614429aaa0e1fbeb68483387d5bc0b5c55ed390b8fe3eeb317e1529ccaa9c7a41f0446d7a4a079f4c602db086d596a7a47dbcaaa6a0d6e75f1ccc8165907610e76c93abfbfe580b893f891a09f74f37bdf07e2d73e875c821a01a72ec9e3e5362cddcfa3521296476e6da1f880808080a038d9d8c71f2ce4639e7da19dc4c9d007e8b899611eea89b30c58963958b9e1a08080a0c50af823ddd4cfc519ce820f15c121ca41cdebf06b1c3ef90f864120f4aefc468080808080a0eb10e85ce708885e53342a886ef11efbe1d3594a12fa790df637d075eac860b18080b853f8518080808080a06faf57464a2fd95b0ab5ca730e0bcb746ddf4998391c1f0c25a1c7aecd71b4c8808080808080808080a070498144d3ce4caf58f156fda7f3056e5cb58bfca06621a384d441c6c691a2ae80aae99f3787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace88872386f26fc10000";
        bytes32 storageRoot =
            0xed339d10818912537ecff9846d024bbb91f43c025e0cf6bd9776170b11a77233;
        bytes32 storageSlot =
            0x0000000000000000000000000000000000000000000000000000000000000002;
        bytes32 slotValue = verifier.verifyStorage(storageRoot, storageSlot, storageProof);
        assert(slotValue != EMPTY_CODE_HASH);
    }
    
}
