use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum Status {
    Inactive = 0,
    Active = 1,
    Paused = 2,
    Deleted = 3,
    Pending = 4,
}

impl From<i16> for Status {
    fn from(val: i16) -> Self {
        match val {
            1 => Status::Active,
            2 => Status::Paused,
            3 => Status::Deleted,
            4 => Status::Pending,
            _ => Status::Inactive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum BidType {
    CPM = 1,  // Cost per mille
    CPC = 2,  // Cost per click
    CPA = 3,  // Cost per action
    OCPM = 4, // Optimized CPM
}

impl From<i16> for BidType {
    fn from(val: i16) -> Self {
        match val {
            1 => BidType::CPM,
            2 => BidType::CPC,
            3 => BidType::CPA,
            4 => BidType::OCPM,
            _ => BidType::CPM,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum CreativeType {
    Banner = 1,
    Native = 2,
    Video = 3,
    Interstitial = 4,
}

impl From<i16> for CreativeType {
    fn from(val: i16) -> Self {
        match val {
            1 => CreativeType::Banner,
            2 => CreativeType::Native,
            3 => CreativeType::Video,
            4 => CreativeType::Interstitial,
            _ => CreativeType::Banner,
        }
    }
}

impl CreativeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CreativeType::Banner => "banner",
            CreativeType::Native => "native",
            CreativeType::Video => "video",
            CreativeType::Interstitial => "interstitial",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum EventType {
    Impression = 1,
    Click = 2,
    Conversion = 3,
}

impl From<i16> for EventType {
    fn from(val: i16) -> Self {
        match val {
            1 => EventType::Impression,
            2 => EventType::Click,
            3 => EventType::Conversion,
            _ => EventType::Impression,
        }
    }
}

impl EventType {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "impression" | "imp" => Some(EventType::Impression),
            "click" | "clk" => Some(EventType::Click),
            "conversion" | "conv" => Some(EventType::Conversion),
            _ => None,
        }
    }
}
