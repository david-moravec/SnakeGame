use piston_window::*;
use piston_window::types::Color;

use rand::{thread_rng, Rng};

use crate::snake::{Direction, Snake};
use crate::draw::{draw_block, draw_rect};

const FOOD_COLOR: Color = [0.80, 0.00, 0.00, 1.0];
const BORDER_COLOR:Color = [0.00, 0.00, 0.00, 1.0];
const GAMEOVER_COLOR:Color = [0.90, 0.00, 0.00, 0.5];

const MOVING_PERIOD: f64 = 0.1;
const RESTART_TIME: f64 = 1.0;

pub struct Game {
  snake: Snake,

  food_exists: bool,
  food_x: i32,
  food_y: i32,

  width: i32,
  height: i32,

  game_over: bool,
  waiting_time: f64,
}

impl Game {
  pub fn new(width: i32, height: i32) -> Game {
    Game {
      snake: Snake::new(2, 2),
      waiting_time: 0.0,
      food_exists: true,
      food_x: 6,
      food_y: 4,
      width,
      height,
      game_over: false
    }
  }

  pub fn key_pressed(&mut self, key: Key) {
    if self.game_over {
      return
    }

    let dir = match key {
      Key::Up => Some(Direction::Up),
      Key::Down => Some(Direction::Down),
      Key::Right => Some(Direction::Right),
      Key::Left => Some(Direction::Left),
      _ => None,
    };

    self.update_snake(dir);
  }

  pub fn draw(&self, con: &Context, g: &mut G2d) {
    self.snake.draw(con, g);
    
    if self.food_exists {
      draw_block(FOOD_COLOR, self.food_x, self.food_y,con, g)
    }

    draw_rect(BORDER_COLOR, 0, 0, self.width, 1, con, g);
    draw_rect(BORDER_COLOR, 0, self.height -1, self.width, 1, con, g);
    draw_rect(BORDER_COLOR, 0, 0, 1, self.width, con, g);
    draw_rect(BORDER_COLOR, 0, self.width -1, 0, 1, con, g);

    if self.game_over {
      draw_rect(GAMEOVER_COLOR, 0, 0, self.width, self.height, con, g)
    }
  }

  pub fn update(&mut self, delta_time: f64) {
    self.waiting_time += delta_time;

    if self.game_over {
      if self.waiting_time > RESTART_TIME {
        self.restart();
      }
      
      return 
    }

    if !self.food_exists {
      self.add_food();
    }

    if self.waiting_time > MOVING_PERIOD {
      self.update_snake(None);
    }
  }

  fn check_eating(&mut self) {
    if self.food_exists && self.snake.has_head_at(self.food_x, self.food_y) {
       self.snake.grow();
       self.food_exists = false;
    }
  }

  fn is_snake_alive(&self, dir: Option<Direction>) -> bool {
    let (next_x ,next_y) = self.snake.next_head_coords();

    if self.snake.is_crawling_over(next_x, next_y) {
      return false;
    }

    0 < next_x && next_x < self.width  - 1  && 
    0 < next_y && next_y < self.height - 1
  }

  fn add_food(&mut self) {
    let mut rng = thread_rng();
    let candidate_x = rng.gen_range(1..self.width  - 1);
    let candidate_y = rng.gen_range(1..self.height - 1);

    // we do not want to put food where snake body is
    if self.snake.is_crawling_over(candidate_x, candidate_y) {
      self.add_food();
      return
    }

    self.food_x = candidate_x;
    self.food_y = candidate_y;
    self.food_exists = true;
  }

  fn update_snake(&mut self, dir: Option<Direction>) {
    if self.is_snake_alive(dir) {
      self.snake.move_forward(dir);
      self.check_eating();
    } else {
      self.game_over = true;
    }
    self.waiting_time = 0.0
    }

  fn restart(&mut self) {
    self.snake = Snake::new(2, 2);
    self.waiting_time = 0.0;
    self.game_over = false;
    self.add_food();
  }
}