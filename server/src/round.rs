use geoutils::Location;
use shared::RoundData;

use crate::server::State;
use crate::{Server, error::Error, images::images};
use shared::Player;

pub async fn new(server: &mut Server, old: Option<&RoundData>) -> Result<State, Error> {
    eprintln!("server: initializing round");
    server.clients.iter_mut().for_each(|x| x.ready = false);
    let lobby = server.lobby().await;
    server
        .broadcast(&shared::Packet::RoundLoading { lobby }, None)
        .await;

    let number = old.and_then(|x| Some(x.number + 1)).unwrap_or(0);
    let players: Vec<Player> = if let Some(old_round) = old {
        old_round
            .players
            .iter()
            .map(|p| Player {
                guess: None,
                points: p.points,
                id: p.id,
            })
            .collect()
    } else {
        server
            .clients
            .iter()
            .map(|c| Player {
                guess: None,
                points: 0,
                id: c.id,
            })
            .collect()
    };

    eprintln!("server: fetching image...");
    let (bytes, data) = images().await?;
    eprintln!("server: fetched image from {}", data.address);

    server
        .broadcast(
            &shared::Packet::Round {
                number,
                image: bytes[1].clone(),
            },
            None,
        )
        .await;

    eprintln!("server: starting round {number}");
    Ok(State::Round(RoundData {
        answer: data.coordinates,
        number,
        players,
    }))
}

pub fn results(round: &mut RoundData) {
    let answer = Location::new(round.answer.latitude, round.answer.longitude);

    for player in &mut round.players {
        let distance = Location::new(
            player.guess.unwrap().latitude,
            player.guess.unwrap().longitude,
        )
        .haversine_distance_to(&answer)
        .meters() / 1000.0;

        const MAX_SCORE: f64 = 1000.0;
        const BEST_DISTANCE: f64 = 250.0;
        const DECAY: f64 = 1500.0;
        let score = MAX_SCORE * f64::exp(-(distance - BEST_DISTANCE) / DECAY);
        player.points += score.clamp(0.0, 1000.0).round() as u64;
    }
}
