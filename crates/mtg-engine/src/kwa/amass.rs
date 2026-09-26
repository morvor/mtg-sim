//! CR 701.47: amass.
//!
//! * "Amass [subtype] N": if you don't control an Army creature, create a 0/0 black
//!   [subtype] Army creature token; choose an Army creature you control; put N +1/+1
//!   counters on it; if it isn't a [subtype], it becomes one in addition to its other
//!   types (CR 701.47a).
//! * A player "amassed" once the process is complete, even if some or all of it was
//!   impossible (CR 701.47b): an `"amass"` event (`Event::Custom`) is reported.
//! * "The Army you amassed" is the chosen creature, whether or not it got counters
//!   (CR 701.47c): it's stored in [`kvars::AMASSED`].
//! * Older cards printed "amass N" were given the subtype Zombies (CR 701.47d); their
//!   Oracle text reads "amass Zombies N".

use super::*;
use crate::types::counters;

/// `Event::Custom` name reported when a player amasses.
pub const AMASSED_EVENT: &str = "amass";

fn army_token(subtype: &Subtype) -> TokenSpec {
    TokenSpec {
        name: SmolStr::default(),
        colors: ColorSet::single(Color::Black),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec![subtype.clone(), SmolStr::new("Army")],
        power: Some(0),
        toughness: Some(0),
        abilities: vec![],
        scryfall_name: None,
    }
}

fn armies(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.permanents()
        .filter(|o| o.controller == p && o.is_creature() && o.chars.has_subtype("Army"))
        .map(|o| o.id)
        .collect()
}

/// `p` amasses [subtype] N (CR 701.47a–c). Returns the Army they chose.
pub fn amass(g: &mut Game, p: PlayerId, subtype: &Subtype, n: u32, ctx: &mut Ctx) -> Option<ObjectId> {
    if g.dirty {
        g.recompute();
    }
    let mut c = ctx.clone();
    c.controller = p;
    if armies(g, p).is_empty() {
        g.exec(
            &Effect::CreateToken {
                spec: army_token(subtype),
                count: Value::c(1),
                controller: PlayerRef::You,
                tapped: false,
                attacking: false,
            },
            &mut c,
        );
        g.recompute();
    }
    let cands = armies(g, p);
    let army = match cands.as_slice() {
        [] => None,
        [one] => Some(*one),
        _ => g
            .ask_objects(p, ctx.source, "Amass: choose an Army", cands.clone(), 1, 1)
            .first()
            .copied()
            .or(Some(cands[0])),
    };
    if let Some(army) = army {
        g.add_counters(Entity::Object(army), counters::PLUS1, n, ctx.source);
        g.recompute();
        if on_battlefield(g, army) && !g.obj(army).chars.has_subtype(subtype) {
            c.set_var(kvars::AMASSED, vec![Entity::Object(army)]);
            g.exec(
                &Effect::Modify {
                    what: Sel::Var(kvars::AMASSED),
                    mods: vec![Modification::AddSubtypes(vec![subtype.clone()])],
                    duration: Duration::Permanent,
                },
                &mut c,
            );
        }
    }
    ctx.set_var(
        kvars::AMASSED,
        army.map(Entity::Object).into_iter().collect(),
    );
    emit(g, AMASSED_EVENT, p, army, n as i32);
    army
}

pub struct Amass;

impl KeywordActionRules for Amass {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Amass]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        // CR 701.47d: an amass instruction without a subtype amasses Zombies.
        let subtype = a
            .spec
            .and_then(|s| s.subtype.clone())
            .unwrap_or_else(|| SmolStr::new("Zombie"));
        for p in g.eval_players(a.who, ctx) {
            amass(g, p, &subtype, n, ctx);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Amass) }
