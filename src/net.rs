pub const PORT:u16 = 34865;

// this code is only used by the server, so the client issues warnings about it
#[allow(dead_code)]
pub enum ProtocolError {
    UnknownCommand,
    MissingArg,
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
        }
    }
}


// this code is only used by the server, so the client issues warnings about it
#[allow(dead_code)]
pub enum ProtocolMessage {
    Error(ProtocolError, String),
}

