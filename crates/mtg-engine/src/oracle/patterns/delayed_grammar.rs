//! Delayed triggered abilities created by instructions (CR 603.7), as grammar:
//!
//! * "[instruction] at the beginning of [the next|your next|that player's next|their next]
//!   [step]" and "At the beginning of ..., [instruction]" ("Exile it at the beginning of
//!   your next upkeep.", "Return that card to the battlefield under its owner's control at
//!   the beginning of that player's next end step.", "At this turn's next end of combat,
//!   destroy all creatures that blocked or were blocked by it this turn."). The instruction
//!   is read with the referents of the text around it ("it", "that card", "those
//!   creatures", "the exiled cards"), and everything it refers to from the creating ability
//!   — its targets, its source, its trigger event — is captured as the delayed ability is
//!   created (CR 603.7c: it still affects those objects, and only while they remain in the
//!   zone they're expected to be in). Targets the instruction names itself ("target land
//!   deals 3 damage to that creature") are the delayed ability's own, chosen when it
//!   triggers (CR 603.3d).
//! * "When/Whenever [referent] [event] this turn, [effect]" ("When that creature dies this
//!   turn, surveil 1.", "Whenever that creature deals combat damage to a player this turn,
//!   you draw two cards.", "When ~ leaves the battlefield this turn, destroy that
//!   creature.", "Whenever a creature dealt damage this way dies this turn, populate."):
//!   the trigger condition is the ordinary trigger grammar with the referent — an object
//!   the creating ability chose or affected, captured as the delayed ability is created —
//!   as its subject. A stated duration ("this turn") lets it trigger more than once
//!   (CR 603.7b).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, parse_effect_text, parse_sentence, Builder};
use crate::oracle::patterns::oracle_hardening_referents as refs;
use crate::oracle::phrases::{end, parse_object_phrase};

/// "That player" of a delayed step trigger ("at the beginning of that player's next end
/// step"), captured as the delayed ability is created.
const WHOSE: Var = vars::USER + 6401;
/// The object a referent delayed trigger is about ("that creature"), captured as it's
/// created.
pub(crate) const REFERENT: Var = vars::USER + 6402;
/// First variable used to capture the creating ability's references.
const CAPTURE_BASE: Var = vars::USER + 6410;

// ---------------------------------------------------------------------------------------
// The delay phrase
// ---------------------------------------------------------------------------------------

/// Whose step a delay phrase names.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Whose {
    /// "the next end step", "the next turn's upkeep".
    Any,
    /// "your next upkeep".
    You,
    /// "that player's next end step", "their next upkeep".
    ThatPlayer,
}

/// A parsed delay phrase.
#[derive(Clone, Debug)]
struct Delay {
    steps: Vec<TriggerStep>,
    whose: Whose,
    /// "this turn's next end of combat", "your next main phase this turn": it ends with
    /// the turn if it hasn't triggered by then.
    this_turn: bool,
}

impl Delay {
    fn trigger(&self) -> TriggerCond {
        let whose = match self.whose {
            Whose::Any => PlayerRel::Any,
            Whose::You => PlayerRel::You,
            Whose::ThatPlayer => PlayerRel::Var(WHOSE),
        };
        let mut conds: Vec<TriggerCond> = self
            .steps
            .iter()
            .map(|&step| TriggerCond::BeginningOf { step, whose })
            .collect();
        let t = if conds.len() == 1 {
            conds.remove(0)
        } else {
            TriggerCond::AnyOf(conds)
        };
        if self.this_turn {
            TriggerCond::ThisTurn(Box::new(t))
        } else {
            t
        }
    }
}

/// The step part of a delay phrase: "end step", "upkeep", "main phase", ...
fn step_words(s: &str) -> Option<Vec<TriggerStep>> {
    use TriggerStep::*;
    Some(match s {
        "end step" => vec![End],
        "upkeep" | "upkeep step" => vec![Upkeep],
        "draw step" => vec![Draw],
        "cleanup step" => vec![Cleanup],
        "combat" => vec![BeginningOfCombat],
        "declare attackers step" => vec![DeclareAttackers],
        // CR 505.1: "your next main phase" is whichever main phase of yours comes next.
        "main phase" => vec![PrecombatMain, PostcombatMain],
        "first main phase" | "precombat main phase" => vec![PrecombatMain],
        "postcombat main phase" => vec![PostcombatMain],
        _ => return None,
    })
}

/// A delay phrase without its leading "at ": "the beginning of the next end step", "the
/// beginning of their next upkeep", "end of combat", "this turn's next end of combat".
fn delay_phrase(s: &str) -> Option<Delay> {
    let eoc = |this_turn| Delay {
        steps: vec![TriggerStep::EndOfCombat],
        whose: Whose::Any,
        this_turn,
    };
    match s {
        "end of combat" | "the end of combat" => return Some(eoc(false)),
        "this turn's next end of combat" => return Some(eoc(true)),
        _ => {}
    }
    let r = s.strip_prefix("the beginning of ")?;
    let (r, this_turn) = match r.strip_suffix(" this turn") {
        Some(x) => (x, true),
        None => (r, false),
    };
    let (whose, step) = [
        ("the next turn's ", Whose::Any),
        ("the next ", Whose::Any),
        ("your next ", Whose::You),
        ("that player's next ", Whose::ThatPlayer),
        ("their next ", Whose::ThatPlayer),
    ]
    .into_iter()
    .find_map(|(p, w)| r.strip_prefix(p).map(|x| (w, x)))?;
    if r.starts_with("the next turn's ") && step != "upkeep" {
        return None;
    }
    Some(Delay {
        steps: step_words(step)?,
        whose,
        this_turn,
    })
}

/// Splits "[delay], [instruction]" or "[instruction] [delay]".
fn split_delay(l: &str) -> Option<(Delay, &str)> {
    if let Some(r) = l.strip_prefix("at ") {
        for (i, _) in r.match_indices(", ") {
            if let Some(d) = delay_phrase(&r[..i]) {
                return Some((d, &r[i + 2..]));
            }
        }
    }
    for (i, _) in l.match_indices(" at ") {
        if let Some(d) = delay_phrase(&l[i + 4..]) {
            return Some((d, &l[..i]));
        }
    }
    None
}

// ---------------------------------------------------------------------------------------
// Capturing the creating ability's references
// ---------------------------------------------------------------------------------------

/// What [`capture_refs`] rewrites into variables.
#[derive(Clone, Copy, PartialEq)]
enum Capture {
    /// The source, the creating ability's targets, and its trigger event (the delayed
    /// ability's own event is a different one).
    All,
    /// The source and the creating ability's targets only: the effect's trigger
    /// references are the delayed ability's own event.
    SourceAndTargets,
}

