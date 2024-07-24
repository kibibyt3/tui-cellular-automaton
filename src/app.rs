use clap::{Parser, Subcommand};
use rand::{thread_rng, Rng};

#[derive(Debug)]
pub struct Model {
    cells: Vec<Vec<bool>>,
    rule: Rule,
    state: State,
    current_coords: Coords,
    max_coords: Coords,
    tickrate: u16,
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    pub birth_list: Vec<u8>,
    pub survival_list: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Editing,
    Running,
    Done,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Coords {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Move(Direction),
    ToggleCellState,
    ToggleEditing,
    Idle,
    Quit,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {

    #[arg(short, long)]
    pub rulestring: Option<String>,
    
    #[arg(short, long)]
    pub preset_string: Option<String>,

    #[arg(short, long)]
    pub tickrate: Option<u16>
}

pub struct Config {
    pub rule: Rule,
    pub preset: Preset,
    pub tickrate: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub enum Preset {
    Blinker,
    Mold,
    Random,
    Empty,
}

impl Model {
    pub fn new(max_y: i16, max_x: i16, birth_list: Vec<u8>, survival_list: Vec<u8>, tickrate: u16) -> Model {
        for birth in &birth_list {
            if *birth > 8 {
                panic!("Geometrically impossible birth constraint.");
            }
        }

        for survival in &survival_list {
            if *survival > 8 {
                panic!("Geometrically impossible survival constraint.");
            }
        }

        if (max_x <= 0) || (max_y <= 0) {
            panic!("Max coords are too small.");
        }

        let mut outer = Vec::with_capacity(max_y as usize);
        for _ in 0..=max_y {
            let mut inner = Vec::with_capacity(max_x as usize);
            for _ in 0..=max_x {
                inner.push(false);
            }
            outer.push(inner);
        }

        Model {
            cells: outer,
            rule: Rule {
                birth_list,
                survival_list,
            },
            state: State::Editing,
            current_coords: Coords { x: 0, y: 0 },
            max_coords: Coords { x: max_x, y: max_y },
            tickrate,
        }
    }

    pub fn load_preset(&mut self, preset: Preset) {
        let cells = match preset {
            Preset::Mold => vec![
                vec![false, false, false, true, true, false],
                vec![false, false, true, false, false, true],
                vec![true, false, false, true, false, true],
                vec![false, false, false, false, true, false],
                vec![true, false, true, true, false, false],
                vec![false, true, false, false, false, false],
            ],
            
            Preset::Blinker => vec![
                vec![false, false, false],
                vec![true, true, true],
                vec![false, false, false],
            ],

            Preset::Random => {
                let mut rng = thread_rng();
                let mut outer = Vec::with_capacity((self.max_coords.y + 1) as usize);
                for _ in 0..=self.max_coords.y {
                    let mut inner: Vec<bool> = Vec::with_capacity((self.max_coords.x + 1) as usize);
                    for _ in 0..=self.max_coords.x {
                        inner.push(rng.gen_bool(0.3));
                    }
                    outer.push(inner);
                }
                outer
            }

            Preset::Empty => vec![vec![false]],
        };

        self.insert_cells(cells);
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Move(dir) => self.move_cursor_in_direction(dir),
            Message::ToggleCellState => self.toggle_current_cell(),
            Message::ToggleEditing => self.toggle_editing_state(),
            Message::Idle => self.pass_tick(),
            Message::Quit => self.quit(),
        }
    }

    pub fn current_coords(&self) -> &Coords {
        &self.current_coords
    }

    pub fn update_cell(&mut self, y: usize, x: usize, val: bool) {
        if (y as i16 <= self.max_coords.y) && (x as i16 <= self.max_coords.x) {
            self.cells[y][x] = val;
        }
    }

    pub fn cells(&self) -> &Vec<Vec<bool>> {
        &self.cells
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn tickrate(&self) -> u16 {
        self.tickrate
    }

    pub fn rulestring(&self) -> String {
        let mut result = String::from("B");
        for birth_rule in &self.rule.birth_list {
            result.push_str(&birth_rule.to_string());
        }

        result.push_str("/S");

        for survival_rule in &self.rule.survival_list {
            result.push_str(&survival_rule.to_string());
        }
        result
    }

    pub fn pass_tick(&mut self) {
        if *self.state() != State::Running {
            return;
        }

        let cells_prev = self.cells().clone();
        for (y, line) in cells_prev.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                let mut active_neighbors = 0;

                // find total active neighbors
                let can_go_up = if y > 0 { true } else { false };

                let can_go_down = if (y as i16) < self.max_coords.y {
                    true
                } else {
                    false
                };

                let can_go_left = if x > 0 { true } else { false };

                let can_go_right = if (x as i16) < self.max_coords.x {
                    true
                } else {
                    false
                };

                // take care of upper, upper-left, and upper-right neighbors
                if can_go_up {
                    if cells_prev[y - 1][x] {
                        active_neighbors += 1
                    }

                    if can_go_left {
                        if cells_prev[y - 1][x - 1] {
                            active_neighbors += 1
                        }
                    }

                    if can_go_right {
                        if cells_prev[y - 1][x + 1] {
                            active_neighbors += 1
                        }
                    }
                }

                // take care of lower, lower-left, and lower-right neighbors
                if can_go_down {
                    if cells_prev[y + 1][x] {
                        active_neighbors += 1
                    }

                    if can_go_left {
                        if cells_prev[y + 1][x - 1] {
                            active_neighbors += 1
                        }
                    }

                    if can_go_right {
                        if cells_prev[y + 1][x + 1] {
                            active_neighbors += 1
                        }
                    }
                }

                // take care of left neighbor
                if can_go_left {
                    if cells_prev[y][x - 1] {
                        active_neighbors += 1
                    }
                }

                // take care of right neighbor
                if can_go_right {
                    if cells_prev[y][x + 1] {
                        active_neighbors += 1
                    }
                }

                if *cell {
                    // check if living cell survives
                    let mut kill_cell = true;
                    for criterion in &self.rule.survival_list.clone() {
                        if active_neighbors == *criterion {
                            kill_cell = false;
                        }
                    }
                    if kill_cell {
                        self.update_cell(y, x, false);
                    }
                } else {
                    // check if cell is born
                    for criterion in &self.rule.birth_list.clone() {
                        if active_neighbors == *criterion {
                            self.update_cell(y, x, true);
                        }
                    }
                }
            }
        }
    }

    fn insert_cells(&mut self, cells: Vec<Vec<bool>>) {
        for (y, line) in cells.iter().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                self.cells[y][x] = *cell;
            }
        }
    }

    fn set_cell(&mut self, y: usize, x: usize, val: bool) {
        self.cells[y][x] = val;
    }

    fn toggle_current_cell(&mut self) {
        let Coords { x: xp, y: yp } = self.current_coords();
        let (x, y) = (*xp, *yp);
        self.cells[y as usize][x as usize] = !self.cells[y as usize][x as usize];
    }

    fn toggle_editing_state(&mut self) {
        if self.state == State::Editing {
            self.state = State::Running;
        } else if self.state == State::Running {
            self.state = State::Editing;
        }
    }

    fn quit(&mut self) {
        self.state = State::Done
    }

    fn move_cursor_in_direction(&mut self, dir: Direction) {
        match dir {
            Direction::Up => self.move_cursor(0, -1),
            Direction::Down => self.move_cursor(0, 1),
            Direction::Left => self.move_cursor(-1, 0),
            Direction::Right => self.move_cursor(1, 0),
        }
    }

    fn move_cursor(&mut self, x_delta: i16, y_delta: i16) {
        if self.state == State::Editing {
            let temp_x = self.current_coords.x + x_delta;
            if temp_x <= 0 {
                self.current_coords.x = 0;
            } else if temp_x >= self.max_coords.x {
                self.current_coords.x = self.max_coords.x;
            } else {
                self.current_coords.x = temp_x;
            }

            let temp_y = self.current_coords.y + y_delta;
            if temp_y <= 0 {
                self.current_coords.y = 0;
            } else if temp_y >= self.max_coords.y {
                self.current_coords.y = self.max_coords.y;
            } else {
                self.current_coords.y = temp_y;
            }
        }
    }
}

