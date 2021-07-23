use std::io::Read;
use rand::thread_rng;
use rand::seq::SliceRandom;
use itertools::iproduct;
use itertools::enumerate;
use std::char;
use radix_fmt::radix_36;

const HELP_TEXT: &str = "H: print this again
Q: quit
U/z: undo
R/y: redo
r: restart (max undo)

S/s: push from stack
Cn: complete nth pile suit
Mxyz: move card of xth pile at index y to zth pile
c: smart complete all finished suits
mxz: smart move from xth pile to zth pile
";

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
enum Face {King, Queen, Jack, Ten, Nine, Eight, Seven, Six, Five, Four, Three, Two, Ace}
impl Face {
    const FACES: [Self; 13] = [
        Face::King,
        Face::Queen,
        Face::Jack,
        Face::Ten,
        Face::Nine,
        Face::Eight,
        Face::Seven,
        Face::Six,
        Face::Five,
        Face::Four,
        Face::Three,
        Face::Two,
        Face::Ace,
    ];

    fn succ(&self) -> Option<Self> {  // TODO: implement as a Face^2->Bool predicate?
        match self {
            Face::King  => Some(Face::Queen),
            Face::Queen => Some(Face::Jack),
            Face::Jack  => Some(Face::Ten),
            Face::Ten   => Some(Face::Nine),
            Face::Nine  => Some(Face::Eight),
            Face::Eight => Some(Face::Seven),
            Face::Seven => Some(Face::Six),
            Face::Six   => Some(Face::Five),
            Face::Five  => Some(Face::Four),
            Face::Four  => Some(Face::Three),
            Face::Three => Some(Face::Two),
            Face::Two   => Some(Face::Ace),
            Face::Ace   => None,
        }
    }

