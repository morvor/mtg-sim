//! The grammar of activation restrictions (CR 602.5b, 602.5d, 602.5e, 602.2): a list of
//! clauses joined by "and only" / ", only" after "Activate only", "Any player may activate
//! this ability but only" or "Only your opponents may activate this ability and only":
//!
//! * timing: "as a sorcery", "as an instant", "during [a turn, step or phase]", "before
//!   [a point in combat]" ([`during`], [`combat_point`]);
//! * conditions: "if [condition]" (the condition grammar; "if A or if B" is either);
//! * limits: "once each turn", "twice each turn", "no more than three times each turn",
//!   "once" (over the object's existence, CR 400.7);
//! * who may activate: "Any player may activate this ability", "Only your opponents may
//!   activate this ability" (CR 602.2);
//! * "You can't activate this ability during combat."
//!
//! The same timing grammar serves "Cast ~ only during ..." (see `restrictions_timing.rs`).

use super::AbilityPattern;
use crate::ability::*;
use crate::game::Game;
use crate::oracle::CompileContext;

/// `Condition::Custom`: it's the draw step (CR 504).
pub const DRAW_STEP: &str = "activation:draw_step";
/// `Condition::Custom`: the turn hasn't reached its end step (CR 513) yet.
pub const BEFORE_END_STEP: &str = "activation:before_end_step";

/// Evaluates this module's custom conditions.
pub fn custom_condition(g: &Game, name: &str, _ctx: &crate::eval::Ctx) -> Option<bool> {
    use crate::turn::Step;
    match name {
        DRAW_STEP => Some(g.turn.step == Step::Draw),
        BEFORE_END_STEP => Some(!matches!(g.turn.step, Step::End | Step::Cleanup)),
        _ => None,
    }
}

/// One restriction clause.
#[derive(Clone, Debug)]
pub(crate) enum Clause {
    /// "as a sorcery" (CR 602.5d).
    Sorcery,
    /// "as an instant" (CR 602.5e).
    AsInstant,
    /// A timing ("during your upkeep"): the condition, and the activation timing that
    /// says the same thing, if there is one.
    When(Condition, Option<ActivationTiming>),
    /// "if [condition]".
    If(Condition),
    /// "once each turn", "no more than twice each turn".
    PerTurn(u32),
    /// "once".
    Total(u32),
}

fn and2(a: Condition, b: Condition) -> Condition {
    Condition::And(vec![a, b])
}

/// A point in combat after "before"/"after" (CR 506.8): "attackers are declared",
/// "blockers are declared", "the combat damage step", "the end of combat step", "combat".
pub(crate) fn combat_point(r: &str) -> Option<CombatPoint> {
    Some(match r {
        "combat" => CombatPoint::Combat,
        "attackers are declared" => CombatPoint::AttackersDeclared,
        "blockers are declared" => CombatPoint::BlockersDeclared,
        "the combat damage step" => CombatPoint::CombatDamageStep,
        "the end of combat step" => CombatPoint::EndOfCombatStep,
        _ => return None,
    })
}

fn combat_window(point: CombatPoint, after: bool, during_combat: bool) -> Clause {
    let t = CombatTiming {
        point,
        after,
        during_combat,
    };
    Clause::When(
        Condition::CombatTiming(t),
        Some(ActivationTiming::CombatWindow(t)),
    )
}

