use macroquad::prelude::*;
use crate::board::{Board, BoardSquareCoords, Square, SquareEdge, SquareOccupant};
use std::fmt;
use std::cell::Cell;

#[derive(PartialEq, Clone, Copy)]
pub enum Team {
    Purple,
    White,
    Neutral,
}

#[allow(dead_code)]
impl Team {
    pub fn from_string<S: AsRef<str>>(str: S) -> Option<Self> {
        let str_ref = str.as_ref();
        match str_ref {
            "purple" | "PURPLE" | "Purple" => Some(Team::Purple),
            "white"  | "WHITE" | "White" => Some(Team::White),
            _ => None,
        }
    }
    pub fn opposite(&self) -> Self {
        match self {
            Self::Purple => Self::White,
            Self::White => Self::Purple,
            Self::Neutral => panic!("Cannot invert neutral team, this is a programming error"),
        }
    }
    pub fn as_network_string(&self) -> String {
        match self {
            Self::Purple => "purple".to_string(),
            Self::White => "white".to_string(),
            Self::Neutral => "neutral".to_string(),
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Team::Purple => write!(f, "Purple"),
            Team::White => write!(f, "White"),
            Team::Neutral => write!(f, "Neutral"),
        }
    }
}

#[allow(dead_code)]
pub enum WinState {
    PurpleWin,
    WhiteWin,
    Draw
}

#[allow(dead_code)]
pub struct GameState {
    pub turn: Team,
    board: Board,
    pub game_over: Option<WinState>,
    pub murder_happened: Cell<bool>,
}

#[allow(dead_code)]
fn is_path_clear(board: &Board, from: BoardSquareCoords, to: BoardSquareCoords) -> bool {
    let mut current = from;

    // Determine the direction of the move
    let (dx, dy) = (to.0 as i32 - from.0 as i32, to.1 as i32 - from.1 as i32);
    // make sure we are only moving in a single direction (i.e orthoganally)
    assert!(dx == 0 || dy == 0);

    // Determine the step direction
    let (step_x, step_y) = (dx.signum(), dy.signum());

    // Iterate over the squares in the path
    while current != to {
        current.0 = (current.0 as i32 + step_x) as u32;
        current.1 = (current.1 as i32 + step_y) as u32;

        if let Some(square) = board.squares.get(&current) {
            // Check for walls in the direction of the move
            if (step_x == 1 && square.wall.contains(&SquareEdge::West))
                || (step_x == -1 && square.wall.contains(&SquareEdge::East))
                || (step_y == 1 && square.wall.contains(&SquareEdge::North))
                || (step_y == -1 && square.wall.contains(&SquareEdge::South))
            {
                return false; // There is a wall in the path
            }
            // check for occupied squares in the direction of the move
            if square.occupant != SquareOccupant::Empty {
                return false;
            }
        } else {
            // The square is not on the board, consider it blocked
            return false;
        }
    }

    true // No walls in the path
}

