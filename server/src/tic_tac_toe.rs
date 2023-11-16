use std::{sync::mpsc, fmt};


pub const TIC_TAC_TOE_MOVES: [&str; 9] = ["1","2","3","4","5","6","7","8","9"];

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum State {
    None,
    X,
    O,
    Draw,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_str = match *self {
            State::None => "_",
            State::X => "X",
            State::O => "O",
            State::Draw => "Draw",
        };
        write!(f, "{}", state_str)
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub board: [State; 9],
    pub player1: String,
    pub player2: String,
    pub player1channel: mpsc::Sender<String>,
    pub player2channel: mpsc::Sender<String>,
    pub turn: String,
    pub legal_moves: Vec<usize>,
    pub last_move: Option<usize>,
    pub win: State,
}

impl Game {

    pub fn new(player1: String, player2: String, channel1: mpsc::Sender<String>, channel2: mpsc::Sender<String>) -> Game {
        Game {
            board: [State::None; 9],
            player1: player1.clone(),
            player2: player2,
            player1channel: channel1,
            player2channel: channel2,
            turn: player1,
            legal_moves: vec![1,2,3,4,5,6,7,8,9],
            last_move: None,
            win: State::None,
        }     
    }

    pub fn play_move(&mut self,username: &str, mut square: usize) -> bool{
        if self.turn != username {
            let error_message = format!("\nit is not your turn");
             match &self.turn {
                player1_name if player1_name == &self.player1 => {
                    self.player2channel.send(error_message).unwrap();
                },
                player2_name if player2_name == &self.player2 => {
                    self.player1channel.send(error_message).unwrap();
                },
                _ => {}
            }
            return false;
        }
        if self.legal_moves.contains(&square) {
            self.last_move = Some(square);
            
            self.legal_moves.retain(|&x| x != square);
            square -= 1;
            match &self.turn {
                player1_name if player1_name == &self.player1 => {
                    self.board[square] = State::X;
                    self.turn = self.player2.clone().to_string();
                }
                player2_name if player2_name == &self.player2 => {
                    self.board[square] = State::O;
                    self.turn = self.player1.clone().to_string();
                }
                _ => {}
            }
            return true;
        }
        else {
             let error_message = format!("\n{:?} is not a legal move", square);
             match &self.turn {
                player1_name if player1_name == &self.player1 => {
                    self.player1channel.send(error_message).unwrap();
                },
                player2_name if player2_name == &self.player2 => {
                    self.player2channel.send(error_message).unwrap();
                },
                _ => {}
              }
              return false;

        }
    } 

    pub fn check_for_result(&mut self) -> bool {
        let players = [State::X, State::O];
        for player in players.iter() {
            if 
            self.board[1-1] == *player && self.board[2-1] == *player && self.board[3-1] == *player || 
            self.board[4-1] == *player && self.board[5-1] == *player && self.board[6-1] == *player || 
            self.board[7-1] == *player && self.board[8-1] == *player && self.board[9-1] == *player || 
            self.board[1-1] == *player && self.board[4-1] == *player && self.board[7-1] == *player || 
            self.board[2-1] == *player && self.board[5-1] == *player && self.board[8-1] == *player || 
            self.board[3-1] == *player && self.board[6-1] == *player && self.board[9-1] == *player || 
            self.board[1-1] == *player && self.board[5-1] == *player && self.board[9-1] == *player || 
            self.board[3-1] == *player && self.board[5-1] == *player && self.board[7-1] == *player {
                self.win = *player;
                self.legal_moves = vec![];
                self.player1channel.send(format!("\nX: {} O: {}\n{:?} {:?} {:?}\n{:?} {:?} {:?}\n{:?} {:?} {:?}", self.player1, self.player2 ,self.board[0], self.board[1], self.board[2], self.board[3], self.board[4], self.board[5], self.board[6], self.board[7], self.board[8])).unwrap();
                self.player2channel.send(format!("\nX: {} O: {}\n{:?} {:?} {:?}\n{:?} {:?} {:?}\n{:?} {:?} {:?}", self.player1, self.player2 ,self.board[0], self.board[1], self.board[2], self.board[3], self.board[4], self.board[5], self.board[6], self.board[7], self.board[8])).unwrap();
                self.player1channel.send(format!("{:?} Wins!\n", self.win)).unwrap();
                self.player2channel.send(format!("{:?} Wins!\n", self.win)).unwrap();
                return true;
            }
            
        }
        if self.legal_moves.is_empty() && self.win == State::None {
            self.win = State::Draw;
            self.player1channel.send(format!("\nX: {} O: {}\n{:?} {:?} {:?}\n{:?} {:?} {:?}\n{:?} {:?} {:?}", self.player1, self.player2 ,self.board[0], self.board[1], self.board[2], self.board[3], self.board[4], self.board[5], self.board[6], self.board[7], self.board[8])).unwrap();
            self.player2channel.send(format!("\nX: {} O: {}\n{:?} {:?} {:?}\n{:?} {:?} {:?}\n{:?} {:?} {:?}", self.player1, self.player2 ,self.board[0], self.board[1], self.board[2], self.board[3], self.board[4], self.board[5], self.board[6], self.board[7], self.board[8])).unwrap();
            self.player1channel.send(format!("Draw!\n")).unwrap();
            self.player2channel.send(format!("Draw!\n")).unwrap();
            return true;
        }
        self.send_update();
        return false;
    }

    pub fn send_update(&self) {
        let main = format!("\nX: {} O: {}\n{:?} {:?} {:?}\n{:?} {:?} {:?}\n{:?} {:?} {:?}", self.player1, self.player2 ,self.board[0], self.board[1], self.board[2], self.board[3], self.board[4], self.board[5], self.board[6], self.board[7], self.board[8]);
        let own_turn: &str = "\nYour turn: type 1-9 to play a move";
        let other_turn: &str = "\nWaiting for opponent...";
        let own = main.clone()+own_turn;
        let other = main+other_turn;
    
        match &self.turn {
            player1_name if player1_name == &self.player1 => {
                self.player1channel.send(own).unwrap();
                self.player2channel.send(other).unwrap();
            },
            player2_name if player2_name == &self.player2 => {
                self.player1channel.send(other).unwrap();
                self.player2channel.send(own).unwrap();
            },
            _ => {}
        }
    }
    }