use log::*;
use serde_derive::{Deserialize, Serialize};
use yew::services::Task;
use yew::{html, Callback, Component, ComponentLink, Html, Renderable, ShouldRender};

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

#[derive(Serialize, Deserialize, Clone)]
struct Piece {
    color: Color,
    position: Position,
}

impl Piece {
    pub fn new() -> Self {
        // using JS because rand doesn't play well with wasm lol
        let random_js_number = js! { return Math.floor(Math.random() * 7) };
        match random_js_number.try_into().unwrap() {
            0 => Piece {
                color: Color::Yellow,
                position: (4, 5, 14, 15),
            },
            1 => Piece {
                color: Color::Green,
                position: (14, 15, 5, 6)
            },
            2 => Piece {
                color: Color::Red,
                position: (4, 5, 15, 16)
            },
            3 => Piece {
                color: Color::Purple,
                position: (5, 14, 15, 16)
            },
            4 => Piece {
                color: Color::Orange,
                position: (14, 15, 16, 6)
            },
            5 => Piece {
                color: Color::Blue,
                position: (4, 14, 15, 16)
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
                    GameEvent::RotateCurrentPiece => {}
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
    if move_is_legal(board, &piece_position, &direction) {
        calculate_new_position(piece_position, direction)
    } else {
        *piece_position
    }
}

fn move_is_legal(board: &Board, position: &Position, direction: &Direction) -> bool {
    let (w, x, y, z) = *position;

    for cell in &[w, x, y, z] {
        match (*cell, direction) {
            // out of bounds cases
            (index, Direction::Up) if index < BOARD_WIDTH => return false,
            (index, Direction::Down) if index > BOARD_WIDTH * (BOARD_HEIGHT - 1) => return false,
            (index, Direction::Left) if index % BOARD_WIDTH == 0 => return false,
            (index, Direction::Right) if (index + 1) % BOARD_WIDTH == 0 => return false,
            // collision cases
            (index, Direction::Up) if board[index - BOARD_WIDTH].color.is_some() => return false,
            (index, Direction::Down) if board[index + BOARD_WIDTH].color.is_some() => return false,
            (index, Direction::Left) if board[index - 1].color.is_some() => return false,
            (index, Direction::Right) if board[index + 1].color.is_some() => return false,
            _ => {}
        }
    }
    true
}

fn calculate_new_position(piece_position: &Position, direction: Direction) -> Position {
    let (w, x, y, z) = *piece_position;
    match direction {
        Direction::Down => (
            w + BOARD_WIDTH,
            x + BOARD_WIDTH,
            y + BOARD_WIDTH,
            z + BOARD_WIDTH,
        ),
        Direction::Up => (
            w - BOARD_WIDTH,
            x - BOARD_WIDTH,
            y - BOARD_WIDTH,
            z - BOARD_WIDTH,
        ),
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
