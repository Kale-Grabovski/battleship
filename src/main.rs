extern crate sdl2;
extern crate rand;

use rand::Rng;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use std::time::Duration;
use sdl2::video::{Window, WindowContext};
use sdl2::rect::{Rect};
use sdl2::render::{Canvas, Texture, TextureCreator};

#[derive(Clone, Copy, PartialEq)]
enum CellState {Alive, Injured, Dead, Empty, Miss}

#[derive(Clone)]
struct Cell {
	state: CellState,
	coords: [usize; 4],
}

impl Cell {
	fn new() -> Cell {
        Cell{state: CellState::Empty, coords: [100, 100, 100, 100]}
	}
}

#[derive(Copy, Clone, PartialEq)]
enum State {Me, Enemy, Won, Lost}

struct Game<'a, 'b> {
	canvas: &'a mut Canvas<Window>,
	tc: &'b TextureCreator<WindowContext>,
	me: Vec<Cell>,
	enemy: Vec<Cell>,
	cell_size: u32,
	field_size: u32,
	xy: [i32; 2],
	fields_space_px: i32,
    state: State,
}

impl<'a, 'b> Game<'a, 'b> {
	pub fn new(
		canvas: &'a mut Canvas<Window>,
		tc: &'b TextureCreator<WindowContext>,
		cell_size: u32,
		xy: [i32; 2],
	) -> Game<'a, 'b> {
		let mut me = vec![Cell::new(); 100];
		let mut enemy = vec![Cell::new(); 100];

		Game::gen_ships(&mut me);
		Game::gen_ships(&mut enemy);

		Game {
			canvas,
            tc,
			me,
			enemy,
			cell_size,
			field_size: cell_size * 10 + 1,
			xy,
			fields_space_px: 50,
            state: State::Me,
		}
	}

	pub fn get_state(&self) -> State {self.state}
	pub fn set_state(&mut self, state: State) {self.state = state;}

