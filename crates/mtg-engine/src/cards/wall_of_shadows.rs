//! Wall of Shadows: "~ can't be the target of spells that can target only Walls or of
//! abilities that can target only Walls."
//!
//! Whether a spell or ability "can target only Walls" looks at all its target slots in all
//! its modes: a modal spell with a mode that can target a non-Wall can target it (ruling).

use super::{has_marker, marker, ManualAbility};
use crate::ability::{AbilityKind, Body, Filter, TargetKind, TargetSpec};
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::object::StackKind;
use crate::types::{Entity, ObjectId};

const NOT_BY_WALL_ONLY: &str = "card:Wall of Shadows:can't be targeted by Wall-only spells or abilities";
const TEXT: &str = "~ can't be the target of spells that can target only Walls or of abilities that can target only Walls.";

inventory::submit! { ManualAbility {
    card: "Wall of Shadows",
    face: 0,
    text: TEXT,
    build: |_| vec![marker(NOT_BY_WALL_ONLY, TEXT)],
    reason: "can't be targeted by spells or abilities that can target only Walls: unique targeting test",
} }

/// Whether everything the filter matches is a Wall.
fn only_walls(f: &Filter) -> bool {
    match f {
        Filter::Subtype(s) => s.as_str() == "Wall",
        Filter::And(v) => v.iter().any(only_walls),
        Filter::Or(v) => !v.is_empty() && v.iter().all(only_walls),
        _ => false,
    }
}

fn slot_only_walls(spec: &TargetSpec) -> bool {
    matches!(&spec.what, TargetKind::Object(f) if only_walls(f))
}

/// All target slots of a body, in all its modes.
fn slots(body: &Body) -> Vec<&TargetSpec> {
    let mut v: Vec<&TargetSpec> = body.targets.iter().collect();
    if let Some(m) = &body.modal {
        for mode in &m.modes {
            v.extend(mode.targets.iter());
        }
    }
    v
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn target_forbidden(
        &self,
        g: &Game,
        spec: &TargetSpec,
        e: Entity,
        source: Option<ObjectId>,
    ) -> bool {
        let Entity::Object(o) = e else {
            return false;
        };
        if !slot_only_walls(spec) || !has_marker(g, o, NOT_BY_WALL_ONLY) {
            return false;
        }
        // The other target slots of the spell or ability (in every mode) must be
        // Wall-only too.
        let Some(s) = source else {
            return true;
        };
        let bodies: Vec<Body> = match g.obj(s).stack.as_deref().map(|si| &si.kind) {
            Some(StackKind::Activated { ability, .. }) | Some(StackKind::Triggered { ability, .. }) => {
                match &ability.kind {
                    AbilityKind::Activated(a) => vec![a.body.clone()],
                    AbilityKind::Triggered(t) => vec![t.body.clone()],
                    _ => vec![],
                }
            }
            _ => g
                .obj(s)
                .chars
                .abilities
                .iter()
                .filter_map(|a| match &a.kind {
                    AbilityKind::Spell(sp) => Some(sp.body.clone()),
                    _ => None,
                })
                .collect(),
        };
        bodies
            .iter()
            .all(|b| slots(b).into_iter().all(slot_only_walls))
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
