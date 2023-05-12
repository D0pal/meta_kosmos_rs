// SPDX-License-Identifier: agpl-3.0
pragma solidity ^0.8.17;
pragma abicoder v2;

interface IMuteSyncAtomicSwap {
    function atomicSwapWeth(
        uint256 wethAmountToFirstMarket,
        uint8 firstMarket,
        address mutePoolAddress
    ) external payable;
}
