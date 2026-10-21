//! Undiscovered Paradise: "{T}: Add one mana of any color. During your next untap step, as
//! you untap your permanents, return ~ to its owner's hand." Not a triggered ability (no
//! one has priority in the untap step, CR 502.4): it returns as your permanents untap, if
//! it's still on the battlefield, even if it doesn't untap (rulings).

use super::{marker, parse, ManualAbility};
use crate::ability::{AbilityDef, LibraryPosition};
use crate::events::{Event, MoveCause};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv};
use crate::types::{ObjectId, PlayerId};

const RETURNS: &str = "card:Undiscovered Paradise:returns to its owner's hand during your next untap step";
const TEXT: &str = "{T}: Add one mana of any color. During your next untap step, as you untap your permanents, return ~ to its owner's hand.";

inventory::submit! { ManualAbility {
    card: "Undiscovered Paradise",
    face: 0,
    text: TEXT,
    build: |ctx| {
        let mut v: Vec<_> = parse(ctx, "{T}: Add one mana of any color.")
            .iter()
            .map(|a| AbilityDef::new(a.kind.clone(), TEXT))
            .collect();
        v.push(marker(RETURNS, TEXT));
        v
    },
    reason: "a delayed return to hand as you untap during your next untap step (not a trigger): only this card",
} }

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    /// Its mana ability resolved: (land, you, turn).
    fn on_event(&self, g: &mut Game, ev: &Event) {
        if let Event::TappedForMana { obj, player, .. } = ev {
            if super::has_marker(g, *obj, RETURNS) {
                let row = vec![i64::from(obj.0), i64::from(player.0), i64::from(g.turn.number)];
                g.cards.push(RETURNS, row);
            }
        }
    }
    fn untap_step(&self, g: &mut Game, active: PlayerId, _to_untap: &mut Vec<ObjectId>) {
        let turn = i64::from(g.turn.number);
        let due: Vec<ObjectId> = g
            .cards
            .get(RETURNS)
            .iter()
            .filter(|r| r[1] == i64::from(active.0) && r[2] < turn)
            .map(|r| ObjectId(r[0] as u32))
            .collect();
        g.cards
            .retain(RETURNS, |r| !(r[1] == i64::from(active.0) && r[2] < turn));
        for land in due {
            if !g.is_live(land) || g.obj(land).zone != Zone::Battlefield {
                continue;
            }
            let owner = g.obj(land).owner;
            g.move_object_ev(MoveEv {
                obj: land,
                to: Zone::Hand(owner),
                pos: LibraryPosition::Top,
                cause: MoveCause::Return,
                by: Some(active),
                etb: EtbInfo::default(),
                source: Some(land),
            });
        }
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
