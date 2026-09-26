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
//!
//! "You may collect evidence N rather than pay the mana cost for spells you cast"
//! ([`ALT_COST_PREFIX`], Conspiracy Unraveler) is an alternative cost for the spells its
//! controller casts from their hand ([`EvidenceInsteadOfMana`]); paying it isn't the
//! linked additional cost, so "if evidence was collected" stays false.

use super::*;

/// `Event::Custom` name reported when a player collects evidence (the amount is N).
pub const COLLECTED_EVIDENCE: &str = "collect evidence";
/// `Event::Custom` name reported when a player forages.
pub const FORAGED: &str = "forage";
/// The name of the optional additional cost "you may collect evidence N" (CR 701.59c).
pub const EVIDENCE_COST: &str = "collect evidence";
/// `StaticEffect::Custom` name prefix, followed by N: "You may collect evidence N rather
/// than pay the mana cost for spells you cast."
pub const ALT_COST_PREFIX: &str = "collect evidence instead of mana cost:";
/// The `CastMethod::Alternative` id of that alternative cost.
pub const ALT_COST_METHOD: u64 = 0x0E1D_E4CE_0701_0059;

fn total_mana_value(g: &Game, cards: &[ObjectId]) -> u32 {
    cards.iter().map(|c| g.mana_value_of(*c)).sum()
}

/// The cards in `p`'s graveyard that could be exiled to collect evidence, other than
/// `excluded`.
fn evidence(g: &Game, p: PlayerId, excluded: &[ObjectId]) -> Vec<ObjectId> {
    g.player(p)
        .graveyard
        .iter()
        .copied()
        .filter(|c| !excluded.contains(c))
        .collect()
}

/// Whether `p` could collect evidence N (CR 701.59b).
pub fn can_collect_evidence(g: &Game, p: PlayerId, n: u32) -> bool {
    can_collect_evidence_without(g, p, n, &[])
}

/// Whether `p` could collect evidence N without exiling the cards `excluded` (cards the
/// same instruction exiles otherwise: "exile it and collect evidence 4").
pub fn can_collect_evidence_without(g: &Game, p: PlayerId, n: u32, excluded: &[ObjectId]) -> bool {
    total_mana_value(g, &evidence(g, p, excluded)) >= n
}

/// `p` collects evidence N (CR 701.59a): they choose cards in their graveyard with total
/// mana value N or greater and exile them. Returns false (doing nothing) if they can't.
pub fn collect_evidence(g: &mut Game, p: PlayerId, n: u32, src: Option<ObjectId>) -> bool {
    collect_evidence_without(g, p, n, src, &[])
}

/// `p` collects evidence N without exiling the cards `excluded`.
fn collect_evidence_without(
    g: &mut Game,
    p: PlayerId,
    n: u32,
    src: Option<ObjectId>,
    excluded: &[ObjectId],
) -> bool {
    if !can_collect_evidence_without(g, p, n, excluded) {
        return false;
    }
    let gy = evidence(g, p, excluded);
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

    /// "Collect evidence N". `what`, if any, are cards that can't be exiled for it,
    /// because the same instruction exiles them otherwise ("you may exile it and collect
    /// evidence 4": it can't also be evidence).
    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let n = number(g, a.n, ctx);
        let excluded = excluded(g, a, ctx);
        let mut all = true;
        for p in g.eval_players(a.who, ctx) {
            all &= collect_evidence_without(g, p, n, ctx.source, &excluded);
        }
        // "If you do" (CR 701.59b: it may be impossible).
        ctx.prev_happened = all;
    }

    fn can_choose(&self, g: &Game, a: &Args, ctx: &Ctx) -> bool {
        let n = number(g, a.n, ctx);
        let excluded = excluded(g, a, ctx);
        g.eval_players(a.who, ctx)
            .into_iter()
            .all(|p| can_collect_evidence_without(g, p, n, &excluded))
    }
}

/// The cards an instruction to collect evidence can't exile (its `what`).
fn excluded(g: &Game, a: &Args, ctx: &Ctx) -> Vec<ObjectId> {
    if matches!(a.what, Sel::None) {
        return vec![];
    }
    g.eval_sel(a.what, ctx)
        .into_iter()
        .filter_map(|e| e.object())
        .map(|o| g.current(o))
        .collect()
}

inventory::submit! { KeywordActionRegistration(&CollectEvidence) }

pub struct Forage;

impl KeywordActionRules for Forage {
    fn actions(&self) -> &'static [KeywordAction] {
        &[KeywordAction::Forage]
    }

    fn perform(&self, g: &mut Game, a: &Args, ctx: &mut Ctx) {
        let mut all = true;
        for p in g.eval_players(a.who, ctx) {
            all &= forage(g, p, ctx.source);
        }
        // "If you do": a player who can do neither doesn't forage.
        ctx.prev_happened = all;
    }

    fn can_choose(&self, g: &Game, a: &Args, ctx: &Ctx) -> bool {
        g.eval_players(a.who, ctx)
            .into_iter()
            .all(|p| can_forage(g, p))
    }
}

inventory::submit! { KeywordActionRegistration(&Forage) }

/// Casting spells from hand by collecting evidence rather than paying their mana cost.
pub struct EvidenceInsteadOfMana;

impl crate::kw::KeywordRules for EvidenceInsteadOfMana {
    fn kinds(&self) -> &'static [crate::keywords::KeywordKind] {
        &[]
    }

    fn global_cast_options(
        &self,
        g: &Game,
        p: PlayerId,
        card: ObjectId,
    ) -> Vec<crate::casting::CastOption> {
        if g.obj(card).zone != Zone::Hand(p) {
            return vec![];
        }
        let Some(n) = g
            .statics
            .customs
            .iter()
            .filter(|(_, ctl, _)| *ctl == p)
            .filter_map(|(_, _, name)| name.strip_prefix(ALT_COST_PREFIX)?.parse::<u32>().ok())
            .min()
        else {
            return vec![];
        };
        let mut opt = crate::casting::CastOption::normal(crate::object::FaceState::Front);
        opt.method = crate::object::CastMethod::Alternative(ALT_COST_METHOD);
        opt.alt_cost = Some(Cost::free().with(CostPart::CollectEvidence(n)));
        vec![opt]
    }
}

inventory::submit! { crate::kw::KeywordRegistration(&EvidenceInsteadOfMana) }
