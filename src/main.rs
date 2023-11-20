use macroquad::prelude::*;
use std::fmt::Write;
pub mod board;
mod game;
use crate::board::{BoardSquareCoords, SquareEdge, SquareOccupant};
use crate::game::{WinState, GameState, Team};

fn conf() -> Conf {
    Conf {
        window_title: String::from("Guards!"),
        fullscreen: true,
        ..Default::default()
    }
}

const SQUARE_SIZE: f32 = 100.0;
const GUARD_SIZE: f32 = 45.0;
const CITIZEN_SIZE: f32 = 45.0;
const MAGISTRATE_SIZE: f32 = 47.5;
const DOT_SIZE: f32 = 5.0;
const WALL_THICKNESS: f32 = 3.0;

struct PlayerState {
    selected_square: Option<BoardSquareCoords>,
}

impl PlayerState {
    fn new() -> Self {
        Self {
            selected_square: None,
        }
    }
}

fn render_game_state(
    game_state: &mut GameState,
    mouse_pos: (f32, f32),
    player_state: &mut PlayerState,
) {
    let (screen_width, screen_height) = (screen_width(), screen_height());
    match &game_state.game_over {
        Some(win) => {
            let rect_height = 100.0;
            let rect_width = 250.0;
            let rect_x = (screen_width - rect_width) / 2.0;
            let rect_y = (screen_height - rect_height) / 2.0;
            let text_width = 60.0; 
            let text_height = 32.0;
            draw_rectangle(rect_x, rect_y , rect_width, rect_height, RED);
            draw_text("Game over!", rect_x + (rect_width / 2.0) - text_width, rect_y + text_height, text_height, WHITE);
            match win {
                WinState::PurpleWin => {
                    let text_width = 66.0;
                    draw_text("Purple won!", rect_x + (rect_width / 2.0) - text_width, rect_y + (text_height * 2.0), text_height, WHITE);
                }
                WinState::WhiteWin => {
                    let text_width = 66.0;
                    draw_text("White won!", rect_x + (rect_width / 2.0) - text_width, rect_y + (text_height * 2.0), text_height, WHITE);
                }
                WinState::Draw => {
                    let text_width = 5.0 * 6.0;
                    draw_text("Draw!", rect_x + (rect_width / 2.0) - text_width, rect_y + (text_height * 2.0),text_height, WHITE);
                }
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                game_state.reset();
            }
        }
        None => {
            let mut s = String::new();
            let _ = write!(s, "Current turn: {}", game_state.get_turn());
            let (mouse_x, mouse_y) = mouse_pos;
            draw_text(s.as_str(), 0.0, 32.0, 32.0, WHITE);


            let board_width = 7.0 * SQUARE_SIZE;
            let board_height = 9.0 * SQUARE_SIZE;

            let board_x = (screen_width - board_width) / 2.0;
            let board_y = (screen_height - board_height) / 2.0;
            let mut player_move: Option<(BoardSquareCoords, BoardSquareCoords)> = None;

            draw_rectangle(
                board_x,
                board_y,
                board_width,
                board_height,
                Color::new(0.9, 0.8, 0.6, 1.0),
            );

            for (coord, square) in game_state.get_board().get_squares() {
                let x = coord.0 as f32 * SQUARE_SIZE + board_x;
                let y = coord.1 as f32 * SQUARE_SIZE + board_y;

                // Draw black outline rectangle for the square
                draw_rectangle_lines(x, y, SQUARE_SIZE, SQUARE_SIZE, 1.0, BLACK);

                if mouse_x > x
                    && mouse_x < x + SQUARE_SIZE
                    && mouse_y > y
                    && mouse_y < y + SQUARE_SIZE
                {
                    draw_rectangle(
                        x,
                        y,
                        SQUARE_SIZE,
                        SQUARE_SIZE,
                        Color::new(0.5, 0.5, 0.5, 0.5),
                    );
                    if is_mouse_button_pressed(MouseButton::Left) {
                        if let Some(player_sq) = player_state.selected_square {
                            if game_state.valid_move(player_sq, *coord) {
                                draw_rectangle(
                                    x,
                                    y,
                                    SQUARE_SIZE,
                                    SQUARE_SIZE,
                                    Color::new(0.5, 0.7, 0.5, 1.0),
                                );
                                player_move = Some((player_sq, *coord));
                            } else {
                                draw_rectangle(
                                    x,
                                    y,
                                    SQUARE_SIZE,
                                    SQUARE_SIZE,
                                    Color::new(0.7, 0.5, 0.5, 1.0),
                                );
                            }
                            player_state.selected_square = None;
                        } else {
                            player_state.selected_square = Some(coord.clone());
                            draw_rectangle(
                                x,
                                y,
                                SQUARE_SIZE,
                                SQUARE_SIZE,
                                Color::new(0.5, 0.5, 0.5, 1.0),
                            );
                        }
                    }
                    if is_mouse_button_pressed(MouseButton::Right) {
                        player_state.selected_square = None;
                    }
                }

                if let Some(player_sq) = player_state.selected_square {
                    if player_sq == *coord {
                        draw_rectangle(
                            x,
                            y,
                            SQUARE_SIZE,
                            SQUARE_SIZE,
                            Color::new(0.7, 0.7, 0.7, 1.0),
                        );
                    }
                }

                // Draw walls with increased thickness
                if square.wall.contains(&SquareEdge::North) {
                    draw_line(x, y, x + SQUARE_SIZE, y, WALL_THICKNESS, BLACK);
                }
                if square.wall.contains(&SquareEdge::East) {
                    draw_line(
                        x + SQUARE_SIZE,
                        y,
                        x + SQUARE_SIZE,
                        y + SQUARE_SIZE,
                        WALL_THICKNESS,
                        BLACK,
                    );
                }
                if square.wall.contains(&SquareEdge::South) {
                    draw_line(
                        x,
                        y + SQUARE_SIZE,
                        x + SQUARE_SIZE,
                        y + SQUARE_SIZE,
                        WALL_THICKNESS,
                        BLACK,
                    );
                }
                if square.wall.contains(&SquareEdge::West) {
                    draw_line(x, y, x, y + SQUARE_SIZE, WALL_THICKNESS, BLACK);
                }

                match &square.occupant {
                    SquareOccupant::Empty => {}
                    SquareOccupant::Guard(team) => {
                        let dot_color = match team {
                            Team::Purple => Some(PURPLE),
                            Team::White => Some(WHITE),
                            _ => None,
                        };
                        draw_circle(
                            x + SQUARE_SIZE / 2.0,
                            y + SQUARE_SIZE / 2.0,
                            GUARD_SIZE,
                            RED,
                        );
                        if let Some(dot_color) = dot_color {
                            draw_circle(
                                x + SQUARE_SIZE / 2.0,
                                y + SQUARE_SIZE / 2.0,
                                DOT_SIZE,
                                dot_color,
                            );
                        }
                    }
                    SquareOccupant::Citizen(team) => {
                        let col = match team {
                            Team::Purple => PURPLE,
                            Team::White => WHITE,
                            _ => PINK, // should never happen (panic instead?)
                        };
                        draw_circle(
                            x + SQUARE_SIZE / 2.0,
                            y + SQUARE_SIZE / 2.0,
                            CITIZEN_SIZE,
                            col,
                        );
                    }
                    SquareOccupant::Magistrate(team) => {
                        let dot_color = match team {
                            Team::Purple => Some(PURPLE),
                            Team::White => Some(WHITE),
                            _ => None,
                        };
                        draw_circle(
                            x + SQUARE_SIZE / 2.0,
                            y + SQUARE_SIZE / 2.0,
                            MAGISTRATE_SIZE,
                            BLACK,
                        );
                        if let Some(dot_color) = dot_color {
                            draw_circle(
                                x + SQUARE_SIZE / 2.0,
                                y + SQUARE_SIZE / 2.0,
                                DOT_SIZE,
                                dot_color,
                            );
                        }
                    }
                }
            }
            if let Some(intended_move) = player_move {
                if !game_state.make_move(intended_move.0, intended_move.1) {
                    draw_text("Could not make move!", screen_width - 100.0, 0.0, 32.0, RED);
                }
            }
            if game_state.murder_happened.get() {
                draw_text("There's been a murder!", 0.0, 64.0, 32.0, RED);
            }
        }
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut game_state = GameState::new();
    let mut player_state = PlayerState::new();

    loop {
        // --- frame init ---
        let _delta_time = get_frame_time();
        let (mouse_x, mouse_y) = mouse_position();

        // --- input handling ---
        if is_key_pressed(KeyCode::Q) {
            break;
        }

        // --- rendering ---
        clear_background(BLACK);
        render_game_state(&mut game_state, (mouse_x, mouse_y), &mut player_state);

        next_frame().await
    }
}