/// Rewrites references to the creating ability's context into variables, returning the
/// effects that fill them as the delayed ability is created. Target slots from
/// `outer_targets` on are the delayed ability's own targets and are renumbered from 0.
fn capture_refs(e: &Effect, outer_targets: u8, what: Capture) -> Option<(Vec<Effect>, Effect)> {
    use serde_json::Value as J;
    struct W {
        stores: Vec<Effect>,
        used: Vec<Var>,
        outer: u8,
        what: Capture,
    }
    fn var_json(var: Var) -> J {
        let mut m = serde_json::Map::new();
        m.insert("Var".into(), J::Number(var.into()));
        J::Object(m)
    }
    impl W {
        fn store(&mut self, var: Var, eff: Effect) -> J {
            if !self.used.contains(&var) {
                self.used.push(var);
                self.stores.push(eff);
            }
            var_json(var)
        }
        fn walk(&mut self, v: J) -> J {
            match v {
                J::String(s) => {
                    let sel = |sel: Sel, var: Var| Effect::Store { var, sel };
                    let all = self.what == Capture::All;
                    match s.as_str() {
                        "This" => self.store(CAPTURE_BASE, sel(Sel::This, CAPTURE_BASE)),
                        "TriggerObject" if all => {
                            self.store(CAPTURE_BASE + 1, sel(Sel::TriggerObject, CAPTURE_BASE + 1))
                        }
                        "TriggerLki" if all => {
                            self.store(CAPTURE_BASE + 2, sel(Sel::TriggerLki, CAPTURE_BASE + 2))
                        }
                        "TriggerOtherObject" if all => self.store(
                            CAPTURE_BASE + 3,
                            sel(Sel::TriggerOtherObject, CAPTURE_BASE + 3),
                        ),
                        "TriggerSpell" if all => {
                            self.store(CAPTURE_BASE + 4, sel(Sel::TriggerSpell, CAPTURE_BASE + 4))
                        }
                        "TriggerPlayer" if all => self.store(
                            CAPTURE_BASE + 5,
                            sel(Sel::TriggerPlayer, CAPTURE_BASE + 5),
                        ),
                        "EventAmount" if all => self.store(
                            CAPTURE_BASE + 6,
                            Effect::StoreValue {
                                var: CAPTURE_BASE + 6,
                                value: Value::EventAmount,
                            },
                        ),
                        _ => J::String(s),
                    }
                }
                J::Object(m) => {
                    if m.len() == 1 {
                        if let Some(J::Number(n)) = m.get("Target") {
                            let slot = n.as_u64().unwrap_or(0) as u8;
                            if slot >= self.outer {
                                let mut o = serde_json::Map::new();
                                o.insert("Target".into(), J::Number((slot - self.outer).into()));
                                return J::Object(o);
                            }
                            let var = CAPTURE_BASE + 10 + slot as Var;
                            return self.store(
                                var,
                                Effect::Store {
                                    var,
                                    sel: Sel::Target(slot),
                                },
                            );
                        }
                    }
                    J::Object(m.into_iter().map(|(k, v)| (k, self.walk(v))).collect())
                }
                J::Array(a) => J::Array(a.into_iter().map(|x| self.walk(x)).collect()),
                other => other,
            }
        }
    }
    let mut w = W {
        stores: vec![],
        used: vec![],
        outer: outer_targets,
        what,
    };
    let json = serde_json::to_value(e).ok()?;
    let rewritten = w.walk(json);
    // A reference that can't be a variable there fails to deserialize: the text stays
    // unsupported.
    let effect: Effect = serde_json::from_value(rewritten).ok()?;
    Some((w.stores, effect))
}

/// The Builder state an inner parse may change.
struct Saved {
    it: Sel,
    it_player: PlayerRef,
    group: Option<super::pronoun_groups::GroupRef>,
    named: Vec<(String, Sel)>,
    chosen: Option<(u8, String)>,
    in_trigger: bool,
}

fn save(b: &Builder) -> Saved {
    Saved {
        it: b.it.clone(),
        it_player: b.it_player.clone(),
        group: b.group.clone(),
        named: b.named.clone(),
        chosen: b.chosen_creature.clone(),
        in_trigger: b.in_trigger,
    }
}

fn restore(b: &mut Builder, s: Saved) {
    b.it = s.it;
    b.it_player = s.it_player;
    b.group = s.group;
    b.named = s.named;
    b.chosen_creature = s.chosen;
    b.in_trigger = s.in_trigger;
}

/// Target specs an inner parse added, which become the delayed ability's own; `None` if
/// one of them depends on the creating ability's targets.
fn own_targets(b: &mut Builder, n0: usize) -> Option<Vec<TargetSpec>> {
    let new = b.targets.split_off(n0);
    let independent = new
        .iter()
        .all(|t| t.distinct_from.is_empty() && t.related_to.is_none() && t.together.is_none());
    independent.then_some(new)
}

// ---------------------------------------------------------------------------------------
// "[instruction] at the beginning of the next end step"
// ---------------------------------------------------------------------------------------

/// "That player" of "at the beginning of that player's next end step": the owner or
/// controller the instruction names ("return that card to the battlefield under its
/// owner's control at the beginning of that player's next end step"), else the player the
/// text named before.
fn that_player(inner: &str, b: &Builder) -> Option<PlayerRef> {
    if refs::is_no_referent(&b.it) {
        return None;
    }
    if inner.contains("its owner's") {
        return Some(PlayerRef::OwnerOf(Box::new(b.it.clone())));
    }
    if inner.contains("its controller's") {
        return Some(PlayerRef::ControllerOf(Box::new(b.it.clone())));
    }
    if refs::is_no_player_referent(&b.it_player) || matches!(b.it_player, PlayerRef::You) {
        return None;
    }
    Some(b.it_player.clone())
}

