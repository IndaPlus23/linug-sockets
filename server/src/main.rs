use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
    thread,
    fmt::{self},
    sync::{Arc, Mutex, mpsc::{self}}
};
use threadpool::ThreadPool;

use crate::tic_tac_toe::*;
mod tic_tac_toe;


struct Player {
    username: String,
    game: Option<usize>,
    challenges: Vec<String>,
    transmission_channel: mpsc::Sender<String>
}

impl fmt::Debug for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Player")
         .field("name", &self.username)
         .field("game", &self.game)
         .field("challanges", &self.challenges)
         .finish()
    }
}

const MAX_PLAYERS: usize = 10;
const IP_ADDRS: &str = "0.0.0.0:8080";

fn main() {
    let pool = ThreadPool::new(MAX_PLAYERS);
    let listener = TcpListener::bind(IP_ADDRS)
    .unwrap_or_else(|e| panic!("Error binding to port: {}", e));

    println!("Running server...");   

    let players: Arc<Mutex<Vec<Player>>> = Arc::new(Mutex::new(vec![]));
    let games: Arc<Mutex<Vec<Game>>> = Arc::new(Mutex::new(vec![]));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                dbg!("new connection");
                let players = players.clone();
                let games = games.clone();
                pool.execute(move || handle_connection(stream, players, games));
            }
            Err(e) => {
                println!("Error while accepting connection: {}", e);    
            }
        }
    }
    pool.join();

}

