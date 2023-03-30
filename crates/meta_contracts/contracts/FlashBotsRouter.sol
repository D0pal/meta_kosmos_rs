//SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;
pragma abicoder v2;
import "openzeppelin-contracts/utils/Strings.sol";
import {IFlashBotsRouter} from './interfaces/FlashBotsInterface.sol';

interface IERC20 {
  event Approval(address indexed owner, address indexed spender, uint256 value);
  event Transfer(address indexed from, address indexed to, uint256 value);

  function name() external view returns (string memory);

  function symbol() external view returns (string memory);

  function decimals() external view returns (uint8);

  function totalSupply() external view returns (uint256);

  function balanceOf(address owner) external view returns (uint256);

  function allowance(address owner, address spender) external view returns (uint256);

  function approve(address spender, uint256 value) external returns (bool);

  function transfer(address to, uint256 value) external returns (bool);

  function transferFrom(
    address from,
    address to,
    uint256 value
  ) external returns (bool);
}

interface IWETH is IERC20 {
  function deposit() external payable;

  function withdraw(uint256) external;
}

// This contract simply calls multiple targets sequentially, ensuring WETH balance before and after

contract FlashBotsRouter is IFlashBotsRouter {
  address private immutable owner;
  address private immutable executor;
  uint256 private lastWethBalance;
  IWETH private WETH;

  modifier onlyExecutor() {
    require(msg.sender == executor || tx.origin == executor);
    _;
  }

  modifier onlyOwner() {
    require(msg.sender == owner);
    _;
  }

  constructor(address _executor, address wethAddress) public payable {
    owner = msg.sender;
    executor = _executor;
    lastWethBalance = 0;
    WETH = IWETH(wethAddress);
    if (msg.value > 0) {
      WETH.deposit{value: msg.value}();
    }
  }

  receive() external payable {}

  function claimWETH(uint256 amount) external onlyOwner {
    uint256 balance = WETH.balanceOf(address(this));
    WETH.transfer(msg.sender, amount < balance ? amount : balance);
  }

  function claimETH(uint256 amount) external onlyOwner {
    uint256 _ethBalance = address(this).balance;
    address payable receiver = payable(msg.sender);
    receiver.transfer(amount < _ethBalance ? amount : _ethBalance);
  }

  /**
  * params: parameters to multiple dex
  * returnProfit: return arbi profit to msg.sender
  */
  function uniswapWeth(UniswapWethParams calldata params, bool returnProfit) external payable override onlyExecutor {
    uint256 wethAmountToFirstMarket = params.wethAmountToFirstMarket;
    uint256 ethAmountToCoinbase = params.ethAmountToCoinbase;
    address[] memory targets = params.targets;
    bytes[] memory payloads = params.payloads;

    require(targets.length == payloads.length, 'target payload lenght not match');
    uint256 _wethBalanceBefore = WETH.balanceOf(address(this));

    require(_wethBalanceBefore > wethAmountToFirstMarket, 'not enought weth balance');
 
    WETH.transfer(targets[0], wethAmountToFirstMarket);
    for (uint256 i = 0; i < targets.length; i++) {
      (bool _success, bytes memory _response) = targets[i].call(payloads[i]);
      require(
        _success, 
        string(abi.encodePacked("swap failed ", Strings.toHexString(uint160(targets[i]), 20), string(_response) ))
      );
      _response;
    }

    uint256 _wethBalanceAfter = WETH.balanceOf(address(this));

    require(_wethBalanceAfter > _wethBalanceBefore + ethAmountToCoinbase, 
    string(abi.encodePacked("non profitable swap: after balance", Strings.toString(_wethBalanceAfter), "before balance",Strings.toString(_wethBalanceBefore) ))
    );
    uint256 _ethBalance = address(this).balance;
    if (_ethBalance < ethAmountToCoinbase) {
      uint256 amtToWithDraw = ethAmountToCoinbase - _ethBalance;
      WETH.withdraw(amtToWithDraw);
      _wethBalanceAfter = _wethBalanceAfter - amtToWithDraw;
    }

    if(ethAmountToCoinbase >0) {
      block.coinbase.transfer(ethAmountToCoinbase);
    }

    if (returnProfit) {
      uint256 profit = _wethBalanceAfter - lastWethBalance;
      WETH.transfer(msg.sender, profit);
    } else {
      lastWethBalance = _wethBalanceAfter;
    }
  }

  function call(
    address payable _to,
    uint256 _value,
    bytes calldata _data
  ) external payable onlyOwner returns (bytes memory) {
    require(_to != address(0));
    (bool _success, bytes memory _result) = _to.call{value: _value}(_data);
    require(_success);
    return _result;
  }

  fallback() external payable {}
}