fn delayed_instruction(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (delay, inner) = split_delay(l)?;
    let inner = inner.trim();
    // A list ("you gain 2 life, and you return ~ ...") is the older pattern's (only the
    // last instruction waits); a time inside the instruction isn't this grammar's.
    if inner.is_empty() || inner.contains(" at the beginning of ") || inner.starts_with("at ") {
        return None;
    }
    // "At the beginning of the next end step, you may pay {3}{B}. If you don't, ...": what
    // the payment decides is in the next sentence, which the delayed ability must include
    // (the payment patterns read the pair).
    if inner.starts_with("you may pay ") || inner.starts_with("pay ") {
        return None;
    }
    let whose = match delay.whose {
        Whose::ThatPlayer => Some(that_player(inner, b)?),
        _ => None,
    };
    // "a creature card from its owner's graveyard" names another object's owner, which the
    // zone grammar reads as any graveyard.
    if inner.contains(" from its owner's ") {
        return None;
    }
    let saved = save(b);
    let n0 = b.targets.len();
    // "At the beginning of the next end step, target land deals 3 damage to that
    // creature.": "that creature" is still what the text named before, even though the
    // instruction names a new target.
    let referent = match &b.chosen_creature {
        Some((slot, text)) if b.targets.get(*slot as usize).is_some_and(|t| &t.text == text) => {
            Sel::Target(*slot)
        }
        _ => b.it.clone(),
    };
    if !matches!(referent, Sel::This) && !refs::is_no_referent(&referent) {
        for p in ["that creature", "that card", "that permanent", "that token"] {
            b.named.push((p.to_string(), referent.clone()));
        }
    }
    let mut parsed = parse_sentence(inner, b);
    // "return those cards to the battlefield ... and those creatures gain haste until end
    // of turn": the second instruction is about what the first returned.
    if parsed.is_none() {
        if let Some((first, second)) = inner.split_once(" and ") {
            if super::pronoun_groups::plural_pronoun(second).is_some() {
                b.targets.truncate(n0);
                parsed = parse_sentence(first, b)
                    .and_then(|a| Some(Effect::seq(vec![a, parse_sentence(second, b)?])));
            }
        }
    }
    let result_it = b.it.clone();
    let targets = own_targets(b, n0);
    restore(b, saved);
    let (effect, targets) = (parsed?, targets?);
    // "... you may return it. If you do, ...": the next sentence is about whether the
    // delayed instruction happened, which only the delayed ability knows (see
    // `f_delayed_continues` for those it includes).
    if let Some(next) = next_sentence_start(l) {
        if next.starts_with("if you don't") || (next.starts_with("if you do") && !enters_battlefield(&effect)) {
            return None;
        }
        // "... you may cast that card without paying its mana cost. If that spell would be
        // put into a graveyard, exile it instead.": about the spell the delayed ability
        // casts.
        let casts = serde_json::to_string(&effect).is_ok_and(|j| j.contains("CastCard"));
        if casts && next.starts_with("if ") {
            return None;
        }
    }
    // "Return it to the battlefield ... at the beginning of your next upkeep. It gains
    // haste.": a permanent the delayed instruction puts onto the battlefield is what the
    // next sentences talk about (see `f_delayed_continues`).
    if enters_battlefield(&effect) {
        b.named.push((DELAYED_RESULT.to_string(), result_it));
    }
    let (mut stores, effect) = capture_refs(&effect, n0 as u8, Capture::All)?;
    if let Some(p) = whose {
        stores.push(Effect::Store {
            var: WHOSE,
            sel: Sel::Players(p),
        });
    }
    stores.push(Effect::DelayedTrigger {
        trigger: delay.trigger(),
        body: Box::new(Body {
            targets,
            effect,
            modal: None,
        }),
        once: true,
    });
    Some(Effect::seq(stores))
}

inventory::submit! { EffectPattern { name: "delayed grammar: [instruction] at the beginning of the next [step]", priority: 990, parse: delayed_instruction } }

/// How the sentence after `l` in the card's text begins (lowercase), if it can be found.
fn next_sentence_start(l: &str) -> Option<String> {
    let raw = crate::oracle::raw_text().to_lowercase();
    // The end of the sentence, without the card's name (`~` in `l`).
    let tail: String = l.chars().rev().take(30).collect::<Vec<_>>().into_iter().rev().collect();
    if tail.contains('~') {
        return None;
    }
    let i = raw.find(&tail)?;
    let after = raw[i + tail.len()..].trim_start_matches('.').trim_start();
    Some(after.chars().take(20).collect())
}

/// A [`Builder::named`] key (never a phrase of Oracle text) recording what "it" is after a
/// delayed instruction that puts a permanent onto the battlefield.
const DELAYED_RESULT: &str = "\u{1}delayed result";

/// Whether the effect puts objects onto the battlefield (or creates them there).
fn enters_battlefield(e: &Effect) -> bool {
    match e {
        Effect::Move { to, .. } => to.zone == ZoneKind::Battlefield,
        Effect::CreateToken { .. } | Effect::CreateTokenWithPT { .. } | Effect::CreateTokenCopy { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(enters_battlefield),
        Effect::May { effect, .. } => enters_battlefield(effect),
        Effect::If { then, .. } => enters_battlefield(then),
        _ => false,
    }
}

/// The delayed ability this grammar's last instruction created, if `e` ends with one.
fn last_delayed(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Seq(v) => last_delayed(v.last_mut()?),
        Effect::If { then, .. } => last_delayed(then),
        d @ Effect::DelayedTrigger { .. } => Some(d),
        _ => None,
    }
}

/// The sequence the delayed ability is the last effect of (to put captures before it); a
/// delayed ability on its own is made a sequence of one.
fn delayed_seq(e: &mut Effect) -> Option<&mut Vec<Effect>> {
    if matches!(e, Effect::DelayedTrigger { .. }) {
        let d = std::mem::take(e);
        *e = Effect::Seq(vec![d]);
    }
    match e {
        Effect::Seq(v) => {
            if matches!(v.last(), Some(Effect::DelayedTrigger { .. })) {
                return Some(v);
            }
            delayed_seq(v.last_mut()?)
        }
        Effect::If { then, .. } => delayed_seq(then),
        _ => None,
    }
}

/// Whether a sentence refers back to an object ("It gains haste.", "If it entered under
/// your control, ...", "It can't be blocked that combat.").
fn mentions_object(l: &str) -> bool {
    l.split(|c: char| !c.is_alphanumeric() && c != '\'').any(|w| {
        matches!(w, "it" | "its" | "they" | "them" | "their")
    }) || l.contains("that creature")
        || l.contains("that card")
        || l.contains("that permanent")
}