impl Preset {
    pub fn from(preset_string: &str) -> Preset {
        match &preset_string[..] {
            "Blinker" => Preset::Blinker,
            "Mold" => Preset::Mold,
            "Random" => Preset::Random,
            _ => Preset::Empty,
        }
    }
}

impl Rule {
    pub fn from(rulestring: &str) -> Rule {
        let mut in_born = false;
        let mut in_survival = false;
        
        let mut birth_list = vec![];
        let mut survival_list = vec![];
        for ch in rulestring.chars() {
            if ch == 'B' {
                in_survival = false;
                in_born = true;
            } else if ch == 'S' {
                in_born = false;
                in_survival = true;
            } else if ch.is_alphabetic() {
                return Rule::default();
            }

            if !in_born && !in_survival {
                return Rule::default();
            }

            if ch.is_digit(10) {
                if in_born {
                    birth_list.push(ch.to_digit(10).unwrap() as u8);
                } else if in_survival {
                    survival_list.push(ch.to_digit(10).unwrap() as u8);
                } else {
                    return Rule::default();
                }
            }
        }

        Rule {
            birth_list,
            survival_list,
        }
    }

    pub fn default() -> Rule {
        Rule {
            birth_list: vec![3],
            survival_list: vec![2, 3]
        }
    }
}

