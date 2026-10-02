//! Objects that have or gain the abilities of other objects (CR 113.10, 613.1f): "~ has
//! all activated abilities of all creature cards in all graveyards" (Necrotic Ooze),
//! "creatures you control have all activated abilities of all land cards exiled with ~"
//! (Steward of the Harvest), "~ gains all activated abilities of target creature until
//! end of turn" (Quicksilver Elemental), "~ has all activated and triggered abilities of
//! the exiled card" (Idris, Soul of the TARDIS). See [`Modification::AddAbilitiesOf`].
//!
//! * **Which abilities.** Those of the selected kinds that the selected objects have as
//!   their characteristics stand when the effect applies in layer 6: an ability another
//!   layer-6 effect gives one of them is included once that effect has applied. Since
//!   applying such an effect changes what this one does, this one depends on it and waits
//!   for it (CR 613.8a–b; the layer system compares [`copied_uids`]). A static ability's
//!   effect is reevaluated each time characteristics are computed; a resolving spell's or
//!   ability's effect gains the abilities the object has as the effect is created
//!   (CR 608.2h, see [`snapshot`]).
//! * **Keyword abilities.** "Some keywords are activated abilities": a keyword that stands
//!   only for abilities of the selected kinds (cycling, equip, prowess) is gained as the
//!   keyword; of a keyword standing for abilities of several kinds, only the abilities of
//!   the selected kinds are gained. Keywords that stand for static abilities (flying) and
//!   static abilities aren't gained.
//! * **Identity.** Each gained ability is a distinct ability: restrictions such as
//!   "activate only once each turn" apply to it as acquired from that object (CR 602.5c).
//!   Abilities gained together from one object keep the links they had with each other on
//!   that object and are linked to no other ability (CR 607.5); one whose linked ability
//!   isn't gained is linked to nothing, and a choice it refers to is undefined (CR 607.5a).
//! * **Names.** Compiled abilities refer to the object they're on as "this object"
//!   ([`Sel::This`]), which is the object that has the ability: an ability that names the
//!   card it's printed on names the object that gained it instead (CR 201.5b).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Whether an activated ability belongs to a class (CR 605, 606).
fn in_class(class: AbilityClass, a: &AbilityDef, act: &ActivatedAbility) -> bool {
    match class {
        AbilityClass::Any => true,
        AbilityClass::Loyalty => {
            act.is_loyalty
                || act
                    .cost
                    .parts
                    .iter()
                    .any(|p| matches!(p, CostPart::Loyalty(_)))
        }
        AbilityClass::Mana => act.is_mana_ability,
        AbilityClass::Keyword(k) => crate::keyword_impls::ability_from_keyword(a) == Some(k),
    }
}

/// Whether a (non-keyword) ability is of the selected kinds.
pub fn selected(which: &AbilitySelection, a: &AbilityDef) -> bool {
    match &a.kind {
        AbilityKind::Activated(act) => {
            which.activated
                && which.only.is_none_or(|c| in_class(c, a, act))
                && !which.except.is_some_and(|c| in_class(c, a, act))
        }
        AbilityKind::Triggered(_) => which.triggered,
        _ => false,
    }
}

/// The abilities of `from` (as its characteristics currently stand) of the selected kinds,
/// as they are on that object: plain abilities, keywords standing only for such
/// abilities, and the abilities of such kinds of other keywords.
pub fn gained_from(g: &Game, which: &AbilitySelection, from: ObjectId) -> Vec<Ability> {
    let mut out = Vec::new();
    let abilities = &g.obj(from).chars.abilities;
    for a in abilities {
        match &a.kind {
            AbilityKind::Keyword(k) => {
                let derived = crate::keyword_impls::derived_abilities(k);
                // A keyword whose abilities the object already has expanded (Lion Sash's
                // reconfigure): those abilities are gained as they are, once each.
                if derived.is_empty()
                    || derived
                        .iter()
                        .any(|d| abilities.iter().any(|x| x.uid == d.uid))
                {
                    continue;
                }
                if derived.iter().all(|d| selected(which, d)) {
                    out.push(a.clone());
                } else {
                    out.extend(derived.into_iter().filter(|d| selected(which, d)));
                }
            }
            _ if selected(which, a) => out.push(a.clone()),
            _ => {}
        }
    }
    out
}

