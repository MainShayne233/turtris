use log::*;
use serde_derive::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, ToString};
use yew::events::IKeyboardEvent;
use yew::format::Json;
use yew::services::storage::{Area, StorageService};
use yew::{html, Component, ComponentLink, Href, Html, Renderable, ShouldRender};

const KEY: &'static str = "yew.todomvc.self";

pub struct App {
    storage: StorageService,
    state: State,
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

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 24;

fn init_board() -> Board {
    vec![Cell { color: None }; BOARD_WIDTH * BOARD_HEIGHT]
}

impl Renderable<App> for App {
    fn view(&self) -> Html<Self> {
        info!("rendered!");
        html! {
            <div class="app">
              { view_board(&self.state.board) }
            </div>
        }
    }
}

fn view_board(board: &Board) -> Html<App> {
    html! {
        <div class="board">
          { for board.iter().enumerate().map(view_cell) }
        </div>
    }
}

fn view_cell((index, cell): (usize, &Cell)) -> Html<App> {
    html! {
      <div class="cell" style={ format!("background-color: {}", cell_color(cell)) }>
      </div>
    }
}

fn cell_color(cell: &Cell) -> &str {
    match &cell.color {
        Some(Color::Turquoise) => "#40e0d0",
        Some(Color::Blue) => "#4169e1",
        Some(Color::Orange) => "#ffa500",
        Some(Color::Yellow) => "#ffff00",
        Some(Color::Green) => " #00FF00",
        Some(Color::Purple) => "#800080",
        Some(Color::Red) => "#ff0000",
        _ => "white",
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
