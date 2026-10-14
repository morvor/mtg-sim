//! Each player's own running totals kept while a spell or ability resolves
//! ([`Effect::AddToPlayerTally`], [`Value::PlayerTally`]): "Starting with you, each player
//! may pay any amount of life. Repeat this process until no one pays life. Each player
//! creates a 1/1 black Rat creature token for each 1 life they paid this way." (Plague of
//! Vermin): each payment is its own life payment (CR 119.4), and afterwards each player's
//! total over all the passes is calculated (Plague of Vermin rulings). They last for the
//! rest of the resolution, across "each player" instructions and passes of a process.

use crate::ability::{vars, Var};

/// The life each player paid this way.
pub const LIFE_PAID: Var = vars::USER + 9608;
/// The mana each player paid this way.
pub const MANA_PAID: Var = vars::USER + 9609;
/// The number of cards each player's latest card action affected ("for each card they
/// revealed this way"; recorded with `kw::hand_graveyard_actions::RECORD_THAT_MANY`).
pub const CARDS_ACTED_ON: Var = vars::USER + 9610;
/// Set (to 1) during a pass of a process in which some player did the optional
/// instruction ("Repeat this process until no one puts a card onto the battlefield").
pub const SOMEONE_DID: Var = vars::USER + 9611;

/// What a tally counts, as the text names it.
pub fn words(var: Var) -> &'static str {
    match var {
        LIFE_PAID => "the life they paid this way",
        MANA_PAID => "the amount of mana they paid this way",
        CARDS_ACTED_ON => "the number of cards they revealed this way",
        _ => "their total",
    }
}
