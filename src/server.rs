use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

mod board;
mod game;
mod net;
use crate::game::Team;
use crate::net::{ProtocolError, ProtocolMessage, PORT};

use tracing::{error, info, warn};
use tracing_subscriber;

struct ClientConnection {
    stream: TcpStream,
    #[allow(dead_code)] // <-- todo: remove this when we get to using the team
    team: Option<Team>,
}

struct Server {
    clients: Vec<ClientConnection>,
}
impl Server {
    // set client_idx to None to broadcast to all clients
    fn send(&self, client_idx: Option<usize>, msg: ProtocolMessage) {
        match client_idx {
            Some(client_idx) => {
                if let Some(client) = self.clients.get(client_idx) {
                    match msg {
                        ProtocolMessage::TeamJoin(team) => {
                            let mut stream = &client.stream;
                            let _ = stream.write_all(format!("join {team}\n").as_bytes());
                        }
                        ProtocolMessage::Error(num, msg) => {
                            let mut stream = &client.stream;
                            let _ = stream.write_all(format!("error {num} {msg}\n").as_bytes());
                        }
                    }
                } else {
                    error!("Invalid client index {client_idx}");
                }
            }
            None => {
                for i in 0..self.clients.len() {
                    self.send(Some(i), msg.clone());
                }
            }
        }
    }
    fn handle_client(&mut self, stream: TcpStream) {
        let client_idx = self.clients.len();
        self.clients.push(ClientConnection { stream, team: None });
        let client = &self.clients[client_idx];

        let reader = BufReader::new(&client.stream);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let command: Vec<&str> = line.split(' ').collect();
                    info!("Command: {:?}", command);
                    match command[0] {
                        "join" => {
                            if command.len() >= 2 {
                                let team = Team::from_string(command[1]);
                                match team {
                                    Some(team) => self.send(None, ProtocolMessage::TeamJoin(team)),
                                    None => {
                                        self.send(
                                            Some(client_idx),
                                            ProtocolMessage::Error(
                                                ProtocolError::InvalidTeam,
                                                format!("Unrecognised team"),
                                            ),
                                        );
                                    }
                                }
                            } else {
                                self.send(
                                    Some(client_idx),
                                    ProtocolMessage::Error(
                                        ProtocolError::MissingArg,
                                        format!("JOIN requires an argument <team>").to_string(),
                                    ),
                                );
                            }
                        }
                        _ => {
                            self.send(
                                Some(client_idx),
                                ProtocolMessage::Error(
                                    ProtocolError::UnknownCommand,
                                    format!("unknown command {}", command[0]).to_string(),
                                ),
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("Could not read from client: {e}");
                }
            }
        }
    }
    fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener =
        TcpListener::bind(format!("0.0.0.0:{}", PORT)).expect("Could not bind to address");
    let server = Arc::new(Mutex::new(Server::new()));
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set up logging library");
    info!("Server listening on port {PORT}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // todo: for now we use one thread per client, this isn't super efficient but we
                // don't expect to have more than 2 clients per game anyway so it should be fine
                let server = server.clone();
                thread::spawn(move || {
                    let mut server = server.lock().unwrap();
                    server.handle_client(stream)
                });
            }
            Err(e) => {
                warn!("Could not accept client connection: {e}");
            }
        }
    }

    Ok(())
}
