//! Splice (CR 702.47): as a spell is cast, its controller may reveal cards with splice
//! from their hand. The spell gains the rules text of each revealed card, following its
//! own (a text-changing effect, CR 612.10, 702.47c), and each splice cost is paid as an
//! additional cost (CR 601.2b, 601.2f–h).

use crate::ability::*;
use crate::casting::add_cost;
use crate::eval::Ctx;
use crate::game::*;
use crate::keywords::KeywordKind;
use crate::types::*;
use crate::{Answer, Decision};
use smol_str::SmolStr;

/// CR 601.2b, 702.47a–b: offers `p` to splice cards from their hand onto `spell`, which
/// they're casting with the additional costs `extra` so far. Returns the splice costs to
/// add to the spell's total cost.
pub fn offer_splices(g: &mut Game, p: PlayerId, spell: ObjectId, extra: &Cost) -> Vec<Cost> {
    let spell_chars = g.obj(spell).chars.clone();
    // What will be paid so far: the mana cost plus the additional costs.
    let mut committed = extra.clone();
    if let Some(m) = &spell_chars.mana_cost {
        add_cost(&mut committed, &Cost::mana(m.clone()));
    }
    let spell_ctx = Ctx::new(Some(spell), p);
    let mut chosen: Vec<(ObjectId, Cost)> = Vec::new();
    for card in g.player(p).hand.clone() {
        let chars = g.obj(card).chars.clone();
        let Some(kw) = chars
            .keywords()
            .find(|k| k.kind == KeywordKind::Splice)
            .cloned()
        else {
            continue;
        };
        let quality = kw.filter.clone().unwrap_or(Filter::Any);
        if !g.matches(spell, &quality, &Ctx::new(Some(card), p)) {
            continue;
        }
        // CR 702.47b: the card's required choices (targets) must be possible.
        let targets_ok = chars.abilities.iter().all(|a| match &a.kind {
            AbilityKind::Spell(s) => {
                s.body.modal.is_some() || g.targets_possible(&s.body.targets, &spell_ctx, spell)
            }
            _ => true,
        });
        let cost = kw.cost.clone().unwrap_or_else(Cost::free);
        let mut with = committed.clone();
        add_cost(&mut with, &cost);
        if !targets_ok || !g.can_pay_cost_optimistic(p, &with, Some(spell), &spell_chars) {
            continue;
        }
        let yes = matches!(
            g.ask(
                p,
                Decision::OptionalCost {
                    source: card,
                    name: "splice".to_string(),
                    repeatable: false,
                },
            ),
            Answer::Bool(true)
        );
        if yes {
            committed = with;
            chosen.push((card, cost));
        }
    }
    if chosen.is_empty() {
        return vec![];
    }
    // CR 702.47b: the cards are revealed all at once, and their controller chooses the
    // order in which their effects happen (after the main spell's).
    if chosen.len() > 1 {
        let names: Vec<String> = chosen
            .iter()
            .map(|(c, _)| g.obj(*c).chars.name.to_string())
            .collect();
        let order = g.ask_order(p, "Order the effects of the spliced cards", names);
        let old = std::mem::take(&mut chosen);
        chosen = order.into_iter().map(|i| old[i].clone()).collect();
    }
    for (card, _) in &chosen {
        g.emit(crate::events::Event::Custom {
            name: "revealed".into(),
            player: Some(p),
            obj: Some(*card),
            amount: 0,
        });
    }
    // CR 702.47c: the spell gains the rules text of each spliced card, after its own;
    // the main spell's effects happen first (CR 702.47b). The effect ends when the spell
    // leaves the stack (CR 702.47e), as it then becomes a new object.
    let mut abilities = Vec::new();
    let mut text: Vec<String> = Vec::new();
    for (card, _) in &chosen {
        let c = &g.obj(*card).chars;
        abilities.extend(c.abilities.iter().cloned());
        text.push(c.rules_text.to_string());
        g.log(|g| format!("{} splices {}", p, g.describe(*card)));
    }
    let id = g.new_effect_id();
    let ts = g.new_timestamp();
    g.effects.push(ContinuousEffect {
        id,
        source: Some(spell),
        controller: p,
        timestamp: ts,
        duration: Duration::Permanent,
        affected: Affected::Objects(vec![spell]),
        mods: vec![Modification::AddText {
            abilities,
            text: SmolStr::new(text.join("\n")),
        }],
        layer1: None,
        created_turn: g.turn.number,
    });
    g.recompute();
    chosen.into_iter().map(|(_, c)| c).collect()
}

/// Whether a continuous effect is the text a spell gained from cards spliced onto it.
fn is_splice_effect(e: &ContinuousEffect, spell: ObjectId) -> bool {
    e.source == Some(spell)
        && matches!(&e.affected, Affected::Objects(v) if v.as_slice() == [spell])
        && e.mods
            .iter()
            .all(|m| matches!(m, Modification::AddText { .. }))
}

/// A copy of a spell copies the choices made while casting it, including the cards
/// spliced onto it (CR 707.10): the copy gains the same text.
pub fn copy_splices(g: &mut Game, from: ObjectId, to: ObjectId) {
    let spliced: Vec<ContinuousEffect> = g
        .effects
        .iter()
        .filter(|e| is_splice_effect(e, from))
        .cloned()
        .collect();
    for mut e in spliced {
        e.id = g.new_effect_id();
        e.source = Some(to);
        e.affected = Affected::Objects(vec![to]);
        g.effects.push(e);
        g.dirty = true;
    }
}
