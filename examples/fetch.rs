use tibber::*;

fn main() {
    let api_token = match std::env::var("TIBBER_API_TOKEN") {
        Ok(v) => v,
        _ => {
            panic!("Set TIBBER_API_TOKEN environmental variable")
        }
    };

    let conn = TibberSession::new(api_token);
    let user = conn.get_user().unwrap();
    println!("{:#?}", user);

    for h in &user.homes {
        let home = conn.get_home(h);
        println!("{:#?}", home);
        let price = conn.get_current_price(h);
        println!("{:#?}", price);
        let price = conn.get_prices_today(h);
        println!("{:#?}", price);
        let price = conn.get_prices_tomorrow(h);
        println!("{:#?}", price);
        let history = conn.get_consuption(h, TimeResolution::Daily, 10);
        println!("{:#?}", history);
    }
}
