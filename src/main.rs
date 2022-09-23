#![allow(dead_code)]

use std::error::Error;
use std::hash::Hash;
use std::path::Path;

use geoutils::Location;
use serde::Deserialize;

use std::io::Write;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Entry {
    #[serde(rename = "YEAR")]
    time_period: String,

    #[serde(rename = "STATION")]
    station: String,

    #[serde(rename = "Entries 0600-1000")]
    #[serde(deserialize_with = "csv::invalid_option")]
    entries_morning: Option<i32>,

    #[serde(rename = "Exits 0600-1000")]
    #[serde(deserialize_with = "csv::invalid_option")]
    exits_morning: Option<i32>,

    #[serde(rename = "Entries 1000-1500")]
    #[serde(deserialize_with = "csv::invalid_option")]
    entries_midday: Option<i32>,

    #[serde(rename = "Exits 1000-1500")]
    #[serde(deserialize_with = "csv::invalid_option")]
    exits_midday: Option<i32>,

    #[serde(rename = "Entries 1500-1900")]
    #[serde(deserialize_with = "csv::invalid_option")]
    entries_evening: Option<i32>,

    #[serde(rename = "Exits 1500-1900")]
    #[serde(deserialize_with = "csv::invalid_option")]
    exits_evening: Option<i32>,

    #[serde(rename = "Entries 1900 -0600")]
    #[serde(deserialize_with = "csv::invalid_option")]
    entries_midnight: Option<i32>,

    #[serde(rename = "Exits 1900 -0600")]
    #[serde(deserialize_with = "csv::invalid_option")]
    exits_midnight: Option<i32>,

    #[serde(rename = "Entries 0000-2359")]
    #[serde(deserialize_with = "csv::invalid_option")]
    entries_total: Option<i32>,

    #[serde(rename = "Exits 0000-2359")]
    #[serde(deserialize_with = "csv::invalid_option")]
    exits_total: Option<i32>,

    #[serde(rename = "LAT")]
    latitude: f64,

    #[serde(rename = "LONG")]
    longitude: f64,
}
#[derive(Clone)]
struct LatLong {
    latitude: f64,
    longitude: f64
}

#[derive(Clone)]
struct BetterEntry {
    thoroughfare_usage_morning: i32,
    thoroughfare_usage_afternoon: i32,
    thoroughfare_usage_evening: i32,
    thoroughfare_usage_night: i32,
    thoroughfare_usage_total: i32,
    period: String,
    location: LatLong,
}

impl BetterEntry {
    fn get_usages(&self) -> HashMap<String, i32> {
        let usagesMap = HashMap::new();
        usagesMap.insert(String::from("Morning"), self.thoroughfare_usage_morning);
        usagesMap.insert(String::from("Afternoon"), self.thoroughfare_usage_afternoon);
        usagesMap.insert(String::from("Evening"), self.thoroughfare_usage_evening);
        usagesMap.insert(String::from("Night"), self.thoroughfare_usage_night);

        usagesMap
    }

    fn get_busiest_time(&self) -> (String, i32) {
        let key_values = vec![];
        for key in self.get_usages().keys() {
            key_values.push(self.get_usages().get_key_value(key))
        }

        key_values.into_iter().max_by_key(|tuple| tuple.1)
    }
}

fn toBetterEntries(data: Vec<Entry>) -> HashMap<String, Vec<BetterEntry>> {
    let mut entries: HashMap<String, Vec<BetterEntry>> = HashMap::new();
    for station in data {
        let numMutator = |entries, exits| {
            let entry_num = match entries {
                Some(entry) => entry,
                None => 0,
            };
            let exit_num = match exits {
                Some(exit) => exit,
                None => 0
            };
            
            entry_num + exit_num
        };

        let newEntry = BetterEntry {
            thoroughfare_usage_morning: numMutator(station.entries_morning, station.exits_morning),
            thoroughfare_usage_afternoon: numMutator(station.entries_midday, station.exits_midday),
            thoroughfare_usage_evening: numMutator(station.entries_evening, station.exits_evening),
            thoroughfare_usage_night: numMutator(station.entries_midnight, station.exits_midnight),
            thoroughfare_usage_total: numMutator(station.entries_total, station.exits_total),
            period: station.time_period,
            location: LatLong {
                latitude: station.latitude,
                longitude: station.longitude
            }
        };
        
        match entries.get(&station.station) {
            Some(entry) => { 
                let mut cloned_entry = entry.clone();
                cloned_entry.push(newEntry);
                entries.insert(station.station, cloned_entry);
            },
            None => { entries.insert(station.station, vec![newEntry]); },
        };
    }

    entries
}

