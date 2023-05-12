pub use mute_switch_factory::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod mute_switch_factory {
    #[rustfmt::skip]
    const __ABI: &str = "[  {    \"inputs\": [],    \"stateMutability\": \"nonpayable\",    \"type\": \"constructor\"  },  {    \"anonymous\": false,    \"inputs\": [      {        \"indexed\": true,        \"internalType\": \"address\",        \"name\": \"token0\",        \"type\": \"address\"      },      {        \"indexed\": true,        \"internalType\": \"address\",        \"name\": \"token1\",        \"type\": \"address\"      },      {        \"indexed\": false,        \"internalType\": \"bool\",        \"name\": \"stable\",        \"type\": \"bool\"      },      {        \"indexed\": false,        \"internalType\": \"address\",        \"name\": \"pair\",        \"type\": \"address\"      },      {        \"indexed\": false,        \"internalType\": \"uint256\",        \"name\": \"\",        \"type\": \"uint256\"      },      {        \"indexed\": false,        \"internalType\": \"uint256\",        \"name\": \"fee\",        \"type\": \"uint256\"      }    ],    \"name\": \"PairCreated\",    \"type\": \"event\"  },  {    \"anonymous\": false,    \"inputs\": [      {        \"indexed\": false,        \"internalType\": \"uint256\",        \"name\": \"_protocolFeeDynamic\",        \"type\": \"uint256\"      }    ],    \"name\": \"ProtocolFeeDynamicChange\",    \"type\": \"event\"  },  {    \"anonymous\": false,    \"inputs\": [      {        \"indexed\": false,        \"internalType\": \"uint256\",        \"name\": \"_protocolFeeFixed\",        \"type\": \"uint256\"      }    ],    \"name\": \"ProtocolFeeFixedChange\",    \"type\": \"event\"  },  {    \"anonymous\": false,    \"inputs\": [      {        \"indexed\": false,        \"internalType\": \"address\",        \"name\": \"_feeTo\",        \"type\": \"address\"      }    ],    \"name\": \"ProtocolFeeToChange\",    \"type\": \"event\"  },  {    \"inputs\": [      {        \"internalType\": \"uint256\",        \"name\": \"\",        \"type\": \"uint256\"      }    ],    \"name\": \"allPairs\",    \"outputs\": [      {        \"internalType\": \"address\",        \"name\": \"\",        \"type\": \"address\"      }    ],    \"stateMutability\": \"view\",    \"type\": \"function\"  },  {    \"inputs\": [],    \"name\": \"allPairsLength\",    \"outputs\": [      {        \"internalType\": \"uint256\",        \"name\": \"\",        \"type\": \"uint256\"      }    ],    \"stateMutability\": \"view\",    \"type\": \"function\"  },  {    \"inputs\": [      {        \"internalType\": \"address\",        \"name\": \"tokenA\",        \"type\": \"address\"      },      {        \"internalType\": \"address\",        \"name\": \"tokenB\",        \"type\": \"address\"      },      {        \"internalType\": \"uint256\",        \"name\": \"feeType\",        \"type\": \"uint256\"      },      {        \"internalType\": \"bool\",        \"name\": \"stable\",        \"type\": \"bool\"      }    ],    \"name\": \"createPair\",    \"outputs\": [      {        \"internalType\": \"address\",        \"name\": \"pair\",        \"type\": \"address\"      }    ],    \"stateMutability\": \"nonpayable\",    \"type\": \"function\"  },  {    \"inputs\": [],    \"name\": \"feeTo\",    \"outputs\": [      {        \"internalType\": \"address\",        \"name\": \"\",        \"type\": \"address\"      }    ],    \"stateMutability\": \"view\",    \"type\": \"function\"  },  {    \"inputs\": [      {        \"internalType\": \"address\",        \"name\": \"\",        \"type\": \"address\"      },      {        \"internalType\": \"address\",        \"name\": \"\",        \"type\": \"address\"      },      {        \"internalType\": \"bool\",        \"name\": \"\",        \"type\": \"bool\"      }    ],    \"name\": \"getPair\",    \"outputs\": [      {        \"internalType\": \"address\",        \"name\": \"\",        \"type\": \"address\"      }    ],    \"stateMutability\": \"view\",    \"type\": \"function\"  },  {    \"inputs\": [],    \"name\": \"protocolFeeDynamic\",    \"outputs\": [      {        \"internalType\": \"uint256\",        \"name\": \"\",        \"type\": \"uint256\"      }    ],    \"stateMutability\": \"view\",    \"type\": \"function\"  },  {    \"inputs\": [],    \"name\": \"protocolFeeFixed\",    \"outputs\": [      {        \"internalType\": \"uint256\",        \"name\": \"\",        \"type\": \"uint256\"      }    ],    \"stateMutability\": \"view\",    \"type\": \"function\"  },  {    \"inputs\": [      {        \"internalType\": \"address\",        \"name\": \"_feeTo\",        \"type\": \"address\"      }    ],    \"name\": \"setFeeTo\",    \"outputs\": [],    \"stateMutability\": \"nonpayable\",    \"type\": \"function\"  },  {    \"inputs\": [      {        \"internalType\": \"uint256\",        \"name\": \"_protocolFeeDynamic\",        \"type\": \"uint256\"      }    ],    \"name\": \"setProtocolFeeDynamic\",    \"outputs\": [],    \"stateMutability\": \"nonpayable\",    \"type\": \"function\"  },  {    \"inputs\": [      {        \"internalType\": \"uint256\",        \"name\": \"_protocolFeeFixed\",        \"type\": \"uint256\"      }    ],    \"name\": \"setProtocolFeeFixed\",    \"outputs\": [],    \"stateMutability\": \"nonpayable\",    \"type\": \"function\"  }]";
    ///The parsed JSON ABI of the contract.
    pub static MUTESWITCHFACTORY_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(|| {
        ::ethers::core::utils::__serde_json::from_str(__ABI)
            .expect("ABI is always valid")
    });
    pub struct MuteSwitchFactory<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for MuteSwitchFactory<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for MuteSwitchFactory<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for MuteSwitchFactory<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for MuteSwitchFactory<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(MuteSwitchFactory)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> MuteSwitchFactory<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    MUTESWITCHFACTORY_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `allPairs` (0x1e3dd18b) function
        pub fn all_pairs(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([30, 61, 209, 139], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `allPairsLength` (0x574f2ba3) function
        pub fn all_pairs_length(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([87, 79, 43, 163], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createPair` (0xb2e916d6) function
        pub fn create_pair(
            &self,
            token_a: ::ethers::core::types::Address,
            token_b: ::ethers::core::types::Address,
            fee_type: ::ethers::core::types::U256,
            stable: bool,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([178, 233, 22, 214], (token_a, token_b, fee_type, stable))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `feeTo` (0x017e7e58) function
        pub fn fee_to(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([1, 126, 126, 88], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getPair` (0x6801cc30) function
        pub fn get_pair(
            &self,
            p0: ::ethers::core::types::Address,
            p1: ::ethers::core::types::Address,
            p2: bool,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([104, 1, 204, 48], (p0, p1, p2))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `protocolFeeDynamic` (0xc348d898) function
        pub fn protocol_fee_dynamic(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([195, 72, 216, 152], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `protocolFeeFixed` (0xa1f177d5) function
        pub fn protocol_fee_fixed(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([161, 241, 119, 213], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setFeeTo` (0xf46901ed) function
        pub fn set_fee_to(
            &self,
            fee_to: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([244, 105, 1, 237], fee_to)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setProtocolFeeDynamic` (0x1d0f85bd) function
        pub fn set_protocol_fee_dynamic(
            &self,
            protocol_fee_dynamic: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([29, 15, 133, 189], protocol_fee_dynamic)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setProtocolFeeFixed` (0x4b7efec1) function
        pub fn set_protocol_fee_fixed(
            &self,
            protocol_fee_fixed: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([75, 126, 254, 193], protocol_fee_fixed)
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `PairCreated` event
        pub fn pair_created_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            PairCreatedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ProtocolFeeDynamicChange` event
        pub fn protocol_fee_dynamic_change_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ProtocolFeeDynamicChangeFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ProtocolFeeFixedChange` event
        pub fn protocol_fee_fixed_change_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ProtocolFeeFixedChangeFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ProtocolFeeToChange` event
        pub fn protocol_fee_to_change_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ProtocolFeeToChangeFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            MuteSwitchFactoryEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for MuteSwitchFactory<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "PairCreated",
        abi = "PairCreated(address,address,bool,address,uint256,uint256)"
    )]
    pub struct PairCreatedFilter {
        #[ethevent(indexed)]
        pub token_0: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub token_1: ::ethers::core::types::Address,
        pub stable: bool,
        pub pair: ::ethers::core::types::Address,
        pub p4: ::ethers::core::types::U256,
        pub fee: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "ProtocolFeeDynamicChange",
        abi = "ProtocolFeeDynamicChange(uint256)"
    )]
    pub struct ProtocolFeeDynamicChangeFilter {
        pub protocol_fee_dynamic: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ProtocolFeeFixedChange", abi = "ProtocolFeeFixedChange(uint256)")]
    pub struct ProtocolFeeFixedChangeFilter {
        pub protocol_fee_fixed: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ProtocolFeeToChange", abi = "ProtocolFeeToChange(address)")]
    pub struct ProtocolFeeToChangeFilter {
        pub fee_to: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum MuteSwitchFactoryEvents {
        PairCreatedFilter(PairCreatedFilter),
        ProtocolFeeDynamicChangeFilter(ProtocolFeeDynamicChangeFilter),
        ProtocolFeeFixedChangeFilter(ProtocolFeeFixedChangeFilter),
        ProtocolFeeToChangeFilter(ProtocolFeeToChangeFilter),
    }
    impl ::ethers::contract::EthLogDecode for MuteSwitchFactoryEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = PairCreatedFilter::decode_log(log) {
                return Ok(MuteSwitchFactoryEvents::PairCreatedFilter(decoded));
            }
            if let Ok(decoded) = ProtocolFeeDynamicChangeFilter::decode_log(log) {
                return Ok(
                    MuteSwitchFactoryEvents::ProtocolFeeDynamicChangeFilter(decoded),
                );
            }
            if let Ok(decoded) = ProtocolFeeFixedChangeFilter::decode_log(log) {
                return Ok(
                    MuteSwitchFactoryEvents::ProtocolFeeFixedChangeFilter(decoded),
                );
            }
            if let Ok(decoded) = ProtocolFeeToChangeFilter::decode_log(log) {
                return Ok(MuteSwitchFactoryEvents::ProtocolFeeToChangeFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for MuteSwitchFactoryEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::PairCreatedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::ProtocolFeeDynamicChangeFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ProtocolFeeFixedChangeFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ProtocolFeeToChangeFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<PairCreatedFilter> for MuteSwitchFactoryEvents {
        fn from(value: PairCreatedFilter) -> Self {
            Self::PairCreatedFilter(value)
        }
    }
    impl ::core::convert::From<ProtocolFeeDynamicChangeFilter>
    for MuteSwitchFactoryEvents {
        fn from(value: ProtocolFeeDynamicChangeFilter) -> Self {
            Self::ProtocolFeeDynamicChangeFilter(value)
        }
    }
    impl ::core::convert::From<ProtocolFeeFixedChangeFilter>
    for MuteSwitchFactoryEvents {
        fn from(value: ProtocolFeeFixedChangeFilter) -> Self {
            Self::ProtocolFeeFixedChangeFilter(value)
        }
    }
    impl ::core::convert::From<ProtocolFeeToChangeFilter> for MuteSwitchFactoryEvents {
        fn from(value: ProtocolFeeToChangeFilter) -> Self {
            Self::ProtocolFeeToChangeFilter(value)
        }
    }
    ///Container type for all input parameters for the `allPairs` function with signature `allPairs(uint256)` and selector `0x1e3dd18b`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "allPairs", abi = "allPairs(uint256)")]
    pub struct AllPairsCall(pub ::ethers::core::types::U256);
    ///Container type for all input parameters for the `allPairsLength` function with signature `allPairsLength()` and selector `0x574f2ba3`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "allPairsLength", abi = "allPairsLength()")]
    pub struct AllPairsLengthCall;
    ///Container type for all input parameters for the `createPair` function with signature `createPair(address,address,uint256,bool)` and selector `0xb2e916d6`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "createPair", abi = "createPair(address,address,uint256,bool)")]
    pub struct CreatePairCall {
        pub token_a: ::ethers::core::types::Address,
        pub token_b: ::ethers::core::types::Address,
        pub fee_type: ::ethers::core::types::U256,
        pub stable: bool,
    }
    ///Container type for all input parameters for the `feeTo` function with signature `feeTo()` and selector `0x017e7e58`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "feeTo", abi = "feeTo()")]
    pub struct FeeToCall;
    ///Container type for all input parameters for the `getPair` function with signature `getPair(address,address,bool)` and selector `0x6801cc30`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getPair", abi = "getPair(address,address,bool)")]
    pub struct GetPairCall(
        pub ::ethers::core::types::Address,
        pub ::ethers::core::types::Address,
        pub bool,
    );
    ///Container type for all input parameters for the `protocolFeeDynamic` function with signature `protocolFeeDynamic()` and selector `0xc348d898`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "protocolFeeDynamic", abi = "protocolFeeDynamic()")]
    pub struct ProtocolFeeDynamicCall;
    ///Container type for all input parameters for the `protocolFeeFixed` function with signature `protocolFeeFixed()` and selector `0xa1f177d5`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "protocolFeeFixed", abi = "protocolFeeFixed()")]
    pub struct ProtocolFeeFixedCall;
    ///Container type for all input parameters for the `setFeeTo` function with signature `setFeeTo(address)` and selector `0xf46901ed`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "setFeeTo", abi = "setFeeTo(address)")]
    pub struct SetFeeToCall {
        pub fee_to: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `setProtocolFeeDynamic` function with signature `setProtocolFeeDynamic(uint256)` and selector `0x1d0f85bd`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "setProtocolFeeDynamic", abi = "setProtocolFeeDynamic(uint256)")]
    pub struct SetProtocolFeeDynamicCall {
        pub protocol_fee_dynamic: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `setProtocolFeeFixed` function with signature `setProtocolFeeFixed(uint256)` and selector `0x4b7efec1`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "setProtocolFeeFixed", abi = "setProtocolFeeFixed(uint256)")]
    pub struct SetProtocolFeeFixedCall {
        pub protocol_fee_fixed: ::ethers::core::types::U256,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum MuteSwitchFactoryCalls {
        AllPairs(AllPairsCall),
        AllPairsLength(AllPairsLengthCall),
        CreatePair(CreatePairCall),
        FeeTo(FeeToCall),
        GetPair(GetPairCall),
        ProtocolFeeDynamic(ProtocolFeeDynamicCall),
        ProtocolFeeFixed(ProtocolFeeFixedCall),
        SetFeeTo(SetFeeToCall),
        SetProtocolFeeDynamic(SetProtocolFeeDynamicCall),
        SetProtocolFeeFixed(SetProtocolFeeFixedCall),
    }
    impl ::ethers::core::abi::AbiDecode for MuteSwitchFactoryCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <AllPairsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::AllPairs(decoded));
            }
            if let Ok(decoded)
                = <AllPairsLengthCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::AllPairsLength(decoded));
            }
            if let Ok(decoded)
                = <CreatePairCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CreatePair(decoded));
            }
            if let Ok(decoded)
                = <FeeToCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::FeeTo(decoded));
            }
            if let Ok(decoded)
                = <GetPairCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetPair(decoded));
            }
            if let Ok(decoded)
                = <ProtocolFeeDynamicCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ProtocolFeeDynamic(decoded));
            }
            if let Ok(decoded)
                = <ProtocolFeeFixedCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ProtocolFeeFixed(decoded));
            }
            if let Ok(decoded)
                = <SetFeeToCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SetFeeTo(decoded));
            }
            if let Ok(decoded)
                = <SetProtocolFeeDynamicCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SetProtocolFeeDynamic(decoded));
            }
            if let Ok(decoded)
                = <SetProtocolFeeFixedCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SetProtocolFeeFixed(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for MuteSwitchFactoryCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::AllPairs(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AllPairsLength(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CreatePair(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FeeTo(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetPair(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ProtocolFeeDynamic(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ProtocolFeeFixed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetFeeTo(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetProtocolFeeDynamic(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetProtocolFeeFixed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for MuteSwitchFactoryCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AllPairs(element) => ::core::fmt::Display::fmt(element, f),
                Self::AllPairsLength(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreatePair(element) => ::core::fmt::Display::fmt(element, f),
                Self::FeeTo(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetPair(element) => ::core::fmt::Display::fmt(element, f),
                Self::ProtocolFeeDynamic(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ProtocolFeeFixed(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetFeeTo(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetProtocolFeeDynamic(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetProtocolFeeFixed(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<AllPairsCall> for MuteSwitchFactoryCalls {
        fn from(value: AllPairsCall) -> Self {
            Self::AllPairs(value)
        }
    }
    impl ::core::convert::From<AllPairsLengthCall> for MuteSwitchFactoryCalls {
        fn from(value: AllPairsLengthCall) -> Self {
            Self::AllPairsLength(value)
        }
    }
    impl ::core::convert::From<CreatePairCall> for MuteSwitchFactoryCalls {
        fn from(value: CreatePairCall) -> Self {
            Self::CreatePair(value)
        }
    }
    impl ::core::convert::From<FeeToCall> for MuteSwitchFactoryCalls {
        fn from(value: FeeToCall) -> Self {
            Self::FeeTo(value)
        }
    }
    impl ::core::convert::From<GetPairCall> for MuteSwitchFactoryCalls {
        fn from(value: GetPairCall) -> Self {
            Self::GetPair(value)
        }
    }
    impl ::core::convert::From<ProtocolFeeDynamicCall> for MuteSwitchFactoryCalls {
        fn from(value: ProtocolFeeDynamicCall) -> Self {
            Self::ProtocolFeeDynamic(value)
        }
    }
    impl ::core::convert::From<ProtocolFeeFixedCall> for MuteSwitchFactoryCalls {
        fn from(value: ProtocolFeeFixedCall) -> Self {
            Self::ProtocolFeeFixed(value)
        }
    }
    impl ::core::convert::From<SetFeeToCall> for MuteSwitchFactoryCalls {
        fn from(value: SetFeeToCall) -> Self {
            Self::SetFeeTo(value)
        }
    }
    impl ::core::convert::From<SetProtocolFeeDynamicCall> for MuteSwitchFactoryCalls {
        fn from(value: SetProtocolFeeDynamicCall) -> Self {
            Self::SetProtocolFeeDynamic(value)
        }
    }
    impl ::core::convert::From<SetProtocolFeeFixedCall> for MuteSwitchFactoryCalls {
        fn from(value: SetProtocolFeeFixedCall) -> Self {
            Self::SetProtocolFeeFixed(value)
        }
    }
    ///Container type for all return fields from the `allPairs` function with signature `allPairs(uint256)` and selector `0x1e3dd18b`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct AllPairsReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `allPairsLength` function with signature `allPairsLength()` and selector `0x574f2ba3`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct AllPairsLengthReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `createPair` function with signature `createPair(address,address,uint256,bool)` and selector `0xb2e916d6`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreatePairReturn {
        pub pair: ::ethers::core::types::Address,
    }
    ///Container type for all return fields from the `feeTo` function with signature `feeTo()` and selector `0x017e7e58`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct FeeToReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `getPair` function with signature `getPair(address,address,bool)` and selector `0x6801cc30`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetPairReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `protocolFeeDynamic` function with signature `protocolFeeDynamic()` and selector `0xc348d898`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ProtocolFeeDynamicReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `protocolFeeFixed` function with signature `protocolFeeFixed()` and selector `0xa1f177d5`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ProtocolFeeFixedReturn(pub ::ethers::core::types::U256);
}