/// The text after "during" (or a whole "during ..., before ..." phrase without its
/// "during"): whose turn, which step or phase. "Your"/"their" is the player activating
/// or casting (CR 602.2), who is "you" as the condition is checked.
pub(crate) fn during(r: &str) -> Option<Clause> {
    let custom = |s: &str| Condition::Custom(s.into());
    let phase = Condition::Phase;
    // "during your turn, before attackers are declared": both (CR 506.8g).
    if let Some((a, b)) = r.split_once(", before ") {
        let Clause::When(turn, _) = during(a)? else {
            return None;
        };
        let point = combat_point(b)?;
        let t = CombatTiming {
            point,
            after: false,
            during_combat: false,
        };
        return Some(Clause::When(
            and2(turn, Condition::CombatTiming(t)),
            None,
        ));
    }
    // "during their turn before the end step".
    if let Some(a) = r.strip_suffix(" before the end step") {
        let Clause::When(turn, _) = during(a)? else {
            return None;
        };
        return Some(Clause::When(and2(turn, custom(BEFORE_END_STEP)), None));
    }
    // "during combat before blockers are declared" (CR 506.8b).
    if let Some((a, b)) = r.split_once(" before ").filter(|(a, _)| *a == "combat") {
        let _ = a;
        let point = combat_point(b)?;
        return Some(if point == CombatPoint::BlockersDeclared {
            Clause::When(
                Condition::CombatTiming(CombatTiming {
                    point,
                    after: false,
                    during_combat: true,
                }),
                Some(ActivationTiming::BeforeBlockers),
            )
        } else {
            combat_window(point, false, true)
        });
    }
    if let Some(b) = r.strip_prefix("combat after ") {
        return Some(combat_window(combat_point(b)?, true, true));
    }
    let (c, t) = match r {
        "your turn" | "their turn" => (Condition::YourTurn, Some(ActivationTiming::YourTurn)),
        "an opponent's turn" => (
            Condition::NotYourTurn,
            Some(ActivationTiming::OpponentsTurn),
        ),
        "your upkeep" | "their upkeep" => (
            and2(Condition::YourTurn, phase(PhaseCond::Upkeep)),
            Some(ActivationTiming::YourUpkeep),
        ),
        "any upkeep step" | "any upkeep" | "each upkeep" => (phase(PhaseCond::Upkeep), None),
        "an opponent's upkeep" => (and2(Condition::NotYourTurn, phase(PhaseCond::Upkeep)), None),
        "your draw step" | "their draw step" => {
            (and2(Condition::YourTurn, custom(DRAW_STEP)), None)
        }
        "your end step" | "their end step" => {
            (and2(Condition::YourTurn, phase(PhaseCond::EndStep)), None)
        }
        "combat" => (phase(PhaseCond::Combat), Some(ActivationTiming::Combat)),
        "the declare attackers step" => (phase(PhaseCond::DeclareAttackers), None),
        "the declare blockers step" => (
            custom(super::restrictions_timing::DECLARE_BLOCKERS_STEP),
            None,
        ),
        // "During the end of combat step": during combat, once that step has begun
        // (CR 506.8).
        "the end of combat step" => {
            return Some(combat_window(CombatPoint::EndOfCombatStep, true, true))
        }
        _ => return None,
    };
    Some(Clause::When(c, t))
}

/// An "if" clause: "if A or if B" is either condition.
fn if_clause(r: &str, ctx: &CompileContext) -> Option<Condition> {
    let parts: Vec<&str> = r.split(" or if ").collect();
    let mut conds = Vec::new();
    for p in &parts {
        // Conditions about where the source is change where the ability functions
        // (CR 113.6); they aren't conditions checked from its zone.
        if ["~ is ", "~ isn't ", "this card is "]
            .iter()
            .any(|x| p.contains(x))
            && ["graveyard", "hand", "exile", "library", "battlefield", "stack"]
                .iter()
                .any(|z| p.contains(z))
        {
            return None;
        }
        conds.push(condition(p, ctx)?);
    }
    Some(if conds.len() == 1 {
        conds.pop()?
    } else {
        Condition::Or(conds)
    })
}

/// A condition in a restriction: the condition grammar, where "it" is the source, and
/// "[object] is A and B" ("enchanted creature is white and untapped").
pub(crate) fn condition(c: &str, ctx: &CompileContext) -> Option<Condition> {
    let c = c.trim();
    if let Some(cond) = crate::oracle::statics::parse_condition(c, ctx) {
        return Some(cond);
    }
    if let Some((cond, _)) =
        super::statics_conditions::parse_static_condition(c, Some(&Sel::This), ctx)
    {
        return Some(cond);
    }
    let (subject, state) = c.split_once(" is ")?;
    let (a, b) = state.split_once(" and ")?;
    let ca = condition(&format!("{subject} is {a}"), ctx)?;
    let cb = condition(&format!("{subject} is {b}"), ctx)?;
    Some(Condition::And(vec![ca, cb]))
}

