extern crate ndarray;

use ndarray::prelude::*;
use std::fmt;

#[derive(Clone, PartialEq)]
pub enum Player {
    PlayerCross,
    PlayerCircle
}

pub struct Game {
    field: Array2<Option<Player>>,
    current_player: Player,
    fieldsize: usize,
}

pub enum TurnOutcome {
    Continue,
    TurnInvalid,
    PlayerWon(Player),
    Stalemate,
}

impl Game {
    fn new(size: usize, starting_player: Player) -> Game {
        Game {
            field: Array::from_elem((size, size), None),
            current_player: starting_player,
            fieldsize: size,
        }
    }

    fn has_won(&self) -> bool {
        let horizontal = self.field.axis_iter(Axis(0)).any(|row|
            row.iter().all(|field| field == &Some(self.current_player.clone()))
        );
        let vertical = self.field.axis_iter(Axis(1)).any(|coloumn|
            coloumn.iter().all(|field| field == &Some(self.current_player.clone()))
        );
        let descending = (0..self.fieldsize).all(|i|
            self.field[[i, i]] == Some(self.current_player.clone())
        );
        let ascending = (0..self.fieldsize).all(|i|
            self.field[[i, self.fieldsize - 1 - i]] == Some(self.current_player.clone())
        );

        horizontal || vertical || descending || ascending
    }

    fn is_stalemate(&self) -> bool {
        self.field.iter().all(|i| !i.is_none())
    }

    fn make_turn(&mut self, x: usize, y: usize) -> TurnOutcome {
        if x >= self.fieldsize || y >= self.fieldsize || !self.field[[y, x]].is_none() {
            return TurnOutcome::TurnInvalid;
        }

        self.field[[y, x]] = Some(self.current_player.clone());

        if self.has_won() {
            return TurnOutcome::PlayerWon(self.current_player.clone());
        } else if self.is_stalemate() {
            return TurnOutcome::Stalemate;
        }

        self.current_player = match self.current_player {
            Player::PlayerCross => Player::PlayerCircle,
            Player::PlayerCircle => Player::PlayerCross,
        };

        return TurnOutcome::Continue;
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (pos, row) in self.field.axis_iter(Axis(0)).enumerate() {
            let row_formatted = row.iter()
                .map(|i| match i {
                    &None => " ",
                    &Some(Player::PlayerCircle) => "o",
                    &Some(Player::PlayerCross) => "x",
                }).fold("|".to_owned(), |a, i| a + i + "|");
            write!(f, "{} {}\n", pos, row_formatted)?;
        }
        Ok(())
    }
}

fn read_usize(message: &str) -> usize {
    use std::io::Write;
    loop {
        print!("{}", message);
        std::io::stdout().flush().expect("Your console's broken");
        let mut input = String::new();
        let result = std::io::stdin().read_line(&mut input).map_err(|_| ());
        let x = result.and_then(|_| input.trim().parse().map_err(|_| ()));
        if let Ok(value) = x {
            return value;
        } else {
            println!("Invalid Input. Try again");
        }
    }
}

fn main() {
    println!("Hello, world!");
    let mut game = Game::new(3, Player::PlayerCircle);
    println!("{}", game);
    loop {
        let x = read_usize("Enter the x-coordinate: ");
        let y = read_usize("Enter the y-coordinate: ");

        let result = game.make_turn(x, y);
        println!("{}", game);

        match result {
            TurnOutcome::Continue => continue,
            TurnOutcome::TurnInvalid => println!("Invalid turn! Try again"),
            TurnOutcome::Stalemate => {
                println!("There was no winner");
                break;
            }
            TurnOutcome::PlayerWon(Player::PlayerCross) => {
                println!("Player cross won");
                break;
            }
            TurnOutcome::PlayerWon(Player::PlayerCircle) => {
                println!("Player circle won");
                break;
            }
        }
    };
}
