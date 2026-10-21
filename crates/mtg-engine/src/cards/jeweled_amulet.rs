//! Jeweled Amulet: "{1}, {T}: Put a charge counter on ~. Note the type of mana spent to
//! pay this activation cost. Activate only if there are no charge counters on ~." and
//! "{T}, Remove a charge counter from ~: Add one mana of ~'s last noted type." The two
//! abilities are linked (CR 607.2e): the note belongs to this object (a new object after
//! a zone change has none, CR 400.7). With no type noted yet, the mana ability adds no
//! mana, even if a charge counter got there some other way (ruling, CR 607.5a). If mana
//! of more than one type was spent (a cost increase), its controller chooses which of
//! those types is noted (interpretation `jeweled-amulet-several-types`).

use super::{map_effect, parse, ManualAbility};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::mana::ManaType;
use crate::types::ObjectId;

const NOTE: &str = "card:Jeweled Amulet:note the type of mana spent to pay this activation cost";
const LAST_NOTED: &str = "card:Jeweled Amulet:~'s last noted type";
const NOTE_TEXT: &str = "{1}, {T}: Put a charge counter on ~. Note the type of mana spent to pay this activation cost. Activate only if there are no charge counters on ~.";
const ADD_TEXT: &str = "{T}, Remove a charge counter from ~: Add one mana of ~'s last noted type.";

inventory::submit! { ManualAbility {
    card: "Jeweled Amulet",
    face: 0,
    text: NOTE_TEXT,
    build: |ctx| {
        parse(ctx, "{1}, {T}: Put a charge counter on ~. Activate only if there are no charge counters on ~.")
            .iter()
            .map(|a| {
                map_effect(a, NOTE_TEXT, |e| {
                    Effect::Seq(vec![e, Effect::Custom(NOTE.into())])
                })
            })
            .collect()
    },
    reason: "notes the type of mana spent to activate it: no other legal card notes a mana type",
} }

inventory::submit! { ManualAbility {
    card: "Jeweled Amulet",
    face: 0,
    text: ADD_TEXT,
    build: |ctx| {
        parse(ctx, "{T}, Remove a charge counter from ~: Add {C}.")
            .iter()
            .map(|a| {
                map_effect(a, ADD_TEXT, |e| match e {
                    Effect::AddMana { who, restriction, .. } => Effect::AddMana {
                        who,
                        mana: ManaProduction::Custom(LAST_NOTED.into()),
                        restriction,
                    },
                    e => e,
                })
            })
            .collect()
    },
    reason: "adds mana of the type its linked ability last noted: no other legal card notes a mana type",
} }

const ORDER: [ManaType; 6] = ManaType::ALL;

/// The type last noted for `obj`.
fn last_noted(g: &Game, obj: ObjectId) -> Option<ManaType> {
    g.cards
        .get(NOTE)
        .iter()
        .rev()
        .find(|r| r.first() == Some(&(obj.0 as i64)))
        .and_then(|r| r.get(1))
        .and_then(|i| ORDER.get(*i as usize).copied())
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != NOTE {
            return false;
        }
        let (Some(src), Some(ab)) = (ctx.source, ctx.stack_obj) else {
            return true;
        };
        let spent: Vec<ManaType> = g
            .try_obj(ab)
            .and_then(|o| o.stack.as_deref())
            .map(|si| si.cast.mana_spent.clone())
            .unwrap_or_default();
        let mut types: Vec<ManaType> = ORDER.iter().copied().filter(|t| spent.contains(t)).collect();
        // Several types spent (a cost increase): its controller chooses which one is noted.
        if types.len() > 1 {
            let names = types.iter().map(|t| format!("{t:?}")).collect();
            let i = g.ask_option(ctx.controller, Some(src), "Note which type of mana spent?", names);
            types = vec![types[i.min(types.len() - 1)]];
        }
        if let Some(t) = types.first().copied() {
            let i = ORDER.iter().position(|x| *x == t).unwrap_or(0);
            g.cards.push(NOTE, vec![src.0 as i64, i as i64]);
            g.log(|g| format!("{} notes {t:?}", g.describe(src)));
        }
        true
    }
    fn custom_mana_types(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<Vec<ManaType>> {
        if name != LAST_NOTED {
            return None;
        }
        Some(ctx.source.and_then(|s| last_noted(g, s)).into_iter().collect())
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
