use futures::{sink::SinkExt, stream::StreamExt};
use irc::client::prelude::*;
use matrix_client::Client;
use tokio::net::TcpListener;
use tokio_util::codec::{FramedRead, FramedWrite};

mod error;
mod matrix_client;

const HOMESERVER: &'static str = "https://mozilla.modular.im";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::spawn(async move {
        let mut app = tide::new();
        app.at("/").get(|_| async move {
            tide::Response::new(302).set_header(
                "location",
                format!(
                    "{}/_matrix/client/r0/login/sso/redirect/?redirectUrl=http://localhost:8080/callback",
                    HOMESERVER
                ),
            )
        });

        app.at("/callback").get(|r: tide::Request<()>| async move {
            #[derive(Debug, serde::Deserialize)]
            struct CallbackOptions {
                #[serde(rename = "loginToken")]
                token: String,
            }

            let q: CallbackOptions = r.query().unwrap();

            let access_token = Client::get_access_token(HOMESERVER, "", &q.token)
                .await
                .unwrap();

            access_token
        });

        println!("HTTP listening on 127.0.0.1:8080");

        app.listen("127.0.0.1:8080").await.unwrap();
    });

    let addr = "127.0.0.1:6667";
    let mut listener = TcpListener::bind(&addr).await?;

    println!("IRC listening on {}", listener.local_addr().unwrap());

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let (read, write) = socket.split();
            let mut read_framed = FramedRead::new(read, irc::proto::IrcCodec::new("utf8").unwrap());
            let mut write_framed =
                FramedWrite::new(write, irc::proto::IrcCodec::new("utf8").unwrap());

            while let Some(data) = read_framed.next().await {
                if let Ok(data) = data {
                    match data.command {
                        Command::PASS(pass) => {
                            let client = Client::new(HOMESERVER, &pass);
                            write_framed
                                .send(
                                    Message::new(
                                        None,
                                        "Response",
                                        vec![&format!(
                                            "Logged in as {} using access token {}",
                                            "",
                                            client.access_token()
                                        )],
                                    )
                                    .unwrap(),
                                )
                                .await
                                .unwrap();
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