#[allow(dead_code)]
impl GameState {
    pub fn new() -> GameState {
        GameState {
            turn: Team::White,
            game_over: None,
            board: Board::new(),
            murder_happened: Cell::new(false),
        }
    }
    pub fn reset(&mut self) {
        self.turn = Team::White;
        self.game_over = None;
        self.board = Board::new();
        self.murder_happened.set(false);
    }
    pub fn get_board(&self) -> &Board {
        &self.board
    }
    pub fn get_turn(&self) -> &Team {
        &self.turn
    }
    pub fn check_neighbours<F>(&self, pos: BoardSquareCoords, callback: F)
    where
        F: Fn(&Square, &BoardSquareCoords, &SquareEdge),
    {
        let offsets = [
            (0, -1),  // North
            (1, 0),  // East
            (0, 1), // South
            (-1, 0), // West
        ];

        for (i, offset) in offsets.iter().enumerate() {
            let neighbour_pos = (pos.0 as i32 + offset.0, pos.1 as i32 + offset.1);
            let edge = match i {
                0 => SquareEdge::North,
                1 => SquareEdge::East,
                2 => SquareEdge::South,
                3 => SquareEdge::West,
                _ => panic!("Too many offsets!"),
            };

            if neighbour_pos.0 >= 0
                && neighbour_pos.0 < self.board.width as i32
                && neighbour_pos.1 >= 0
                && neighbour_pos.1 < self.board.height as i32
            {
                let npos = &BoardSquareCoords(neighbour_pos.0 as u32, neighbour_pos.1 as u32);
                if let Some(neighbour_square) = self.board.squares.get(npos) {
                    callback(neighbour_square, npos, &edge);
                }
            }
        }
    }
    pub fn make_move(&mut self, team: Team, from: BoardSquareCoords, to: BoardSquareCoords) -> bool {
        if team != self.turn {
            return false;
        }
        if self.valid_move(from, to) {
            if let Some(mut from_square) = self.board.squares.remove(&from) {
                if let Some(to_square) = self.board.squares.get_mut(&to) {
                    to_square.occupant =
                        std::mem::replace(&mut from_square.occupant, SquareOccupant::Empty);
                    self.board.squares.insert(from, from_square);
                    let murder_victim: Cell<Option<(BoardSquareCoords, Team)>> = Cell::new(None);

                    self.check_neighbours(to, |neighbour, position, _| {
                        match neighbour.occupant {
                            SquareOccupant::Guard(_) |
                            SquareOccupant::Magistrate(_) |
                            SquareOccupant::Empty => {},
                            SquareOccupant::Citizen(team) => {
                                if team != self.turn {
                                    let murdered = Cell::new(true);
                                    self.check_neighbours(*position, |neighbour, _, dir| {
                                        let opposite_dir = dir.get_opposite();
                                        if !neighbour.wall.contains(&opposite_dir) &&
                                            match neighbour.occupant {
                                                SquareOccupant::Empty => true,
                                                SquareOccupant::Magistrate(_) => false,
                                                SquareOccupant::Guard(team) |
                                                SquareOccupant::Citizen(team) => {
                                                    team != self.turn
                                                }
                                            }
                                        {
                                            murdered.set(false);
                                        }
                                    });
                                    if murdered.get() {
                                        murder_victim.set(Some((position.clone(), team)));
                                    }
                                }
                            }
                        }
                    });
                    if let Some((victim_location, victim_team)) = murder_victim.get() {
                        self.murder_happened.set(true);
                        if let Some(square) = self.board.squares.get_mut(&victim_location) {
                            square.occupant = SquareOccupant::Empty;
                        }
                        self.flip_guards(victim_team);
                    }
                    self.end_turn();
                    return true;
                }
            } 
        } 
        false
    }
    pub fn flip_guards(&mut self, victim_team: Team) {
        for (_, square) in &mut self.board.squares {
            match square.occupant {
                SquareOccupant::Guard(team) => {
                    let new_team = match team {
                        Team::Purple => Team::White,
                        Team::White => Team::Purple,
                        Team::Neutral => victim_team,
                    };
                    square.occupant = SquareOccupant::Guard(new_team);
                },
                SquareOccupant::Magistrate(team) => {
                    let new_team = match team {
                        Team::Purple => Team::White,
                        Team::White => Team::Purple,
                        Team::Neutral => victim_team.opposite(),
                    };
                    square.occupant = SquareOccupant::Magistrate(new_team);
                },
                _ => {},
            }
        }
    }
    pub fn valid_move(&self, from: BoardSquareCoords, to: BoardSquareCoords) -> bool {
        // if there is an occupant in the to square, this is an invalid move
        if let Some(to_square) = self.board.squares.get(&to) {
            if to_square.occupant != SquareOccupant::Empty {
                return false;
            }
        }
        if let Some(from_square) = self.board.squares.get(&from) {
            match &from_square.occupant {
                SquareOccupant::Empty => false,
                SquareOccupant::Guard(team)
                | SquareOccupant::Magistrate(team)
                | SquareOccupant::Citizen(team) => {
                    if *team == self.turn {
                        if from.0 == to.0 || from.1 == to.1 {
                            return is_path_clear(&self.board, from, to);
                        }
                    }
                    false
                }
            }
        } else {
            false
        }
    }
    pub fn end_turn(&mut self) {
        // evaluate win condition - all opponents are dead
        let mut purple_count = 0;
        let mut white_count = 0;
        for (_, square) in &self.board.squares {
            match square.occupant  {
                SquareOccupant::Citizen(team) => {
                    match team {
                        Team::Purple => { purple_count += 1; }
                        Team::White => {  white_count += 1; }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        if purple_count == 0 && white_count == 0 {
            self.game_over = Some(WinState::Draw);
        }
        else if purple_count == 0 {
            self.game_over = Some(WinState::WhiteWin);
        }
        else if white_count == 0 {
            self.game_over = Some(WinState::PurpleWin);
        }
        else {
            self.turn = match self.turn {
                Team::White => Team::Purple,
                Team::Purple => Team::White,
                Team::Neutral => {
                    panic!("Neutral player should never get a turn!");
                }
            }
        }
    }
}
