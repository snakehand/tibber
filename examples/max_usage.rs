use chrono::Timelike;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use tibber::*;

/// Find max hourly usages last n days
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
        println!("usage: histogram n  ( n last days )");
        return;
    }

    let last = 24 * args[0].parse::<u32>().unwrap_or(10);

    let mut consumption = conn
        .get_consuption(&user.homes[0], TimeResolution::Hourly, last)
        .unwrap_or_default();

    consumption.sort_by(|a, b| b.energy.partial_cmp(&a.energy).unwrap());

    let max: Vec<Consumption> = consumption.into_iter().take(3).collect();

    let sum: f64 = max
        .iter()
        .map(|c| {
            if let EnergyUnits::kWh(e) = c.energy {
                e
            } else {
                0.0
            }
        })
        .sum();

    if !max.is_empty() {
        println!("Max usage: {}", sum / max.len() as f64);
    }

    println!("Max {:#?}", max);
}
