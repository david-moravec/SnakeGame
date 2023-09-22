use std::collections::LinkedList;
use piston_window::{Context, G2d};
use piston_window::types::Color;

use crate::draw::draw_block;

const SNAKE_COLOR: Color = [0.00, 0.80, 0.00, 1.0];

#[derive(Copy, Clone, PartialEq)]
pub enum Direction {
  Up,
  Down,
  Left,
  Right,
}

impl Direction {
  /// Returns the opposite of this [`Direction`].
pub fn opposite(&self) -> Direction {
    match *self {
      Direction::Up => Direction::Down,
      Direction::Down => Direction::Up,
      Direction::Left => Direction::Right,
      Direction::Right => Direction::Left,
    }
  }
}

#[derive(Clone, Debug)]
struct Block {
  x: i32,
  y: i32,
}

pub struct Snake{
  direction: Direction,
  body: LinkedList<Block>,
  tail: Option<Block>,
}

impl Snake {
  pub fn new(x: i32, y: i32) -> Snake {
    let mut body: LinkedList<Block> = LinkedList::new();
    
    body.push_back(Block{x: x + 2, y,});
    body.push_back(Block{x: x + 1, y,});
    body.push_back(Block{x, y,});

    Snake{direction: Direction::Right, body, tail: None,}
  }

  pub fn draw(&self, con: &Context, g: &mut G2d) {
    for block in &self.body {
      draw_block(SNAKE_COLOR, block.x, block.y, con, g);
    }
  }

  fn head_position(&self) -> (i32, i32) {
    match self.body.front() {
      Some(ref block) => (block.x, block.y),
      None => (2,2)
    }
  }

  pub fn has_head_at(&self, x: i32, y: i32) -> bool {
    match self.body.front() {
      Some(ref block) => x == block.x && y == block.y,
      None => false,
    }
  }

  pub fn move_forward(&mut self, dir: Option<Direction>) {
    let (new_x, new_y) = self.next_head_coords(dir);
    self.tail = self.body.pop_back();
    self.body.push_front(Block{x: new_x, y: new_y});
  }

  pub fn head_direction(&self) -> Direction{
    self.direction
  }

  pub fn next_head_coords(&self, dir: Option<Direction>) -> (i32, i32) {
    let (head_x, head_y): (i32, i32) = self.head_position();

    match dir.unwrap_or(self.direction) {
      Direction::Up => (head_x, head_y - 1),
      Direction::Down => (head_x, head_y +1),
      Direction::Left => (head_x - 1, head_y),
      Direction::Right => (head_x + 1, head_y),
    }
  }

  pub fn grow(&mut self) {
    let block = self.tail.clone().unwrap();
    self.body.push_back(block);
  }

  pub fn is_crawling_over(&self, x: i32, y: i32) -> bool{
    self.body.iter().all(|block: &Block| -> bool {x == block.x && y == block.y})
  }
  
}