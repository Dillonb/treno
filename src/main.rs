use amtrak_api::{Client, Train};
use chrono::TimeDelta;

fn arrival_delta_to_human_string(mut delta: TimeDelta) -> String {
    let mut time = String::new();

    let early_or_late = if delta.num_seconds() < 0 {
        delta = -delta;
        "early"
    } else {
        "late"
    };

    let s_or_not = |n: i64| if n == 1 { "" } else { "s" };

    time.push_str(&format!(
        "{} hour{} ",
        delta.num_hours(),
        s_or_not(delta.num_hours())
    ));
    delta = delta - chrono::Duration::hours(delta.num_hours());

    time.push_str(&format!(
        "{} minute{} ",
        delta.num_minutes(),
        s_or_not(delta.num_minutes())
    ));
    delta = delta - chrono::Duration::minutes(delta.num_minutes());

    time.push_str(&format!(
        "{} second{}",
        delta.num_seconds(),
        s_or_not(delta.num_seconds())
    ));
    return format!("{} {}", time, early_or_late);
}

fn display_train(train: &Train) {
    let next_station = &train.event_code;

    println!(
        "Train ID {} from {} to {} \"{}\"",
        train.train_id, train.origin_code, train.destination_code, train.route_name
    );

    let next_station = train
        .stations
        .iter()
        .find(|s| s.code == *next_station)
        .unwrap();
    println!(
        "Next station: {} ({})",
        next_station.name, next_station.code
    );

    let prev_station = train
        .stations
        .iter()
        .take_while(|s| s.code != next_station.code)
        .last()
        .map(|s| s.code.clone())
        .unwrap_or("".to_string());

    println!(
        "\tNext Station: {} ({})",
        next_station.name, next_station.code
    );

    {
        println!("\t\tScheduled: {}", next_station.schedule_arrival);
        println!(
            "\t\tActual: {:?}",
            next_station
                .arrival
                .map(|a| a.to_string())
                .unwrap_or("Unknown".to_string())
        );
        let arrival_delta = next_station
            .arrival
            .map(|arrival| arrival - next_station.schedule_arrival)
            .map(|delta| arrival_delta_to_human_string(delta))
            .unwrap_or("Unknown".to_string());
        println!("\t\t{}", arrival_delta);
    }

    let longest_station_code_len = train
        .stations
        .iter()
        .map(|s| s.code.len())
        .max()
        .unwrap_or(0);
    let spaces = " ".repeat(longest_station_code_len + 1);

    let train_icon = "󰣄";
    let lines = train
        .stations
        .iter()
        .flat_map(|station| {
            let pipes = "|".repeat((longest_station_code_len + 1) - station.code.len());
            let station_line = format!("{}{}-", station.code, pipes);

            let spacer_line = format!("{}-", spaces);
            let train_icon_line = if station.code == prev_station {
                format!("{}{}", spaces, train_icon)
            } else {
                spacer_line.clone()
            };

            if station.code == train.destination_code {
                vec![station_line]
            } else {
                vec![station_line, spacer_line, train_icon_line]
            }
        })
        .collect::<Vec<String>>();

    let line_len = lines.first().unwrap().len();
    for i in 0..line_len {
        for line in &lines {
            print!("{}", line.chars().nth(i).unwrap());
        }
        println!("");
    }
}

async fn display_train_id(train_id: &str) -> Result<(), Box<dyn std::error::Error>> {
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
        eprintln!(
            "No train found with ID: {}. Check the date and timetables, has the train left the origin station yet?",
            train_id
        );
        return Ok(());
    }
    let trains = maybe_trains.unwrap();
    display_train(trains.first().unwrap());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} [<train_id>|<train_number>|all]", args[0]);
        eprintln!(
            "Train ID format: '7-27' for the train 7 that left the station on the 27th, or 7-4 for train 7 that left the station on the 4th of the month."
        );
        return Ok(());
    }

    let train_id = &args[1];
    if train_id.contains("-") {
        display_train_id(train_id).await
    } else if train_id == "all" {
        let client = Client::new();
        let trains = client.trains().await?;
        trains
            .values()
            .flat_map(|trains| trains)
            .filter(|train| train.provider == "Amtrak")
            .for_each(|train| {
                display_train(&train);
                println!();
            });
        Ok(())
    } else {
        eprintln!("Invalid train ID format. Expected format: <train_number>-<date> or 'all'");
        Ok(())
    }
}
