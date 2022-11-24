use counter::Counter;
use itertools::Itertools;
use rayon::prelude::*;

pub fn simulate_hands(reps: usize) -> Counter<Value, usize> {
    const HANDS_PER_SHUFFLE: usize = 52 / 5;

    let outer_reps = reps / HANDS_PER_SHUFFLE;

    let counts: Counter<Value, usize> = (0..outer_reps)
        .into_par_iter()
        .fold(
            || Counter::<Value, usize>::new(),
            |mut counter: Counter<Value, usize>, _| {
                let mut deck = Deck::standard();
                deck.shuffle();
                let cards = &(deck.cards[..]);
                cards
                    .chunks_exact(5)
                    .for_each(|hand| counter[&get_hand_value(&hand)] += 1);
                counter
            },
        )
        .reduce(
            || Counter::<Value, usize>::new(),
            |mut a, b| {
                a.extend(&b);
                a
            },
        );
    counts
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum Value {
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

#[derive(Debug, Clone)]
pub struct Card {
    rank: u32,
    suit: u8,
}

// type Card = (u8, u32);
type Hand = [Card];

#[derive(Debug, Clone)]
pub struct Deck {
    pub cards: Vec<Card>,
}

// An ace has to be treated specially, as it can show up
// as part of a royal straight or the lowest straight
const ACE: u32 = u32::pow(2, 12);

impl Deck {
    pub fn standard() -> Self {
        let cards: Vec<Card> = (0..4)
            .cartesian_product(0..13)
            .map(|(suit_index, rank_index)| Card {
                suit: u8::pow(2, suit_index),
                rank: u32::pow(2, rank_index),
            })
            .collect();
        Deck { cards }
    }
    pub fn shuffle(&mut self) {
        fastrand::shuffle(&mut self.cards);
    }
}

/// Bitwise-or the card suits together.  If the suits collapse down to a single
/// bit, then the hand is a flush.
fn is_flush(hand: &Hand) -> bool {
    assert!(hand.len() == 5);

    let suits = hand[0].suit | hand[1].suit | hand[2].suit | hand[3].suit | hand[4].suit;
    suits.count_ones() == 1
}

fn check_straight(hand: &Hand) -> Value {
    assert!(hand.len() == 5);
    let mut ranks = [
        hand[0].rank,
        hand[1].rank,
        hand[2].rank,
        hand[3].rank,
        hand[4].rank,
    ];
    ranks.sort();

    if (ranks[0] * 16) != ranks[4] {
        if 8 == ranks[3] && ranks[4] == ACE {
            // Ace, 2, 3, 4, 5
            Value::Straight
        } else {
            Value::HighCard
        }
    } else if ranks[4] == u32::pow(2, 12) {
        Value::RoyalStraight
    } else {
        Value::Straight
    }
}

fn max_same_kind(hand: &Hand) -> usize {
    let counts = hand.iter().map(|card| card.rank).collect::<Counter<_>>();
    counts.k_most_common_ordered(1)[0].1
}

pub fn get_hand_value(hand: &Hand) -> Value {
    let bit_ranks = hand.iter().fold(0, |bits, card| bits | card.rank);
    let value = match bit_ranks.count_ones() {
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
