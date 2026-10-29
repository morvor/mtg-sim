//! Time Vault: "If you would begin your turn while ~ is tapped, you may skip that turn
//! instead. If you do, untap ~." A replacement of beginning a turn (CR 614.10): the choice
//! is made as the turn would begin, only if Time Vault is tapped then; a skipped turn
//! never begins, and Time Vault untaps before anything else happens in the next turn
//! (rulings, CR 614.10a).

use super::{has_marker, marker, ManualAbility};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::types::{ObjectId, PlayerId};

const TEXT: &str =
    "If you would begin your turn while ~ is tapped, you may skip that turn instead. If you do, untap ~.";
const SKIP: &str = "card:Time Vault:skip a turn you would begin while it's tapped";

inventory::submit! { ManualAbility {
    card: "Time Vault",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(SKIP, TEXT)],
    reason: "replaces beginning a turn (CR 614.10): the only card that does",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn replace_turn_begin(&self, g: &mut Game, p: PlayerId) -> bool {
        if g.dirty {
            g.recompute();
        }
        let vaults: Vec<ObjectId> = g
            .permanents_controlled_by(p)
            .into_iter()
            .filter(|&o| {
                let obj = g.obj(o);
                obj.zone == Zone::Battlefield && obj.tapped && has_marker(g, o, SKIP)
            })
            .collect();
        for v in vaults {
            if g.ask_yes_no(p, Some(v), "Skip this turn and untap it?", false) {
                g.log(|g| format!("{p:?} skips a turn ({})", g.describe(v)));
                g.untap(v);
                return true;
            }
        }
        false
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
