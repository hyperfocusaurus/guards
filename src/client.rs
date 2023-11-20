use std::io::{Write as IOWrite, Read};
use std::env::current_exe;
use std::thread::sleep;
use std::net::TcpStream;
use macroquad::prelude::*;
use std::fmt::Write;
use std::time;
use std::process::{Command, Child};
pub mod board;
mod game;
mod menu;
mod net;
use crate::board::{BoardSquareCoords, SquareEdge, SquareOccupant};
use crate::game::{WinState, GameState, Team};
use crate::menu::{render_menu, MenuState, MenuOption, MENU_FONT_SIZE};
use crate::net::{PORT};


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

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop (&mut self) {
        match self.0.kill() {
            Err(e) => println!("Could not kill child process: {e}"),
            Ok(_) => println!("Child killed successfully"),
        }
    }
}

struct PlayerState {
    selected_square: Option<BoardSquareCoords>,
    playing_as: Option<Team>,
}

impl PlayerState {
    fn new() -> Self {
        Self {
            selected_square: None,
            playing_as: None,
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

pub enum Scene {
    InGame,
    MainMenu
}

struct ResourceBundle {
    logo: Texture2D,
    menu_item_bg: Texture2D,
}

struct TeamPickerMenuState {
    selected_team_index: u32
}

impl TeamPickerMenuState {
    pub fn new() -> Self {
        Self {
            selected_team_index: 0,
        }
    }
}

// todo: move to menu.rs?
fn render_team_picker(resources: &ResourceBundle, state: &mut TeamPickerMenuState) -> Option<Team> {
    let (screen_width, screen_height) = (screen_width(), screen_height());
    let (mouse_x, mouse_y) = mouse_position();
    let ResourceBundle { menu_item_bg, .. } = resources;
    if is_key_pressed(KeyCode::Down) {
        state.selected_team_index = (state.selected_team_index + 1).clamp(0,1);
    }
    if is_key_pressed(KeyCode::Up) {
        state.selected_team_index = (state.selected_team_index as i32 - 1).clamp(0,1) as u32;
    }
    if is_key_pressed(KeyCode::Enter) {
        return Some(match state.selected_team_index {
            0 => Team::Purple,
            1 => Team::White,
            _ => panic!("Somehow selected a team that doesn't exist!"),
        });
    }
    for i in 0..2u32 {
        let item_width = menu_item_bg.width();
        let item_height = menu_item_bg.height();
        let item_x = (screen_width - item_width) / 2.0;
        let item_y = screen_height  / 2.0 + (item_height * i as f32);
        let texture_color = if state.selected_team_index == i {
            WHITE
        } else {
            GRAY
        };
        draw_texture(menu_item_bg, item_x, item_y, texture_color);
        let item_label = match i {
            0 => "Purple",
            1 => "White",
            _ => panic!("Tried to render menu item that does not exist"), 
        };
        let item_text_size = measure_text(item_label, None, MENU_FONT_SIZE as u16, 1.0);
        draw_text(item_label, item_x + ((menu_item_bg.width() - item_text_size.width) / 2.0), item_y + (menu_item_bg.height() / 2.0), MENU_FONT_SIZE, BEIGE);
        if mouse_x > item_x && mouse_x < item_x + item_width &&
            mouse_y > item_y && mouse_y < item_y + item_height {
                state.selected_team_index = i;
        }
    }
    // if the player has not yet picked a team, return None
    None
}

#[macroquad::main(conf)]
async fn main() {
    let mut game_state = GameState::new();
    let mut player_state = PlayerState::new();
    let mut menu_state = MenuState::new();
    let mut team_menu_state = TeamPickerMenuState::new();
    let mut connection: Option<TcpStream> = None;
    let mut scene = Scene::MainMenu;
    let mut _child: Option<ChildGuard> = None;
    let logo = load_texture("logo.png").await.unwrap();
    let menu_item_bg = load_texture("menu-item-bg.png").await.unwrap();
    let resources = ResourceBundle {
        logo,
        menu_item_bg
    };
    let mut rx_buf = vec![];

    loop {
        // --- frame init ---
        let _delta_time = get_frame_time();
        let (mouse_x, mouse_y) = mouse_position();

        // --- general input handling ---
        if is_key_pressed(KeyCode::Q) {
            scene = Scene::MainMenu;
        }

        // --- rendering ---
        clear_background(BLACK);
        match scene {
            Scene::InGame => {
                match &mut connection {
                    Some(stream) => {
                        if let Some(_) = player_state.playing_as {
                            match stream.read_to_end(&mut rx_buf) { Ok(_) => {
                                    if rx_buf.len() > 0 {
                                        println!("Received: {:?}", rx_buf);
                                    }
                                }
                                // ignore errors, including EWOULDBLOCK
                                _ => {}
                            }
                            render_game_state(&mut game_state, (mouse_x, mouse_y), &mut player_state);
                        } else {
                            player_state.playing_as = render_team_picker(&resources, &mut team_menu_state);
                            if let Some(team) = player_state.playing_as {
                                let mut s = String::new();
                                write!(s, "join {}", team.as_network_string()).unwrap();
                                stream.write(s.as_bytes()).unwrap();
                            }
                        }
                    }
                    None => {
                        render_game_state(&mut game_state, (mouse_x, mouse_y), &mut player_state);
                    }
                }
            }
            Scene::MainMenu => {
                if let Some(option) = render_menu(&mut menu_state, &resources) {
                    match option {
                        MenuOption::Quit => {
                            break;
                        }
                        MenuOption::LocalGame => {
                            game_state.reset(); // shouldn't be required, but just in case
                            scene = Scene::InGame;
                        }
                        MenuOption::HostMultiplayer => {
                            if let Ok(mut path_to_executable) = current_exe() {
                                path_to_executable.pop();
                                if cfg!(target_os = "windows") {
                                    path_to_executable.push("guardsd.exe");
                                } else {
                                    path_to_executable.push("guardsd");
                                }
                                _child = Some(ChildGuard(Command::new(path_to_executable).spawn().expect("Could not run server executable")));
                            }
                            // wait a moment for the server to start listening (todo: can this be
                            // implemented differently? maybe give the user some feedback that this
                            // is what we're doing?)
                            sleep(time::Duration::from_millis(500));

                            let mut constr = String::new();
                            let _ = write!(constr, "127.0.0.1:{}", PORT);
                            let stream = TcpStream::connect(constr.as_str()).expect("Failed to connect to server");
                            stream.set_nonblocking(true).expect("Could not set stream as non-blocking");

                            connection = Some(stream);
                            scene = Scene::InGame;
                        }
                        _ => {}
                    }
                }
            }
        }

        next_frame().await
    }
}
