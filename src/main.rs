use futures::stream::StreamExt;
use irc::client::prelude::*;
use ruma_client::Client;
use tokio::net::TcpListener;
use tokio_util::codec::FramedRead;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:6667";
    let mut listener = TcpListener::bind(&addr).await?;

    println!("Listening on {}", listener.local_addr().unwrap());

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let (read, _write) = socket.split();
            let mut read_framed = FramedRead::new(read, irc::proto::IrcCodec::new("utf8").unwrap());

            let mut access_token = None;

            while let Some(data) = read_framed.next().await {
                if let Ok(data) = data {
                    match data.command {
                        Command::PASS(pass) => {
                            access_token = Some(pass);
                        }
                        Command::USER(user, _mode, _realname) => {
                            use ruma_client_api::r0::session::login;
                            println!("logging in as {}", user);
                            let client =
                                Client::https("https://mozilla.org".parse().unwrap(), None);
                            let response = client
                                .request(login::Request {
                                    user: login::UserInfo::MatrixId(user),
                                    login_info: login::LoginInfo::Token {
                                        token: access_token.take().unwrap(),
                                    },
                                    device_id: None,
                                    initial_device_display_name: None,
                                })
                                .await;
                            println!("response {:?}", response);
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}
