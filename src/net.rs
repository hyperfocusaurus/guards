use crate::Team;
use crate::board::BoardSquareCoords;
pub const PORT:u16 = 34865;

// this code is only used by the server, so the client issues warnings about it
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ProtocolError {
    UnknownCommand,
    MissingArg,
    InvalidTeam,
    InvalidMove,
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UnknownCommand => {
                write!(f, "UNKNOWN")
            }
            Self::MissingArg => {
                write!(f, "MISSINGARG")
            }
            Self::InvalidTeam => {
                write!(f, "INVALIDTEAM")
            }
            Self::InvalidMove => {
                write!(f, "INVALIDMOVE")
            }
        }
    }
}


// this code is only used by the server, so the client issues warnings about it
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ProtocolMessage {
    Error(ProtocolError, String),
    TeamJoin(Team),
    Move(Team, BoardSquareCoords, BoardSquareCoords),
}

impl std::fmt::Display for ProtocolMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Error(e, msg) => {
                write!(f, "ERROR {} {}", e, msg)
            }
            Self::TeamJoin(team) => {
                write!(f, "TEAMJOIN {}", team)
            }
            Self::Move(team, from, to) => {
                write!(f, "MOVE {} {} {}", team, from, to)
            }
        }
    }
}
