#![deny(unsafe_code)]
#![deny(missing_docs)]

//! Simple bindings to Tibber GraphQL API
//!
//! Docs of underlying API : https://developer.tibber.com/docs/overview
#[cfg(feature = "reqwest")]
use ::reqwest::blocking::Client;
use chrono::{DateTime, FixedOffset};
#[cfg(feature = "reqwest")]
use graphql_client::reqwest::post_graphql_blocking as post_graphql;
use graphql_client::GraphQLQuery;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tibber/tibber.json",
    query_path = "tibber/view.graphql",
    response_derives = "Debug"
)]
struct Viewer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tibber/tibber.json",
    query_path = "tibber/price.graphql",
    response_derives = "Debug"
)]
struct Price;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tibber/tibber.json",
    query_path = "tibber/price_today.graphql",
    response_derives = "Debug"
)]
struct PriceToday;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tibber/tibber.json",
    query_path = "tibber/price_tomorrow.graphql",
    response_derives = "Debug"
)]
struct PriceTomorrow;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tibber/tibber.json",
    query_path = "tibber/home.graphql",
    response_derives = "Debug"
)]
struct Home;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tibber/tibber.json",
    query_path = "tibber/consumption.graphql",
    response_derives = "Debug"
)]
struct ConsumptionHistory;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tibber/tibber.json",
    query_path = "tibber/production.graphql",
    response_derives = "Debug"
)]
struct ProductionHistory;

#[cfg(feature = "reqwest")]
fn make_request<Q: GraphQLQuery>(
    api_token: &str,
    variables: <Q as GraphQLQuery>::Variables,
) -> Result<graphql_client::Response<Q::ResponseData>, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("graphql-rust/0.14.0")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_token))?,
            ))
            .collect(),
        )
        .build()?;

    Ok(post_graphql::<Q, _>(
        &client,
        "https://api.tibber.com/v1-beta/gql/",
        variables,
    )?)
}

#[cfg(feature = "ureq")]
fn make_request<Q: GraphQLQuery>(
    api_token: &str,
    variables: <Q as GraphQLQuery>::Variables,
) -> Result<graphql_client::Response<Q::ResponseData>, Box<dyn std::error::Error>> {
    let agent = ureq_crate::AgentBuilder::new()
        .user_agent("graphql-rust/0.14.0")
        .build();

    let body = Q::build_query(variables);
    Ok(agent
        .post("https://api.tibber.com/v1-beta/gql/")
        .set("Authorization", &format!("Bearer {}", api_token))
        .send_json(&body)?
        .into_json()?)
}

fn fetch_data<T: GraphQLQuery>(
    api_token: &str,
    variables: <T as GraphQLQuery>::Variables,
) -> Result<<T as GraphQLQuery>::ResponseData, Box<dyn std::error::Error>> {
    let response_body = make_request::<T>(api_token, variables)?;

    let response_data = match response_body.data {
        Some(d) => d,
        None => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "no response body",
            )))
        }
    };
    Ok(response_data)
}

#[derive(Debug, Clone)]
/// ID used to represent a house / home
pub struct HomeId(String);

#[derive(Debug, Clone)]
/// User info for current authenticated user
pub struct User {
    /// Login name of the user
    pub login: String,
    /// User id
    pub user_id: String,
    /// (Legal) name of the user
    pub name: String,
    /// Account types, as a list
    pub account_type: Vec<String>,
    /// IDs of available homes
    pub homes: Vec<HomeId>,
}

#[derive(Debug, Clone)]
/// Type / building-category of house
pub enum HouseType {
    /// An apartment in a block
    Apartment,
    /// A unit in a row of houses ( Condo )
    RowHouse,
    /// A free standing house
    House,
    /// A cottage or recreational house
    Cottage,
    /// Something else
    Other(String),
}

impl HouseType {
    fn new(htype: home::HomeType) -> Self {
        match htype {
            home::HomeType::APARTMENT => HouseType::Apartment,
            home::HomeType::ROWHOUSE => HouseType::RowHouse,
            home::HomeType::HOUSE => HouseType::House,
            home::HomeType::COTTAGE => HouseType::Cottage,
            home::HomeType::Other(s) => HouseType::Other(s),
        }
    }
}

#[derive(Debug, Clone)]
/// Primary source of heating
pub enum HeatingSource {
    /// AC unit with inverter
    Air2AairHeatPump,
    /// Electrical ovens
    Electricity,
    /// Ground heat
    Ground,
    /// Remote heating, typically water-borne
    DistrictHeating,
    /// Electrical boiler with floor heating or radiator
    ElectricBoiler,
    /// Heatpump extracting heat from some body of water
    Air2WaterHeatPump,
    /// Something else
    Other(Option<String>),
    /// Not specified
    Unknown,
}

