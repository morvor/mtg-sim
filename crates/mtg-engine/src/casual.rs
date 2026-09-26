//! Casual variants (CR 900–905): each variant's default setup, the checks of a game's
//! setup that its variant adds, and the construction rules of the supplementary decks the
//! variants use — planar decks (CR 901.3, 901.15a) and scheme decks (CR 904.3, 904.13d).
//!
//! The variants' game rules live with the cards and actions they concern: `planechase.rs`
//! (CR 901), `life_totals.rs` and `start.rs` (vanguard modifiers, CR 902; starting life,
//! CR 903.7, 904.5), `commander_rules.rs` and `deck.rs` (CR 903), `variants.rs` (schemes,
//! CR 904), `draft.rs` (drafts, CR 903.13, 905.1–905.2) and `start.rs` (conspiracies,
//! CR 905.4).

use crate::card::CardDef;
use crate::deck::DeckProblem;
use crate::game::{GameConfig, Variant};
use crate::multiplayer::setup::SetupError;
use crate::types::*;
use std::collections::BTreeMap;
use std::sync::Arc;

impl GameConfig {
    /// A Planechase game (CR 901.2): two-player or multiplayer; the default multiplayer
    /// setup is Free-for-All with the attack multiple players option and without the
    /// limited range of influence option.
    pub fn planechase_game() -> GameConfig {
        GameConfig {
            variant: Variant::Planechase,
            attack_multiple_players: true,
            range_of_influence: None,
            ..Default::default()
        }
    }

    /// A Two-Headed Giant Planechase game (CR 901.12).
    pub fn two_headed_giant_planechase(teams: Vec<u8>) -> GameConfig {
        GameConfig {
            planechase: true,
            ..GameConfig::two_headed_giant(teams)
        }
    }

    /// A Grand Melee Planechase game (CR 901.14).
    pub fn grand_melee_planechase() -> GameConfig {
        GameConfig {
            planechase: true,
            ..GameConfig::grand_melee()
        }
    }

    /// A Vanguard game (CR 902.2): two-player or multiplayer.
    pub fn vanguard_game() -> GameConfig {
        GameConfig {
            variant: Variant::Vanguard,
            attack_multiple_players: true,
            ..Default::default()
        }
    }

    /// A Commander game (CR 903.2): two-player or multiplayer; the default multiplayer
    /// setup is Free-for-All with the attack multiple players option and without the
    /// limited range of influence option.
    pub fn commander_game() -> GameConfig {
        GameConfig {
            variant: Variant::Commander,
            attack_multiple_players: true,
            range_of_influence: None,
            ..Default::default()
        }
    }

    /// A Commander game with the Brawl option (CR 903.12).
    pub fn brawl_game() -> GameConfig {
        GameConfig {
            brawl: true,
            ..GameConfig::commander_game()
        }
    }

    /// An Archenemy game (CR 904.2): Team vs. Team with exactly two teams, one of them a
    /// single player (the archenemy), with the attack multiple players and shared team
    /// turns options and no other multiplayer options. `teams` gives each seat's team.
    pub fn archenemy_game(teams: Vec<u8>) -> GameConfig {
        GameConfig {
            variant: Variant::Archenemy,
            teams: Some(teams),
            attack_multiple_players: true,
            shared_team_turns: true,
            ..Default::default()
        }
    }

    /// A Supervillain Rumble game (CR 904.12): a Free-for-All game in which every player
    /// is an archenemy with their own scheme deck, with the attack multiple players option
    /// and no other multiplayer options.
    pub fn supervillain_rumble() -> GameConfig {
        GameConfig {
            variant: Variant::Archenemy,
            teams: None,
            attack_multiple_players: true,
            ..Default::default()
        }
    }

    /// An Archenemy Commander game (CR 904.13): a Commander game using the Archenemy
    /// rules.
    pub fn archenemy_commander(teams: Vec<u8>) -> GameConfig {
        GameConfig {
            variant: Variant::Commander,
            archenemy: true,
            teams: Some(teams),
            attack_multiple_players: true,
            shared_team_turns: true,
            ..Default::default()
        }
    }

    /// A Conspiracy Draft game (CR 905.3): a multiplayer limited game; the default setup
    /// is Free-for-All with the attack multiple players option and without the limited
    /// range of influence option.
    pub fn conspiracy_draft_game() -> GameConfig {
        GameConfig {
            limited: true,
            ..GameConfig::free_for_all()
        }
    }
}

