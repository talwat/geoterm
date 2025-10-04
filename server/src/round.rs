use geoutils::Location;
use shared::{Round, Text};

use crate::server::State;
use crate::{Server, error::Error, images::images};
use shared::Player;

pub async fn new(server: &mut Server, old: Option<&Round>) -> Result<State, Error> {
    eprintln!("server: initializing round");
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
                images: bytes.clone(),
                text: Text {
                    street: data.address.split(", ").next().unwrap().to_owned(),
                    additional: Vec::new(),
                },
            },
            None,
        )
        .await;
    eprintln!("server: sent data to players");

    eprintln!("server: starting round {number}");
    Ok(State::Round(Round {
        answer: data.coordinates,
        number,
        players,
    }))
}

pub fn results(round: &mut Round) {
    let answer = Location::new(round.answer.0, round.answer.1);

    for player in &mut round.players {
        let distance = Location::new(player.guess.unwrap().0, player.guess.unwrap().1)
            .haversine_distance_to(&answer)
            .meters();
        const SIGMA: f64 = 3000.0 * 1000.0;

        // Gauss something or other, I'm an engineer not a mathematician.
        let score = 1000.0 * f64::exp(-0.5 * (distance / SIGMA).powi(2));
        player.points += score.round() as u64;
    }
}