/// "you've been attacked this step" (CR 508.1).
fn attacked_this_step(c: &str) -> Option<Condition> {
    matches!(c, "you've been attacked this step" | "you were attacked this step").then(|| {
        Condition::Custom(super::restrictions_timing::ATTACKED_THIS_STEP.into())
    })
}

/// Conditions restrictions use about the source in combat, its abilities and hands:
/// "~ is blocked", "at least one creature is blocking ~", "~ doesn't have defender",
/// "you have seven or more cards in your hand", "a player has one or fewer cards in hand".
fn restriction_state(c: &str) -> Option<Condition> {
    use super::statics_conditions::amount_cmp;
    let c = crate::oracle::phrases::end(c);
    match c {
        "~ is blocked" => return Some(Condition::SelMatches(Sel::This, Filter::Blocked)),
        "~ is unblocked" => return Some(Condition::SelMatches(Sel::This, Filter::Unblocked)),
        "at least one creature is blocking ~" | "a creature is blocking ~" => {
            return Some(Condition::Exists(Filter::and(vec![
                Filter::Type(crate::types::CardType::Creature),
                Filter::BlockingSource,
            ])))
        }
        _ => {}
    }
    if let Some(k) = c
        .strip_prefix("~ doesn't have ")
        .or_else(|| c.strip_prefix("~ does not have "))
        .and_then(crate::keywords::KeywordKind::from_name)
    {
        return Some(Condition::Not(Box::new(Condition::SelMatches(
            Sel::This,
            Filter::HasKeyword(k),
        ))));
    }
    let hand = |r: &str| -> Option<(Cmp, Value)> {
        let (cmp, n, rest) = amount_cmp(r)?;
        matches!(rest.trim(), "cards in hand" | "cards in your hand" | "card in hand")
            .then_some((cmp, n))
    };
    if let Some((cmp, n)) = c.strip_prefix("you have ").and_then(hand) {
        return Some(Condition::Compare(Value::HandSize(PlayerRef::You), cmp, n));
    }
    if let Some((cmp, n)) = c.strip_prefix("a player has ").and_then(|r| {
        let (cmp, n, rest) = amount_cmp(r)?;
        matches!(rest.trim(), "cards in hand" | "card in hand" | "cards in their hand")
            .then_some((cmp, n))
    }) {
        return Some(Condition::Compare(
            Value::CountPlayers(PlayerFilter::HandSize(cmp, Box::new(n))),
            Cmp::Ge,
            Value::c(1),
        ));
    }
    None
}

/// "you've cast another spell this turn", "you've cast another green spell this turn"
/// (spells other than the one being cast, CR 601.2).
fn cast_another_spell(c: &str) -> Option<Condition> {
    let c = crate::oracle::phrases::end(c);
    match c {
        "you've cast another spell this turn" => {
            return Some(Condition::Custom(crate::spell_costs::CAST_ANOTHER_SPELL.into()))
        }
        "you've cast another instant or sorcery spell this turn" => {
            return Some(Condition::Custom(
                crate::spell_costs::CAST_ANOTHER_INSTANT_OR_SORCERY.into(),
            ))
        }
        _ => {}
    }
    let r = c
        .strip_prefix("you've cast another ")?
        .strip_suffix(" spell this turn")?;
    if r.is_empty() {
        return None;
    }
    let color = crate::types::Color::from_word(r)?;
    Some(Condition::Custom(
        format!("you_cast_another_spell_this_turn:{}", color.letter()).into(),
    ))
}

inventory::submit! { super::ConditionPattern { name: "activation restrictions: you've cast another [color] spell this turn", priority: 100, parse: cast_another_spell } }

