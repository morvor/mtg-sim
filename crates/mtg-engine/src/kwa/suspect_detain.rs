//! CR 701.60: suspect, and CR 701.35: detain.
//!
//! * Suspecting a creature makes it suspected until it leaves the battlefield or an effect
//!   makes it no longer suspected (CR 701.60a). Suspected is a designation of permanents,
//!   neither an ability nor a copiable value (CR 701.60b; `GameObject::suspected`). A
//!   suspected permanent has menace and "This creature can't block" for as long as it's
//!   suspected (CR 701.60c): a continuous effect with the timestamp of when it became
//!   suspected, so an effect that later removes all abilities removes them too while it
//!   stays suspected. A suspected permanent can't become suspected again (CR 701.60d).
//! * Detaining a permanent: until the next turn of the controller of the spell or ability
//!   that detained it, it can't attack or block and its activated abilities (including
//!   mana abilities) can't be activated (CR 701.35a).

use super::*;
use crate::game::{Affected, ContinuousEffect};
use crate::keywords::{Keyword, KeywordKind};

/// `Filter::Custom` name: "suspected" (a suspected permanent); also the `Event::Custom`
/// name reported when a permanent becomes suspected.
pub const SUSPECTED: &str = "suspected";

/// Whether `id` is a suspected permanent.
pub fn is_suspected(g: &Game, id: ObjectId) -> bool {
    on_battlefield(g, id) && g.obj(id).suspected
}

/// The abilities a suspected permanent has (CR 701.60c).
fn suspected_mods() -> Vec<Modification> {
    vec![
        Modification::AddKeyword(Keyword::new(KeywordKind::Menace)),
        Modification::AddAbility(AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(
                Restriction::CantBlock(Filter::Source),
            ))),
            "This creature can't block.",
        )),
    ]
}

/// Suspects `obj` (CR 701.60a, 701.60d). Returns true if it became suspected.
pub fn suspect(g: &mut Game, obj: ObjectId) -> bool {
    if !on_battlefield(g, obj) || g.obj(obj).suspected {
        return false;
    }
    g.objects[obj.0 as usize].suspected = true;
    let controller = g.obj(obj).controller;
    let id = g.new_effect_id();
    let ts = g.new_timestamp();
    g.effects.push(ContinuousEffect {
        id,
        source: Some(obj),
        controller,
        timestamp: ts,
        // For as long as it's suspected (and it's still this permanent).
        duration: Duration::WhileCondition(Condition::SelMatches(
            Sel::This,
            Filter::Custom(SmolStr::new(SUSPECTED)),
        )),
        affected: Affected::Objects(vec![obj]),
        mods: suspected_mods(),
        layer1: None,
        created_turn: g.turn.number,
    });
    g.dirty = true;
    g.log(|g| format!("{} becomes suspected", g.describe(obj)));
    emit(g, SUSPECTED, controller, Some(obj), 0);
    true
}

/// `obj` is no longer suspected (CR 701.60a).
pub fn unsuspect(g: &mut Game, obj: ObjectId) {
    if g.obj(obj).suspected {
        g.objects[obj.0 as usize].suspected = false;
        g.dirty = true;
    }
}

pub struct Suspect;

impl KeywordActionRules for Suspect {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Suspect]
    }

    /// "Suspect [creatures]", or (`undo`) "[creatures] are no longer suspected".
    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let objs = g.resolve_objects(a.what, ctx);
        if a.spec.is_some_and(|s| s.undo) {
            for obj in objs {
                unsuspect(g, obj);
            }
            return;
        }
        let mut any = false;
        for obj in objs {
            any |= suspect(g, obj);
        }
        ctx.prev_happened = any;
    }
}

inventory::submit! { KeywordActionRegistration(&Suspect) }

struct SuspectedRules;

impl crate::kw::KeywordRules for SuspectedRules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn custom_filter(&self, g: &Game, name: &str, id: ObjectId, _ctx: &Ctx) -> Option<bool> {
        (name == SUSPECTED).then(|| is_suspected(g, id))
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&SuspectedRules) }

/// Detains `objs` (CR 701.35a) for the controller of the resolving spell or ability
/// (`ctx`).
pub fn detain(g: &mut Game, objs: &[ObjectId], ctx: &mut Ctx) {
    let objs: Vec<ObjectId> = objs
        .iter()
        .copied()
        .filter(|o| on_battlefield(g, *o))
        .collect();
    if objs.is_empty() {
        return;
    }
    let these = Filter::Objects(objs.clone());
    for restriction in [
        Restriction::CantAttackOrBlock(these.clone()),
        Restriction::CantActivate {
            who: PlayerFilter::Any,
            sources: these.clone(),
            include_mana: true,
        },
    ] {
        g.exec(
            &Effect::AddRestriction {
                restriction,
                duration: Duration::UntilYourNextTurn,
            },
            ctx,
        );
    }
    for o in &objs {
        g.log(|g| format!("{} is detained", g.describe(*o)));
    }
}

pub struct Detain;

impl KeywordActionRules for Detain {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Detain]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let objs = g.resolve_objects(a.what, ctx);
        detain(g, &objs, ctx);
    }
}

inventory::submit! { KeywordActionRegistration(&Detain) }
