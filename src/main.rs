use std::io::Read;
use rand::thread_rng;
use rand::seq::SliceRandom;
use itertools::iproduct;
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
            if hidden {0x1F0A0} else {
                0x1F000 | match self.suit {
                    Suit::Spades   => 0xA0,
                    Suit::Hearts   => 0xB0,
                    Suit::Diamonds => 0xC0,
                    Suit::Clubs    => 0xD0,
                } | match self.value {
                    Value::King  => 0xE,
                    Value::Queen => 0xD,
                    Value::Jack  => 0xB,
                    Value::Ten   => 0xA,
                    Value::Nine  => 0x9,
                    Value::Eight => 0x8,
                    Value::Seven => 0x7,
                    Value::Six   => 0x6,
                    Value::Five  => 0x5,
                    Value::Four  => 0x4,
                    Value::Three => 0x3,
                    Value::Two   => 0x2,
                    Value::Ace   => 0x1,
                }
            }
        ).expect("something went very wrong")
    }
}

struct GameState {  // TODO: change value to face
    stack: Vec<Card>,
    piles: [Vec<Card>; 10],
    hidden: [usize; 10],
    completed: u8,
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
        hidden: [0; 10],
        completed: 0,
    }
}

fn is_sequence(pile: &Vec<Card>, index: usize) -> bool {
    if index + 1 == pile.len() {return true}

    let slice1 = &pile[index..];
    let slice2 = {let i = index+1; &pile[i..]};
 
    for (pred, succ) in slice1.iter().zip(slice2) {
        if pred.value.succ().is_none() {return false}
        if pred.value.succ().unwrap() != succ.value {return false}
        if pred.suit != succ.suit {return false}
    }
    true
}

fn game_step(game: &mut GameState, input: Input) {
    use Input::*;
    match input {
        NewGame => {println!("New Game...\nUndoing...");},
        Quit => {println!("Bye!"); panic!("bazinga")},  // TODO
        Undo => {println!("Undoing...\nRedoing...");},
        Redo => {println!("Redoing...\nUndoing...");},
        Help => {
            println!("H: print this again\nN: new game\nQ: quit\nS: push from stack\nU: undo\nR: redo\nCn: complete nth pile suit\nMxyz: move card of xth pile at index y to zth pile");
        },
        Move {source, index, target} => {
            if !is_sequence(&game.piles[source], index) {return}  // lower cards in a row
            if game.piles[target].len() > 0 {
                if game.piles[target].last().unwrap().value.succ().unwrap()
                    != game.piles[source][index].value {return}  // source matches target
            }
            let temp = &mut game.piles[source].drain(index..).collect();
            game.piles[target].append(temp);
        },
        CompleteSuit{pos} => {
            let i = game.piles[pos].len() - 13;
            if !is_sequence(&game.piles[pos], i) {return}
            game.piles[pos].truncate(i);  // TODO: test
            game.completed += 1;
            if game.completed == 10 {println!("You win!")};
        },
        Stack => {
            if game.stack.len() == 0 {println!("Stack exhausted"); return}
            for pile in &mut game.piles {
                pile.push(game.stack.pop().unwrap());
            }
        },
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
        print!("{:x} ", i);
        for pile in &game.piles {
            match pile.get(i) {
                None       => print!("   "),
                Some(card) => print!("{}  ", card.to_char(false)),
            }
        }
        println!(" ");
    }
    println!("   0  1  2  3  4  5  6  7  8  9");

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