/// A whole value phrase ("the number of Caves you control plus ..."), with no targets.
fn whole_value(s: &str) -> Option<Value> {
    let tl = crate::types::TypeLine::default();
    let ctx = CompileContext {
        card_name: "",
        full_name: "",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let mut b = crate::oracle::effects::Builder::new(&ctx);
    let (v, rest) = super::value_grammar::parse_value(s, &mut b)?;
    (rest.trim().is_empty() && b.targets.is_empty()).then_some(v)
}

/// Comparisons with a counted value: "the number of other Caves you control plus the
/// number of Cave cards in your graveyard is three or greater", "there are four or more
/// permanent types among cards in your graveyard", "you have exactly zero or seven cards
/// in hand".
fn value_comparison(c: &str) -> Option<Condition> {
    use super::statics_conditions::amount_cmp;
    let c = crate::oracle::phrases::end(c);
    // "you have exactly zero or seven cards in hand": either count.
    if let Some(r) = c
        .strip_prefix("you have exactly ")
        .and_then(|r| r.strip_suffix(" cards in hand"))
    {
        let (a, b) = r.split_once(" or ")?;
        let n = |w: &str| {
            if w == "zero" {
                return Some(Value::c(0));
            }
            let (v, rest) = crate::oracle::phrases::parse_number(w)?;
            rest.trim().is_empty().then_some(v)
        };
        let hand = || Value::HandSize(PlayerRef::You);
        return Some(Condition::Or(vec![
            Condition::Compare(hand(), Cmp::Eq, n(a)?),
            Condition::Compare(hand(), Cmp::Eq, n(b)?),
        ]));
    }
    if let Some(r) = c.strip_prefix("there are ") {
        if r.contains(" among ") {
            let (cmp, n, rest) = amount_cmp(r)?;
            let v = whole_value(&format!("the number of {}", rest.trim()))?;
            return Some(Condition::Compare(v, cmp, n));
        }
        return None;
    }
    let (v, rest) = c.rsplit_once(" is ")?;
    if !v.starts_with("the number of ") {
        return None;
    }
    let (cmp, n, tail) = amount_cmp(rest)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let v = whole_value(v)?;
    Some(Condition::Compare(v, cmp, n))
}

inventory::submit! { super::ConditionPattern { name: "activation restrictions: counted value comparisons", priority: 110, parse: value_comparison } }

inventory::submit! { super::ConditionPattern { name: "activation restrictions: source and hand states", priority: 100, parse: restriction_state } }

inventory::submit! { super::ConditionPattern { name: "activation restrictions: you've been attacked this step", priority: 100, parse: attacked_this_step } }

/// "Equip [cost]. Activate only once each turn." (CR 602.5b): the keyword, with the
/// restriction kept in its text (see `kw/equip.rs`).
fn equip_once_each_turn(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let head = t
        .strip_suffix(" Activate only once each turn.")
        .filter(|h| h.starts_with("Equip"))?;
    let mut abilities = crate::oracle::parse_ability(head.trim_end_matches('.'), ctx)?;
    if abilities.len() != 1 {
        return None;
    }
    let a = abilities.pop()?;
    let AbilityKind::Keyword(kw) = &a.kind else {
        return None;
    };
    if kw.kind != crate::keywords::KeywordKind::Equip {
        return None;
    }
    let kw = kw.clone().text(t);
    Some(crate::oracle::keywords::compile_keyword(kw, t))
}

inventory::submit! { AbilityPattern { name: "activation restrictions: equip, activate only once each turn", priority: 100, parse: equip_once_each_turn } }

fn times(w: &str) -> Option<u32> {
    Some(match w {
        "once" => 1,
        "twice" => 2,
        _ => {
            let n = w.strip_suffix(" times")?;
            crate::oracle::phrases::parse_number(n)
                .and_then(|(v, rest)| rest.trim().is_empty().then_some(v))?
                .as_const()? as u32
        }
    })
}

/// One clause after "only".
pub(crate) fn clause(s: &str, ctx: &CompileContext) -> Option<Clause> {
    let s = s.trim();
    match s {
        "as a sorcery" => return Some(Clause::Sorcery),
        "as an instant" => return Some(Clause::AsInstant),
        _ => {}
    }
    if let Some(r) = s.strip_prefix("during ") {
        return during(r);
    }
    if let Some(r) = s.strip_prefix("before ") {
        let point = combat_point(r)?;
        return Some(if point == CombatPoint::BlockersDeclared {
            Clause::When(
                Condition::CombatTiming(CombatTiming {
                    point,
                    after: false,
                    during_combat: false,
                }),
                Some(ActivationTiming::BeforeBlockers),
            )
        } else {
            combat_window(point, false, false)
        });
    }
    if let Some(r) = s.strip_prefix("after ") {
        return Some(combat_window(combat_point(r)?, true, false));
    }
    if let Some(r) = s.strip_prefix("if ") {
        return Some(Clause::If(if_clause(r, ctx)?));
    }
    if let Some(r) = s.strip_suffix(" each turn") {
        return Some(Clause::PerTurn(times(r)?));
    }
    Some(Clause::Total(times(s)?))
}

/// A list of clauses: "as a sorcery and only once each turn", "during the declare
/// blockers step, only if ..., and only once each turn".
pub(crate) fn clause_list(s: &str, ctx: &CompileContext) -> Option<Vec<Clause>> {
    let mut out = Vec::new();
    let mut rest = s.trim();
    loop {
        let next = [", and only ", " and only ", ", only "]
            .iter()
            .filter_map(|sep| rest.find(sep).map(|i| (i, sep.len())))
            .min();
        match next {
            Some((i, n)) => {
                out.push(clause(&rest[..i], ctx)?);
                rest = &rest[i + n..];
            }
            None => {
                out.push(clause(rest, ctx)?);
                return Some(out);
            }
        }
    }
}

/// Who may activate the ability.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Who {
    #[default]
    Controller,
    AnyPlayer,
    Opponents,
}