impl HeatingSource {
    fn new(hsource: Option<home::HeatingSource>) -> Self {
        if hsource.is_none() {
            return HeatingSource::Unknown;
        }
        match hsource.unwrap() {
            home::HeatingSource::AIR2AIR_HEATPUMP => HeatingSource::Air2AairHeatPump,
            home::HeatingSource::ELECTRICITY => HeatingSource::Electricity,
            home::HeatingSource::GROUND => HeatingSource::Ground,
            home::HeatingSource::DISTRICT_HEATING => HeatingSource::DistrictHeating,
            home::HeatingSource::ELECTRIC_BOILER => HeatingSource::ElectricBoiler,
            home::HeatingSource::AIR2WATER_HEATPUMP => HeatingSource::Air2WaterHeatPump,
            home::HeatingSource::OTHER => HeatingSource::Other(None),
            home::HeatingSource::Other(s) => HeatingSource::Other(Some(s)),
        }
    }
}

#[derive(Debug, Clone)]
/// Address of the home / house
pub struct Address {
    /// Street adress
    pub address1: Option<String>,
    /// Supplemental address information
    pub address2: Option<String>,
    /// Supplemental address information (2)
    pub address3: Option<String>,
    /// Post code
    pub postal_code: Option<String>,
    /// City
    pub city: Option<String>,
    /// Country
    pub country: Option<String>,
    /// Lattitude, assumed to be WGS84
    pub latitude: Option<f64>,
    /// Longitude, assumed to be WGS84
    pub longitude: Option<f64>,
}

impl Address {
    fn new(addr: home::HomeViewerHomeAddress) -> Self {
        Address {
            address1: addr.address1,
            address2: addr.address2,
            address3: addr.address3,
            postal_code: addr.postal_code,
            city: addr.city,
            country: addr.country,
            latitude: match addr.latitude {
                Some(l) => l.parse::<f64>().ok(),
                _ => None,
            },
            longitude: match addr.longitude {
                Some(l) => l.parse::<f64>().ok(),
                _ => None,
            },
        }
    }
}

#[derive(Debug, Default, Clone)]
/// Other features of the home / house
pub struct Features {
    /// Realtime consumption data is available
    pub real_time_consumption_enabled: Option<bool>,
}