/// To create a location, run:
///
/// ```rust
/// let berlin = Location::new(52.518611, 13.408056);
/// ```
///
/// then pass two locations into this function for a
/// distance in meters.
fn distance_in_meters(point1: Location, point2: Location) -> f64 {
    point1.distance_to(&point2).unwrap().meters()
}

fn query_user(prompt: String) -> String {
    print!("{prompt}");
    std::io::stdout().flush().unwrap();
    let mut response = String::new();
    std::io::stdin().read_line(&mut response).expect("Failed to read user input. Fuck");
    
    response
}

fn load_station_data(path: &Path) -> Result<HashMap<String, Vec<BetterEntry>>, Box<dyn Error>> {
    let entries: Vec<Entry> = csv::Reader::from_path(&path)?
        .deserialize()
        .collect::<Result<_, _>>()?;

    Ok(toBetterEntries(entries))
} 

fn load_station(name: String, data: HashMap<String, Vec<BetterEntry>>) -> Option<Vec<BetterEntry>> {
    let station_data = data.get(&name);
    match station_data {
        Some(entries) => {
            let cloned_entry = entries.clone();
            return Some(cloned_entry);
        },
        None => return None,
    }
}

fn output_station_busiest_times(station_name: String, data: HashMap<String, Vec<BetterEntry>>) {
    let station_data = load_station(station_name, data);
    match station_data {
        Some(entries) => {
            let mut max = (String::new(), 0_i32);
            for entry in entries {
                if entry.get_busiest_time().1 > max.1 {
                    max = entry.get_busiest_time();
                }
            }
            println!("Busiest time at station {station_name} was {}", max.0);
        },
        None => println!("No data found for station {station_name}"),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = Path::new("trains.csv");
    let data = load_station_data(path)?;
    max_min(data);

    Ok(())
}

fn max_min(entries:HashMap<String, Vec<BetterEntry>>) -> () {
    let mut max_morning:i32 = 0;
    let mut max_midday:i32 = 0;
    let mut max_evening:i32 = 0;    
    let mut max_midnight:i32 = 0;

    let mut min_morning:i32 = i32::MAX;
    let mut min_midday:i32 = i32::MAX;
    let mut min_evening:i32 = i32::MAX;
    let mut min_midnight:i32 = i32::MAX;

    let mut min_morning_station:String = String::new();
    let mut max_morning_station:String = String::new();    
    let mut min_midday_station:String = String::new();    
    let mut max_midday_station:String = String::new();    
    let mut min_evening_station:String = String::new();    
    let mut max_evening_station:String = String::new();    
    let mut min_midnight_station:String = String::new();
    let mut max_midnight_station:String = String::new();

    // loop over hashmap keys/values
    // for every pair, go through vec of thoroughfare data
    for (name, datas) in &entries {
        for entry in datas{
            if max_morning < entry.thoroughfare_usage_morning {
                max_morning = entry.thoroughfare_usage_morning;
                max_morning_station = name.to_string();
            }
            if max_midday < entry.thoroughfare_usage_afternoon {
                max_midday = entry.thoroughfare_usage_afternoon;
                max_midday_station = name.to_string();
            }
            if max_evening < entry.thoroughfare_usage_evening {
                max_evening = entry.thoroughfare_usage_evening;
                max_evening_station = name.to_string();
            }
            if max_midnight < entry.thoroughfare_usage_night {
                max_midnight = entry.thoroughfare_usage_night;
                max_midnight_station = name.to_string();
            }
            if min_morning > entry.thoroughfare_usage_morning {
                max_morning = entry.thoroughfare_usage_morning;
                max_morning_station = name.to_string();
            }
            if min_midday > entry.thoroughfare_usage_afternoon {
                max_midday = entry.thoroughfare_usage_afternoon;
                max_midday_station = name.to_string();
            }
            if min_evening > entry.thoroughfare_usage_evening {
                max_evening = entry.thoroughfare_usage_evening;
                max_evening_station = name.to_string();
            }
            if min_midnight > entry.thoroughfare_usage_night {
                max_midnight = entry.thoroughfare_usage_night;
                max_midnight_station = name.to_string();
            }
        }
    }

    println!("Morning - Min {}, Max {}", min_morning_station, max_morning_station);
    println!("Midday - Min {}, Max {}", min_midday_station, max_midday_station);
    println!("Evening - Min {}, Max {}", min_evening_station, max_evening_station);
    println!("Midnight - Min {}, Max {}", min_midnight_station, max_midnight_station);
}

// fn closest_furthest(entries:HashMap<String, Vec<BetterEntry>>) -> () {
    // let mut distances:HashMap<array,f64>;
    // for (name, data1) in entries {
        // (<array<String>>, f64)

        // double-loop over all stations, find all distances
        // use min/max_by_key to find closest/furthest
    // }
// }