/// A restriction sentence (lowercase, with or without its period): its clauses and who
/// may activate the ability.
pub(crate) fn restriction_sentence(s: &str, ctx: &CompileContext) -> Option<(Vec<Clause>, Who)> {
    let s = s.trim();
    let s = s.strip_suffix('.').unwrap_or(s);
    if let Some(r) = s.strip_prefix("activate only ") {
        return Some((clause_list(r, ctx)?, Who::Controller));
    }
    if let Some(r) = s
        .strip_prefix("activate no more than ")
        .and_then(|r| r.strip_suffix(" each turn"))
    {
        return Some((vec![Clause::PerTurn(times(r)?)], Who::Controller));
    }
    for (head, who) in [
        ("any player may activate this ability", Who::AnyPlayer),
        ("only your opponents may activate this ability", Who::Opponents),
    ] {
        if let Some(r) = s.strip_prefix(head) {
            if r.is_empty() {
                return Some((Vec::new(), who));
            }
            let r = r
                .strip_prefix(" but only ")
                .or_else(|| r.strip_prefix(" and only "))?;
            return Some((clause_list(r, ctx)?, who));
        }
    }
    // "You can't activate this ability during combat."
    if let Some(r) = s.strip_prefix("you can't activate this ability during ") {
        let Clause::When(c, _) = during(r)? else {
            return None;
        };
        return Some((
            vec![Clause::If(Condition::Not(Box::new(c)))],
            Who::Controller,
        ));
    }
    None
}

/// Applies restriction clauses to an activated ability. Fails if two clauses can't both
/// hold as written (two different sorcery/instant timings).
pub(crate) fn apply(act: &mut ActivatedAbility, clauses: Vec<Clause>, who: Who) -> Option<()> {
    let mut conds: Vec<Condition> = act.condition.take().into_iter().collect();
    // Sorcery and instant timing only exist as activation timings.
    for c in &clauses {
        let t = match c {
            Clause::Sorcery => ActivationTiming::Sorcery,
            Clause::AsInstant => ActivationTiming::AsInstant,
            _ => continue,
        };
        if act.timing != ActivationTiming::Instant && act.timing != t {
            return None;
        }
        act.timing = t;
    }
    for c in clauses {
        match c {
            Clause::Sorcery | Clause::AsInstant => {}
            Clause::When(cond, t) => match t {
                Some(t) if act.timing == ActivationTiming::Instant => act.timing = t,
                _ => conds.push(cond),
            },
            Clause::If(cond) => conds.push(cond),
            Clause::PerTurn(n) => act.max_per_turn = Some(act.max_per_turn.map_or(n, |m| m.min(n))),
            Clause::Total(n) => act.max_total = Some(act.max_total.map_or(n, |m| m.min(n))),
        }
    }
    act.condition = match conds.len() {
        0 => None,
        1 => conds.pop(),
        _ => Some(Condition::And(conds)),
    };
    match who {
        Who::Controller => {}
        Who::AnyPlayer => act.any_player = true,
        Who::Opponents => act.only_opponents = true,
    }
    Some(())
}