    fn to_char(&self) -> char {
        match self {
            Face::King  => 'K',
            Face::Queen => 'Q',
            Face::Jack  => 'J',
            Face::Ten   => 'T',
            Face::Nine  => '9',
            Face::Eight => '8',
            Face::Seven => '7',
            Face::Six   => '6',
            Face::Five  => '5',
            Face::Four  => '4',
            Face::Three => '3',
            Face::Two   => '2',
            Face::Ace   => 'A',
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Card {
    face: Face,
    suit: Suit,
}
impl Card {
    fn to_string(&self, hidden: bool) -> String {
        if hidden {
            String::from("? ?")
        } else {
            format!("{} {}", self.suit.to_char(), self.face.to_char())
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
                } | match self.face {
                    Face::King  => 0x0E,
                    Face::Queen => 0x0D,
                    Face::Jack  => 0x0B,
                    Face::Ten   => 0x0A,
                    Face::Nine  => 0x09,
                    Face::Eight => 0x08,
                    Face::Seven => 0x07,
                    Face::Six   => 0x06,
                    Face::Five  => 0x05,
                    Face::Four  => 0x04,
                    Face::Three => 0x03,
                    Face::Two   => 0x02,
                    Face::Ace   => 0x01,
                }
            }
        ).expect("Grigri has refused to make an informative error message, but something is bad :(")
    }
}

struct GameState {
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
        if self.hidden[pile] == self.piles[pile].len() && self.hidden[pile] > 0 {
            self.hidden[pile] -= 1;
            true
        } else {
            false
        }
    }
    fn is_sequence(&self, pile: usize, index: usize) -> bool {
        if self.hidden[pile] > index {return false}  // block sequence for hidden cards

        let pile = &self.piles[pile];
        if index + 1 == pile.len() {return true}

        let slice1 = &pile[index..];
        let slice2 = &pile[(index+1)..];
     
        for (pred, succ) in slice1.iter().zip(slice2) {
            if pred.face.succ().is_none() {
                return false
            }
            if pred.face.succ().unwrap() != succ.face {
                return false
            }
            if pred.suit != succ.suit {
                return false
            }
        }
        return true
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
    let mut cards: Vec<Card> = iproduct!(suits, Face::FACES)
        .map(|(suit, face)| Card{suit, face})
        .collect();
    // TODO: make 3 decks as const arrays and just choose one of them?

    // shuffle deck
    let mut rng = thread_rng();
    cards.shuffle(&mut rng);
    cards
}

fn init_game(deck: Vec<Card>) -> GameState {
    GameState {
        stack: (&deck[..50]).to_vec(),
        piles: [
            //{let mut v: Vec<Card> = Vec::with_capacity(25); v.extend_from_slice(&deck[50..55]); v},
            // TODO: set the capacities? Like line above but do it nicely
            (&deck[50..55]).to_vec(),
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
            println!("{}", HELP_TEXT);
        },
        Move {source, index, target} => {
            if !game.is_sequence(source, index) {
                println!("Not in sequence"); return;
            }  // lower cards in a row
            if game.piles[target].len() > 0 {
                match game.piles[target].last().unwrap().face.succ() {
                    None => {println!("can't move onto ace"); return;},
                    Some(f) if f != game.piles[source][index].face => {
                        println!("source not succ of target"); return;
                    },
                    Some(_) => {}, // source matches target
                }
            }
            let target_i = game.piles[target].len();
            let cards = &mut game.piles[source].drain(index..).collect();
            game.piles[target].append(cards);
            let discover = game.discover(source);
            game.write(Action::Move{source, source_i: index, target, target_i, discover});
        },
        CompleteSuit{pos} => {
            let i = game.piles[pos].len() - 13;
            if !game.is_sequence(pos, i) {
                println!("Not in sequence"); return;
            }
            let suit = game.piles[pos].last().unwrap().suit;
            game.piles[pos].truncate(i);
            game.completed += 1;
            if game.completed == 8 {println!("You win!")};
            let discover = game.discover(pos);
            game.write(Action::CompleteSuit{pos, suit, discover});
        },
        Stack => {
            if game.stack.len() == 0 {
                println!("Stack exhausted"); return;
            }
            for pile in &mut game.piles {
                pile.push(game.stack.pop().unwrap());
            }
            game.write(Action::Stack);
        },
        SmartComp => {
            for pos in 0..10 {  // do with for pile in piles?
                if game.piles[pos].len() < 13 {continue;}
                if game.piles[pos].last().unwrap().face != Face::Ace {continue;}
                let i = game.piles[pos].len() - 13;
                if !game.is_sequence(pos, i) {continue;}
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
                println!("No cards to move"); return;
            }
            if game.piles[target].is_empty() {
                let len = game.piles[source].len();
                if len == 1 || !game.is_sequence(source, len-2) {
                    // no sequence of length 2 => only one possible move
                    let card = game.piles[source].pop().unwrap();
                    let discover = game.discover(source);
                    game.piles[target].push(card);
                    game.write(Action::Move{
                        source,
                        source_i: len-2,
                        target,
                        target_i: 0,
                        discover,
                    });
                } else {println!("Multiple moves possible; use Mxyz notation")}
                return;
            }
            let face = match game.piles[target].last().unwrap().face.succ() {
                Some(f) => f,
                None    => {println!("can't move onto ace"); return;},
            };
            let top = game.piles[source].len();  // TODO: avoid declaring this?
            for source_i in (0..top).rev() {
                if game.piles[source][source_i].face == face {
                    if !game.is_sequence(source, source_i) {
                        println!("No valid move found"); return;
                        // TODO: add prints to all returns in this function
                    }
                    let target_i = game.piles[target].len();
                    let cards = &mut game.piles[source].drain(source_i..).collect();
                    game.piles[target].append(cards);
                    let discover = game.discover(source);
                    game.write(Action::Move{source, source_i, target, target_i, discover});
                    return;
                }
            }
            println!("No valid move found");
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
            for face in Face::FACES {
                game.piles[pos].push(Card{suit, face});
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

    match first_byte {
        10   => return Err(""),  // TODO: something's very wrong here  @Berg
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
        print!("{}   ", radix_36(i));
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

