use futures::{sink::SinkExt, stream::StreamExt};
use irc::client::prelude::*;
use matrix_client::Client;
use tokio::net::TcpListener;
use tokio_util::codec::{FramedRead, FramedWrite};

mod error;
mod matrix_client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:6667";
    let mut listener = TcpListener::bind(&addr).await?;

    println!("Listening on {}", listener.local_addr().unwrap());

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let (read, write) = socket.split();
            let mut read_framed = FramedRead::new(read, irc::proto::IrcCodec::new("utf8").unwrap());
            let mut write_framed =
                FramedWrite::new(write, irc::proto::IrcCodec::new("utf8").unwrap());

            let mut access_token = None;

            while let Some(data) = read_framed.next().await {
                if let Ok(data) = data {
                    match data.command {
                        Command::PASS(pass) => {
                            println!("access_token: {}", pass);
                            access_token = Some(pass);
                        }
                        Command::USER(user, _mode, _realname) => {
                            let client = Client::create(
                                "https://mozilla.modular.im",
                                &user,
                                &access_token.take().unwrap(),
                            )
                            .await
                            .unwrap();
                            println!("access_token: {}", client.access_token());
                            write_framed
                                .send(
                                    Message::new(
                                        None,
                                        "Response",
                                        vec![&format!(
                                            "Logged in as {} using access token {}",
                                            user,
                                            client.access_token()
                                        )],
                                    )
                                    .unwrap(),
                                )
                                .await
                                .unwrap();
                            // let sync = client.sync().await.unwrap();
                            // println!("sync {:#?}", sync);
                        }
                        Command::JOIN(chanlist, _chankeys, _realname) => {
                            println!("chanlist {}", chanlist);
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}
