use amtrak_api::Client;
use chrono::TimeDelta;

// const STATION_CODE: &str = "SEA";

fn arrival_delta_to_human_string(mut delta: TimeDelta) -> String {
    let mut time = String::new();

    let early_or_late = if delta.num_seconds() < 0 {
        delta = -delta;
        "early"
    } else {
        "late"
    };

    time.push_str(&format!("{} hours ", delta.num_hours()));
    delta = delta - chrono::Duration::hours(delta.num_hours());

    time.push_str(&format!("{} minutes ", delta.num_minutes()));
    delta = delta - chrono::Duration::minutes(delta.num_minutes());

    time.push_str(&format!("{} seconds", delta.num_seconds()));
    return format!("{} {}", time, early_or_late);
    // println!("Train is {} {}", time, early_or_late);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <train_id>", args[0]);
        eprintln!("Example: '7-27' for the train 7 that left the station on the 27th, or 7-4 for train 7 that left the station on the 4th of the month.");
        return Ok(());
    }

    let train_id = &args[1];
    let train_id_parts: Vec<&str> = train_id.split('-').collect();
    if train_id_parts.len() != 2 {
        eprintln!("Invalid train ID format. Expected format: <train_number>-<date>");
        return Ok(());
    }
    let train_number = train_id_parts[0];

    let client = Client::new();

    let result = client.train(&train_id).await?;
    let maybe_trains = result.get(train_number);
    if maybe_trains.is_none() {
        eprintln!("No train found with ID: {}. Check the date and timetables, has the train left the origin station yet?", train_id);
        return Ok(());
    }
    let trains = maybe_trains.unwrap();
    let train = trains.first().unwrap();
    let next_station = &train.event_code;

    println!(
        "Train ID {} from {} to {}",
        train.train_id, train.origin_code, train.destination_code
    );
    let next_station = train
        .stations
        .iter()
        .find(|s| s.code == *next_station)
        .unwrap();

    println!("\tNext Station: {} ({})", next_station.name, next_station.code);

    {
        println!("\t\tScheduled: {}", next_station.schedule_arrival);
        println!("\t\tActual: {:?}", next_station.arrival.unwrap());
        let arrival_delta = next_station.arrival.unwrap() - next_station.schedule_arrival;
        println!("\t\t{}", arrival_delta_to_human_string(arrival_delta));
    }

    Ok(())
}
