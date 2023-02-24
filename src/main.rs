use nickel::{HttpRouter, Nickel, QueryString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[macro_use]
extern crate nickel;

#[derive(Debug, Serialize, Deserialize)]
pub struct Horoscope {
    date_range: String,
    current_date: String,
    description: String,
    compatibility: String,
    mood: String,
    color: String,
    lucky_number: String,
    lucky_time: String,
}

impl Default for Horoscope {
    fn default() -> Horoscope {
        Horoscope {
            date_range: String::from("default"),
            current_date: String::from("default"),
            description: String::from("default"),
            compatibility: String::from("default"),
            mood: String::from("default"),
            color: String::from("default"),
            lucky_number: String::from("default"),
            lucky_time: String::from("default"),
        }
    }
}

fn get_horoscope(stored_types: &mut HashMap<String, Horoscope>, horo_type: String) -> String {
    let horo = horo_type;

    if stored_types.contains_key(&horo) {
        println!("Cache hit for: {horo}");

        horostring(stored_types.get(&horo).unwrap())
    } else {
        println!("Hitting API for {horo}");

        let fresh_val = hit_api(&horo);

        println!("Caching {horo}");

        stored_types.insert(horo.to_string(), fresh_val);

        horostring(stored_types.get(&horo).unwrap())
    }
}

fn horostring(cancer_horoscope: &Horoscope) -> String {
    serde_json::to_string(cancer_horoscope).unwrap()
}

fn hit_api(horo_type: &str) -> Horoscope {
    println!("hitting API");
    let client = reqwest::blocking::Client::new();

    let resp = client
        .post(format!(
            "https://aztro.sameerkumar.website/?sign={horo_type}&day=today",
        ))
        .send()
        .unwrap()
        .json::<Horoscope>()
        .unwrap();

    resp
}

fn main() {
    let config_wait_time = 60 * 60 * 12; //12 hours

    let stored_types: Arc<Mutex<HashMap<String, Horoscope>>> = Arc::new(Mutex::new(HashMap::new()));

    let thread_handle_to_map = Arc::clone(&stored_types);
    let main_handle_to_map = Arc::clone(&stored_types);

    let handle = thread::spawn(move || loop {
        let mut hashmap_handle = thread_handle_to_map.lock().unwrap();
        let mut temp: HashMap<String, Horoscope> = HashMap::new();

        for sign in hashmap_handle.iter() {
            println!("Loop hitting API");
            let new_val = hit_api(sign.0);

            println!("Successfully received {} response", sign.0);

            println!("Cached resp: {:?}", sign.1);
            temp.insert(sign.0.to_string(), new_val);
        }
        hashmap_handle.extend(temp);

        drop(hashmap_handle);

        thread::sleep(Duration::from_secs(config_wait_time));
    });

    //Idle/Serve
    let mut server = Nickel::new();
    server.get(
        "/horoscope",
        middleware! {|request, response|
                println!("asked for: {0:?}", request.query().all("horoType").unwrap());
            let mut main_handle = main_handle_to_map.lock().unwrap();

            let resp=get_horoscope(&mut main_handle, request.query().all("horoType").unwrap().get(0).unwrap().to_string());

            std::mem::drop(main_handle);

            resp
        },
    );

    server.listen("0.0.0.0:3030").unwrap();
}
