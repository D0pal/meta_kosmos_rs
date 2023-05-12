// SPDX-License-Identifier: agpl-3.0
pragma solidity ^0.8.17;
pragma abicoder v2;

interface IFlashBotsRouter {
    struct UniswapWethParams {
        uint256 wethAmountToFirstMarket;
        uint256 ethAmountToCoinbase;
        address[] targets;
        bytes[] payloads;
    }

    function uniswapWeth(
        UniswapWethParams calldata params,
        bool returnProfit
    ) external payable;
}
