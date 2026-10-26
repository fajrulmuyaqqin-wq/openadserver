use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbBidRequest {
    pub id: String,
    pub imp: Vec<OpenRtbImp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site: Option<OpenRtbSite>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<OpenRtbApp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<OpenRtbDevice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<OpenRtbUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tmax: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcat: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badv: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbImp {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<OpenRtbBanner>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<OpenRtbVideo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bidfloor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bidfloorcur: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbBanner {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbVideo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minduration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxduration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbSite {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cat: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbApp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cat: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbDevice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ua: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo: Option<OpenRtbGeo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub make: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub osv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ifa: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbGeo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lon: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbUser {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyeruid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yob: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbBidResponse {
    pub id: String,
    pub seatbid: Vec<OpenRtbSeatBid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bidid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cur: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbr: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbSeatBid {
    pub bid: Vec<OpenRtbBid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRtbBid {
    pub id: String,
    pub impid: String,
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nurl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adomain: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iurl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<i32>,
}
