//! Shared helpers for the tests of rulings batch S01 (`r_s01_*.rs`): adamant, adapt,
//! addendum, affinity, afflict, afterlife, aftermath, airbend, alliance, amass,
//! annihilator, ascend, assist, augment, awaken.

#![allow(dead_code)]

use mtg_engine::card::{card, CardDef, Layout};
use mtg_engine::decision::Decision;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// Asserts that the real card's oracle text compiled completely, so the test exercises
/// the card as printed.
pub fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// A custom card compiled from oracle text with the real oracle compiler (asserting it
/// compiled completely).
pub fn custom_card(
    name: &str,
    type_line: &str,
    cost: &str,
    pt: Option<(i32, i32)>,
    text: &str,
) -> CardDef {
    let tl = TypeLine::parse(type_line);
    let p = pt.map(|x| x.0.to_string());
    let tt = pt.map(|x| x.1.to_string());
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: p.as_deref(),
        toughness: tt.as_deref(),
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(
        compiled.unsupported.is_empty(),
        "{name}: unsupported text {:?}",
        compiled.unsupported
    );
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        mana_cost: mtg_engine::mana::ManaCost::parse(cost),
        supertypes: tl.supertypes,
        card_types: tl.card_types,
        subtypes: tl.subtypes.into_iter().collect(),
        abilities: compiled.abilities,
        power: pt.map(|x| x.0),
        toughness: pt.map(|x| x.1),
        rules_text: Arc::from(text),
        ..Default::default()
    })
}

/// Puts real cards on top of `p`'s library, the first one on top.
pub fn stack_library(t: &mut TestGame, p: PlayerId, top_first: &[&str]) -> Vec<ObjectId> {
    top_first
        .iter()
        .rev()
        .map(|n| t.library_top(p, n))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

/// Lands for a spell's mana cost (basic lands for each colored symbol, Wastes for generic).
pub fn give_mana_for(t: &mut TestGame, p: PlayerId, name: &str) {
    let c = card(name);
    let cost = c.front().chars.mana_cost.clone().unwrap_or_default();
    let text = format!("{cost}");
    let mut generic = 0usize;
    for sym in text.split('}').filter_map(|s| s.strip_prefix('{')) {
        let land = match sym {
            "W" => "Plains",
            "U" => "Island",
            "B" => "Swamp",
            "R" => "Mountain",
            "G" => "Forest",
            n => {
                generic += n.parse::<usize>().unwrap_or(1);
                continue;
            }
        };
        t.lands(p, land, 1);
    }
    t.lands(p, "Wastes", generic);
}

/// Permanents on the battlefield controlled by `p` with the given subtype.
pub fn with_subtype(t: &TestGame, p: PlayerId, subtype: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype(subtype))
        .map(|o| o.id)
        .collect()
}

/// Tokens on the battlefield controlled by `p`.
pub fn tokens(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token())
        .map(|o| o.id)
        .collect()
}

/// Creatures on the battlefield controlled by `p`.
pub fn creatures(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Creature))
        .map(|o| o.id)
        .collect()
}

/// Whether any lands `p` controls are tapped.
pub fn tapped_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land) && o.tapped)
        .count()
}

/// The decisions asked since index `from` of the decision log.
pub fn asked_since(t: &TestGame, from: usize) -> Vec<(PlayerId, Decision)> {
    t.asked()[from..].to_vec()
}

/// Puts the card `name` into `zone` for `p` (a real card).
pub fn place(t: &mut TestGame, p: PlayerId, name: &str, zone: Zone) -> ObjectId {
    t.custom(p, (*card(name)).clone(), zone)
}

/// Number of triggered abilities on the stack whose text contains `text`.
pub fn triggers_on_stack(t: &TestGame, text: &str) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id).stack.as_ref().is_some_and(|si| {
                matches!(&si.kind, mtg_engine::object::StackKind::Triggered { ability, .. }
                    if ability.text.contains(text))
            })
        })
        .count()
}

/// Game state seen by one of a player's decisions (see [`watch`]).
pub type Seen<T> = Arc<std::sync::Mutex<Vec<T>>>;

/// Wraps `p`'s agent: whenever `p` is asked a decision for which `when` holds, `look`
/// records something about the game at that moment; the answer is still the scripted one.
pub fn watch<T: Send + 'static>(
    t: &mut TestGame,
    p: PlayerId,
    when: fn(&mtg_engine::decision::Decision) -> bool,
    look: fn(&mtg_engine::game::Game) -> T,
) -> Seen<T> {
    struct Watch<T> {
        inner: Box<dyn mtg_engine::decision::Agent>,
        when: fn(&mtg_engine::decision::Decision) -> bool,
        look: fn(&mtg_engine::game::Game) -> T,
        seen: Seen<T>,
    }
    impl<T: Send> mtg_engine::decision::Agent for Watch<T> {
        fn decide(
            &mut self,
            g: &mtg_engine::game::Game,
            p: PlayerId,
            d: &mtg_engine::decision::Decision,
        ) -> mtg_engine::decision::Answer {
            if (self.when)(d) {
                self.seen.lock().unwrap().push((self.look)(g));
            }
            self.inner.decide(g, p, d)
        }
    }
    let seen: Seen<T> = Default::default();
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(Watch {
        inner,
        when,
        look,
        seen: seen.clone(),
    });
    seen
}

/// Declares attackers for the active player and advances into the declare attackers step
/// (attackers declared, triggers on the stack, no blocks yet).
pub fn attack_with(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    use mtg_engine::decision::Answer;
    use mtg_engine::turn::{Stage, Step};
    let ap = t.g.turn.active;
    if t.g.turn.step != Step::BeginningOfCombat {
        t.set_step(ap, Step::BeginningOfCombat);
    }
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareAttackers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "attackers not declared");
    t.settle();
}

/// Finishes combat from the declare attackers step with the given blocks, advancing to
/// the end of combat step.
pub fn block_and_finish(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    use mtg_engine::decision::Answer;
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    let ap = t.g.turn.active;
    t.advance_to(ap, mtg_engine::turn::Step::EndOfCombat);
}
