use shared::Round;

use crate::{Player, Server, State, error::Error, images::images};

pub async fn new(server: &mut Server) -> Result<State, Error> {
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

    let (bytes, metadata) = images().await?;
    server
        .broadcast(
            &shared::Packet::Round {
                number,
                players: players.clone(),
                images: bytes.clone(),
            },
            None,
        )
        .await;

    Ok(State::Round(Round {
        answer: metadata.coordinates,
        number,
        players,
    }))
}
