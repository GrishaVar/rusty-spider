use std::io::Read;
use rand::thread_rng;
use rand::seq::SliceRandom;
use itertools::iproduct;
use itertools::enumerate;
use std::char;

enum Input {
    NewGame,
    Quit,
    Move{source:usize, index:usize, target:usize},
    Stack,
    CompleteSuit{pos:usize},
    Undo,
    Redo,
    Help,
    SmartMove{source:usize, target:usize},  // guesses index
    SmartComp,  // finds a stack to complete
    Restart,
}

#[derive(Clone)]
enum Action {
    Move{source:usize, source_i:usize, target:usize, target_i:usize, discover:bool},
    Stack,
    CompleteSuit{pos:usize, suit:Suit, discover:bool},
}

#[derive(Clone, Copy, PartialEq)]
enum Suit {Hearts, Spades, Diamonds, Clubs}
impl Suit {
    fn to_char(&self) -> char {
        match self {
            Suit::Hearts   => '♡',
            Suit::Spades   => '♠',
            Suit::Diamonds => '♢',
            Suit::Clubs    => '♣',
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Value {King, Queen, Jack, Ten, Nine, Eight, Seven, Six, Five, Four, Three, Two, Ace}
impl Value {
    const VALUES: [Self; 13] = [
        Value::King,
        Value::Queen,
        Value::Jack,
        Value::Ten,
        Value::Nine,
        Value::Eight,
        Value::Seven,
        Value::Six,
        Value::Five,
        Value::Four,
        Value::Three,
        Value::Two,
        Value::Ace,
    ];

    fn succ(&self) -> Option<Self> {  // TODO: implement as a Value^2->Bool predicate?
        match self {
            Value::King  => Some(Value::Queen),
            Value::Queen => Some(Value::Jack),
            Value::Jack  => Some(Value::Ten),
            Value::Ten   => Some(Value::Nine),
            Value::Nine  => Some(Value::Eight),
            Value::Eight => Some(Value::Seven),
            Value::Seven => Some(Value::Six),
            Value::Six   => Some(Value::Five),
            Value::Five  => Some(Value::Four),
            Value::Four  => Some(Value::Three),
            Value::Three => Some(Value::Two),
            Value::Two   => Some(Value::Ace),
            Value::Ace   => None,
        }
    }

    fn to_char(&self) -> char {
        match self {
            Value::King  => 'K',
            Value::Queen => 'Q',
            Value::Jack  => 'J',
            Value::Ten   => 'T',
            Value::Nine  => '9',
            Value::Eight => '8',
            Value::Seven => '7',
            Value::Six   => '6',
            Value::Five  => '5',
            Value::Four  => '4',
            Value::Three => '3',
            Value::Two   => '2',
            Value::Ace   => 'A',
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Card {
    value: Value,  // TODO use char or u8?
    suit: Suit,
}
impl Card {
    fn to_string(&self, hidden: bool) -> String {
        if hidden {
            String::from("? ?")
        } else {
            format!("{} {}", self.suit.to_char(), self.value.to_char())
        }
    }
    fn to_char(&self, hidden: bool) -> char {
        char::from_u32(
            if hidden {
                0x01F0A0
            } else {
                0x01F000 | match self.suit {
                    Suit::Spades   => 0xA0,
                    Suit::Hearts   => 0xB0,
                    Suit::Diamonds => 0xC0,
                    Suit::Clubs    => 0xD0,
                } | match self.value {
                    Value::King  => 0x0E,
                    Value::Queen => 0x0D,
                    Value::Jack  => 0x0B,
                    Value::Ten   => 0x0A,
                    Value::Nine  => 0x09,
                    Value::Eight => 0x08,
                    Value::Seven => 0x07,
                    Value::Six   => 0x06,
                    Value::Five  => 0x05,
                    Value::Four  => 0x04,
                    Value::Three => 0x03,
                    Value::Two   => 0x02,
                    Value::Ace   => 0x01,
                }
            }
        ).expect("Grigri has refused to make an informative error message, but something is bad :(")
    }
}

struct GameState {  // TODO: change value to face
    stack: Vec<Card>,
    piles: [Vec<Card>; 10],
    hidden: [usize; 10],
    completed: u8,
    history: Vec<Action>,
    history_head: usize,
}
impl GameState {
    fn write(&mut self, action: Action) {
        self.history.truncate(self.history_head);
        self.history.push(action);
        self.history_head += 1;
    }

    fn discover(&mut self, pile: usize) -> bool {
        println!("{} {} {}", pile, self.hidden[pile], self.piles[pile].len());
        if self.hidden[pile] == self.piles[pile].len() && self.hidden[pile] > 0 {
            self.hidden[pile] -= 1;
            return true
        } else {
            return false
        }
    }
}

fn generate_deck(suits: u8) -> Vec<Card> {
    use Suit::*;

    let suits: [Suit; 8] = match suits {
        1 => [Spades; 8],
        2 => [Spades, Spades, Spades, Spades, Hearts, Hearts, Hearts, Hearts],  // TODO: concatinate?
        4 => [Hearts, Spades, Diamonds, Clubs, Hearts, Spades, Diamonds, Clubs],
        _ => panic!("Invalid number of suits"),
    };

    // cartesian product - makes a card for every suit/face combination
    let mut cards: Vec<Card> = iproduct!(suits, Value::VALUES)
        .map(|(suit, value)| Card{suit, value})
        .collect();

    // shuffle deck
    let mut rng = thread_rng();
    cards.shuffle(&mut rng);
    cards
}

fn init_game(deck: Vec<Card>) -> GameState {
    GameState {
        stack: (&deck[..50]).to_vec(),
        piles: [
            {let mut v: Vec<Card> = Vec::with_capacity(25); v.extend_from_slice(&deck[50..55]); v},
            //(Vec::with_capacity(25)).extend_from_slice(&deck[50..55]),
            (&deck[55..60]).to_vec(),
            (&deck[60..65]).to_vec(),
            (&deck[65..70]).to_vec(),
            (&deck[70..75]).to_vec(),
            (&deck[75..80]).to_vec(),
            (&deck[80..86]).to_vec(),
            (&deck[86..92]).to_vec(),
            (&deck[92..98]).to_vec(),
            (&deck[98..]).to_vec(),
        ],
        hidden: [4, 4, 4, 4, 4, 4, 5, 5, 5, 5],
        completed: 0,
        history: Vec::with_capacity(50),
        history_head: 0,
    }
}

fn is_sequence(pile: &Vec<Card>, index: usize) -> bool {
    if index + 1 == pile.len() {return true}

    let slice1 = &pile[index..];
    let slice2 = {let i = index+1; &pile[i..]};
 
    for (pred, succ) in slice1.iter().zip(slice2) {
        if pred.value.succ().is_none() {
            return false
        }
        if pred.value.succ().unwrap() != succ.value {
            return false
        }
        if pred.suit != succ.suit {
            return false
        }
    }
    return true
}

fn game_step(game: &mut GameState, input: Input) {
    use Input::*;
    match input {
        NewGame => {println!("New Game...\nUndoing...")},
        Quit => {println!("Bye!"); panic!("Success")},  // TODO exis properly
        Undo => {
            if game.history_head == 0 {
                println!("Nothing to undo");
            } else {
                game.history_head -= 1;
                undo_action(game, game.history[game.history_head].clone());
            }
        },
        Redo => {
            if game.history_head == game.history.len() {
                println!("Nothing to redo");
            } else {
                redo_action(game, game.history[game.history_head].clone());
                game.history_head += 1;
            }
        },
        Restart => {
            while game.history_head > 0 {
                game.history_head -= 1;
                undo_action(game, game.history[game.history_head].clone());
            }
        },
        Help => {
            println!("H: print this again\nN: new game\nQ: quit\nS: push from stack\nU: undo\nR: redo\nCn: complete nth pile suit\nMxyz: move card of xth pile at index y to zth pile");
        },
        Move {source, index, target} => {
            if !is_sequence(&game.piles[source], index) {return}  // lower cards in a row
            if game.piles[target].len() > 0 {
                if game.piles[target].last().unwrap().value.succ().unwrap()
                    != game.piles[source][index].value {return}  // source matches target
            }
            let target_i = game.piles[target].len();
            let cards = &mut game.piles[source].drain(index..).collect();
            game.piles[target].append(cards);
            let discover = game.discover(source);
            game.write(Action::Move{source, source_i: index, target, target_i, discover});
        },
        CompleteSuit{pos} => {
            let i = game.piles[pos].len() - 13;
            if !is_sequence(&game.piles[pos], i) {return}
            let suit = game.piles[pos].last().unwrap().suit;
            game.piles[pos].truncate(i);
            game.completed += 1;
            if game.completed == 8 {println!("You win!")};
            let discover = game.discover(pos);
            game.write(Action::CompleteSuit{pos, suit, discover});
        },
        Stack => {
            if game.stack.len() == 0 {
                println!("Stack exhausted");
                return;
            }
            for pile in &mut game.piles {
                pile.push(game.stack.pop().unwrap());
            }
            game.write(Action::Stack);
        },
        SmartComp => {
            for pos in 0..8 {  // do with for pile in piles?
                if game.piles[pos].len() < 13 {continue;}
                if game.piles[pos].last().unwrap().value != Value::Ace {continue;}
                let i = game.piles[pos].len() - 13;
                if !is_sequence(&game.piles[pos], i) {continue;}
                let suit = game.piles[pos].last().unwrap().suit;
                game.piles[pos].truncate(i);
                game.completed += 1;
                if game.completed == 8 {println!("You win!")};
                let discover = game.discover(pos);
                game.write(Action::CompleteSuit{pos, suit, discover});
            }
        },
        SmartMove{source, target} => {
            if game.piles[source].is_empty() {
                println!("No cards to move");
                return;
            }
            if game.piles[target].is_empty() {
                if game.piles[source].len() == 1 {
                    game.piles[target].push(game.piles[source][0]);
                    game.write(Action::Move{
                        source,
                        source_i: 0,
                        target,
                        target_i: 0,
                        discover: false
                    });
                } else {println!("Multiple moves possible; use Mxyz notation")}
                // TODO: allow move if only one card in sequence
                return;
            }
            let value = match game.piles[target].last().unwrap().value.succ() {
                Some(v) => v,
                None    => {println!("can't move onto ace"); return;},
            };
            let top = game.piles[source].len();  // TODO: avoid declaring this?
            for source_i in (0..top).rev() {
                if game.piles[source][source_i].value == value {
                    if !is_sequence(&game.piles[source], source_i) {
                        println!("No valid move found"); return;
                        // TODO: add prints to all returns in this function
                    }
                    let target_i = game.piles[target].len();
                    let cards = &mut game.piles[source].drain(source_i..).collect();
                    game.piles[target].append(cards);
                    let discover = game.discover(source);
                    game.write(Action::Move{source, source_i, target, target_i, discover});
                    return;  // TODO: don't allow movement of hidden cards
                }
            }
        },
    }
}

fn undo_action(game: &mut GameState, action: Action) {
    use Action::*;
    match action {
        Stack => {
            for pile in &mut game.piles {  // TODO: reverse this in a nice way
                game.stack.push(pile.pop().unwrap());
            }
        }
        CompleteSuit {pos, suit, discover} => {
            for value in Value::VALUES {
                game.piles[pos].push(Card{suit, value});
            }
            if discover {game.hidden[pos] += 1;}
        }
        Move {source, source_i:_, target, target_i, discover} => {
            let cards = &mut game.piles[target].drain(target_i..).collect();
            game.piles[source].append(cards);
            if discover {game.hidden[source] += 1;}
        }
    }
}

fn redo_action(game: &mut GameState, action: Action) {
    // TODO: these are all basically the same as in game_step
    // abstract into a do_action function?
    use Action::*;
    match action {
        Stack => {
            for pile in &mut game.piles {
                pile.push(game.stack.pop().unwrap());
            }
        }
        CompleteSuit {pos, suit:_, discover} => {
            let i = game.piles[pos].len() - 13;
            game.piles[pos].truncate(i);
            if discover {game.hidden[pos] -= 1;}
        }
        Move {source, source_i, target, target_i:_, discover} => {
            let cards = &mut game.piles[source].drain(source_i..).collect();
            game.piles[target].append(cards);
            if discover {game.hidden[source] -= 1;}
        }
    }
}

fn parse_text_input() -> Result<Input, &'static str> {
    use Input::*;

    let mut bytes = std::io::stdin().bytes();
    let first_byte = match bytes.next() {
        Some(Ok(byte)) => byte,
        Some(Err(_))   => return Err("Failed to read"),
        None           => return Err("Empty input; try H for help"),
    };

    //let (arg1, arg2, arg3) = (bytes.next(), bytes.next(), bytes.next());
    //while bytes.next().is_some() {}  // nice

    match first_byte {
        10   => return Err(""),  // TODO: something's very wrong here
        b'H' => Ok(Help),
        b'N' => Ok(NewGame),
        b'Q' => Ok(Quit),
        b'S' => Ok(Stack),
        b'U' => Ok(Undo),
        b'R' => Ok(Redo),
        b'C' => match bytes.next() {
            Some(Ok(n)) if (b'0'<=n && n<=b'9') => Ok(CompleteSuit{pos: (n-b'0') as usize}),
            Some(Ok(_))    => return Err("second char should be a digit!"),
            Some(Err(_))   => return Err("Failed to read"),
            None           => return Err("Column not provided"),
        },
        b'M' => {  // TODO: fix this nonsense
            let s: usize = match bytes.next() {
                Some(Ok(n)) if (b'0'<=n && n<=b'9') => (n-b'0') as usize,
                Some(Ok(_))    => return Err("2nd char should be a digit!"),
                Some(Err(_))   => return Err("Failed to read"),
                None           => return Err("Data not provided"),
            };
            let i: usize = match bytes.next() {  // TODO: more digits!
                Some(Ok(n)) if (b'0'<=n && n<=b'9') => (n-b'0') as usize,
                Some(Ok(n)) if (b'a'<=n && n<=b'z') => (n-b'a'+10) as usize,
                Some(Ok(_))    => return Err("3rd char is a base (base 36; lowercase)"),
                Some(Err(_))   => return Err("Failed to read"),
                None           => return Err("Data not provided"),
            };
            let t: usize = match bytes.next() {
                Some(Ok(n)) if (b'0'<=n && n<=b'9') => (n-b'0') as usize,
                Some(Ok(_))    => return Err("4th char should be a digit!"),
                Some(Err(_))   => return Err("Failed to read"),
                None           => return Err("Data not provided"),
            };
            Ok(Move{source: s, target: t, index: i})
        },
        // lower chars for big brain
        b'z' => Ok(Undo),
        b'y' => Ok(Redo),
        b'r' => Ok(Restart),
        b's' => Ok(Stack),
        b'c' => Ok(SmartComp),
        b'm' => {  // TODO: fix this nonsense
            let source: usize = match bytes.next() {
                Some(Ok(n)) if (b'0'<=n && n<=b'9') => (n-b'0') as usize,
                Some(Ok(_))    => return Err("2nd char should be a digit!"),
                Some(Err(_))   => return Err("Failed to read"),
                None           => return Err("Data not provided"),
            };
            let target: usize = match bytes.next() {
                Some(Ok(n)) if (b'0'<=n && n<=b'9') => (n-b'0') as usize,
                Some(Ok(_))    => return Err("3rd char should be a digit!"),
                Some(Err(_))   => return Err("Failed to read"),
                None           => return Err("Data not provided"),
            };
            Ok(SmartMove{source, target})
        },
        _    => return Err("Invalid char; try H for help"),
    }
}

fn print_game(game: &GameState) {
    let max: usize = {
        let mut max = 0;
        for pile in &game.piles{if max < pile.len() {max = pile.len()}};
        max  // TODO: there is def. a better way.
    };

    for i in (0..max).rev() {
        print!("{:x}   ", i);
        for (j, pile) in enumerate(&game.piles) {
            match pile.get(i) {
                None       => print!("   "),
                Some(card) => print!(
                    "{}  ",
                    card.to_char(game.hidden[j] > i),
                ),
            }
        }
        println!(" ");
    }
    println!("     0  1  2  3  4  5  6  7  8  9");

}

fn main() {
    println!("Select number of suits (1/2/4): ");

    let suits: u8 = std::io::stdin()
        .bytes()
        .next()
        .expect("Empty input")
        .expect("Failed to read")   // TODO: think about this. Result inside Option
        - b'0';  // convert ascii number to number number

    let deck: Vec<Card> = generate_deck(suits);
    let mut game: GameState = init_game(deck);

    println!("");
    loop {
        print_game(&game);
        let input = loop {
            match parse_text_input() {
                Ok(res) => break res,
                Err(er) => println!("{}", er)
            }
        };
        game_step(&mut game, input);
    }
}