	fn gen_ships(cells: &mut Vec<Cell>) {
		enum Dir {Hor, Ver}

		let mut rng = rand::thread_rng();

        for i in &[4, 3, 3, 2, 2, 2, 1, 1, 1, 1] {
			loop {
				let dir = match rng.gen_range(0, 2) {
					0 => Dir::Hor,
					_ => Dir::Ver,
				};

				let shift = match dir {
					Dir::Hor => 1,
					Dir::Ver => 10,
				};
				let upper = match dir {
					Dir::Hor => (10 - i + 1, 10),
					Dir::Ver => (10, 10 - i + 1),
				};

				let row = rng.gen_range(0, upper.0);
				let col = rng.gen_range(0, upper.1);

				let mut is_ok = true;
				let mut coords = vec![];
				'outer: for k in 0..*i {
					let t = row + col * 10 + k * shift;
					let check_cells: [i16; 9] = [
						t, t + 1, t - 1,
						t + 11, t + 10, t + 9,
						t - 11, t - 10, t - 9,
					];
					for c in &check_cells {
                        if *c >= 0 && *c < 100 && cells[*c as usize].state != CellState::Empty {
							is_ok = false;
							break 'outer;
						}
					}

					coords.push(t);
				}

				if is_ok {
					for t in &coords {
                        cells[*t as usize].state = CellState::Alive;
                        // save ship coordinates at each ship cell
						for (k, z) in coords.iter().enumerate() {
							cells[*t as usize].coords[k] = *z as usize;
						}
					}
					break;
				}
			}
		}
	}

	pub fn draw(&mut self) {
		if self.state == State::Enemy {
			self.enemy_turn();
		}

		self.canvas.set_draw_color(Color::RGB(0, 0, 0));
		self.canvas.clear();

		let me = self.draw_field(false);
		let enemy = self.draw_field(true);

		self.canvas.copy(
			&me,
			None,
			Rect::new(self.xy[0], self.xy[1], self.field_size, self.field_size),
		).unwrap();
		self.canvas.copy(
			&enemy,
			None,
			Rect::new(
				self.xy[0] + (self.field_size as i32) + self.fields_space_px,
				self.xy[1],
				self.field_size,
				self.field_size,
			),
		).unwrap();

		self.canvas.present();
	}

	fn enemy_turn(&mut self) {
        loop {
            let mut rng = rand::thread_rng();
            let cell = rng.gen_range(0, 100);
			match self.me[cell].state {
				CellState::Alive => {
					self.me[cell].state = CellState::Injured;
					self.check_dead_ship(false, cell);
					self.check_game_over();
				},
				CellState::Empty => {
					self.me[cell].state = CellState::Miss;
					self.set_state(State::Me);
					break;
                }
				_ => {},
			}
		}
	}

	fn check_game_over(&mut self) {
        let mut alive = false;
		for i in &self.me {
            if i.state == CellState::Alive {
                alive = true;
                break;
            }
        }

        if !alive {
            return self.set_state(State::Lost);
        }

        alive = false;
        for i in &self.enemy {
            if i.state == CellState::Alive {
                alive = true;
                break;
            }
        }

        if !alive {
            return self.set_state(State::Won);
        }
	}

	fn check_dead_ship(&mut self, is_enemy: bool, cell: usize) {
		let cells = match is_enemy {
			true => &mut self.enemy,
			false => &mut self.me,
		};

		let mut is_dead = true;
		for i in &cells[cell].coords {
			if i < &100 && cells[*i].state == CellState::Alive {
				is_dead = false;
			}
		}

		if !is_dead {
			return;
		}
        for i in &cells[cell].coords.clone() {
            if i == &100 {
                continue;
            }

            let u = *i as i16;
            let around_cell: [i16; 8] = [
                u + 1, u - 1,
                u + 11, u + 10, u + 9,
                u - 11, u - 10, u - 9,
            ];

            for k in &around_cell {
                if k < &0 || k >= &100 {
                    continue;
                }

				match cells[*k as usize].state {
					CellState::Injured | CellState::Dead => {},
					_ => cells[*k as usize].state = CellState::Miss,
				}
            }

			cells[*i as usize].state = CellState::Dead;
        }
    }

	pub fn shot(&mut self, x: i32, y: i32) {
		let cell_x = (x - (self.xy[0] + (self.field_size as i32) + self.fields_space_px)) / self.cell_size as i32;
		let cell_y = (y - self.xy[1]) / self.cell_size as i32;
		if cell_x < 0 || cell_y < 0 || cell_x > 9 || cell_y > 9 {
			return;
		}

		let cell = (cell_x + cell_y * 10) as usize;
        match self.enemy[cell].state {
            CellState::Alive => {
				self.enemy[cell].state = CellState::Injured;
				self.check_dead_ship(true, cell);
				self.check_game_over();
			},
			CellState::Injured | CellState::Dead | CellState::Miss => {},
			CellState::Empty => {
				self.enemy[cell].state = CellState::Miss;
				self.set_state(State::Enemy);
			},
		}
	}

	fn draw_field(&mut self, is_enemy: bool) -> Texture<'b> {
		let field_size = &self.field_size;
		let mut t: Texture = self.tc.create_texture_target(None, self.field_size, self.field_size).unwrap();

		let cell_size = &self.cell_size;
        let cells = match is_enemy {
			true => &self.enemy,
			false => &self.me,
		};

		self.canvas.with_texture_canvas(&mut t, |tc| {
			tc.set_draw_color(Color::RGB(0, 0, 0));
			tc.clear();

			tc.set_draw_color(Color::RGB(99, 159, 255));
			for row in 0..11 {
				tc.fill_rect(Rect::new(0, (row * cell_size) as i32, cell_size * 10, 1));
				tc.fill_rect(Rect::new((row * cell_size) as i32, 0, 1, cell_size * 10));
			}

			for (i, cell) in cells.iter().enumerate() {
				let x = (i as u32 % 10 * cell_size) as i32 + 1;
				let y = (i as u32 / 10 * cell_size) as i32 + 1;

				match cell.state {
					CellState::Miss => {
						tc.set_draw_color(Color::RGB(255, 255, 255));
						tc.draw_rect(Rect::new(
							x + (cell_size / 2) as i32,
							y + (cell_size / 2) as i32,
							2,
							2,
						));
					},
					CellState::Alive if !is_enemy => {
						tc.set_draw_color(Color::RGB(225, 225, 225));
						tc.fill_rect(Rect::new(x, y, cell_size - 1, cell_size - 1));
					},
					CellState::Injured => {
						tc.set_draw_color(Color::RGB(0, 155, 155));
						tc.fill_rect(Rect::new(x, y, cell_size - 1, cell_size - 1));
					},
					CellState::Dead => {
						tc.set_draw_color(Color::RGB(215, 215, 0));
						tc.fill_rect(Rect::new(x, y, cell_size - 1, cell_size - 1));
					},
                    _ => {},
				}
			}
		});

		t
	}
}

fn main() {
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let window = video_subsystem.window("Ship", 660, 350).build().unwrap();

	let mut canvas: Canvas<Window> = window.into_canvas()
		.present_vsync()
		.build()
        .unwrap();

    let texture_creator : TextureCreator<_> = canvas.texture_creator();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut game = Game::new(&mut canvas, &texture_creator, 25, [50, 50]);

	'running: loop {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					break 'running
				},
				Event::MouseButtonDown { x, y, mouse_btn: MouseButton::Left, .. } => {
                    game.shot(x, y);
				},
				_ => {}
			}
		}

        match game.get_state() {
            State::Won => {
                println!("You won");
                break;
            },
            State::Lost => {
                println!("You lost");
                break;
            },
            _ => {},
        }

		game.draw();

		::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
	}
}
