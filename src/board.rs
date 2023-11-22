use std::fmt::{Write, Display, Formatter, Error};
use std::collections::{HashMap, HashSet};
use std::fs;

use crate::game::Team;

#[derive(PartialEq, Clone, Copy)]
#[allow(dead_code)] // this is not used by the server
pub enum SquareOccupant {
    Empty,
    // guards are controlled by one player at a time, flipping sides every time a kill
    // occurs
    Guard(Team),
    // citizens are player-controlled pieces
    Citizen(Team),
    // the magistrate is controlled by the player who doesn't control the guards, and
    // cannot be used to kill but can be used to block pieces in.
    Magistrate(Team),
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)] // this is not used by the server
pub enum SquareEdge {
    North,
    East,
    South,
    West,
}

#[allow(dead_code)] // not used by the server
impl SquareEdge {
    pub fn get_opposite(&self) -> SquareEdge {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }
}

#[allow(dead_code)] // not used by the server
pub struct Square {
    pub occupant: SquareOccupant,
    pub wall: HashSet<SquareEdge>,
}

impl Square {
    #[allow(dead_code)]
    pub fn new(occupant: SquareOccupant, wall: HashSet<SquareEdge>) -> Self {
        Self { occupant, wall }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct BoardSquareCoords(pub u32, pub u32);

impl Display for BoardSquareCoords {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let BoardSquareCoords (x, y) = self;
        write!(f, "({x},{y})")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsePointError;

impl std::str::FromStr for BoardSquareCoords {
    type Err = ParsePointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.split_once(','))
            .ok_or(ParsePointError)?;

        let x_fromstr = x.parse::<u32>().map_err(|_| ParsePointError)?;
        let y_fromstr = y.parse::<u32>().map_err(|_| ParsePointError)?;

        Ok( Self ( x_fromstr, y_fromstr ))
    }
}

#[allow(dead_code)]
pub struct Board {
    pub squares: HashMap<BoardSquareCoords, Square>,
    pub width: u32, // width in tiles
    pub height: u32, // height in tiles
}


#[allow(dead_code)]
impl Board {
    pub fn new() -> Self {
        let mut squares = HashMap::new();
        let board_txt = fs::read_to_string("board.txt").map_err(|err| panic!("Board file missing: {err}")).unwrap();

        let board_lines: Vec<&str> = board_txt.lines().collect();
        let mut width: u32 = 0;
        let height: u32 = board_lines.len() as u32;

        for y in 0..board_lines.len() {
            let line = board_lines[y];
            if line.len() > width as usize {
                width = line.len() as u32;
            }
            for x in 0..line.len() {
                // for hysterical raisins, the coordinates are reversed for this match statement
                // only
                let occupant = match (y, x) {
                    (0, 6) | (8, 0) => SquareOccupant::Guard(Team::Neutral),
                    (2, 2) | (2, 3) | (2, 4) |
                    (3, 2) | (3, 3) | (3, 4) |
                    (4, 4) => SquareOccupant::Citizen(Team::Purple),

                    (4, 2) |
                    (5, 2) | (5, 3) | (5, 4) |
                    (6, 2) | (6, 3) | (6, 4)
                    => SquareOccupant::Citizen(Team::White),

                    (4, 3) => SquareOccupant::Magistrate(Team::Neutral),

                    _ => SquareOccupant::Empty,
                };
                let mut wall = HashSet::new();
                let mut s = String::new();
                let _ = write!(s, "Board char does not exist at {x},{y}");
                let wall_char = line.chars().nth(x).expect(s.as_str());
                s.clear();
                let _ = write!(s, "Invalid hex digit: {wall_char}");
                let wall_val = u8::from_str_radix(&wall_char.to_string(), 16).expect(s.as_str());

                if wall_val & 0b0001 != 0 {
                    wall.insert(SquareEdge::West);
                }
                if wall_val & 0b0010 != 0 {
                    wall.insert(SquareEdge::North);
                }
                if wall_val & 0b0100 != 0 {
                    wall.insert(SquareEdge::East);
                }
                if wall_val & 0b1000 != 0 {
                    wall.insert(SquareEdge::South);
                }

                let coords = BoardSquareCoords(x as u32, y as u32);
                squares.insert(coords, Square::new(occupant, wall));
            }
        }
        Self { squares, height, width }
    }

    pub fn get_squares(&self) -> &HashMap<BoardSquareCoords, Square> {
        &self.squares
    }
}
