use log::*;
use serde_derive::{Deserialize, Serialize};
use yew::services::Task;
use yew::{html, Callback, Component, ComponentLink, Html, Renderable, ShouldRender};

use std::convert::TryFrom;

mod keydown_service;
use keydown_service::KeydownService;

use crate::stdweb::unstable::TryInto;
use stdweb::traits::IKeyboardEvent;
use stdweb::web::event::KeyDownEvent;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 24;

pub struct App {
    state: State,
    keydown_service: KeydownService,
    keydown_cb: Callback<KeyDownEvent>,
    keydown_job: Option<Box<dyn Task>>,
}

type Position = (usize, usize, usize, usize);

type TheoritcalPosition = (i16, i16, i16, i16);

fn position_to_theoritical(position: Position) -> TheoritcalPosition {
    let (w, x, y, z) = position;
    (
        i16::try_from(w).unwrap(),
        i16::try_from(x).unwrap(),
        i16::try_from(y).unwrap(),
        i16::try_from(z).unwrap(),
    )
}

fn position_from_theoritical(theoritcal: TheoritcalPosition) -> Position {
    let (w, x, y, z) = theoritcal;
    (
        usize::try_from(w).unwrap(),
        usize::try_from(x).unwrap(),
        usize::try_from(y).unwrap(),
        usize::try_from(z).unwrap(),
    )
}

#[derive(Serialize, Deserialize, Clone)]
struct Piece {
    color: Color,
    position: Position,
}

impl Piece {
    pub fn new() -> Self {
        // using JS because rand doesn't play well with wasm lol
        let random_js_number: usize = js! { return Math.floor(Math.random() * 7) }
            .try_into()
            .unwrap();
        match random_js_number {
            0 => Piece {
                color: Color::Yellow,
                position: (4, 5, 14, 15),
            },
            1 => Piece {
                color: Color::Green,
                position: (14, 15, 5, 6),
            },
            2 => Piece {
                color: Color::Red,
                position: (4, 5, 15, 16),
            },
            3 => Piece {
                color: Color::Purple,
                position: (5, 14, 15, 16),
            },
            4 => Piece {
                color: Color::Orange,
                position: (14, 15, 16, 6),
            },
            5 => Piece {
                color: Color::Blue,
                position: (4, 14, 15, 16),
            },
            _ => Piece {
                color: Color::Turquoise,
                position: (4, 14, 24, 34),
            },
        }
    }