/// The number of players on each team, for a game of `teams` (seat → team).
fn team_sizes(teams: &[u8]) -> BTreeMap<u8, usize> {
    let mut out = BTreeMap::new();
    for t in teams {
        *out.entry(*t).or_insert(0) += 1;
    }
    out
}

/// The setup problems particular to the casual variants, for a game seated as `teams`
/// (seat → team): an Archenemy game is between exactly two teams, one of which is a
/// single player (CR 904.2, 904.2a, 904.2b), unless every player plays alone
/// (Supervillain Rumble, CR 904.12a); either way only the attack multiple players option
/// (and between teams the shared team turns option) is used.
pub fn setup_errors(config: &GameConfig, teams: &[u8]) -> Vec<SetupError> {
    let mut errors = Vec::new();
    let archenemy = config.variant == Variant::Archenemy
        || (config.archenemy && config.variant == Variant::Commander);
    if archenemy {
        let sizes = team_sizes(teams);
        let rumble = sizes.values().all(|n| *n == 1);
        if !rumble && (sizes.len() != 2 || !sizes.values().any(|n| *n == 1)) {
            errors.push(SetupError::ArchenemyTeams);
        }
        let other_options = config.range_of_influence.is_some()
            || !config.player_ranges.is_empty()
            || config.deploy_creatures
            || config.attack_side.is_some()
            || (rumble && config.shared_team_turns);
        if other_options || !config.attack_multiple_players {
            errors.push(SetupError::VariantOptions);
        }
    }
    errors
}

// --- Supplementary decks ---------------------------------------------------------------

fn has_type(card: &CardDef, t: CardType) -> bool {
    card.faces.iter().any(|f| f.chars.card_types.contains(t))
}

/// Cards whose English names repeat more than `max` times.
fn repeated_names(cards: &[Arc<CardDef>], max: usize) -> Vec<DeckProblem> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for c in cards {
        *counts.entry(c.name.as_str()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .filter(|(_, n)| *n > max)
        .map(|(name, n)| DeckProblem::TooManyCopies {
            name: name.to_string(),
            have: n,
            max,
        })
        .collect()
}

/// Checks a planar deck (CR 901.3): at least ten plane and/or phenomenon cards, no more
/// than two of them phenomena, each with a different English name. With the single
/// planar deck option (`players` = the number of players), the communal deck needs at
/// least forty cards or ten per player, whichever is smaller, and no more phenomena than
/// twice the number of players (CR 901.15a).
pub fn check_planar_deck(
    cards: &[Arc<CardDef>],
    single_deck_players: Option<usize>,
) -> Vec<DeckProblem> {
    let mut problems = Vec::new();
    for c in cards {
        if !has_type(c, CardType::Plane) && !has_type(c, CardType::Phenomenon) {
            problems.push(DeckProblem::WrongCardType {
                name: c.name.to_string(),
                expected: "plane or phenomenon".into(),
            });
        }
    }
    let (min, max_phenomena) = match single_deck_players {
        Some(n) => (40.min(10 * n), 2 * n),
        None => (10, 2),
    };
    if cards.len() < min {
        problems.push(DeckProblem::TooFewCards {
            have: cards.len(),
            min,
        });
    }
    let phenomena = cards
        .iter()
        .filter(|c| has_type(c, CardType::Phenomenon))
        .count();
    if phenomena > max_phenomena {
        problems.push(DeckProblem::TooManyPhenomena {
            have: phenomena,
            max: max_phenomena,
        });
    }
    problems.extend(repeated_names(cards, 1));
    problems
}

/// Checks a scheme deck (CR 904.3): at least twenty scheme cards, no more than two of any
/// card with a particular English name. With the Archenemy Commander option, at least ten
/// cards, each with a different English name (CR 904.13d).
pub fn check_scheme_deck(cards: &[Arc<CardDef>], archenemy_commander: bool) -> Vec<DeckProblem> {
    let mut problems: Vec<DeckProblem> = cards
        .iter()
        .filter(|c| !has_type(c, CardType::Scheme))
        .map(|c| DeckProblem::WrongCardType {
            name: c.name.to_string(),
            expected: "scheme".into(),
        })
        .collect();
    let (min, max_copies) = if archenemy_commander {
        (10, 1)
    } else {
        (20, 2)
    };
    if cards.len() < min {
        problems.push(DeckProblem::TooFewCards {
            have: cards.len(),
            min,
        });
    }
    problems.extend(repeated_names(cards, max_copies));
    problems
}
