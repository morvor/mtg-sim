//! Bidding life (`bidding.rs`): the whole paragraph of a spell
//!
//! ```text
//! bidders ". you start the bidding with a bid of " (N | "any number")
//! ". in turn order, each player may top the high bid. the bidding ends if the high bid
//! stands." result*
//! bidders := "each player may bid life" [" for control of " target]
//!          | "you and target spell's controller bid life"
//! result  := "the high bidder " predicate        (with "the high bid" its amount)
//!          | "if you win the bidding, " instruction
//! ```
//!
//! (Illicit Auction, Pain's Reward, Mages' Contest.)

use super::AbilityPattern;
use crate::ability::*;
use crate::bidding::{effect_name, HIGH_BID, HIGH_BIDDER};
use crate::oracle::effects::{parse_effect_text, split_sentences, Builder};
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

const MIDDLE: &str =
    "in turn order, each player may top the high bid. the bidding ends if the high bid stands.";

fn high_bidder() -> PlayerRef {
    PlayerRef::Var(HIGH_BIDDER)
}

/// One instruction after the bidding.
fn result(s: &str, b: &mut Builder) -> Option<Effect> {
    let s = end(s);
    if let Some(r) = s.strip_prefix("if you win the bidding, ") {
        let r = r.replace("that spell", "it").replace("the creature", "it");
        let e = parse_effect_text(&r, b)?;
        return Some(Effect::If {
            cond: Condition::PlayerMatches(
                PlayerRef::You,
                PlayerFilter::Ref(Box::new(high_bidder())),
            ),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        });
    }
    let pred = s.strip_prefix("the high bidder ")?;
    let pred = pred
        .replace("loses life equal to the high bid", "loses x life")
        .replace("the creature", "it");
    let one = super::player_subjects::as_you(&pred)?;
    let e = parse_effect_text(&format!("you {one}"), b)?;
    Some(Effect::AsPlayer {
        who: high_bidder(),
        effect: Box::new(Effect::seq(vec![
            Effect::SetX {
                value: Value::Var(HIGH_BID),
            },
            e,
        ])),
    })
}

fn bidding(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_spell() || block.contains('\n') {
        return None;
    }
    let lower = block.to_lowercase();
    let (head, rest) = lower.split_once(&format!(" {MIDDLE}"))?;
    let (bidders, start) = head.split_once(". you start the bidding with a bid of ")?;
    let start = match end(start) {
        "any number" => None,
        n => match parse_number(n)? {
            (Value::Const(n), r) if end(r).is_empty() => Some(n as i64),
            _ => return None,
        },
    };
    let mut b = Builder::new(ctx);
    let each = if let Some(t) = bidders.strip_prefix("each player may bid life") {
        if let Some(target) = t.strip_prefix(" for control of ") {
            let (spec, tail) = parse_target(target)?;
            if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Object(_)) {
                return None;
            }
            let slot = b.add_target(spec, "target");
            b.it = Sel::Target(slot);
        } else if !t.is_empty() {
            return None;
        }
        true
    } else if bidders == "you and target spell's controller bid life" {
        let (spec, _) = parse_target("target spell")?;
        let slot = b.add_target(spec, "target spell");
        b.it = Sel::Target(slot);
        false
    } else {
        return None;
    };
    let mut effects = vec![Effect::Custom(effect_name(each, start))];
    for s in split_sentences(rest.trim()) {
        effects.push(result(&s.to_lowercase(), &mut b)?);
    }
    if effects.len() < 2 {
        return None;
    }
    let body = Body {
        targets: b.targets,
        effect: Effect::seq(effects),
        modal: None,
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Spell(SpellAbility { body }),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "bidding life", priority: 60, parse: bidding } }
