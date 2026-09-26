//! CR 701.59: collect evidence; CR 701.61: forage.
//!
//! * To collect evidence N, a player exiles any number of cards from their graveyard with
//!   total mana value N or greater (CR 701.59a); the player chooses the cards.
//! * A player who can't exile cards with total mana value N or greater can't choose to
//!   collect evidence (CR 701.59b), whether it's optional or a cost
//!   (`CostPart::CollectEvidence`, or the optional additional cost [`EVIDENCE_COST`]).
//! * "If evidence was collected" refers to the linked optional additional cost
//!   (CR 701.59c): `Condition::CostPaid(EVIDENCE_COST)`.
//! * To forage means "Exile three cards from your graveyard or sacrifice a Food"
//!   (CR 701.61a).
//!
//! Both report an event (`Event::Custom`) for "whenever you collect evidence" / "whenever
//! you forage" triggers.

use super::*;

/// `Event::Custom` name reported when a player collects evidence (the amount is N).
pub const COLLECTED_EVIDENCE: &str = "collect evidence";
/// `Event::Custom` name reported when a player forages.
pub const FORAGED: &str = "forage";
/// The name of the optional additional cost "you may collect evidence N" (CR 701.59c).
pub const EVIDENCE_COST: &str = "collect evidence";

fn total_mana_value(g: &Game, cards: &[ObjectId]) -> u32 {
    cards.iter().map(|c| g.mana_value_of(*c)).sum()
}

/// Whether `p` could collect evidence N (CR 701.59b).
pub fn can_collect_evidence(g: &Game, p: PlayerId, n: u32) -> bool {
    total_mana_value(g, &g.player(p).graveyard) >= n
}

/// `p` collects evidence N (CR 701.59a): they choose cards in their graveyard with total
/// mana value N or greater and exile them. Returns false (doing nothing) if they can't.
pub fn collect_evidence(g: &mut Game, p: PlayerId, n: u32, src: Option<ObjectId>) -> bool {
    if !can_collect_evidence(g, p, n) {
        return false;
    }
    let gy = g.player(p).graveyard.clone();
    let picked = g.ask_objects(
        p,
        src,
        &format!("Collect evidence {n}: exile cards with total mana value {n} or greater"),
        gy.clone(),
        u32::from(n > 0),
        gy.len() as u32,
    );
    let mut distinct = picked.clone();
    distinct.sort();
    distinct.dedup();
    let valid = distinct.len() == picked.len()
        && picked.iter().all(|c| gy.contains(c))
        && total_mana_value(g, &picked) >= n;
    let chosen = if valid {
        picked
    } else {
        // Fall back to the cards with the highest mana values.
        let mut sorted = gy;
        sorted.sort_by_key(|c| std::cmp::Reverse(g.mana_value_of(*c)));
        let mut chosen = Vec::new();
        let mut sum = 0;
        for c in sorted {
            if sum >= n {
                break;
            }
            sum += g.mana_value_of(c);
            chosen.push(c);
        }
        chosen
    };
    for c in chosen {
        g.exile_object(c, src);
    }
    g.log(|_| format!("{p} collects evidence {n}"));
    emit(g, COLLECTED_EVIDENCE, p, None, n as i32);
    true
}

fn foods(g: &Game, p: PlayerId) -> Vec<ObjectId> {
    g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Food"))
        .map(|o| o.id)
        .collect()
}

/// Whether `p` could forage.
pub fn can_forage(g: &Game, p: PlayerId) -> bool {
    g.player(p).graveyard.len() >= 3 || !foods(g, p).is_empty()
}

/// `p` forages (CR 701.61a): exiles three cards from their graveyard or sacrifices a
/// Food. Returns false (doing nothing) if they can do neither.
pub fn forage(g: &mut Game, p: PlayerId, src: Option<ObjectId>) -> bool {
    if g.dirty {
        g.recompute();
    }
    let foods = foods(g, p);
    let gy = g.player(p).graveyard.clone();
    let can_exile = gy.len() >= 3;
    if !can_exile && foods.is_empty() {
        return false;
    }
    let use_food = if can_exile && !foods.is_empty() {
        g.ask_option(
            p,
            src,
            "Forage",
            vec![
                "Exile three cards from your graveyard".into(),
                "Sacrifice a Food".into(),
            ],
        ) == 1
    } else {
        !can_exile
    };
    if use_food {
        let f = g
            .ask_objects(p, src, "Choose a Food to sacrifice", foods.clone(), 1, 1)
            .first()
            .copied()
            .filter(|f| foods.contains(f))
            .unwrap_or(foods[0]);
        g.sacrifice(f, p);
    } else {
        let mut pick = g.ask_objects(p, src, "Choose three cards to exile", gy.clone(), 3, 3);
        pick.sort();
        pick.dedup();
        if pick.len() != 3 || !pick.iter().all(|c| gy.contains(c)) {
            pick = gy[..3].to_vec();
        }
        for c in pick {
            g.exile_object(c, src);
        }
    }
    g.log(|_| format!("{p} forages"));
    emit(g, FORAGED, p, None, 0);
    true
}

pub struct CollectEvidence;

impl KeywordActionRules for CollectEvidence {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::CollectEvidence]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        for p in g.eval_players(a.who, ctx) {
            collect_evidence(g, p, n, ctx.source);
        }
    }

    fn can_choose(&self, g: &Game, a: &Args, ctx: &Ctx) -> bool {
        let n = number(g, a.n, ctx);
        g.eval_players(a.who, ctx)
            .into_iter()
            .all(|p| can_collect_evidence(g, p, n))
    }
}

inventory::submit! { KeywordActionRegistration(&CollectEvidence) }

pub struct Forage;

impl KeywordActionRules for Forage {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Forage]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        for p in g.eval_players(a.who, ctx) {
            forage(g, p, ctx.source);
        }
    }

    fn can_choose(&self, g: &Game, a: &Args, ctx: &Ctx) -> bool {
        g.eval_players(a.who, ctx)
            .into_iter()
            .all(|p| can_forage(g, p))
    }
}

inventory::submit! { KeywordActionRegistration(&Forage) }
