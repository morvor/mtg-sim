//! Space sculptor (CR 702.158): sector designations and the state-based action that
//! assigns them (CR 704.5u).

use super::{KeywordRegistration, KeywordRules};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::StackKind;
use crate::types::*;
use smol_str::SmolStr;

/// The sector designations (CR 702.158b).
pub const SECTORS: [&str; 3] = ["alpha", "beta", "gamma"];

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
}

inventory::submit! { KeywordRegistration(&SpaceSculptor) }
