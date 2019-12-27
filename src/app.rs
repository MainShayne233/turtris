use log::*;
use serde_derive::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, ToString};
use yew::events::IKeyboardEvent;
use yew::format::Json;
use yew::services::storage::{Area, StorageService};
use yew::{html, Component, ComponentLink, Href, Html, Renderable, ShouldRender};

const KEY: &'static str = "yew.turtris.self";

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 24;

pub struct App {
    storage: StorageService,
    state: State,
}

#[derive(Serialize, Deserialize, Clone)]
struct Piece {
    color: Color,
    position: (usize, usize, usize, usize),
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

#[derive(Serialize, Deserialize, Clone)]
struct Cell {
    color: Option<Color>,
}

type Board = Vec<Cell>;

#[derive(Serialize, Deserialize)]
pub struct State {
    entries: Vec<Entry>,
    filter: Filter,
    value: String,
    edit_value: String,
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
    Nope,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local);
        let entries = {
            if let Json(Ok(restored_entries)) = storage.restore(KEY) {
                restored_entries
            } else {
                Vec::new()
            }
        };
        let state = State {
            entries,
            filter: Filter::All,
            value: "".into(),
            edit_value: "".into(),
            board: init_board(),
            current_piece: Some(init_piece()),
        };
        App { storage, state }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Nope => {}
        }
        self.storage.store(KEY, Json(&self.state.entries));
        true
    }
}

fn init_board() -> Board {
    vec![Cell { color: None }; BOARD_WIDTH * BOARD_HEIGHT]
}

fn init_piece() -> Piece {
    Piece {
        color: Color::Yellow,
        position: (4, 5, 14, 15),
    }
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

fn piece_has_index(piece: &Piece, index: usize) -> bool {
    let (w, x, z, y) = piece.position;
    index == w || index == x || index == z || index == y
}

fn view_cell((index, cell): (usize, &Cell), current_piece: Option<&Piece>) -> Html<App> {
    let color = match current_piece {
        Some(piece) if piece_has_index(piece, index) => color_to_hex(&piece.color),
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
        Some(color) => color_to_hex(color),
        _ => String::from("white"),
    }
}

fn color_to_hex(color: &Color) -> String {
    match color {
        Color::Turquoise => String::from("#40e0d0"),
        Color::Blue => String::from("#4169e1"),
        Color::Orange => String::from("#ffa500"),
        Color::Yellow => String::from("#ffff00"),
        Color::Green => String::from(" #00ff00"),
        Color::Purple => String::from("#800080"),
        Color::Red => String::from("#ff0000"),
    }
}

#[derive(EnumIter, ToString, Clone, PartialEq, Serialize, Deserialize)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl<'a> Into<Href> for &'a Filter {
    fn into(self) -> Href {
        match *self {
            Filter::All => "#/".into(),
            Filter::Active => "#/active".into(),
            Filter::Completed => "#/completed".into(),
        }
    }
}

impl State {}
