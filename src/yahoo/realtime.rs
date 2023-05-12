use serde::{Deserialize, Serialize};

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Serialize, Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct PricingData {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(float, tag = "2")]
    pub price: f32,
    #[prost(sint64, tag = "3")]
    pub time: i64,
    #[prost(string, tag = "4")]
    pub currency: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub exchange: ::prost::alloc::string::String,
    #[prost(enumeration = "pricing_data::QuoteType", tag = "6")]
    pub quote_type: i32,
    #[prost(enumeration = "pricing_data::MarketHoursType", tag = "7")]
    pub market_hours: i32,
    #[prost(float, tag = "8")]
    pub change_percent: f32,
    #[prost(sint64, tag = "9")]
    pub day_volume: i64,
    #[prost(float, tag = "10")]
    pub day_high: f32,
    #[prost(float, tag = "11")]
    pub day_low: f32,
    #[prost(float, tag = "12")]
    pub change: f32,
    #[prost(string, tag = "13")]
    pub short_name: ::prost::alloc::string::String,
    #[prost(sint64, tag = "14")]
    pub expire_date: i64,
    #[prost(float, tag = "15")]
    pub open_price: f32,
    #[prost(float, tag = "16")]
    pub previous_close: f32,
    #[prost(float, tag = "17")]
    pub strike_price: f32,
    #[prost(string, tag = "18")]
    pub underlying_symbol: ::prost::alloc::string::String,
    #[prost(sint64, tag = "19")]
    pub open_interest: i64,
    #[prost(enumeration = "pricing_data::OptionType", tag = "20")]
    pub options_type: i32,
    #[prost(sint64, tag = "21")]
    pub mini_option: i64,
    #[prost(sint64, tag = "22")]
    pub last_size: i64,
    #[prost(float, tag = "23")]
    pub bid: f32,
    #[prost(sint64, tag = "24")]
    pub bid_size: i64,
    #[prost(float, tag = "25")]
    pub ask: f32,
    #[prost(sint64, tag = "26")]
    pub ask_size: i64,
    #[prost(sint64, tag = "27")]
    pub price_hint: i64,
    #[prost(sint64, tag = "28")]
    pub vol_24hr: i64,
    #[prost(sint64, tag = "29")]
    pub vol_all_currencies: i64,
    #[prost(string, tag = "30")]
    pub fromcurrency: ::prost::alloc::string::String,
    #[prost(string, tag = "31")]
    pub last_market: ::prost::alloc::string::String,
    #[prost(double, tag = "32")]
    pub circulating_supply: f64,
    #[prost(double, tag = "33")]
    pub marketcap: f64,
}
/// Nested message and enum types in `PricingData`.
pub mod pricing_data {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum QuoteType {
        None = 0,
        Altsymbol = 5,
        Heartbeat = 7,
        Equity = 8,
        Index = 9,
        Mutualfund = 11,
        Moneymarket = 12,
        Option = 13,
        Currency = 14,
        Warrant = 15,
        Bond = 17,
        Future = 18,
        Etf = 20,
        Commodity = 23,
        Ecnquote = 28,
        Cryptocurrency = 41,
        Indicator = 42,
        Industry = 1000,
    }
    impl QuoteType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                QuoteType::None => "NONE",
                QuoteType::Altsymbol => "ALTSYMBOL",
                QuoteType::Heartbeat => "HEARTBEAT",
                QuoteType::Equity => "EQUITY",
                QuoteType::Index => "INDEX",
                QuoteType::Mutualfund => "MUTUALFUND",
                QuoteType::Moneymarket => "MONEYMARKET",
                QuoteType::Option => "OPTION",
                QuoteType::Currency => "CURRENCY",
                QuoteType::Warrant => "WARRANT",
                QuoteType::Bond => "BOND",
                QuoteType::Future => "FUTURE",
                QuoteType::Etf => "ETF",
                QuoteType::Commodity => "COMMODITY",
                QuoteType::Ecnquote => "ECNQUOTE",
                QuoteType::Cryptocurrency => "CRYPTOCURRENCY",
                QuoteType::Indicator => "INDICATOR",
                QuoteType::Industry => "INDUSTRY",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "NONE" => Some(Self::None),
                "ALTSYMBOL" => Some(Self::Altsymbol),
                "HEARTBEAT" => Some(Self::Heartbeat),
                "EQUITY" => Some(Self::Equity),
                "INDEX" => Some(Self::Index),
                "MUTUALFUND" => Some(Self::Mutualfund),
                "MONEYMARKET" => Some(Self::Moneymarket),
                "OPTION" => Some(Self::Option),
                "CURRENCY" => Some(Self::Currency),
                "WARRANT" => Some(Self::Warrant),
                "BOND" => Some(Self::Bond),
                "FUTURE" => Some(Self::Future),
                "ETF" => Some(Self::Etf),
                "COMMODITY" => Some(Self::Commodity),
                "ECNQUOTE" => Some(Self::Ecnquote),
                "CRYPTOCURRENCY" => Some(Self::Cryptocurrency),
                "INDICATOR" => Some(Self::Indicator),
                "INDUSTRY" => Some(Self::Industry),
                _ => None,
            }
        }
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum OptionType {
        Call = 0,
        Put = 1,
    }
    impl OptionType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                OptionType::Call => "CALL",
                OptionType::Put => "PUT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "CALL" => Some(Self::Call),
                "PUT" => Some(Self::Put),
                _ => None,
            }
        }
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum MarketHoursType {
        PreMarket = 0,
        RegularMarket = 1,
        PostMarket = 2,
        ExtendedHoursMarket = 3,
    }
    impl MarketHoursType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                MarketHoursType::PreMarket => "PRE_MARKET",
                MarketHoursType::RegularMarket => "REGULAR_MARKET",
                MarketHoursType::PostMarket => "POST_MARKET",
                MarketHoursType::ExtendedHoursMarket => "EXTENDED_HOURS_MARKET",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "PRE_MARKET" => Some(Self::PreMarket),
                "REGULAR_MARKET" => Some(Self::RegularMarket),
                "POST_MARKET" => Some(Self::PostMarket),
                "EXTENDED_HOURS_MARKET" => Some(Self::ExtendedHoursMarket),
                _ => None,
            }
        }
    }
}
