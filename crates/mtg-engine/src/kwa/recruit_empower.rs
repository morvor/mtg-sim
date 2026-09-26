//! CR 701.70: recruit, and CR 701.71: empower Jace.
//!
//! * "Recruit" means "Draw a card, then discard a card. If you discarded a nonland card
//!   this way, create a 1/1 white Human Soldier creature token" (CR 701.70a).
//! * To empower Jace N: if you don't control a Jace planeswalker token, create a blue Jace
//!   planeswalker token with 0 loyalty, "[−1]: Surveil 1," and "[−3]: Draw a card."; then
//!   choose a Jace planeswalker token you control and put N loyalty counters on it
//!   (CR 701.71a).

use super::*;
use crate::types::counters;

fn token(
    colors: ColorSet,
    types: Vec<CardType>,
    subtypes: &[&str],
    pt: Option<(i32, i32)>,
) -> TokenSpec {
    TokenSpec {
        name: SmolStr::default(),
        colors,
        supertypes: vec![],
        card_types: types,
        subtypes: subtypes.iter().map(|s| SmolStr::new(s)).collect(),
        power: pt.map(|x| x.0),
        toughness: pt.map(|x| x.1),
        abilities: vec![],
        scryfall_name: None,
    }
}

fn create(g: &mut Game, p: PlayerId, spec: TokenSpec, ctx: &Ctx) -> Vec<ObjectId> {
    let mut c = ctx.clone();
    c.controller = p;
    g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut c,
    );
    c.var_objects(vars::CREATED)
}

/// `p` recruits (CR 701.70a). Returns true if a nonland card was discarded.
pub fn recruit(g: &mut Game, p: PlayerId, ctx: &Ctx) -> bool {
    g.draw_cards(p, 1);
    let hand = g.player(p).hand.clone();
    let pick = g.ask_objects(p, ctx.source, "Recruit: discard a card", hand, 1, 1);
    let mut nonland = false;
    for c in pick {
        let is_land = g.obj(c).chars.is_land();
        if g.discard(p, c, ctx.source).is_some() && !is_land {
            nonland = true;
        }
    }
    if nonland {
        let soldier = token(
            ColorSet::single(Color::White),
            vec![CardType::Creature],
            &["Human", "Soldier"],
            Some((1, 1)),
        );
        create(g, p, soldier, ctx);
    }
    nonland
}

pub struct Recruit;

impl KeywordActionRules for Recruit {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Recruit]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        for p in g.eval_players(a.who, ctx) {
            recruit(g, p, ctx);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&Recruit) }

/// The Jace planeswalker token (CR 701.71a).
pub fn jace_token() -> TokenSpec {
    let loyalty = |n: i32, effect: Effect, text: &str| {
        let mut act = ActivatedAbility::new(
            Cost {
                mana: None,
                parts: vec![CostPart::Loyalty(n)],
            },
            Body::effect(effect),
        );
        act.is_loyalty = true;
        AbilityDef::new(AbilityKind::Activated(act), text)
    };
    let mut t = token(
        ColorSet::single(Color::Blue),
        vec![CardType::Planeswalker],
        &["Jace"],
        None,
    );
    t.abilities = vec![
        loyalty(
            -1,
            Effect::Surveil {
                who: PlayerRef::You,
                n: Value::c(1),
            },
            "[−1]: Surveil 1.",
        ),
        loyalty(
            -3,
            Effect::Draw {
                who: PlayerRef::You,
                n: Value::c(1),
            },
            "[−3]: Draw a card.",
        ),
    ];
    t
}

fn jace_tokens(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.permanents()
        .filter(|o| {
            o.controller == p
                && o.is_token()
                && o.is(CardType::Planeswalker)
                && o.chars.has_subtype("Jace")
        })
        .map(|o| o.id)
        .collect()
}

/// `p` empowers Jace N (CR 701.71a). Returns the chosen Jace token.
pub fn empower_jace(g: &mut Game, p: PlayerId, n: u32, ctx: &Ctx) -> Option<ObjectId> {
    if g.dirty {
        g.recompute();
    }
    if jace_tokens(g, p).is_empty() {
        create(g, p, jace_token(), ctx);
        g.recompute();
    }
    let cands = jace_tokens(g, p);
    let jace = match cands.as_slice() {
        [] => return None,
        [one] => *one,
        _ => g
            .ask_objects(
                p,
                ctx.source,
                "Empower Jace: choose a Jace token",
                cands.clone(),
                1,
                1,
            )
            .first()
            .copied()
            .unwrap_or(cands[0]),
    };
    g.add_counters(Entity::Object(jace), counters::LOYALTY, n, ctx.source);
    Some(jace)
}

pub struct EmpowerJace;

impl KeywordActionRules for EmpowerJace {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::EmpowerJace]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        for p in g.eval_players(a.who, ctx) {
            empower_jace(g, p, n, ctx);
        }
    }
}

inventory::submit! { KeywordActionRegistration(&EmpowerJace) }
