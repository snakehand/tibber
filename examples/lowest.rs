use chrono::prelude::*;
use tibber::*;

/// Get prices for next n hours and sort them in ascending order
fn main() {
    let api_token = match std::env::var("TIBBER_API_TOKEN") {
        Ok(v) => v,
        _ => {
            panic!("Set TIBBER_API_TOKEN environmental variable")
        }
    };
    let args: Vec<String> = std::env::args().skip(1).collect();

    let conn = TibberSession::new(api_token);
    let user = conn.get_user().unwrap();
    if user.homes.is_empty() {
        println!("No homes found");
        return;
    }

    if args.is_empty() {
        println!("usage: lowest n  ( n next hours )");
        return;
    }

    let next_h = 3600 * args[0].parse::<i64>().unwrap_or(24);
    let local: chrono::DateTime<FixedOffset> = Local::now().into();

    let mut prices = conn.get_prices_today(&user.homes[0]).unwrap_or_default();
    for p in conn.get_prices_tomorrow(&user.homes[0]).unwrap_or_default() {
        prices.push(p);
    }
    let mut prices: Vec<PriceInfo> = prices
        .into_iter()
        .filter(|p| {
            let diff = p.starts_at.timestamp() - local.timestamp();
            (diff > 0) && (diff < next_h)
        })
        .collect();

    prices.sort_by(|a, b| a.total.partial_cmp(&b.total).unwrap());

    // println!("{:#?}", prices);
    for p in prices {
        println!("{} {}", p.starts_at, p.total);
    }
}
