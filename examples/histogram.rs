use chrono::Timelike;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use tibber::*;

/// Create hourly histogram of usage for last n days
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

    let consumption = conn
        .get_consuption(&user.homes[0], TimeResolution::Hourly, last)
        .unwrap_or_default();

    let mut hist = HashMap::new();
    for c in &consumption {
        if let EnergyUnits::kWh(energy) = c.energy {
            match hist.entry(c.from.hour()) {
                Entry::Occupied(mut entry) => *entry.get_mut() += energy,
                Entry::Vacant(entry) => {
                    entry.insert(energy);
                }
            }
        }
    }

    println!("{:#?}", hist);
    for i in 0..24 {
        println!("{} {}", i, hist.get(&i).unwrap_or(&0.0));
    }
}