/// The objects whose abilities the effect gives to `target` (not `target` itself, which
/// already has its own abilities).
fn sources(g: &Game, from: &Sel, ctx: &Ctx, target: Option<ObjectId>) -> Vec<ObjectId> {
    let mut v: Vec<ObjectId> = g
        .eval_sel_objects(from, ctx)
        .into_iter()
        .filter(|o| Some(*o) != target)
        .collect();
    v.dedup();
    v
}

/// The link an ability with link `link` has when the object the effect from `granter`
/// affects has it because of `from` (CR 607.5): the same for abilities `from` has with
/// the same link, distinct from the links of printed abilities and of abilities gained
/// from other objects or other effects.
fn gained_link(link: u16, granter: u32, from: u32) -> u16 {
    let h = (link as u64 * 131)
        .wrapping_add(granter as u64 * 31)
        .wrapping_add(from as u64 * 7919)
        .wrapping_add(0x3d1);
    0x8000 | (h % 0x7fff) as u16
}

/// Ability `a` of `from` as an object gains it through an effect of `granter`, with a
/// stable identity across recomputation.
fn gained(a: &Ability, granter: Option<ObjectId>, from: ObjectId) -> Ability {
    static CACHE: OnceLock<Mutex<HashMap<(u64, u32, u32), Ability>>> = OnceLock::new();
    let g = granter.map_or(u32::MAX, |s| s.0);
    let m = CACHE.get_or_init(Default::default);
    let mut c = m.lock().unwrap();
    c.entry((a.uid, g, from.0))
        .or_insert_with(|| {
            AbilityDef::with_link(
                a.kind.clone(),
                a.text.clone(),
                gained_link(a.link, g, from.0),
            )
        })
        .clone()
}

/// The abilities a continuously reevaluated effect from `ctx.source` gives `target`.
pub fn abilities_for(
    g: &Game,
    from: &Sel,
    which: &AbilitySelection,
    ctx: &Ctx,
    target: ObjectId,
) -> Vec<Ability> {
    let mut out = Vec::new();
    for o in sources(g, from, ctx, Some(target)) {
        for a in gained_from(g, which, o) {
            out.push(gained(&a, ctx.source, o));
        }
    }
    out
}

/// What the effect currently gives, for dependency checks (CR 613.8a: "what it does to
/// any of the things it applies to"): the uids of the abilities it would copy.
pub fn copied_uids(g: &Game, from: &Sel, which: &AbilitySelection, ctx: &Ctx) -> Vec<i64> {
    let mut out = Vec::new();
    for o in sources(g, from, ctx, None) {
        out.push(-(o.0 as i64) - 1);
        out.extend(gained_from(g, which, o).iter().map(|a| a.uid as i64));
    }
    out
}

/// The abilities a resolving spell's or ability's effect gives, fixed as the effect is
/// created (CR 608.2h): fresh abilities, each distinct from every other ability ("You can
/// activate the ability more than once, collecting abilities from multiple creatures (or
/// the same creature more than once)"), with the links among those gained from one object
/// kept (CR 607.5).
pub fn snapshot(g: &Game, from: &Sel, which: &AbilitySelection, ctx: &Ctx) -> Vec<Modification> {
    let mut out = Vec::new();
    for o in sources(g, from, ctx, None) {
        let mut links: HashMap<u16, u16> = HashMap::new();
        for a in gained_from(g, which, o) {
            let link = *links
                .entry(a.link)
                .or_insert_with(|| 0x8000 | (next_ability_uid() % 0x7fff) as u16);
            out.push(Modification::AddAbility(AbilityDef::with_link(
                a.kind.clone(),
                a.text.clone(),
                link,
            )));
        }
    }
    out
}
