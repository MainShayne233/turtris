use log::*;
use serde_derive::{Deserialize, Serialize};
use yew::services::Task;
use yew::{html, Callback, Component, ComponentLink, Html, Renderable, ShouldRender};

mod keydown_service;
use keydown_service::KeydownService;

use stdweb::traits::IKeyboardEvent;
use stdweb::web::event::KeyDownEvent;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 24;

pub struct App {
    state: State,
    keydown_service: KeydownService,
    keydown_cb: Callback<KeyDownEvent>,
    keydown_job: Option<Box<Task>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Piece {
    color: Color,
    position: (usize, usize, usize, usize),
}

impl Piece {
    pub fn new() -> Self {
        Piece {
            color: Color::Yellow,
            position: (4, 5, 14, 15),
        }
    }

    pub fn occupies_cell(&self, index: usize) -> bool {
        let (w, x, z, y) = self.position;
        index == w || index == x || index == z || index == y
    }
}

#[derive(Serialize, Deserialize, Clone)]
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
    current_piece: Option<Piece>,
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
    Nope,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let state = State {
            board: init_board(),
            current_piece: Some(Piece::new()),
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
        info!("Update!");
        match msg {
            Msg::ListenKeydown => {
                let handle = self.keydown_service.spawn(self.keydown_cb.clone());
                self.keydown_job = Some(Box::new(handle));
            },
            Msg::HandleKeyDown(e) => {
                info!("on key down from rust!");
            }
            _ => {}
        }
        true
    }
}

fn init_board() -> Board {
    vec![Cell { color: None }; BOARD_WIDTH * BOARD_HEIGHT]
}

impl Renderable<App> for App {
    fn view(&self) -> Html<Self> {
        info!("rendered!");
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
          { for state.board.iter().enumerate().map(|cell| view_cell(cell, state.current_piece.as_ref())) }
        </div>
    }
}

fn view_cell((index, cell): (usize, &Cell), current_piece: Option<&Piece>) -> Html<App> {
    let color = match current_piece {
        Some(piece) if piece.occupies_cell(index) => piece.color.to_hex(),
        _ => cell_color(cell),
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
