use super::{handle_error, IdType};
use crate::general_types::*;
use crate::app::general_functions::*;
use axum::extract::ws::{self, WebSocket};
use futures_util::{stream::SplitStream, StreamExt};
use tokio::sync::mpsc;

pub async fn read(
    mut receiver: SplitStream<WebSocket>,
    sender: mpsc::Sender<ws::Message>,
    id: IdType,
    app_state: AppState,
) {
    while let Some(message) = receiver.next().await {
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        };

        let message: real_time::Request =
            match rmp_serde::from_slice(message.into_data().as_slice()) {
                Ok(m) => m,
                Err(e) => {
                    use real_time::Error;
                    let error = Error::Decode(e.to_string());
                    handle_error(error, true, &sender).await;
                    break;
                }
            };

        match message {
            real_time::Request::RemoveUser { user_id } => {
                let host_id = match only_host(&id, &sender).await {
                    Ok(id) => &id.id,
                    Err(_) => break,
                };

                if let Err(error) = kick_user(&user_id, host_id, &app_state.db.pool).await {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::AddSong { song_id } => {
                let id = match only_user(&id, &sender).await {
                    Ok(id) => id,
                    Err(_) => break,
                };

                if let Err(error) = add_song(&song_id, &id.id, &id.jam_id, &app_state.db.pool).await
                {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::RemoveSong { song_id } => {
                let jam_id = match only_host(&id, &sender).await {
                    Ok(id) => &id.jam_id,
                    Err(_) => break,
                };

                if let Err(error) = remove_song(&song_id, jam_id, &app_state.db.pool).await {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::AddVote { song_id } => {
                let id = match only_user(&id, &sender).await {
                    Ok(id) => id,
                    Err(_) => break,
                };

                if let Err(error) = add_vote(&song_id, &id.id, &id.jam_id, &app_state.db.pool).await
                {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::RemoveVote { song_id } => {
                let id = match only_host(&id, &sender).await {
                    Ok(id) => id,
                    Err(_) => break,
                };

                if let Err(error) =
                    remove_vote(&song_id, &id.id, &id.jam_id, &app_state.db.pool).await
                {
                    handle_error(error.into(), false, &sender).await;
                };
            }
            real_time::Request::Update => {
                //notify(chanel, jam_id, pool)
            }
        }
    }
}

use super::Id;

async fn only_host<'a>(id: &'a IdType, sender: &mpsc::Sender<ws::Message>) -> Result<&'a Id, ()> {
    match id {
        IdType::Host(id) => Ok(id),
        IdType::User { .. } => {
            let error = real_time::Error::Forbidden(
                "Only the host can do the requested action, if you see this in prod this is a bug"
                    .to_string(),
            );

            handle_error(error, true, &sender).await;
            Err(())
        }
    }
}

async fn only_user<'a>(id: &'a IdType, sender: &mpsc::Sender<ws::Message>) -> Result<&'a Id, ()> {
    match id {
        IdType::User(id) => Ok(id),
        IdType::Host { .. } => {
            let error = real_time::Error::Forbidden(
                "Only users can do the requested action, if you see this in prod this is a bug"
                    .to_string(),
            );

            handle_error(error, true, &sender).await;
            Err(())
        }
    }
}
