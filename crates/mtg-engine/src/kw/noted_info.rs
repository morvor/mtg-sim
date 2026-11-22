//! Information noted for an object (CR 607.2e): creature types and numbers kept on the
//! object that noted them, for its abilities that refer to "noted" information.
//!
//! * "Note a creature type that hasn't been noted for this Saga." (Long List of the Ents):
//!   its controller chooses a creature type not yet noted for the object; it's noted, and
//!   it's the type the ability's later instructions call "that type" (the linked choice,
//!   CR 607.2d).
//! * "Whenever you cast a creature spell, note one of its creature types that hasn't been
//!   noted for Volo's Journal." (the token Volo, Itinerant Scholar creates): one of the
//!   spell's creature types not yet noted (nothing if there's none left), and "Draw a card
//!   for each creature type noted for target permanent you control named Volo's Journal."
//!   counts them.
//! * "Note the mana value of each card as it's put into exile." / "~'s power is equal to
//!   the greatest number noted for it this turn." (Bell Borca, Spectral Sergeant): a static
//!   ability that notes, while its object is on the battlefield, the mana value of each
//!   card put into exile (as the card is in exile, its rulings); the power is the greatest
//!   number noted this turn, 0 with none (its rulings).
//!
//! The notes belong to the object (CR 400.7): a new object has none. They're kept in the
//! object's linked choices under [`NOTED_LINK`]: a creature type as its name, a number as
//! `#<turn>:<number>`.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::object::Zone;
use crate::types::*;
use smol_str::SmolStr;

/// The linked-choices key of the noted information.
pub const NOTED_LINK: u16 = 0x3fb5;

/// [`Effect::Custom`]: "note a creature type that hasn't been noted for ~".
pub const NOTE_CREATURE_TYPE: &str = "note:a creature type not noted";
/// [`Effect::Custom`]: "note one of its creature types that hasn't been noted for ~" (the
/// spell that triggered the ability).
pub const NOTE_ITS_CREATURE_TYPE: &str = "note:one of its creature types not noted";
/// [`Value::Custom`] prefix: the number of creature types noted for the object in target
/// slot `<slot>`.
pub const NOTED_TYPES_OF_TARGET: &str = "noted:creature types of target:";
/// [`StaticEffect::Custom`]: "Note the mana value of each card as it's put into exile."
pub const NOTE_EXILED_MANA_VALUES: &str = "note:the mana value of each card put into exile";
/// [`Value::Custom`]: "the greatest number noted for it this turn".
pub const GREATEST_NOTED_THIS_TURN: &str = "noted:the greatest number noted this turn";

/// What's noted for `obj`.
pub fn noted(g: &Game, obj: ObjectId) -> Vec<SmolStr> {
    g.try_obj(obj)
        .and_then(|o| o.linked_choices.get(&NOTED_LINK))
        .map(|c| c.list.clone())
        .unwrap_or_default()
}

fn note(g: &mut Game, obj: ObjectId, what: SmolStr) {
    g.objects[obj.0 as usize]
        .linked_choices
        .entry(NOTED_LINK)
        .or_default()
        .list
        .push(what);
}

/// The creature types noted for `obj`.
pub fn noted_types(g: &Game, obj: ObjectId) -> Vec<SmolStr> {
    noted(g, obj)
        .into_iter()
        .filter(|w| !w.starts_with('#'))
        .collect()
}

/// The numbers noted for `obj` during turn `turn`.
pub fn noted_numbers(g: &Game, obj: ObjectId, turn: u32) -> Vec<i64> {
    noted(g, obj)
        .into_iter()
        .filter_map(|w| {
            let (t, n) = w.strip_prefix('#')?.split_once(':')?;
            (t.parse::<u32>().ok()? == turn).then(|| n.parse().ok())?
        })
        .collect()
}