/// "Return it to the battlefield under its owner's control at the beginning of your next
/// upkeep. It gains haste.", "At the beginning of the next end step, return that card to
/// the battlefield under its owner's control. If it entered under your control, put a
/// +1/+1 counter on ~.", "... at the beginning of your next upkeep. If you do, discard cards
/// equal to that creature's toughness.": sentences about the permanent the delayed
/// instruction will put onto the battlefield are part of the delayed ability (they happen
/// when it does, to the new object, CR 400.7).
fn f_delayed_continues(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(result) = b
        .named
        .iter()
        .rev()
        .find(|(k, _)| k == DELAYED_RESULT)
        .map(|(_, s)| s.clone())
    else {
        return false;
    };
    let l = end(l);
    if !(mentions_object(l) || l.starts_with("if you do, ")) || split_delay(l).is_some() {
        return false;
    }
    if last_delayed(prev).is_none() {
        return false;
    }
    let saved = save(b);
    let n0 = b.targets.len();
    b.it = result;
    let parsed = parse_sentence(l, b);
    let new_targets = b.targets.len() > n0;
    b.targets.truncate(n0);
    restore(b, saved);
    let Some(e) = parsed.filter(|_| !new_targets) else {
        return false;
    };
    let Some((stores, e)) = capture_refs(&e, n0 as u8, Capture::All) else {
        return false;
    };
    let Some(seq) = delayed_seq(prev) else {
        return false;
    };
    let Some(Effect::DelayedTrigger { body, .. }) = seq.last_mut() else {
        return false;
    };
    body.effect = Effect::seq(vec![std::mem::take(&mut body.effect), e]);
    let at = seq.len() - 1;
    for st in stores {
        let var_of = |x: &Effect| match x {
            Effect::Store { var, .. } | Effect::StoreValue { var, .. } => Some(*var),
            _ => None,
        };
        if !seq.iter().any(|x| var_of(x).is_some() && var_of(x) == var_of(&st)) {
            seq.insert(at, st);
        }
    }
    true
}


/// "Whenever you attack this turn, create two 1/1 red Warrior creature tokens that are
/// tapped and attacking. Sacrifice them at the beginning of the next end step." (Dalkovan
/// Encampment): a sentence about the tokens a delayed triggered ability creates is part
/// of that ability (it happens each time it resolves, to those tokens).
fn f_tokens_of_delayed(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    if !mentions_object(l) || l.contains("target") {
        return false;
    }
    let Some(Effect::DelayedTrigger { trigger, body, .. }) = last_delayed(prev) else {
        return false;
    };
    if !matches!(trigger, TriggerCond::ThisTurn(_) | TriggerCond::UntilYourNextTurn(_))
        || body.modal.is_some()
    {
        return false;
    }
    fn creates(e: &Effect) -> bool {
        match e {
            Effect::CreateToken { .. } | Effect::CreateTokenWithPT { .. } => true,
            Effect::Seq(v) => v.last().is_some_and(creates),
            _ => false,
        }
    }
    if !creates(&body.effect) {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(vars::CREATED);
    sub.it_player = refs::no_player_referent();
    sub.sentences = 1;
    sub.targets = body.targets.clone();
    let Some(e) = parse_sentence(l, &mut sub) else {
        return false;
    };
    if sub.targets.len() != body.targets.len() {
        return false;
    }
    body.effect = Effect::seq(vec![std::mem::take(&mut body.effect), e]);
    true
}

inventory::submit! { FollowupPattern { name: "delayed grammar: sentences about the tokens a delayed ability creates", priority: 45, apply: f_tokens_of_delayed } }

inventory::submit! { FollowupPattern { name: "delayed grammar: sentences about what the delayed instruction puts onto the battlefield", priority: 45, apply: f_delayed_continues } }

/// The last instruction of an effect, through sequences and optional parts.
fn last_instruction(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_instruction),
        Effect::May { effect, .. } => last_instruction(effect),
        other => other,
    }
}

/// "You may exile ~. If you do, return it to the battlefield under its owner's control at
/// the beginning of your next upkeep.": "it" is the card the source became in exile
/// (CR 400.7), which the move recorded.
fn f_delayed_after_move(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (if_you_do, r) = match l.strip_prefix("if you do, ") {
        Some(r) => (true, r),
        None => (false, l),
    };
    if split_delay(r).is_none() {
        return false;
    }
    // The source moved ("you may exile ~"), or cards exiled ("Exile all creatures you
    // control. At the beginning of the next combat, return those cards ...").
    let moved = match last_instruction(prev) {
        Effect::Exile { what, .. } => matches!(what, Sel::This) || !if_you_do,
        Effect::Move { what, to } => {
            matches!(what, Sel::This) || (to.zone == ZoneKind::Exile && !if_you_do)
        }
        _ => false,
    };
    if !moved {
        return false;
    }
    let saved = (b.it.clone(), b.named.len());
    b.it = Sel::Var(vars::IT);
    for p in ["those cards", "the exiled cards", "the exiled card", "that card"] {
        b.named.push((p.to_string(), Sel::Var(vars::IT)));
    }
    let parsed = delayed_instruction(r, b);
    // (Keeping what the delayed instruction recorded for the sentences after it.)
    let mut i = saved.1;
    while i < b.named.len() {
        if b.named[i].0 == DELAYED_RESULT {
            i += 1;
        } else {
            b.named.remove(i);
        }
    }
    let Some(e) = parsed else {
        b.it = saved.0;
        return false;
    };
    let e = if if_you_do {
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        }
    } else {
        e
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "delayed grammar: return the moved source at the beginning of the next [step]", priority: 44, apply: f_delayed_after_move } }

// ---------------------------------------------------------------------------------------
// "When that creature dies this turn, ..."
// ---------------------------------------------------------------------------------------

/// Splits at the first comma outside quotes.
fn split_at_comma(s: &str) -> Option<(&str, &str)> {
    let mut in_quote = false;
    for (i, ch) in s.char_indices() {
        match ch {
            '"' => in_quote = !in_quote,
            ',' if !in_quote => return Some((&s[..i], s[i + 1..].trim_start())),
            _ => {}
        }
    }
    None
}

/// Rewrites the trigger condition parsed with "~" as its subject so that its subject is
/// `subject` instead; `None` if the condition doesn't name its subject as a filter, or
/// names the source elsewhere too.
fn with_subject(t: &TriggerCond, subject: &Filter) -> Option<TriggerCond> {
    use serde_json::Value as J;
    let subject = serde_json::to_value(subject).ok()?;
    let mut n = 0;
    fn walk(v: J, subject: &J, n: &mut usize) -> J {
        match v {
            J::String(s) if s == "Source" => {
                *n += 1;
                subject.clone()
            }
            J::Object(m) => J::Object(m.into_iter().map(|(k, v)| (k, walk(v, subject, n))).collect()),
            J::Array(a) => J::Array(a.into_iter().map(|x| walk(x, subject, n)).collect()),
            other => other,
        }
    }
    let json = walk(serde_json::to_value(t).ok()?, &subject, &mut n);
    // "When ~ leaves the battlefield or becomes untapped": the same subject each time.
    if n == 0 || json.to_string().contains("\"This\"") {
        return None;
    }
    serde_json::from_value(json).ok()
}