impl Config {
    pub fn build(preset_string: &str, rulestring: &str, tickrate: u16) -> Config {
        Config {
            preset: Preset::from(preset_string),
            rule: Rule::from(rulestring),
            tickrate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_cursor() {
        let mut model = Model::new(10, 10, vec![], vec![]);
        model.move_cursor(-1, -4);
        assert_eq!(Coords { x: 0, y: 0 }, *model.current_coords());
        model.move_cursor(5, 6);
        assert_eq!(Coords { x: 5, y: 6 }, *model.current_coords());
        model.move_cursor(6, 6);
        assert_eq!(Coords { x: 10, y: 10 }, *model.current_coords());
    }

    #[test]
    fn move_cursor_in_direction() {
        let mut model = Model::new(10, 10, vec![], vec![]);
        model.move_cursor_in_direction(Direction::Down);
        assert_eq!(Coords { x: 0, y: 1 }, *model.current_coords());
        model.move_cursor_in_direction(Direction::Right);
        assert_eq!(Coords { x: 1, y: 1 }, *model.current_coords());
        model.move_cursor_in_direction(Direction::Up);
        assert_eq!(Coords { x: 1, y: 0 }, *model.current_coords());
        model.move_cursor_in_direction(Direction::Left);
        assert_eq!(Coords { x: 0, y: 0 }, *model.current_coords());
    }

    #[test]
    #[should_panic(expected = "Geometrically impossible birth")]
    fn too_many_neighbors_birth() {
        Model::new(10, 10, vec![1, 2, 9], vec![1, 2, 3]);
    }

    #[test]
    #[should_panic(expected = "Geometrically impossible survival")]
    fn too_many_neighbors_survival() {
        Model::new(10, 10, vec![4, 4, 4], vec![9, 4, 4]);
    }

    #[test]
    #[should_panic(expected = "Max coords")]
    fn max_x_too_small() {
        Model::new(10, -1, vec![], vec![]);
    }

    #[test]
    #[should_panic(expected = "Max coords")]
    fn max_y_too_small() {
        Model::new(0, 10, vec![], vec![]);
    }

    #[test]
    fn toggle_current_cell() {
        let mut model = Model::new(3, 3, vec![], vec![]);
        model.move_cursor_in_direction(Direction::Down);
        model.move_cursor_in_direction(Direction::Right);
        model.update(Message::ToggleCellState);
        assert_eq!(
            vec![
                vec![false; 4],
                vec![false, true, false, false],
                vec![false; 4],
                vec![false; 4]
            ],
            *model.cells()
        );
    }

    #[test]
    fn toggle_editing_state() {
        let mut model = Model::new(5, 5, vec![], vec![]);
        model.update(Message::ToggleEditing);
        assert_eq!(*model.state(), State::Running);
        model.update(Message::ToggleEditing);
        assert_eq!(*model.state(), State::Editing);
    }

    #[test]
    fn pass_tick_running_blinker() {
        let mut model = Model::new(4, 4, vec![3], vec![2, 3]);
        model.cells = vec![
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![false, true, true, true, false],
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
        ];
        model.update(Message::ToggleEditing);
        model.update(Message::Idle);
        assert_eq!(
            *model.cells(),
            vec![
                vec![false, false, false, false, false],
                vec![false, false, true, false, false],
                vec![false, false, true, false, false],
                vec![false, false, true, false, false],
                vec![false, false, false, false, false],
            ]
        );
        model.update(Message::Idle);
        assert_eq!(
            *model.cells(),
            vec![
                vec![false, false, false, false, false],
                vec![false, false, false, false, false],
                vec![false, true, true, true, false],
                vec![false, false, false, false, false],
                vec![false, false, false, false, false],
            ]
        );
    }

    #[test]
    fn load_preset() {
        let mut model = Model::new(4, 5, vec![3], vec![2, 3]);
        model.load_preset(Preset::Blinker);
        assert_eq!(
            *model.cells(),
            vec![
                vec![false, false, false, false, false, false],
                vec![true, true, true, false, false, false],
                vec![false, false, false, false, false, false],
                vec![false, false, false, false, false, false],
                vec![false, false, false, false, false, false],
            ]
        );
        model.update(Message::ToggleEditing);
        model.update(Message::Idle);
        assert_eq!(
            *model.cells(),
            vec![
                vec![false, true, false, false, false, false],
                vec![false, true, false, false, false, false],
                vec![false, true, false, false, false, false],
                vec![false, false, false, false, false, false],
                vec![false, false, false, false, false, false],
            ]
        );
    }

    #[test]
    fn pass_tick_running_mold() {
        let mut model = Model::new(5, 5, vec![3], vec![2, 3]);
        model.cells = vec![
            vec![false, false, false, true, true, false],
            vec![false, false, true, false, false, true],
            vec![true, false, false, true, false, true],
            vec![false, false, false, false, true, false],
            vec![true, false, true, true, false, false],
            vec![false, true, false, false, false, false],
        ];
        model.update(Message::ToggleEditing);
        model.update(Message::Idle);
        assert_eq!(
            *model.cells(),
            vec![
                vec![false, false, false, true, true, false],
                vec![false, false, true, false, false, true],
                vec![false, false, false, true, false, true],
                vec![false, true, true, false, true, false],
                vec![false, true, true, true, false, false],
                vec![false, true, true, false, false, false],
            ]
        );
    }

    #[test]
    fn rulestring() {
        let model = Model::new(3, 3, vec![2, 3, 5], vec![1, 7]);
        assert_eq!(model.rulestring(), "B235/S17");
    }

    #[test]
    fn rulestring_from() {
        let rule = Rule::from("2983uhjnere");
        assert_eq!(Rule::default(), rule);

        let rule = Rule::from("B45/S10");
        let expected = Rule {
            birth_list: vec![4, 5],
            survival_list: vec![1, 0],
        };

        assert_eq!(rule, expected);
    }
}