/// Whether the core parser handles the sentence by itself (see
/// `costs::split_activation_restrictions`).
fn core_handles(sentence: &str) -> bool {
    let (rest, ..) = crate::oracle::costs::split_activation_restrictions(sentence);
    rest.trim().is_empty()
}

/// "[cost]: [effect]. [restriction sentences]".
fn activated_with_restrictions(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = crate::oracle::strip_ability_word(block.trim());
    let (cost_s, eff_s) = crate::oracle::split_cost(text)?;
    let sentences = crate::oracle::effects::split_sentences(eff_s);
    let mut keep: Vec<String> = Vec::new();
    let mut parsed = Vec::new();
    let mut needed = false;
    for s in &sentences {
        let l = s.to_lowercase();
        match restriction_sentence(&l, ctx) {
            Some(r) if !s.contains('"') => {
                needed |= !core_handles(s);
                parsed.push(r);
            }
            _ => keep.push(s.clone()),
        }
    }
    if !needed || keep.is_empty() {
        return None;
    }
    let new_block = format!("{cost_s}: {}", keep.join(" "));
    let mut abilities = crate::oracle::parse_ability(&new_block, ctx)?;
    if abilities.len() != 1 {
        return None;
    }
    let a = abilities.pop()?;
    let AbilityKind::Activated(act) = &a.kind else {
        return None;
    };
    let mut act = act.clone();
    for (clauses, who) in parsed {
        apply(&mut act, clauses, who)?;
    }
    Some(vec![AbilityDef::new(AbilityKind::Activated(act), block)])
}

inventory::submit! { AbilityPattern { name: "activation restrictions: clause grammar", priority: 105, parse: activated_with_restrictions } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restriction_sentences() {
        let tl = crate::types::TypeLine::default();
        let c = CompileContext {
            card_name: "",
            full_name: "",
            type_line: &tl,
            layout: crate::card::Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let (cl, who) = restriction_sentence(
            "only your opponents may activate this ability and only as a sorcery.",
            &c,
        )
        .unwrap();
        assert_eq!(who, Who::Opponents);
        assert!(matches!(cl[..], [Clause::Sorcery]));
        let (cl, who) = restriction_sentence(
            "any player may activate this ability but only during their turn before the end step",
            &c,
        )
        .unwrap();
        assert_eq!(who, Who::AnyPlayer);
        assert!(matches!(cl[..], [Clause::When(Condition::And(_), None)]));
        let (cl, _) = restriction_sentence("activate no more than three times each turn.", &c).unwrap();
        assert!(matches!(cl[..], [Clause::PerTurn(3)]));
        let (cl, _) = restriction_sentence("activate no more than twice each turn.", &c).unwrap();
        assert!(matches!(cl[..], [Clause::PerTurn(2)]));
        let (cl, _) = restriction_sentence("activate only once.", &c).unwrap();
        assert!(matches!(cl[..], [Clause::Total(1)]));
        let (cl, _) = restriction_sentence(
            "activate only during the declare blockers step, only if you control an artifact, and only once each turn.",
            &c,
        )
        .unwrap();
        assert!(matches!(
            cl[..],
            [Clause::When(..), Clause::If(_), Clause::PerTurn(1)]
        ));
        let (cl, _) =
            restriction_sentence("activate only during an opponent's turn and only before combat.", &c)
                .unwrap();
        assert!(matches!(cl[..], [Clause::When(..), Clause::When(..)]));
        let (cl, _) =
            restriction_sentence("you can't activate this ability during combat.", &c).unwrap();
        assert!(matches!(cl[..], [Clause::If(Condition::Not(_))]));
        assert!(restriction_sentence("activate only during the full moon.", &c).is_none());
        assert!(restriction_sentence("draw a card.", &c).is_none());
    }
}
