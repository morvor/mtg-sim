//! CR 702.85 Cascade.
//!
//! "Cascade" means "When you cast this spell, exile cards from the top of your library
//! until you exile a nonland card whose mana value is less than this spell's mana value.
//! You may cast that card without paying its mana cost if the resulting spell's mana value
//! is less than this spell's mana value. Then put all cards exiled this way that weren't
//! cast on the bottom of your library in a random order." (CR 702.85a). It's a triggered
//! ability that functions while the spell is on the stack; each instance triggers
//! separately (CR 702.85c).
//!
//! Actions a player may take "as you cascade" (CR 702.85b) are static abilities of
//! permanents that player controls ([`StaticEffect::Custom`] [`AS_YOU_CASCADE_LAND`]):
//! they're taken after the last card is exiled, before choosing whether to cast it.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::casting::CastOption;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;

/// `Effect::Custom` of the cascade trigger.
const CASCADE: &str = "cascade";

/// `StaticEffect::Custom`: "As you cascade, you may put a land card from among the exiled
/// cards onto the battlefield tapped." (Averna, the Chaos Bloom).
pub const AS_YOU_CASCADE_LAND: &str =
    "cascade:as you cascade, put a land card from among the exiled cards onto the battlefield tapped";

pub struct Cascade;

impl KeywordRules for Cascade {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Cascade]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut t = TriggeredAbility::new(
            TriggerCond::CastSpell {
                who: PlayerRel::You,
                filter: Filter::Source,
            },
            Body::effect(Effect::Custom(SmolStr::new(CASCADE))),
        );
        t.zone = FunctionZone::Stack;
        Some(vec![AbilityDef::new(
            AbilityKind::Triggered(t),
            KeywordKind::Cascade.name(),
        )])
    }

    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        if name != CASCADE {
            return false;
        }
        cascade(g, ctx);
        true
    }
}

/// The ways `card` could be cast as a spell with mana value less than `mv` (CR 702.85a:
/// "if the resulting spell's mana value is less"), without paying its mana cost: one per
/// face or half it could be cast as, with a description.
fn cheaper_options(g: &Game, card: ObjectId, mv: u32) -> Vec<(String, CastOption)> {
    let layout = g.obj(card).card.as_ref().map(|d| d.layout);
    let faces: Vec<FaceState> = match layout {
        Some(crate::card::Layout::Split) => vec![FaceState::Half(0), FaceState::Half(1)],
        Some(crate::card::Layout::Adventure) => vec![FaceState::Front, FaceState::Half(1)],
        Some(crate::card::Layout::ModalDfc) => vec![FaceState::Front, FaceState::Back],
        _ => vec![FaceState::Front],
    };
    let mut out = Vec::new();
    for face in faces {
        let mut opt = CastOption::normal(face);
        opt.method = CastMethod::Free;
        opt.alt_cost = Some(Cost::free());
        opt.any_time = true;
        let chars = g.option_characteristics(card, &opt);
        if chars.is_land() {
            continue;
        }
        // Cast without paying its mana cost, X is 0 (CR 107.3b).
        let spell_mv = chars.mana_cost.as_ref().map_or(0, |m| m.mana_value_with_x(0));
        if spell_mv < mv {
            out.push((format!("Cast {} (mana value {spell_mv})", chars.name), opt));
        }
    }
    out
}

fn cascade(g: &mut Game, ctx: &mut Ctx) {
    let p = ctx.controller;
    // "This spell": as it last existed on the stack if it has left it (it still cascades
    // if it was countered).
    let Some(spell) = ctx.source else {
        return;
    };
    let mv = g.mana_value_of(spell);
    // Exile cards one at a time, face up, until a nonland card with lesser mana value.
    let mut exiled: Vec<ObjectId> = Vec::new();
    let mut hit: Option<ObjectId> = None;
    let mut left = g.player(p).library.len();
    while let Some(top) = crate::library::top_cards(g, p, 1).first().copied() {
        if left == 0 {
            break;
        }
        left -= 1;
        let Some(card) = g.move_object(top, Zone::Exile, MoveCause::Effect, Some(p)) else {
            break;
        };
        if g.obj(card).zone != Zone::Exile {
            continue;
        }
        exiled.push(card);
        if !g.obj(card).chars.is_land() && g.mana_value_of(card) < mv {
            hit = Some(card);
            break;
        }
    }
    let names: Vec<String> = exiled.iter().map(|c| g.describe(*c)).collect();
    g.log(|_| format!("{p} cascades, exiling {}", names.join(", ")));
    // CR 702.85b: actions taken "as you cascade".
    as_you_cascade(g, p, &exiled);
    if let Some(card) = hit.filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Exile) {
        let mut options = cheaper_options(g, card, mv);
        if !options.is_empty() {
            let mut labels: Vec<String> = options.iter().map(|(l, _)| l.clone()).collect();
            labels.push("Don't cast it".into());
            let i = if labels.len() == 2 {
                let prompt = format!("Cast {} without paying its mana cost?", g.describe(card));
                if g.ask_yes_no(p, Some(card), &prompt, true) {
                    0
                } else {
                    1
                }
            } else {
                g.ask_option(p, Some(card), "Cascade: cast the exiled card?", labels)
            };
            if i < options.len() {
                let (_, opt) = options.swap_remove(i);
                let _ = g.cast_with_option(p, card, opt);
            }
        }
    }
    let rest: Vec<ObjectId> = exiled
        .into_iter()
        .filter(|c| g.is_live(*c) && g.obj(*c).zone == Zone::Exile)
        .collect();
    let mut d = Destination::zone(ZoneKind::Library);
    d.position = LibraryPosition::BottomRandom;
    g.move_to_destination(rest, &d, ctx);
}

/// CR 702.85b: "as you cascade" actions of permanents `p` controls, taken after the last
/// card is exiled and before deciding whether to cast it.
fn as_you_cascade(g: &mut Game, p: PlayerId, exiled: &[ObjectId]) {
    let sources: Vec<(ObjectId, usize)> = g
        .permanents()
        .filter(|o| o.controller == p)
        .map(|o| {
            let n = o
                .chars
                .abilities
                .iter()
                .filter(|a| {
                    matches!(&a.kind, AbilityKind::Static(s)
                        if matches!(&s.effect, StaticEffect::Custom(n) if n == AS_YOU_CASCADE_LAND))
                })
                .count();
            (o.id, n)
        })
        .filter(|(_, n)| *n > 0)
        .collect();
    for (src, n) in sources {
        for _ in 0..n {
            let lands: Vec<ObjectId> = exiled
                .iter()
                .copied()
                .filter(|c| {
                    g.is_live(*c) && g.obj(*c).zone == Zone::Exile && g.obj(*c).chars.is_land()
                })
                .collect();
            if lands.is_empty() {
                return;
            }
            let pick = g.ask_objects(
                p,
                Some(src),
                "As you cascade, put a land card onto the battlefield tapped?",
                lands,
                0,
                1,
            );
            let mut ctx = Ctx::new(Some(src), p);
            g.move_to_destination(pick, &Destination::battlefield().tapped(), &mut ctx);
        }
    }
}

inventory::submit! { KeywordRegistration(&Cascade) }
