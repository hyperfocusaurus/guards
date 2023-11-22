use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;

mod board;
mod game;
mod net;
use crate::board::BoardSquareCoords;
use crate::game::Team;
use crate::net::{ProtocolError, ProtocolMessage, PORT};

use tracing::{debug, error, info, warn};
use tracing_subscriber;

static ID_SEQ: RwLock<u32> = RwLock::new(0);

enum ServerEvent {
    ClientConnected(Arc<ClientConnection>),
    ClientDisconnected(u32),
    ClientMessage(u32, Vec<String>),
}

#[derive(Debug)]
struct ClientConnection {
    stream: Arc<TcpStream>,
    team: Option<Team>,
    id: u32,
}

impl std::fmt::Display for ClientConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.stream)
    }
}

struct Server {
    clients: Vec<Arc<ClientConnection>>,
}
impl Server {
    fn event_loop(&mut self, receiver: Receiver<ServerEvent>) {
        loop {
            match receiver.recv() {
                Ok(event) => match event {
                    ServerEvent::ClientConnected(client) => {
                        info!("Client {:?} connected", client);
                        self.clients.push(client);
                    }
                    ServerEvent::ClientDisconnected(client_id) => {
                        info!("Client {:?} disconnected", client_id);
                        self.clients.retain(|c| {
                            c.id != client_id
                        });
                    }
                    ServerEvent::ClientMessage(client_id, command) => {
                        info!("Client {:?} sent message: {:?}", client_id, command);
                        match command[0].as_str() {
                            "join" => {
                                if command.len() >= 2 {
                                    let team = Team::from_str(command[1].as_str());
                                    match team {
                                        Ok(team) => {
                                            self.set_client_team(client_id, team);
                                            self.send(None, ProtocolMessage::TeamJoin(team));
                                        }
                                        Err(_) => {
                                            self.send(
                                                Some(client_id),
                                                ProtocolMessage::Error(
                                                    ProtocolError::InvalidTeam,
                                                    format!("Unrecognised team"),
                                                ),
                                            );
                                        }
                                    }
                                } else {
                                    self.send(
                                        Some(client_id),
                                        ProtocolMessage::Error(
                                            ProtocolError::MissingArg,
                                            format!("JOIN requires an argument <team>").to_string(),
                                        ),
                                    );
                                }
                            }
                            "move" => {
                                if command.len() >= 4 {
                                    let team = command[1].parse::<Team>();
                                    let from = command[2].parse::<BoardSquareCoords>();
                                    let to = command[3].parse::<BoardSquareCoords>();
                                    match (team, from, to) {
                                        (Ok(team), Ok(from), Ok(to)) => {
                                            let client = self
                                                .clients
                                                .iter()
                                                .find(|c| c.id == client_id)
                                                .expect("Could not find client");
                                            if let Some(client_team) = client.team {
                                                if client_team == team {
                                                    self.send(
                                                        None,
                                                        ProtocolMessage::Move(team, from, to),
                                                    );
                                                } else {
                                                    self.send(
                                                        Some(client_id),
                                                        ProtocolMessage::Error(
                                                            ProtocolError::InvalidMove,
                                                            format!("You are not on team {}", team),
                                                        ),
                                                    );
                                                }
                                            } else {
                                                self.send(
                                                    Some(client_id),
                                                    ProtocolMessage::Error(
                                                        ProtocolError::InvalidMove,
                                                        format!("You have not joined a team"),
                                                    ),
                                                );
                                            }
                                        }
                                        _ => {
                                            self.send(
                                                Some(client_id),
                                                ProtocolMessage::Error(
                                                    ProtocolError::InvalidMove,
                                                    format!("Invalid move"),
                                                ),
                                            );
                                        }
                                    }
                                } else {
                                    self.send(
                                        Some(client_id),
                                        ProtocolMessage::Error(
                                            ProtocolError::MissingArg,
                                            format!("MOVE requires two arguments <from> <to>")
                                                .to_string(),
                                        ),
                                    );
                                }
                            }
                            _ => {
                                self.send(
                                    Some(client_id),
                                    ProtocolMessage::Error(
                                        ProtocolError::UnknownCommand,
                                        format!("unknown command {}", command[0]).to_string(),
                                    ),
                                );
                            }
                        }
                    }
                },
                Err(e) => {
                    error!("Could not receive event: {e}");
                }
            }
        }
    }
    // set client_idx to None to broadcast to all clients
    fn send(&self, client_id: Option<u32>, msg: ProtocolMessage) {
        debug!("Sending {:?} to client {:?}", msg, client_id);
        match client_id {
            Some(client_id) => {
                let client = self
                    .clients
                    .iter()
                    .find(|c| c.id == client_id)
                    .expect("Could not find client");
                match msg {
                    ProtocolMessage::Move(team, from, to) => {
                        let mut stream: &TcpStream = &client.stream;
                        let _ = stream
                            .write_all(format!("move {team} {from} {to}\n").as_bytes())
                            .map_err(|e| {
                                error!("Could not send message to client: {e}");
                            });
                        let _ = stream.flush().map_err(|e| {
                            error!("Could not send message to client: {e}");
                        });
                    }
                    ProtocolMessage::TeamJoin(team) => {
                        let mut stream: &TcpStream = &client.stream;
                        let _ = stream
                            .write_all(format!("join {team}\n").as_bytes())
                            .map_err(|e| {
                                error!("Could not send message to client: {e}");
                            });
                        let _ = stream.flush().map_err(|e| {
                            error!("Could not send message to client: {e}");
                        });
                    }
                    ProtocolMessage::Error(num, msg) => {
                        let mut stream: &TcpStream = &client.stream;
                        let _ = stream
                            .write_all(format!("error {num} {msg}\n").as_bytes())
                            .map_err(|e| {
                                error!("Could not send message to client: {e}");
                            });
                        let _ = stream.flush().map_err(|e| {
                            error!("Could not send message to client: {e}");
                        });
                    }
                }
            },
            None => {
                for client in self.clients.iter() {
                    self.send(Some(client.id), msg.clone());
                }
            }
        }
    }
    fn set_client_team(&mut self, client_id: u32, team: Team) {
        // replace the client in the list with a new one with the team set
        debug!("Setting client {:?} to team {:?}", client_id, team);
        let client_idx = self
            .clients
            .iter()
            .position(|c| c.id == client_id)
            .expect("Could not find client");
        let client = self.clients[client_idx].clone();
        let new_client = Arc::new(ClientConnection {
            stream: client.stream.clone(),
            team: Some(team),
            id: client.id,
        });
        self.clients[client_idx] = new_client;
    }