/// The subject of a referent trigger: what to capture and the filter its events must
/// match. "that creature", "it", "target creature", "~", "a creature you control dealt
/// damage this way".
fn referent_subject(s: &str, b: &mut Builder) -> Option<(Sel, Filter)> {
    // "a creature dealt damage this way": the objects the latest damage instruction dealt
    // damage to, of that kind.
    if let Some(r) = s
        .strip_prefix("a ")
        .or_else(|| s.strip_prefix("an "))
        .and_then(|r| r.strip_suffix(" dealt damage this way"))
    {
        let (f, plural, rest) = parse_object_phrase(r)?;
        if plural || !rest.trim().is_empty() {
            return None;
        }
        let f = Filter::and(vec![f, Filter::In(Box::new(Sel::Var(REFERENT)))]);
        return Some((Sel::Var(vars::DAMAGED), f));
    }
    if s.starts_with("a ") || s.starts_with("an ") || s.starts_with("each ") {
        return None;
    }
    // "When the creature an opponent controls dies this turn" after "target creature an
    // opponent controls": that target.
    if let Some(r) = s.strip_prefix("the ") {
        let wanted = format!("target {r}");
        let slots: Vec<u8> = (0..b.targets.len() as u8)
            .filter(|i| b.targets[*i as usize].text == wanted)
            .collect();
        if let [slot] = slots[..] {
            return Some((Sel::Target(slot), Filter::In(Box::new(Sel::Var(REFERENT)))));
        }
    }
    let (sel, rest) = object_ref(s, b)?;
    if !rest.trim().is_empty() || refs::is_no_referent(&sel) {
        return None;
    }
    match sel {
        Sel::This
        | Sel::Target(_)
        | Sel::Var(_)
        | Sel::TriggerObject
        | Sel::TriggerLki
        | Sel::TriggerOtherObject => {}
        _ => return None,
    }
    Some((sel, Filter::In(Box::new(Sel::Var(REFERENT)))))
}

/// Trigger events the core trigger grammar doesn't word for a single object: "is put into
/// a graveyard" (from the battlefield: once it's anywhere else it's a new object, CR
/// 400.7), "is put into your graveyard", "dies under your control".
fn object_event(rest: &str, subject: &Filter) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let dies = |extra: Option<Filter>| {
        let f = match extra {
            Some(x) => Filter::and(vec![subject.clone(), x]),
            None => subject.clone(),
        };
        (TriggerCond::Dies(f), Sel::This, PlayerRef::You)
    };
    Some(match rest {
        " is put into a graveyard" | " is put into a graveyard from the battlefield" => dies(None),
        " is put into your graveyard" => dies(Some(Filter::OwnedBy(PlayerRel::You))),
        " dies under your control" => dies(Some(Filter::ControlledBy(PlayerRel::You))),
        _ => return None,
    })
}

/// Whether the trigger is about damage the subject deals (its event object is what was
/// dealt damage).
fn deals_damage(t: &TriggerCond) -> bool {
    match t {
        TriggerCond::DealsDamage { .. } => true,
        TriggerCond::Batched { trigger, .. } => deals_damage(trigger),
        _ => false,
    }
}

/// "draw that many cards", "you gain life equal to that damage": the amount of the
/// delayed ability's own event (as "x", replaced by the event amount after parsing).
fn event_amount_text(eff: &str, t: &TriggerCond) -> Option<String> {
    let uses = eff.contains("that many") || eff.contains("life equal to that damage");
    if !uses {
        return Some(eff.to_string());
    }
    if !deals_damage(t) && !matches!(t, TriggerCond::IsDealtDamage { .. } | TriggerCond::Batched { .. }) {
        return None;
    }
    if eff.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
        return None;
    }
    Some(
        eff.replace("that many", "x")
            .replace("gain life equal to that damage", "gain x life"),
    )
}

/// Replaces X with the event's amount.
fn x_is_event_amount(e: &Effect) -> Option<Effect> {
    use serde_json::Value as J;
    fn walk(v: J) -> J {
        match v {
            J::String(s) if s == "X" => J::String("EventAmount".into()),
            J::Object(m) => J::Object(m.into_iter().map(|(k, v)| (k, walk(v))).collect()),
            J::Array(a) => J::Array(a.into_iter().map(walk).collect()),
            other => other,
        }
    }
    serde_json::from_value(walk(serde_json::to_value(e).ok()?)).ok()
}

fn referent_trigger(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (when, r) = if let Some(r) = l.strip_prefix("when ") {
        (true, r)
    } else {
        (false, l.strip_prefix("whenever ")?)
    };
    let (cond, eff) = split_at_comma(r)?;
    // "a creature card from its owner's graveyard" (another object's owner, which the zone
    // grammar reads as any graveyard); "return ~ from exile" (the card the source became,
    // which isn't tracked into the delayed ability).
    if eff.contains(" from its owner's ") || eff.contains("~ from exile") {
        return None;
    }
    let (cond, this_turn) = match cond.strip_suffix(" this turn") {
        Some(c) => (c, true),
        None => (cond, false),
    };
    // Without a stated duration it waits for the one event (CR 603.7b).
    if !this_turn && !when {
        return None;
    }
    let cond = cond.replacen("it's put into ", "it is put into ", 1);
    // "When you lose control of the creature, tap it." (the referent is the object).
    if let Some(r) = cond.strip_prefix("you lose control of ") {
        return lose_control_trigger(r, eff, this_turn, when, b);
    }
    let words: Vec<(usize, &str)> = cond.match_indices(' ').collect();
    for (i, _) in words {
        let (subject_s, rest) = (&cond[..i], &cond[i..]);
        let saved = save(b);
        let n0 = b.targets.len();
        let parsed = referent_subject(subject_s, b).and_then(|(sel, filter)| {
            if let Some((t, it, p)) = object_event(rest, &filter) {
                return Some((sel, t, it, p));
            }
            let (t, it, it_player) =
                crate::oracle::triggers::parse_trigger_condition(&format!("when ~{rest}"))?;
            Some((sel, with_subject(&t, &filter)?, it, it_player))
        });
        let Some((sel, trigger, it, it_player)) = parsed else {
            b.targets.truncate(n0);
            restore(b, saved);
            continue;
        };
        let Some(amount_eff) = event_amount_text(eff, &trigger) else {
            b.targets.truncate(n0);
            restore(b, saved);
            return None;
        };
        let uses_x = amount_eff != eff;
        let eff = amount_eff;
        // The subject's own targets ("whenever target creature deals damage this turn")
        // are the creating ability's.
        let n1 = b.targets.len();
        let source_subject = matches!(sel, Sel::This);
        // "When ~ leaves the battlefield this turn, destroy that creature.": the effect's
        // referents are the creating ability's. Otherwise they're the delayed ability's
        // own event: "it" is the object (the new object after a zone change), or what the
        // trigger condition names ("deals combat damage to a non-Wall creature").
        if !source_subject {
            // After a zone change, "it" is found through the object's last known
            // information (its controller then, Searing Blood's rulings; the new object
            // for instructions that move it, CR 400.7).
            let leaves = matches!(
                trigger,
                TriggerCond::Dies(_) | TriggerCond::LeavesBattlefield(_)
            );
            b.it = match it {
                Sel::This if deals_damage(&trigger) => Sel::Var(REFERENT),
                Sel::This if leaves => Sel::TriggerLki,
                Sel::This => Sel::TriggerObject,
                other => other,
            };
            b.it_player = match it_player {
                PlayerRef::You => refs::no_player_referent(),
                p => p,
            };
            b.named.clear();
            b.chosen_creature = None;
            b.group = None;
            b.in_trigger = true;
        }
        let effect = parse_effect_text(&eff, b);
        let targets = own_targets(b, n1);
        restore(b, saved);
        let effect = match effect {
            Some(e) if uses_x => x_is_event_amount(&e),
            other => other,
        };
        let (Some(effect), Some(targets)) = (effect, targets) else {
            b.targets.truncate(n0);
            return None;
        };
        let what = if source_subject {
            Capture::All
        } else {
            Capture::SourceAndTargets
        };
        let (mut stores, effect) = capture_refs(&effect, n1 as u8, what)?;
        stores.insert(0, Effect::Store { var: REFERENT, sel });
        let trigger = if this_turn {
            TriggerCond::ThisTurn(Box::new(trigger))
        } else {
            trigger
        };
        stores.push(Effect::DelayedTrigger {
            trigger,
            body: Box::new(Body {
                targets,
                effect,
                modal: None,
            }),
            once: !this_turn,
        });
        return Some(Effect::seq(stores));
    }
    None
}

