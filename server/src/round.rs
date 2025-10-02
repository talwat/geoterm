use shared::{Round, Text};

use crate::server::State;
use crate::{Server, error::Error, images::images};
use shared::Player;

pub async fn new(server: &mut Server) -> Result<State, Error> {
    server.broadcast(&shared::Packet::RoundLoading, None).await;
    eprintln!("server: initializing round");

    let (number, players): (usize, Vec<Player>) = if let State::Round(round) = &server.state {
        (round.number + 1, round.players.to_owned())
    } else {
        let players = server
            .clients
            .iter()
            .map(|x| Player {
                guess: None,
                points: 0,
                id: x.id,
            })
            .collect();

        (0, players)
    };

    eprintln!("server: fetching image...");
    let (bytes, data) = images().await?;
    eprintln!("server: fetched image from {}", data.address);

    server
        .broadcast(
            &shared::Packet::Round {
                number,
                players: players.clone(),
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