impl Features {
    fn new(feat: home::HomeViewerHomeFeatures) -> Self {
        Features {
            real_time_consumption_enabled: feat.real_time_consumption_enabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Rough classification of price level compared to a 3 day moving avarage
pub enum PriceLevel {
    /// Much lower than avarage
    VeryCheap,
    /// Lover than avaragre
    Cheap,
    /// Around avarage
    Normal,
    /// Higher than avarage
    Expensive,
    /// Much higher than avarage
    VeryExpensive,
    /// Other
    Other(String),
    /// Missing data
    None,
}

#[derive(Debug, Clone, PartialEq)]
/// Information about price in a particular interval
pub struct PriceInfo {
    /// Total price
    pub total: f64,
    /// Energy cost component of price
    pub energy: f64,
    /// Taxes to be added to energy cost
    pub tax: f64,
    /// When this pricing interval started
    pub starts_at: DateTime<FixedOffset>,
    /// The currency that is used to set price
    pub currency: String,
    /// Classification of price relative to avarage
    pub level: PriceLevel,
}

impl PriceInfo {
    fn new(pinfo: price::PriceViewerHomeCurrentSubscriptionPriceInfoCurrent) -> Option<Self> {
        let total = pinfo.total?;
        let (energy, tax) = match (pinfo.energy, pinfo.tax) {
            (Some(e), Some(t)) => (e, t),
            (Some(e), None) => (e, total - e),
            (None, Some(t)) => (total - t, t),
            _ => (total, 0.0),
        };
        let level = match pinfo.level {
            Some(price::PriceLevel::VERY_CHEAP) => PriceLevel::VeryCheap,
            Some(price::PriceLevel::CHEAP) => PriceLevel::Cheap,
            Some(price::PriceLevel::NORMAL) => PriceLevel::Normal,
            Some(price::PriceLevel::EXPENSIVE) => PriceLevel::Expensive,
            Some(price::PriceLevel::VERY_EXPENSIVE) => PriceLevel::VeryExpensive,
            Some(price::PriceLevel::Other(s)) => PriceLevel::Other(s),
            _ => PriceLevel::None,
        };
        let starts_at = chrono::DateTime::parse_from_rfc3339(
            pinfo.starts_at.ok_or("no starts_at time").ok()?.as_str(),
        )
        .ok()?;
        Some(PriceInfo {
            total,
            energy,
            tax,
            starts_at,
            currency: pinfo.currency,
            level,
        })
    }

    fn new_t(
        pinfo: price_today::PriceTodayViewerHomeCurrentSubscriptionPriceInfoToday,
    ) -> Option<Self> {
        let total = pinfo.total?;
        let (energy, tax) = match (pinfo.energy, pinfo.tax) {
            (Some(e), Some(t)) => (e, t),
            (Some(e), None) => (e, total - e),
            (None, Some(t)) => (total - t, t),
            _ => (total, 0.0),
        };
        let level = match pinfo.level {
            Some(price_today::PriceLevel::VERY_CHEAP) => PriceLevel::VeryCheap,
            Some(price_today::PriceLevel::CHEAP) => PriceLevel::Cheap,
            Some(price_today::PriceLevel::NORMAL) => PriceLevel::Normal,
            Some(price_today::PriceLevel::EXPENSIVE) => PriceLevel::Expensive,
            Some(price_today::PriceLevel::VERY_EXPENSIVE) => PriceLevel::VeryExpensive,
            Some(price_today::PriceLevel::Other(s)) => PriceLevel::Other(s),
            _ => PriceLevel::None,
        };
        let starts_at = chrono::DateTime::parse_from_rfc3339(
            pinfo.starts_at.ok_or("no starts_at time").ok()?.as_str(),
        )
        .ok()?;
        Some(PriceInfo {
            total,
            energy,
            tax,
            starts_at,
            currency: pinfo.currency,
            level,
        })
    }

    fn new_f(
        pinfo: price_tomorrow::PriceTomorrowViewerHomeCurrentSubscriptionPriceInfoTomorrow,
    ) -> Option<Self> {
        let total = pinfo.total?;
        let (energy, tax) = match (pinfo.energy, pinfo.tax) {
            (Some(e), Some(t)) => (e, t),
            (Some(e), None) => (e, total - e),
            (None, Some(t)) => (total - t, t),
            _ => (total, 0.0),
        };
        let level = match pinfo.level {
            Some(price_tomorrow::PriceLevel::VERY_CHEAP) => PriceLevel::VeryCheap,
            Some(price_tomorrow::PriceLevel::CHEAP) => PriceLevel::Cheap,
            Some(price_tomorrow::PriceLevel::NORMAL) => PriceLevel::Normal,
            Some(price_tomorrow::PriceLevel::EXPENSIVE) => PriceLevel::Expensive,
            Some(price_tomorrow::PriceLevel::VERY_EXPENSIVE) => PriceLevel::VeryExpensive,
            Some(price_tomorrow::PriceLevel::Other(s)) => PriceLevel::Other(s),
            _ => PriceLevel::None,
        };
        let starts_at = chrono::DateTime::parse_from_rfc3339(
            pinfo.starts_at.ok_or("no starts_at time").ok()?.as_str(),
        )
        .ok()?;
        Some(PriceInfo {
            total,
            energy,
            tax,
            starts_at,
            currency: pinfo.currency,
            level,
        })
    }
}

#[derive(Debug, Clone)]
/// Information about a home / house
pub struct House {
    /// Time zone where the building is located
    pub time_zone: String,
    /// Nickname given to building in App
    pub app_nickname: Option<String>,
    /// Size of house in square meters if given
    pub size: Option<u32>,
    /// Type of building
    pub house_type: HouseType,
    /// Number of residents if given
    pub number_of_residents: Option<u32>,
    /// Primary heating source
    pub primary_heating_source: HeatingSource,
    /// Does the building have ventilation system
    pub has_ventilation_system: Option<bool>,
    /// Size of main fuse in Amperes if given
    pub main_fuse_size: Option<u32>,
    /// Address information
    pub address: Option<Address>,
    /// Other features
    pub features: Features,
}

#[derive(Debug, Clone)]
/// Resolution of time when requestiong consumption data
pub enum TimeResolution {
    /// Hourely intervals
    Hourly,
    /// Daily intervals
    Daily,
    /// Weekly intervals
    Weekly,
    /// Monthly intervals
    Monthly,
    /// Yearly intervals
    Annual,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
/// Measurement if energy with appropriate units
pub enum EnergyUnits {
    #[allow(non_camel_case_types)]
    /// KiloWatt hours
    kWh(f64),
    /// Not given
    None,
}

#[derive(Debug, Clone)]
/// Consumption data
pub struct Consumption {
    /// Start of interval
    pub from: DateTime<FixedOffset>,
    /// End of interval
    pub to: DateTime<FixedOffset>,
    /// Total price
    pub cost: f64,
    /// Price pr unit
    pub unit_price: f64,
    /// Tax pr unit
    pub unit_price_vat: f64,
    /// Type of units and units
    pub energy: EnergyUnits,
}

impl Consumption {
    fn new(
        node: consumption_history::ConsumptionHistoryViewerHomeConsumptionNodes,
    ) -> Option<Self> {
        let cost = node.cost?;
        let unit_price = node.unit_price?;
        let unit_price_vat = node.unit_price_vat?;
        let energy = match node.consumption_unit {
            Some(s) if s.as_str() == "kWh" => match node.consumption {
                Some(c) => EnergyUnits::kWh(c),
                _ => EnergyUnits::None,
            },
            _ => EnergyUnits::None,
        };
        let from = DateTime::parse_from_rfc3339(node.from.as_str()).ok()?;
        let to = DateTime::parse_from_rfc3339(node.to.as_str()).ok()?;
        Some(Consumption {
            from,
            to,
            cost,
            unit_price,
            unit_price_vat,
            energy,
        })
    }
}

#[derive(Debug, Clone)]
/// Production data
pub struct Production {
    /// Start of interval
    pub from: DateTime<FixedOffset>,
    /// End of interval
    pub to: DateTime<FixedOffset>,
    /// Total price
    pub profit: f64,
    /// Price pr unit
    pub unit_price: f64,
    /// Tax pr unit
    pub unit_price_vat: f64,
    /// Type of units and units
    pub energy: EnergyUnits,
}

impl Production {
    fn new(node: production_history::ProductionHistoryViewerHomeProductionNodes) -> Option<Self> {
        let profit = node.profit?;
        let unit_price = node.unit_price?;
        let unit_price_vat = node.unit_price_vat?;
        let energy = match node.production_unit {
            Some(s) if s.as_str() == "kWh" => match node.production {
                Some(c) => EnergyUnits::kWh(c),
                _ => EnergyUnits::None,
            },
            _ => EnergyUnits::None,
        };
        let from = DateTime::parse_from_rfc3339(node.from.as_str()).ok()?;
        let to = DateTime::parse_from_rfc3339(node.to.as_str()).ok()?;
        Some(Production {
            from,
            to,
            profit,
            unit_price,
            unit_price_vat,
            energy,
        })
    }
}

/// A tibber session, can be shared among threads, only holds the API token
pub struct TibberSession {
    authentication: String,
}

impl TibberSession {
    /// Create a new session object
    pub fn new(authentication: String) -> Self {
        TibberSession { authentication }
    }

    /// Get information about the logged in user
    pub fn get_user(&self) -> Result<User, Box<dyn std::error::Error>> {
        let viewer = fetch_data::<Viewer>(self.authentication.as_str(), viewer::Variables {})?;
        let login = viewer.viewer.login.ok_or("No login")?;
        let user_id = viewer.viewer.user_id.ok_or("No UserId")?;
        let name = viewer.viewer.name.ok_or("No Name")?;
        let account_type = viewer.viewer.account_type;
        let homes = viewer
            .viewer
            .homes
            .into_iter()
            .flatten()
            .map(|h| HomeId(h.id))
            .collect();
        Ok(User {
            login,
            user_id,
            name,
            account_type,
            homes,
        })
    }

    /// Get information about a particular home / house
    pub fn get_home(&self, home_id: &HomeId) -> Result<House, Box<dyn std::error::Error>> {
        let id = home_id.0.to_owned();
        let home = fetch_data::<Home>(self.authentication.as_str(), home::Variables { id })?;
        let time_zone = home.viewer.home.time_zone;
        let app_nickname = home.viewer.home.app_nickname;
        let size = match home.viewer.home.size {
            Some(s) if s >= 0 => Some(s as u32),
            _ => None,
        };
        let house_type = HouseType::new(home.viewer.home.type_);
        let number_of_residents = match home.viewer.home.number_of_residents {
            Some(n) if n >= 0 => Some(n as u32),
            _ => None,
        };
        let primary_heating_source = HeatingSource::new(home.viewer.home.primary_heating_source);
        let has_ventilation_system = home.viewer.home.has_ventilation_system;
        let main_fuse_size = match home.viewer.home.main_fuse_size {
            Some(s) if s >= 0 => Some(s as u32),
            _ => None,
        };
        let address = home.viewer.home.address.map(Address::new);
        let features = match home.viewer.home.features {
            Some(f) => Features::new(f),
            _ => Default::default(),
        };

        Ok(House {
            time_zone,
            app_nickname,
            size,
            house_type,
            number_of_residents,
            primary_heating_source,
            has_ventilation_system,
            main_fuse_size,
            address,
            features,
        })
    }

    /// Get Current price information for a particular house / home
    pub fn get_current_price(
        &self,
        home_id: &HomeId,
    ) -> Result<PriceInfo, Box<dyn std::error::Error>> {
        let id = home_id.0.to_owned();
        let price = fetch_data::<Price>(self.authentication.as_str(), price::Variables { id })?;
        let price = PriceInfo::new(
            price
                .viewer
                .home
                .current_subscription
                .ok_or("No subscription")?
                .price_info
                .ok_or("No Price info")?
                .current
                .ok_or("No current price")?,
        )
        .ok_or("Could not parse price info")?;
        Ok(price)
    }

    /// Get full day price information for a particular house / home
    pub fn get_prices_today(
        &self,
        home_id: &HomeId,
    ) -> Result<Vec<PriceInfo>, Box<dyn std::error::Error>> {
        let id = home_id.0.to_owned();
        let price =
            fetch_data::<PriceToday>(self.authentication.as_str(), price_today::Variables { id })?;
        let prices = price
            .viewer
            .home
            .current_subscription
            .ok_or("No subscription")?
            .price_info
            .ok_or("No Price info")?
            .today;
        let prices = prices
            .into_iter()
            .flatten()
            .map(PriceInfo::new_t)
            .flatten()
            .collect();
        Ok(prices)
    }

    /// Get tomorrows prices (if available) for a particular house / home
    pub fn get_prices_tomorrow(
        &self,
        home_id: &HomeId,
    ) -> Result<Vec<PriceInfo>, Box<dyn std::error::Error>> {
        let id = home_id.0.to_owned();
        let price = fetch_data::<PriceTomorrow>(
            self.authentication.as_str(),
            price_tomorrow::Variables { id },
        )?;
        let prices = price
            .viewer
            .home
            .current_subscription
            .ok_or("No subscription")?
            .price_info
            .ok_or("No Price info")?
            .tomorrow;
        let prices = prices
            .into_iter()
            .flatten()
            .map(PriceInfo::new_f)
            .flatten()
            .collect();
        Ok(prices)
    }

    /// Get historical consumption data for a particular house / home
    pub fn get_consuption(
        &self,
        home_id: &HomeId,
        resolution: TimeResolution,
        last: u32,
    ) -> Result<Vec<Consumption>, Box<dyn std::error::Error>> {
        let id = home_id.0.to_owned();
        let resolution = match resolution {
            TimeResolution::Hourly => consumption_history::EnergyResolution::HOURLY,
            TimeResolution::Daily => consumption_history::EnergyResolution::DAILY,
            TimeResolution::Weekly => consumption_history::EnergyResolution::WEEKLY,
            TimeResolution::Monthly => consumption_history::EnergyResolution::MONTHLY,
            TimeResolution::Annual => consumption_history::EnergyResolution::ANNUAL,
        };
        let variables = consumption_history::Variables {
            id,
            resolution,
            num: last.into(),
        };
        let history = fetch_data::<ConsumptionHistory>(self.authentication.as_str(), variables)?;
        let history = history
            .viewer
            .home
            .consumption
            .ok_or("No History")?
            .nodes
            .ok_or("No history nodes")?
            .into_iter()
            .flatten()
            .map(Consumption::new)
            .flatten()
            .collect();
        Ok(history)
    }

    /// Get historical production data for a particular house / home
    pub fn get_production(
        &self,
        home_id: &HomeId,
        resolution: TimeResolution,
        last: u32,
    ) -> Result<Vec<Production>, Box<dyn std::error::Error>> {
        let id = home_id.0.to_owned();
        let resolution = match resolution {
            TimeResolution::Hourly => production_history::EnergyResolution::HOURLY,
            TimeResolution::Daily => production_history::EnergyResolution::DAILY,
            TimeResolution::Weekly => production_history::EnergyResolution::WEEKLY,
            TimeResolution::Monthly => production_history::EnergyResolution::MONTHLY,
            TimeResolution::Annual => production_history::EnergyResolution::ANNUAL,
        };
        let variables = production_history::Variables {
            id,
            resolution,
            num: last.into(),
        };
        let history = fetch_data::<ProductionHistory>(self.authentication.as_str(), variables)?;
        let history = history
            .viewer
            .home
            .production
            .ok_or("No History")?
            .nodes
            .ok_or("No history nodes")?
            .into_iter()
            .flatten()
            .map(Production::new)
            .flatten()
            .collect();
        Ok(history)
    }
}