    pub fn occupies_cell(&self, index: usize) -> bool {
        let (w, x, z, y) = self.position;
        index == w || index == x || index == z || index == y
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
enum Color {
    Turquoise,
    Blue,
    Orange,
    Yellow,
    Green,
    Purple,
    Red,
}

impl Color {
    fn to_hex(&self) -> String {
        match self {
            Color::Turquoise => String::from("#40e0d0"),
            Color::Blue => String::from("#4169e1"),
            Color::Orange => String::from("#ffa500"),
            Color::Yellow => String::from("#ffff00"),
            Color::Green => String::from(" #00ff00"),
            Color::Purple => String::from("#800080"),
            Color::Red => String::from("#ff0000"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Cell {
    color: Option<Color>,
}

type Board = Vec<Cell>;

#[derive(Serialize, Deserialize)]
pub struct State {
    board: Board,
    current_piece: Piece,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    description: String,
    completed: bool,
    editing: bool,
}

pub enum Msg {
    ListenKeydown,
    HandleKeyDown(KeyDownEvent),
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
enum GameEvent {
    MoveCurrentPiece(Direction),
    RotateCurrentPiece,
    PlaceCurrentPiece,
    NoOP,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let state = State {
            board: init_board(),
            current_piece: Piece::new(),
        };
        let app = App {
            state,
            keydown_service: KeydownService::new(),
            keydown_cb: link.send_back(|e| Msg::HandleKeyDown(e)),
            keydown_job: None,
        };
        link.send_self(Msg::ListenKeydown);
        app
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ListenKeydown => {
                let handle = self.keydown_service.spawn(self.keydown_cb.clone());
                self.keydown_job = Some(Box::new(handle));
            }
            Msg::HandleKeyDown(event) => {
                info!("{}", event.key());
                match decode_event(event) {
                    GameEvent::MoveCurrentPiece(direction) => {
                        self.state.current_piece.position = attempt_move(
                            &self.state.board,
                            &self.state.current_piece.position,
                            direction,
                        );
                    }
                    GameEvent::RotateCurrentPiece => {
                        self.state.current_piece.position =
                            attempt_rotate(&self.state.board, &self.state.current_piece.position);
                    }
                    GameEvent::PlaceCurrentPiece => {
                        let (w, x, y, z) = self.state.current_piece.position;
                        for cell in &[w, x, y, z] {
                            self.state.board[*cell] = Cell {
                                color: Some(self.state.current_piece.color),
                            }
                        }
                        self.state.current_piece = Piece::new()
                    }
                    GameEvent::NoOP => {}
                }
            }
        }
        true
    }
}

fn attempt_move(board: &Board, piece_position: &Position, direction: Direction) -> Position {
    let new_position = calculate_new_position(piece_position, direction);
    if move_is_legal(board, &piece_position, &new_position) {
        position_from_theoritical(new_position)
    } else {
        *piece_position
    }
}

fn attempt_rotate(board: &Board, piece_position: &Position) -> Position {
    let new_position = calculate_rotation(piece_position);
    if move_is_legal(board, &piece_position, &new_position) {
        position_from_theoritical(new_position)
    } else {
        *piece_position
    }
}

fn move_is_legal(
    board: &Board,
    old_position: &Position,
    new_position: &TheoritcalPosition,
) -> bool {
    let (old_w, old_x, old_y, old_z) = *old_position;
    let (new_w, new_x, new_y, new_z) = *new_position;

    for (old, new) in &[
        (old_w, new_w),
        (old_x, new_x),
        (old_y, new_y),
        (old_z, new_z),
    ] {
        match (*old, *new) {
            // top and bottom bounds
            (_, new) if new < 0 || new > 239 => return false,
            // right bound
            (old, new) if (old + 1) % 10 == 0 && new % 10 == 0 => return false,
            // left bound
            (old, new) if old % 10 == 0 && (new + 1) % 10 == 0 => return false,
            // cell is taken
            (_, new) if board[usize::try_from(new).unwrap()].color.is_some() => return false,
            _ => {}
        }
    }
    true
}

fn calculate_rotation(piece_position: &Position) -> TheoritcalPosition {
    let (w, x, y, z) = position_to_theoritical(*piece_position);
    let cells = &[w, x, y, z];
    let horizontal_adjust: i16 = cells.iter().map(|v| v % 10).min().unwrap();
    let vertical_adjust: i16 = cells.iter().map(|v| v / 10).min().unwrap();
    let ((w, x, y, z), additional_adjust) = match (
        w - horizontal_adjust - vertical_adjust * 10,
        x - horizontal_adjust - vertical_adjust * 10,
        y - horizontal_adjust - vertical_adjust * 10,
        z - horizontal_adjust - vertical_adjust * 10,
    ) {
        // yellow
        (0, 1, 10, 11) => ((0, 1, 10, 11), 0),
        // red
        (0, 1, 11, 12) => ((1, 10, 11, 20), 0),
        (1, 10, 11, 20) => ((0, 1, 11, 12), 0),
        // green
        (10, 11, 1, 2) => ((0, 10, 11, 21), 0),
        (0, 10, 11, 21) => ((10, 11, 1, 2), 0),
        // purple
        (1, 10, 11, 12) => ((1, 11, 12, 21), 0),
        (0, 10, 11, 20) => ((9, 10, 11, 20), 0),
        (0, 1, 2, 11) => ((1, 10, 11, 21), -10),
        (1, 10, 11, 21) => ((1, 10, 11, 12), 0),
        // orange
        (10, 11, 12, 2) => ((1, 11, 21, 22), 0),
        (0, 10, 20, 21) => ((10, 11, 12, 20), -1),
        (0, 1, 2, 10) => ((1, 2, 12, 22), -10),
        (0, 1, 11, 21) => ((10, 11, 12, 2), -1),
        // blue
        (0, 10, 11, 12) => ((1, 2, 11, 21), 0),
        (0, 1, 10, 20) => ((0, 1, 2, 12), -1),
        (0, 1, 2, 12) => ((2, 12, 21, 22), 0),
        (1, 11, 20, 21) => ((0, 10, 11, 12), -1),
        // turquoise
        (0, 10, 20, 30) => ((0, 1, 2, 3), 9),
        (0, 1, 2, 3) => ((0, 10, 20, 30), -9),
        (adjusted_w, adjusted_x, adjusted_y, adjusted_z) => {
            info!(
                "{} {} {} {}",
                adjusted_w, adjusted_x, adjusted_y, adjusted_z
            );
            ((adjusted_w, adjusted_x, adjusted_y, adjusted_z), 0)
        }
    };

    (
        w + horizontal_adjust + vertical_adjust * 10 + additional_adjust,
        x + horizontal_adjust + vertical_adjust * 10 + additional_adjust,
        y + horizontal_adjust + vertical_adjust * 10 + additional_adjust,
        z + horizontal_adjust + vertical_adjust * 10 + additional_adjust,
    )
}

fn calculate_new_position(piece_position: &Position, direction: Direction) -> TheoritcalPosition {
    let (w, x, y, z) = position_to_theoritical(*piece_position);
    let width = i16::try_from(BOARD_WIDTH).unwrap();
    match direction {
        Direction::Down => (w + width, x + width, y + width, z + width),
        Direction::Up => (w - width, x - width, y - width, z - width),
        Direction::Left => (w - 1, x - 1, y - 1, z - 1),
        Direction::Right => (w + 1, x + 1, y + 1, z + 1),
    }
}

fn decode_event(event: KeyDownEvent) -> GameEvent {
    match &event.key()[..] {
        "a" => GameEvent::MoveCurrentPiece(Direction::Left),
        "s" => GameEvent::MoveCurrentPiece(Direction::Down),
        "d" => GameEvent::MoveCurrentPiece(Direction::Right),
        "w" => GameEvent::MoveCurrentPiece(Direction::Up),
        "p" => GameEvent::PlaceCurrentPiece,
        " " => GameEvent::RotateCurrentPiece,
        _ => GameEvent::NoOP,
    }
}

fn init_board() -> Board {
    let mut board = vec![Cell { color: None }; BOARD_WIDTH * BOARD_HEIGHT];
    board[232] = Cell {
        color: Some(Color::Red),
    };
    board
}

impl Renderable<App> for App {
    fn view(&self) -> Html<Self> {
        html! {
           <div class="app">
             { view_state( &self.state) }
           </div>
        }
    }
}

fn view_state(state: &State) -> Html<App> {
    html! {
        <div class="board">
          { for state.board.iter().enumerate().map(|cell| view_cell(cell, &state.current_piece)) }
        </div>
    }
}

fn view_cell((index, cell): (usize, &Cell), current_piece: &Piece) -> Html<App> {
    let color = if current_piece.occupies_cell(index) {
        current_piece.color.to_hex()
    } else {
        cell_color(cell)
    };

    html! {
      <div class="cell" style={ format!("background-color: {}", color) }>
        <p>{ index }</p>
      </div>
    }
}

fn cell_color(cell: &Cell) -> String {
    match &cell.color {
        Some(color) => color.to_hex(),
        _ => String::from("white"),
    }
}

impl State {}