    fn new() -> Self {
        Self {
            clients: Vec::new(),
        }
    }
}

fn handle_client(stream: Arc<TcpStream>, event_tx: Sender<ServerEvent>) {
    let client;

    {
        let id = ID_SEQ.read().unwrap().clone();
        let mut id_seq_ref = ID_SEQ.write().unwrap();
        *id_seq_ref += 1;
        client = Arc::new(ClientConnection {
            stream: stream.clone(),
            team: None,
            id,
        });
    }

    let reader = BufReader::new(stream.as_ref());
    let _ = event_tx
        .send(ServerEvent::ClientConnected(client.clone()))
        .expect("Could not send event to server thread");

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let command: Vec<String> = line.split(' ').map(|s| s.to_string()).collect();
                info!("Command: {:?}", command);

                let _ = event_tx
                    .send(ServerEvent::ClientMessage(client.id, command))
                    .expect("Could not send event to server thread");
            }
            Err(e) => {
                error!("Could not read from client: {e}");
            }
        }
    }
    let _ = event_tx
        .send(ServerEvent::ClientDisconnected(client.id))
        .expect("Could not send event to server thread");
}

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let listener =
        TcpListener::bind(format!("0.0.0.0:{}", PORT)).expect("Could not bind to address");
    info!("Server listening on port {PORT}");
    let (event_tx, event_rx) = channel();

    thread::spawn(move || {
        let mut server = Server::new();
        server.event_loop(event_rx)
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let event_tx = event_tx.clone();
                thread::spawn(move || {
                    let stream = Arc::new(stream);
                    handle_client(stream, event_tx);
                });
            }
            Err(e) => {
                warn!("Could not accept client connection: {e}");
            }
        }
    }

    Ok(())
}
