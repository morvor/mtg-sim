//! Space sculptor (CR 702.158): sector designations and the state-based action that
//! assigns them (CR 704.5u).
//!
//! * "Choose a sector" along with an action on each creature in that sector
//!   (CR 702.158d): [`CHOOSE_SECTOR`] asks for one of the three designations and keeps it
//!   in the resolving ability's context, where [`IN_CHOSEN_SECTOR`] finds the creatures
//!   with it ("in the sector of your choice").
//! * "Creatures in each sector can be blocked this turn only by creatures in the same
//!   sector" ([`SAME_SECTOR_BLOCKING`]): two permanents are in the same sector if each has
//!   the same sector designation (CR 702.158e). A creature without a designation (e.g.
//!   once no permanent with space sculptor remains) isn't restricted.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::Var;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::StackKind;
use crate::types::*;
use smol_str::SmolStr;

/// The sector designations (CR 702.158b).
pub const SECTORS: [&str; 3] = ["alpha", "beta", "gamma"];

/// `Effect::Custom`: choose a sector (CR 702.158d).
pub const CHOOSE_SECTOR: &str = "space sculptor:choose a sector";
/// `Filter::Custom`: a permanent in the sector chosen by [`CHOOSE_SECTOR`].
pub const IN_CHOSEN_SECTOR: &str = "space sculptor:in the chosen sector";
/// `Effect::Custom`: "Creatures in each sector can be blocked this turn only by creatures
/// in the same sector."
pub const SAME_SECTOR_BLOCKING: &str =
    "space sculptor:creatures can be blocked this turn only by creatures in the same sector";
/// The chosen sector's index in [`SECTORS`], in the resolving ability's context.
const CHOSEN: Var = crate::ability::vars::USER + 1580;

/// Whether two permanents are in the same sector (CR 702.158e).
pub fn same_sector(g: &Game, a: ObjectId, b: ObjectId) -> bool {
    matches!((&g.obj(a).sector, &g.obj(b).sector), (Some(x), Some(y)) if x == y)
}

pub struct SpaceSculptor;

/// Whether a permanent with space sculptor is on the battlefield.
fn sculptor_on_battlefield(g: &Game) -> bool {
    g.permanents()
        .any(|o| o.chars.has_keyword(KeywordKind::SpaceSculptor))
}

/// Whether `p` controls a permanent with space sculptor.
fn controls_sculptor(g: &Game, p: PlayerId) -> bool {
    g.permanents()
        .any(|o| o.controller == p && o.chars.has_keyword(KeywordKind::SpaceSculptor))
}

/// Whether an ability whose source has space sculptor is on the stack (CR 702.158b).
fn sculptor_ability_on_stack(g: &Game) -> bool {
    g.stack.iter().any(|s| {
        matches!(
            g.obj(*s).stack.as_deref().map(|x| &x.kind),
            Some(StackKind::Activated { source, .. } | StackKind::Triggered { source, .. })
                if g.obj(*source).chars.has_keyword(KeywordKind::SpaceSculptor)
        )
    })
}

impl KeywordRules for SpaceSculptor {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::SpaceSculptor]
    }

    fn state_based_actions(&self, g: &mut Game) -> bool {
        if !sculptor_on_battlefield(g) {
            // CR 702.158b: a permanent keeps its sector designation only until no player
            // controls a permanent with space sculptor or an ability whose source has it.
            if !sculptor_ability_on_stack(g) {
                for id in g.battlefield.clone() {
                    if g.obj(id).sector.is_some() {
                        g.objects[id.0 as usize].sector = None;
                    }
                }
            }
            return false;
        }
        // CR 704.5u, 702.158c: creatures without a sector designation get one. Players who
        // don't control a permanent with space sculptor choose first, then the others.
        let unassigned: Vec<(ObjectId, PlayerId)> = g
            .permanents()
            .filter(|o| o.is(CardType::Creature) && o.sector.is_none())
            .map(|o| (o.id, o.controller))
            .collect();
        if unassigned.is_empty() {
            return false;
        }
        let order = g.apnap();
        for sculptors in [false, true] {
            for &p in &order {
                if controls_sculptor(g, p) != sculptors {
                    continue;
                }
                for &(id, _) in unassigned.iter().filter(|(_, c)| *c == p) {
                    let name = g.obj(id).chars.name.clone();
                    let options = SECTORS.iter().map(|s| format!("{s} sector")).collect();
                    let i =
                        g.ask_option(p, Some(id), &format!("Choose a sector for {name}"), options);
                    g.objects[id.0 as usize].sector = Some(SmolStr::new(SECTORS[i]));
                    g.log(|_| format!("{p} assigns {name} to {} sector", SECTORS[i]));
                }
            }
        }
        true
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        match name {
            CHOOSE_SECTOR => {
                let options = SECTORS.iter().map(|s| format!("{s} sector")).collect();
                let i = g.ask_option(ctx.controller, ctx.source, "Choose a sector", options);
                let i = i.min(SECTORS.len() - 1);
                g.log(|_| format!("{} chooses {} sector", ctx.controller, SECTORS[i]));
                ctx.nums.insert(CHOSEN, i as i64);
                true
            }
            SAME_SECTOR_BLOCKING => {
                g.emit(Event::Custom {
                    name: SAME_SECTOR_BLOCKING.into(),
                    player: Some(ctx.controller),
                    obj: ctx.source,
                    amount: 0,
                });
                true
            }
            _ => false,
        }
    }

    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, ctx: &Ctx) -> Option<bool> {
        if name != IN_CHOSEN_SECTOR {
            return None;
        }
        let chosen = ctx.nums.get(&CHOSEN).and_then(|i| SECTORS.get(*i as usize));
        Some(chosen.is_some_and(|s| g.obj(id).sector.as_deref() == Some(*s)))
    }

    fn block_allowed(&self, g: &Game, blocker: ObjectId, attacker: ObjectId) -> bool {
        let restricted = g
            .turn_events
            .iter()
            .any(|e| matches!(e, Event::Custom { name, .. } if name == SAME_SECTOR_BLOCKING));
        !restricted || g.obj(attacker).sector.is_none() || same_sector(g, blocker, attacker)
    }
}

inventory::submit! { KeywordRegistration(&SpaceSculptor) }