/// "When you lose control of the creature, tap it." (Ray of Command): the creature the
/// creating ability gained control of (CR 603.7c).
fn lose_control_trigger(
    subject: &str,
    eff: &str,
    this_turn: bool,
    when: bool,
    b: &mut Builder,
) -> Option<Effect> {
    if !matches!(subject, "the creature" | "that creature" | "it") || !when || this_turn {
        return None;
    }
    let it = super::pronoun_groups::singular_it(b);
    if !matches!(it, Sel::Target(_) | Sel::Var(_) | Sel::TriggerObject) {
        return None;
    }
    let saved = save(b);
    let n0 = b.targets.len();
    b.it = Sel::TriggerObject;
    b.it_player = refs::no_player_referent();
    b.named.clear();
    b.chosen_creature = None;
    b.group = None;
    b.in_trigger = true;
    let effect = parse_effect_text(eff, b);
    let targets = own_targets(b, n0);
    restore(b, saved);
    let (effect, targets) = (effect?, targets?);
    let (mut stores, effect) = capture_refs(&effect, n0 as u8, Capture::SourceAndTargets)?;
    stores.insert(0, Effect::Store { var: REFERENT, sel: it });
    stores.push(Effect::DelayedTrigger {
        trigger: TriggerCond::LoseControl(Filter::In(Box::new(Sel::Var(REFERENT)))),
        body: Box::new(Body {
            targets,
            effect,
            modal: None,
        }),
        once: true,
    });
    Some(Effect::seq(stores))
}

inventory::submit! { EffectPattern { name: "delayed grammar: when that creature [event] this turn, [effect]", priority: 990, parse: referent_trigger } }

// ---------------------------------------------------------------------------------------
// Instructions the delayed abilities above need
// ---------------------------------------------------------------------------------------

/// "Sacrifice those creatures", "sacrifice them", "sacrifice that token": the objects the
/// text named before, those of them you control (CR 701.21a).
fn sacrifice_referent(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("sacrifice ")?;
    let what = if let Some(found) = super::pronoun_groups::plural_object_ref(r, b) {
        let (sel, rest) = found?;
        if !rest.trim().is_empty() {
            return None;
        }
        sel
    } else {
        let r = ["that token", "that creature", "that permanent"]
            .iter()
            .find_map(|p| r.strip_prefix(p).filter(|x| x.trim().is_empty()))?;
        let _ = r;
        let it = super::pronoun_groups::singular_it(b);
        if matches!(it, Sel::This) || refs::is_no_referent(&it) {
            return None;
        }
        it
    };
    Some(Effect::SacrificeObjects {
        what: Sel::All(Filter::and(vec![
            Filter::In(Box::new(what)),
            Filter::ControlledBy(PlayerRel::You),
        ])),
    })
}

inventory::submit! { EffectPattern { name: "delayed grammar: sacrifice those creatures / that token", priority: 985, parse: sacrifice_referent } }

/// "If it would leave the battlefield, exile it instead of putting it anywhere else."
/// (Gruesome Encore, Whip of Erebos): a replacement effect for that permanent (CR 614.1a);
/// once it has left the battlefield it's a new object the effect doesn't apply to
/// (CR 400.7).
fn exile_if_it_would_leave(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let (who, rest) = r.split_once(" would leave the battlefield, ")?;
    if rest != "exile it instead of putting it anywhere else" {
        return None;
    }
    if !matches!(who, "it" | "that creature" | "that permanent") {
        return None;
    }
    let it = super::pronoun_groups::singular_it(b);
    if matches!(it, Sel::This) || refs::is_no_referent(&it) {
        return None;
    }
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter: Filter::In(Box::new(it)),
                from: Some(ZoneKind::Battlefield),
                to: None,
            },
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::Permanent,
        uses: None,
    })
}

inventory::submit! { EffectPattern { name: "delayed grammar: if it would leave the battlefield, exile it instead", priority: 60, parse: exile_if_it_would_leave } }

/// "When ~ leaves the battlefield this turn, destroy that creature. A creature destroyed
/// this way can't be regenerated." / "... It can't be regenerated.": the delayed
/// ability's destruction can't be regenerated (CR 701.19c).
fn f_delayed_no_regen(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !matches!(
        end(l),
        "it can't be regenerated"
            | "that creature can't be regenerated"
            | "a creature destroyed this way can't be regenerated"
    ) {
        return false;
    }
    let Some(Effect::DelayedTrigger { body, .. }) = last_delayed(prev) else {
        return false;
    };
    fn last_destroy(e: &mut Effect) -> Option<&mut bool> {
        match e {
            Effect::Destroy { no_regen, .. } => Some(no_regen),
            Effect::Seq(v) => last_destroy(v.last_mut()?),
            _ => None,
        }
    }
    match last_destroy(&mut body.effect) {
        Some(n) => {
            *n = true;
            true
        }
        None => false,
    }
}