fn handle_connection(mut stream: TcpStream, players: Arc<Mutex<Vec<Player>>>, games: Arc<Mutex<Vec<Game>>>) {
    
    write_to_stream("Welcome to the Tic Tac Toe server\nType a username",&mut stream);
    let (mut username, _) = read_from_stream(&mut stream);
    username = username.as_str().trim().to_string();
    write_to_stream(format!("Welcome {}!", username).as_str(), &mut stream);

    let (tx, rx) = mpsc::channel::<String>();

    let mut info = Player {
        username: username.to_string(),
        game: None,
        challenges: vec![],
        transmission_channel: tx.clone()};

    { //recovering games from lost connnection
        let mut g = games.lock().unwrap();
        let len = g.len();
        for (game, i) in g.iter_mut().zip(0..len) {
            if game.win == State::None && (game.player1 == username || game.player2 == username) {
                info.game = Some(i);
                if game.player1 == username {
                    game.player1channel = tx.clone();
                }
                else if game.player2 == username {
                    game.player2channel = tx.clone();
                }
                write_to_stream("game successfully recovered", &mut stream);
                game.send_update();
            }
        }

    }

    {
        let mut p = players.lock().unwrap();
        p.push(info);
    }
    
    let mut stream_clone = stream.try_clone().unwrap();
    thread::spawn(move || {
        loop {
            let msg = rx.recv();
            match msg {
                Ok(msg) => {write_to_stream(&msg, &mut stream_clone)}
                _ => {break}} //the player has disconnected
            }
    });
    
    println!("{} has logged in", username);    
    loop {
        let (message, bytes_read) = read_from_stream(&mut stream);
        if bytes_read == 0 {
            {
                let mut p = players.lock().unwrap();
                p.retain(|x| x.username != username);
            }
            println!("{} has disconnected...", username);
            break
        }

        let players_clone = players.clone();
        let in_game = is_in_game(&username, players_clone);

        match in_game {
            Some(game) => {
                if message.starts_with("resign") {
                    let players_clone = players.clone();
                    let games_clone = games.clone();
                    resign(&username, game, players_clone, games_clone);
                }
                else if TIC_TAC_TOE_MOVES.contains(&message.as_str()) {
                    let players_clone = players.clone();
                    let games_clone = games.clone();
                    play_move(&username, &message, game, players_clone, games_clone);
                }
            },
            None => {
                if message.starts_with("online") {
                    let players_clone = players.clone();
                    who_is_online(&mut stream, players_clone);
                }
                else if message.starts_with("dm ") {
                    let players_clone = players.clone();
                    direct_message(&username, &message, &mut stream, players_clone);
                }
                else if message.starts_with("challenge ") {
                    let players_clone = players.clone();
                    challenge(&username, &message, &mut stream, players_clone);
                }
                else if message.starts_with("accept ") {
                    let players_clone = players.clone();
                    let games_clone = games.clone();
                    let (opponent, game_index) = accept(&username, &message, &mut stream, players_clone, games_clone);
                    match opponent {
                        Some(opponent) => {
                            {
                                let mut p = players.lock().unwrap();
                                for player in p.iter_mut() {
                                    if player.username == username || player.username == opponent {
                                        player.game = game_index;
                                        player.challenges.retain(|x|x != &opponent );
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                }
                else {
                    let players_clone = players.clone();
                    global_message(&username, &message, players_clone);
                }
            }
        }
    }
}

fn read_from_stream(stream: &mut TcpStream) -> (String, usize) {
    let mut buffer = [0; 1024];
    
    let bytes_read = stream.read(&mut buffer).unwrap_or_else(|e| {
        eprintln!("Error while reading stream: {}", e);
        return 0;
    });
    let message_len = match buffer.iter().position(|&x| x == b'\0') {
        Some(index) => index,
        None => bytes_read
    };
    
    (String::from_utf8_lossy(&buffer[..message_len]).to_string(), bytes_read)
}

fn write_to_stream(message: &str, stream: &mut TcpStream) {
    stream.write(message.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn global_message(username: &str, message: &str, players: Arc<Mutex<Vec<Player>>>) {
        {
            let p = players.lock().unwrap();
            for player in p.iter() {
                match player.game {
                    Some(_) => {},
                    None => {
                        if player.username != username {
                            player.transmission_channel.send(format!("{}: {}", username, message)).unwrap();
                        }
                    }           
                }
            }
        }
}

fn direct_message(username: &str, message: &str, stream: &mut TcpStream, players: Arc<Mutex<Vec<Player>>>) {
    let split_message = message.split_whitespace().collect::<Vec<&str>>();
    if split_message.len() >= 3 {
        let (player_username, dm) = (split_message[1], split_message[2..].join(" ")); 
        {
            let p = players.lock().unwrap();
            for player in p.iter() {
                if player.username == player_username {
                    player.transmission_channel.send(format!("dm from {}: {}", username, dm)).unwrap();
                    write_to_stream(format!("dm sent to {}", player_username).as_str(), stream);
                }
            }
        }
    }
    else {
        write_to_stream("use format: dm <user> <message>", stream);
    }
}

fn challenge(username: &str, message: &str, stream: &mut TcpStream, players: Arc<Mutex<Vec<Player>>>) {
    let split_message = message.split_whitespace().collect::<Vec<&str>>();
    if split_message.len() == 2 {
        let player_username = split_message[1]; 
        {
            let mut p = players.lock().unwrap();
            for player in p.iter_mut() {
                if player.username == player_username {
                    match player.game {
                        Some(_) => {
                            write_to_stream(format!("{} is in a game, try again later... \n", player_username).as_str(), stream);
                            return;
                        },
                        None => {}
                    }
                    player.challenges.push(username.to_string());
                    player.transmission_channel.send(format!("challenge from {}\nType: accept {} to play", username, username)).unwrap();
                    write_to_stream(format!("challenge sent to {}", player_username).as_str(), stream);
                    return;
                }
            }
            write_to_stream(format!("{} is not online, try again later... \n", player_username).as_str(), stream);
        }
    }
    else {
        write_to_stream("use format: challenge <user>", stream);
    }
}

fn accept(username: &str, message: &str, stream: &mut TcpStream, players: Arc<Mutex<Vec<Player>>>, games: Arc<Mutex<Vec<Game>>>) -> (Option<String>, Option<usize>) {
    
    let split_message = message.split_whitespace().collect::<Vec<&str>>();
    let (mut p1_transmission_channel,mut p2_transmission_channel): 
    (Option<mpsc::Sender<String>>,Option<mpsc::Sender<String>>) = (None, None);

    if split_message.len() == 2 {
        let opponent_username = split_message[1]; 
        {  //checking if challanges contains the opponent and if opponent is in a game
            let p = players.lock().unwrap();
            for player in p.iter() {
                if player.username == username {
                    p2_transmission_channel = Some(player.transmission_channel.clone());
                    if !player.challenges.contains(&opponent_username.to_string()) {
                        return (None, None);
                    }
                        
                }
                else if player.username == opponent_username {
                    p1_transmission_channel = Some(player.transmission_channel.clone());
                }
            }
        }
        match (&p1_transmission_channel, &p2_transmission_channel) {
            (Some(_), Some(_)) => {}
            _ => {return (None, None)} //both players are not online
        }
        {  //accepting the challange
            let mut p = players.lock().unwrap();
            for player in p.iter_mut() {
                if player.username == opponent_username {
                    player.transmission_channel.send(format!("{} has accepted your challange", username)).unwrap();
                    write_to_stream(format!("accepted challege with {}\n", opponent_username).as_str(), stream);       
                    {
                        let mut g = games.lock().unwrap();
                        let game_index = g.len();
                        let new_game = Game::new(opponent_username.to_string(), username.to_string(),
                        p1_transmission_channel.unwrap(), p2_transmission_channel.unwrap());
                        new_game.send_update();
                        g.push(new_game);
                        return (Some(opponent_username.to_string()), Some(game_index));
                    }
                    
                }
            }
            return (None, None)
        }
    }
    else {
        write_to_stream("use format: accept <user>", stream);
        return (None, None);
    }
}

fn is_in_game(username: &str, players: Arc<Mutex<Vec<Player>>>) -> Option<usize>{
    {
        let p = players.lock().unwrap();
        for player in p.iter(){
            if player.username == username {
                match player.game {
                    Some(game) => {return Some(game)},
                    None => {return None}
                }
            }
        }
    }
    None
}

fn play_move(username: &str, m: &str, game_index: usize, players: Arc<Mutex<Vec<Player>>>, games: Arc<Mutex<Vec<Game>>>) {
    let square = m.parse().unwrap();
    {
        let mut g = games.lock().unwrap();
        let game = &mut g[game_index];
        let ok = game.play_move(username, square);
        if !ok {return}
        let is_over = game.check_for_result();
        if is_over {
            {
                let mut p = players.lock().unwrap();
                for player in p.iter_mut() {
                    match player.game {
                        Some(i) => {
                            if i == game_index {
                                player.game = None;
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }
}

fn resign(username: &str, game_index: usize, players: Arc<Mutex<Vec<Player>>>, games: Arc<Mutex<Vec<Game>>>) {
        {
            let mut g = games.lock().unwrap();
            let game = &mut g[game_index];
            game.player1channel.send(format!("{} resigned the game\n", username)).unwrap();
            game.player2channel.send(format!("{} resigned the game\n", username)).unwrap();
            game.win = State::Draw; //preventing the game from being recovered
        }
        {
            let mut p = players.lock().unwrap();
            for player in p.iter_mut() {
                match player.game {
                    Some(i) => {
                        if i == game_index {
                            player.game = None;
                        }
                    }
                    None => {}
                }
            }
        }
}

fn who_is_online(stream: &mut TcpStream, players: Arc<Mutex<Vec<Player>>>,) {
    write_to_stream("Online players:\n", stream);
    {
        let p = players.lock().unwrap();
        for player in p.iter() {
            write_to_stream(format!("{}  ", &player.username).as_str(), stream);
        }
    }
}