/// Asks `p` to choose one of `options` and notes it for `src`; it's also the creature
/// type the rest of the resolving ability calls "that type" (CR 607.2d). Each resolution
/// keeps its type under a fresh link, which becomes the resolution's link: a delayed
/// trigger it creates ("When you next cast a creature spell of that type this turn")
/// keeps looking for that type when a later resolution of the same ability notes
/// another one.
fn choose_and_note(g: &mut Game, p: PlayerId, src: ObjectId, ctx: &mut Ctx, options: Vec<String>) {
    if options.is_empty() || !g.is_live(src) {
        return;
    }
    let link = crate::ability_grants::one_fresh_link(g);
    ctx.link = link;
    let i = g.ask_option(p, Some(src), "Note a creature type", options.clone());
    let t = SmolStr::new(&options[i.min(options.len() - 1)]);
    g.log(|g| format!("{} notes {t}", g.describe(src)));
    note(g, src, t.clone());
    let o = &mut g.objects[src.0 as usize];
    o.choices.creature_type = Some(t.clone());
    o.linked_choices.entry(link).or_default().creature_type = Some(t);
    g.dirty = true;
}

pub struct NotedInfo;

impl KeywordRules for NotedInfo {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        let Some(src) = ctx.source else {
            return matches!(name, NOTE_CREATURE_TYPE | NOTE_ITS_CREATURE_TYPE);
        };
        match name {
            NOTE_CREATURE_TYPE => {
                let done = noted_types(g, src);
                let options: Vec<String> = crate::types::subtype_lists()
                    .creature
                    .iter()
                    .filter(|t| !done.iter().any(|d| d.as_str() == t.as_str()))
                    .cloned()
                    .collect();
                choose_and_note(g, ctx.controller, src, ctx, options);
                true
            }
            NOTE_ITS_CREATURE_TYPE => {
                let done = noted_types(g, src);
                let spell = g
                    .eval_sel(&Sel::TriggerSpell, ctx)
                    .into_iter()
                    .find_map(|e| e.object());
                let Some(spell) = spell else {
                    return true;
                };
                let chars = &g.obj(spell).chars;
                let types: Vec<String> = crate::types::subtype_lists()
                    .creature
                    .iter()
                    .filter(|t| chars.has_subtype(t))
                    .filter(|t| !done.iter().any(|d| d.as_str() == t.as_str()))
                    .cloned()
                    .collect();
                choose_and_note(g, ctx.controller, src, ctx, types);
                true
            }
            _ => false,
        }
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name == GREATEST_NOTED_THIS_TURN {
            let src = ctx.source?;
            return Some(
                noted_numbers(g, src, g.turn.number)
                    .into_iter()
                    .max()
                    .unwrap_or(0),
            );
        }
        let slot: usize = name.strip_prefix(NOTED_TYPES_OF_TARGET)?.parse().ok()?;
        let n = ctx
            .targets
            .get(slot)
            .into_iter()
            .flatten()
            .filter_map(|e| e.object())
            .filter(|o| g.is_live(*o))
            .map(|o| noted_types(g, o).len() as i64)
            .sum();
        Some(n)
    }

    fn on_event(&self, g: &mut Game, ev: &Event) {
        let Event::ZoneChange {
            new,
            to: Zone::Exile,
            ..
        } = ev
        else {
            return;
        };
        let new = *new;
        if !g.is_live(new) || g.obj(new).is_token() {
            return;
        }
        let noters: Vec<ObjectId> = g
            .statics
            .customs
            .iter()
            .filter(|(s, _, n)| {
                n.as_str() == NOTE_EXILED_MANA_VALUES
                    && g.is_live(*s)
                    && g.obj(*s).zone == Zone::Battlefield
            })
            .map(|(s, _, _)| *s)
            .collect();
        if noters.is_empty() {
            return;
        }
        // The card as it is in exile (an {X} in its mana cost is 0, CR 107.3g).
        let mv = g.obj(new).chars.mana_value() as i64;
        let turn = g.turn.number;
        for s in noters {
            note(g, s, SmolStr::new(format!("#{turn}:{mv}")));
            g.dirty = true;
        }
    }
}

inventory::submit! { KeywordRegistration(&NotedInfo) }