inventory::submit! { FollowupPattern { name: "delayed grammar: the delayed destruction can't be regenerated", priority: 45, apply: f_delayed_no_regen } }

/// "Exile that card until ~ leaves the battlefield.", "each opponent exiles a card from
/// their hand until ~ leaves the battlefield", "exile any number of other nonland
/// permanents you control until ~ leaves the battlefield": the exile instruction (any the
/// exile grammar reads), whose objects return to the zones they came from immediately
/// after ~ leaves the battlefield (CR 610.3).
fn exile_until_leaves(l: &str, b: &mut Builder) -> Option<Effect> {
    use serde_json::Value as J;
    let head = end(l).strip_suffix(" until ~ leaves the battlefield")?;
    if !head.contains("exile") {
        return None;
    }
    let saved = save(b);
    let n0 = b.targets.len();
    let Some(e) = parse_sentence(head, b) else {
        b.targets.truncate(n0);
        restore(b, saved);
        return None;
    };
    fn walk(v: J, n: &mut usize) -> J {
        match v {
            J::Object(m) => {
                if let Some(J::Object(ex)) = m.get("Exile") {
                    if m.len() == 1 && ex.get("face_down") == Some(&J::Bool(false)) {
                        *n += 1;
                        let mut inner = serde_json::Map::new();
                        inner.insert("what".into(), ex.get("what").cloned().unwrap_or(J::Null));
                        inner.insert("until".into(), J::String("SourceLeavesBattlefield".into()));
                        let mut o = serde_json::Map::new();
                        o.insert("ExileUntil".into(), J::Object(inner));
                        return J::Object(o);
                    }
                }
                J::Object(m.into_iter().map(|(k, v)| (k, walk(v, n))).collect())
            }
            J::Array(a) => J::Array(a.into_iter().map(|x| walk(x, n)).collect()),
            other => other,
        }
    }
    let mut n = 0;
    let json = walk(serde_json::to_value(&e).ok()?, &mut n);
    let converted = (n == 1).then(|| serde_json::from_value::<Effect>(json).ok()).flatten();
    if converted.is_none() {
        b.targets.truncate(n0);
        restore(b, saved);
    }
    converted
}

inventory::submit! { EffectPattern { name: "delayed grammar: exile [objects] until ~ leaves the battlefield", priority: 995, parse: exile_until_leaves } }

// ---------------------------------------------------------------------------------------
// Reflexive triggers about what an instruction did "this way" (CR 603.12)
// ---------------------------------------------------------------------------------------

/// The cards the latest exile instruction of the ability exiled ("when you exile a card
/// this way"), recorded right after it (each time, for an instruction each player
/// performs).
const EXILED_THIS_WAY: Var = vars::USER + 6403;

/// Puts `store` right after the last instruction of `e` that `is`, through sequences,
/// optional and conditional parts and per-player repetitions. Returns whether it found
/// one.
fn store_after_last(e: &mut Effect, is: &dyn Fn(&Effect) -> bool, store: &Effect) -> bool {
    match e {
        Effect::Seq(v) => {
            for i in (0..v.len()).rev() {
                if is(&v[i]) {
                    v.insert(i + 1, store.clone());
                    return true;
                }
                if store_after_last(&mut v[i], is, store) {
                    return true;
                }
            }
            false
        }
        Effect::May { effect, .. } | Effect::ForEachPlayer { effect, .. } => {
            store_after_last(effect, is, store)
        }
        Effect::If { then, .. } => store_after_last(then, is, store),
        x if is(x) => {
            let old = std::mem::take(x);
            *x = Effect::Seq(vec![old, store.clone()]);
            true
        }
        _ => false,
    }
}

fn is_exile(e: &Effect) -> bool {
    match e {
        Effect::Exile { .. } | Effect::ExileUntil { .. } => true,
        Effect::Move { to, .. } => to.zone == ZoneKind::Exile,
        _ => false,
    }
}

/// "When you exile a card this way, ...", "When you exile a nonland card this way, ...",
/// "When one or more nonland cards are exiled this way, put that many +1/+1 counters on
/// target attacking creature.": a reflexive triggered ability that triggers if the
/// ability's exile instruction exiled such a card (CR 603.12); "that card" / "the exiled
/// card" is it, "that many" how many.
fn f_when_exiled_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some((cond, eff)) = l.strip_prefix("when ").and_then(split_at_comma) else {
        return false;
    };
    let Some(cond) = cond.strip_suffix(" this way") else {
        return false;
    };
    let (noun, many) = if let Some(r) = cond.strip_prefix("you exile ") {
        match r.strip_prefix("one or more ") {
            Some(n) => (n, true),
            None => match r.strip_prefix("a ").or_else(|| r.strip_prefix("an ")) {
                Some(n) => (n, false),
                None => return false,
            },
        }
    } else if let Some(r) = cond
        .strip_prefix("one or more ")
        .and_then(|r| r.strip_suffix(" are exiled"))
    {
        (r, true)
    } else if let Some(r) = cond
        .strip_prefix("a ")
        .or_else(|| cond.strip_prefix("an "))
        .and_then(|r| r.strip_suffix(" is exiled"))
    {
        (r, false)
    } else {
        return false;
    };
    let Some((f, _, rest)) = parse_object_phrase(noun) else {
        return false;
    };
    if !rest.trim().is_empty() {
        return false;
    }
    let found = Sel::All(Filter::and(vec![
        Filter::In(Box::new(Sel::Var(EXILED_THIS_WAY))),
        f,
    ]));
    // "that many": the number of them.
    let eff_x = eff.replace("that many", "x");
    let uses_x = eff_x != eff;
    if uses_x && eff.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = if many { found.clone() } else { Sel::Var(EXILED_THIS_WAY) };
    sub.it_player = b.it_player.clone();
    sub.sentences = 1;
    for p in ["that card", "the exiled card"] {
        if !many {
            sub.named.push((p.to_string(), Sel::Var(EXILED_THIS_WAY)));
        }
    }
    let Some(effect) = parse_effect_text(&eff_x, &mut sub) else {
        return false;
    };
    let effect = if uses_x {
        let count = Value::CountSel(Box::new(found.clone()));
        let Ok(json) = serde_json::to_value(&effect) else {
            return false;
        };
        let Ok(cj) = serde_json::to_value(&count) else {
            return false;
        };
        fn walk(v: serde_json::Value, c: &serde_json::Value) -> serde_json::Value {
            use serde_json::Value as J;
            match v {
                J::String(s) if s == "X" => c.clone(),
                J::Object(m) => J::Object(m.into_iter().map(|(k, v)| (k, walk(v, c))).collect()),
                J::Array(a) => J::Array(a.into_iter().map(|x| walk(x, c)).collect()),
                other => other,
            }
        }
        match serde_json::from_value(walk(json, &cj)) {
            Ok(e) => e,
            Err(_) => return false,
        }
    } else {
        effect
    };
    // Record each exile's cards (accumulated over the players of "each player exiles").
    let store = Effect::Store {
        var: EXILED_THIS_WAY,
        sel: Sel::Union(vec![Sel::Var(EXILED_THIS_WAY), Sel::Var(vars::IT)]),
    };
    let mut new_prev = prev.clone();
    if !store_after_last(&mut new_prev, &is_exile, &store) {
        return false;
    }
    let reflexive = Effect::If {
        cond: Condition::SelNonEmpty(found),
        then: Box::new(Effect::Reflexive {
            body: Box::new(Body {
                targets: sub.targets,
                effect,
                modal: None,
            }),
        }),
        otherwise: Box::new(Effect::Noop),
    };
    *prev = Effect::seq(vec![new_prev, reflexive]);
    true
}

