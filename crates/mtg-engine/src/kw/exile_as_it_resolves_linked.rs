//! "Whenever a player casts an instant or sorcery spell, exile it instead of putting it
//! into a graveyard as it resolves." (Rod of Absorption): the triggered ability gives the
//! spell a replacement effect of where it goes as it resolves (CR 608.2n, 614.1a), and the
//! card exiled this way is "exiled with" the source (CR 607.2a), so the source's other
//! ability can find it ("cards exiled with this artifact").
//!
//! * A spell that's countered, or never resolves, isn't exiled.
//! * With several such sources, each gives the spell the effect; as it resolves its
//!   controller chooses which of them exiles it (and so has it linked).
//! * Each source tracks the cards it exiled: a source that left the battlefield and came
//!   back is a new object with none (CR 400.7).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom`: the triggering spell gets the replacement effect, for the resolving
/// ability's source and link.
pub const EFFECT: &str = "exile the triggering spell as it resolves, exiled with the source";

/// Prefix of the marker ability given to the spell, followed by "<source id>:<link>".
const MARKER: &str = "exiled with a source as it resolves:";

fn marker(src: ObjectId, link: u16) -> Modification {
    let name = SmolStr::new(format!("{MARKER}{}:{link}", src.0));
    let mut s = StaticAbility::new(StaticEffect::Custom(name.clone()));
    s.zone = FunctionZone::Stack;
    Modification::AddAbility(AbilityDef::new(AbilityKind::Static(s), name.as_str()))
}

/// The sources (and links) that would exile the spell as it resolves.
fn markers(g: &Game, spell: ObjectId) -> Vec<(ObjectId, u16)> {
    let mut out = Vec::new();
    for a in &g.obj(spell).chars.abilities {
        let AbilityKind::Static(s) = &a.kind else {
            continue;
        };
        let StaticEffect::Custom(n) = &s.effect else {
            continue;
        };
        let Some((src, link)) = n.strip_prefix(MARKER).and_then(|r| r.split_once(':')) else {
            continue;
        };
        if let (Ok(src), Ok(link)) = (src.parse::<u32>(), link.parse::<u16>()) {
            if !out.contains(&(ObjectId(src), link)) {
                out.push((ObjectId(src), link));
            }
        }
    }
    out
}

pub struct ExileAsItResolvesLinked;

impl KeywordRules for ExileAsItResolvesLinked {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != EFFECT {
            return false;
        }
        let Some(src) = ctx.source else {
            return true;
        };
        let e = Effect::Modify {
            what: Sel::TriggerSpell,
            mods: vec![marker(src, ctx.link)],
            duration: Duration::Permanent,
        };
        g.exec(&e, ctx);
        true
    }

    fn global_resolved_destination(
        &self,
        g: &Game,
        spell: ObjectId,
    ) -> Option<(Zone, LibraryPosition)> {
        (!markers(g, spell).is_empty()).then_some((Zone::Exile, LibraryPosition::Top))
    }

    fn global_after_spell_resolved(&self, g: &mut Game, spell: ObjectId, new: ObjectId) {
        if g.obj(new).zone != Zone::Exile {
            return;
        }
        // Only sources still where they were can have cards exiled with them.
        let sources: Vec<(ObjectId, u16)> = markers(g, spell)
            .into_iter()
            .filter(|(s, _)| g.is_live(*s))
            .collect();
        let (src, link) = match sources.len() {
            0 => return,
            1 => sources[0],
            _ => {
                let p = g.obj(spell).controller;
                let labels = sources
                    .iter()
                    .map(|(s, _)| format!("Exiled with {}", g.describe(*s)))
                    .collect();
                let i = g.ask_option(p, Some(new), "Which one exiles it?", labels);
                sources[i.min(sources.len() - 1)]
            }
        };
        g.objects[src.0 as usize]
            .linked
            .entry(link)
            .or_default()
            .push(new);
    }
}

inventory::submit! { KeywordRegistration(&ExileAsItResolvesLinked) }
