use counter::Counter;
use itertools::Itertools;
use spinners::{Spinner, Spinners};
use std::time::Instant;
use thousands::Separable;

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
enum Value {
    FiveOfAKind,
    RoyalFlush,
    StraightFlush,
    FourOfAKind,
    FullHouse,
    Flush,
    RoyalStraight,
    Straight,
    ThreeOfAKind,
    TwoPair,
    OnePair,
    HighCard,
}

type Card = (u8, u32);
type Hand = [Card];

#[derive(Debug, Clone)]
struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn standard() -> Self {
        let cards: Vec<Card> = (0..4)
            .cartesian_product(0..13)
            .map(|(suit, face)| (u8::pow(2, suit), u32::pow(2, face)))
            .collect();
        Deck { cards }
    }
    pub fn shuffle(&mut self) {
        fastrand::shuffle(&mut self.cards);
    }
}

fn is_flush(hand: &Hand) -> bool {
    assert!(hand.len() == 5);
    let suits = hand[0].0 | hand[1].0 | hand[2].0 | hand[3].0 | hand[4].0;
    suits == 1 || suits == 2 || suits == 4 || suits == 8
}

fn check_straight(hand: &Hand) -> Value {
    assert!(hand.len() == 5);
    let mut faces = [hand[0].1, hand[1].1, hand[2].1, hand[3].1, hand[4].1];

    faces.sort();
    if faces[4] != faces[0] * 16 {
        Value::HighCard
    } else if faces[4] == u32::pow(2, 12) {
        Value::RoyalStraight
    } else {
        Value::Straight
    }
}

fn max_same_kind(hand: &Hand) -> usize {
    let counts = hand.iter().map(|card| card.1).collect::<Counter<_>>();
    counts.k_most_common_ordered(1)[0].1
}

fn get_hand_value(hand: &Hand) -> Value {
    let bit_faces = hand.iter().fold(0, |bits, card| bits | card.1);
    let value = match bit_faces.count_ones() {
        1 => Value::FiveOfAKind,
        2 => {
            if max_same_kind(&hand) == 4 {
                Value::FourOfAKind
            } else {
                Value::FullHouse
            }
        }
        3 => {
            if max_same_kind(&hand) == 3 {
                Value::ThreeOfAKind
            } else {
                Value::TwoPair
            }
        }
        4 => Value::OnePair,
        5 => check_straight(&hand),
        _ => unreachable!(),
    };

    match (value, is_flush(&hand)) {
        (Value::RoyalStraight, true) => Value::RoyalFlush,
        (Value::Straight, true) => Value::StraightFlush,
        (Value::FourOfAKind, false) => Value::FourOfAKind,
        (Value::FullHouse, false) => Value::FullHouse,
        (_, true) => Value::Flush,
        (Value::RoyalStraight, false) => Value::Straight,
        (Value::Straight, false) => Value::Straight,
        (Value::ThreeOfAKind, _) => Value::ThreeOfAKind,
        (Value::TwoPair, _) => Value::TwoPair,
        (Value::OnePair, _) => Value::OnePair,
        (Value::HighCard, _) => Value::HighCard,
        _ => unreachable!(),
    }
}

fn main() {
    const REPS: usize = 100_000_000;
    const HANDS_PER_SHUFFLE: usize = 52 / 5;

    println!("");
    let mut spinner = Spinner::new(
        Spinners::Dots12,
        format!(
            "Simulating {} standard poker hands",
            REPS.separate_with_commas()
        )
        .into(),
    );
    let start = Instant::now();

    let counts = (0..(REPS / HANDS_PER_SHUFFLE))
        .flat_map(|_| {
            let mut deck = Deck::standard();
            deck.shuffle();
            let cards = &(deck.cards[..]);
            cards
                .chunks_exact(5)
                .map(|hand| get_hand_value(&hand))
                .collect::<Vec<_>>()
        })
        .collect::<Counter<Value, usize>>();

    let elapsed = Instant::now() - start;
    spinner.stop_with_message(format!(
        "Finished simulating {} standard poker hands",
        REPS.separate_with_commas()
    ));

    println!("Elapsed time: {:?}\n", elapsed);

    let mut stats = counts.most_common();
    stats.reverse();
    for (k, v) in stats {
        println!(
            "{:14}: {:10.6}%",
            format!("{k:?}"),
            100.0 * v as f32 / REPS as f32
        );
    }
}