inventory::submit! { FollowupPattern { name: "delayed grammar: when you exile a card this way (reflexive)", priority: 46, apply: f_when_exiled_this_way } }

/// "When it connives this way, ...", "When you search your library this way, ...": the
/// action always happens once the instruction is performed (a creature connives even if
/// it has left the battlefield, CR 701.50c; a search happens even if nothing is found,
/// CR 701.23), so the reflexive ability triggers then (CR 603.12).
fn f_when_done_this_way(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some((cond, eff)) = l.strip_prefix("when ").and_then(split_at_comma) else {
        return false;
    };
    let did: fn(&Effect) -> bool = match cond {
        "it connives this way" | "~ connives this way" => |e| {
            matches!(
                e,
                Effect::KeywordAction {
                    action: KeywordAction::Connive,
                    ..
                }
            )
        },
        "you search your library this way" => |e| matches!(e, Effect::Search { .. }),
        _ => return false,
    };
    fn contains(e: &Effect, did: fn(&Effect) -> bool) -> bool {
        match e {
            Effect::Seq(v) => v.iter().any(|x| contains(x, did)),
            x => did(x),
        }
    }
    // Only an instruction performed unconditionally.
    if !contains(prev, did) {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = b.it.clone();
    sub.it_player = b.it_player.clone();
    sub.sentences = 1;
    let Some(effect) = parse_effect_text(eff, &mut sub) else {
        return false;
    };
    let reflexive = Effect::Reflexive {
        body: Box::new(Body {
            targets: sub.targets,
            effect,
            modal: None,
        }),
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, reflexive]);
    true
}

inventory::submit! { FollowupPattern { name: "delayed grammar: when it connives / you search this way (reflexive)", priority: 46, apply: f_when_done_this_way } }

// ---------------------------------------------------------------------------------------
// Extra phases
// ---------------------------------------------------------------------------------------

/// "There is an additional combat phase after this phase, followed by an additional main
/// phase." (CR 500.8): the order of "After this phase, there is an additional combat phase
/// followed by an additional main phase."
fn additional_phase_after(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("there is ")?;
    let (first, then) = r.split_once(" after this phase, followed by ")?;
    parse_sentence(&format!("after this phase, there is {first} followed by {then}"), b)
}

inventory::submit! { EffectPattern { name: "delayed grammar: there is an additional combat phase after this phase, followed by ...", priority: 100, parse: additional_phase_after } }

/// "it's your main phase" (CR 505.1).
fn your_main_phase(c: &str) -> Option<Condition> {
    (end(c) == "it's your main phase").then(|| {
        Condition::And(vec![Condition::YourTurn, Condition::Phase(PhaseCond::MainPhase)])
    })
}

inventory::submit! { super::ConditionPattern { name: "delayed grammar: it's your main phase", priority: 100, parse: your_main_phase } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_phrases() {
        for (s, steps, whose, this_turn) in [
            ("the beginning of the next end step", 1, Whose::Any, false),
            ("the beginning of your next upkeep", 1, Whose::You, false),
            ("the beginning of that player's next end step", 1, Whose::ThatPlayer, false),
            ("the beginning of their next upkeep", 1, Whose::ThatPlayer, false),
            ("the beginning of the next turn's upkeep", 1, Whose::Any, false),
            ("the beginning of your next main phase", 2, Whose::You, false),
            ("the beginning of your next main phase this turn", 2, Whose::You, true),
            ("the beginning of the next cleanup step", 1, Whose::Any, false),
            ("this turn's next end of combat", 1, Whose::Any, true),
            ("the beginning of your next upkeep step", 1, Whose::You, false),
        ] {
            let d = delay_phrase(s).unwrap_or_else(|| panic!("{s}"));
            assert_eq!((d.steps.len(), d.whose, d.this_turn), (steps, whose, this_turn), "{s}");
        }
        assert!(delay_phrase("the beginning of the next turn's end step").is_none());
        assert!(delay_phrase("the beginning of each end step").is_none());
    }

    #[test]
    fn splits_either_order() {
        let (d, i) = split_delay("exile it at the beginning of your next upkeep").unwrap();
        assert_eq!((i, d.whose), ("exile it", Whose::You));
        let (_, i) = split_delay(
            "at the beginning of the next end step, return that card to the battlefield under its owner's control",
        )
        .unwrap();
        assert_eq!(i, "return that card to the battlefield under its owner's control");
        assert!(split_delay("look at the top card of your library").is_none());
    }

    #[test]
    fn captures_outer_targets_and_renumbers_own() {
        let e = Effect::Seq(vec![
            Effect::Destroy {
                what: Sel::Target(0),
                no_regen: false,
            },
            Effect::Destroy {
                what: Sel::Target(1),
                no_regen: false,
            },
            Effect::Draw {
                who: PlayerRef::You,
                n: Value::EventAmount,
            },
        ]);
        let (stores, out) = capture_refs(&e, 1, Capture::SourceAndTargets).unwrap();
        assert_eq!(stores.len(), 1);
        let s = format!("{out:?}");
        assert!(s.contains(&format!("Var({})", CAPTURE_BASE + 10)), "{s}");
        assert!(s.contains("Target(0)") && s.contains("EventAmount"), "{s}");
        let (stores, _) = capture_refs(&e, 1, Capture::All).unwrap();
        assert_eq!(stores.len(), 2);
    }
}
