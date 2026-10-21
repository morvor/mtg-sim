//! Referent grammar IV: alternatives and last-known information (CR 608.2c, 608.2h,
//! 603.10a):
//!
//! - "Destroy target Equipment. If that Equipment was attached to a creature, ~ deals 2
//!   damage to that creature." (Unforge): the permanent the object was attached to as it
//!   last existed on the battlefield (or is attached to, if it's still there);
//! - "Whenever a creature dies, if an Aura you controlled was attached to it, ..." (Dawn
//!   Evangel): the Auras attached to the creature as it last existed on the battlefield
//!   (a leaves-the-battlefield ability looks back in time, CR 603.10a);
//! - "{4}{U}{U}, Exile ~: Draw a card for each verse counter on ~. If it had seven or more
//!   verse counters on it, ..." (Lost Isle Calling): "it" is the source, as it last
//!   existed before its cost moved it.

use super::{ConditionPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, parse_sentence, Builder};
use crate::oracle::phrases::{end, parse_object_phrase};

/// "If that [noun] was attached to a creature, [effect with "that creature"]."
fn if_that_was_attached(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if that ") else {
        return false;
    };
    let Some((noun, r)) = r.split_once(" was attached to ") else {
        return false;
    };
    if noun.contains(' ') || !matches!(noun, "equipment" | "aura" | "fortification") {
        return false;
    }
    let Some((host, x)) = r.split_once(", ") else {
        return false;
    };
    let Some(host_noun) = host.strip_prefix("a ").or_else(|| host.strip_prefix("an ")) else {
        return false;
    };
    let Some((hf, false, tail)) = parse_object_phrase(host_noun) else {
        return false;
    };
    if !end(tail).is_empty() {
        return false;
    }
    // "That Equipment": the object the previous instruction acted on, a target.
    let it = b.it.clone();
    if !matches!(it, Sel::Target(_)) {
        return false;
    }
    let host_sel = Sel::Matching(Box::new(Sel::HostOf(Box::new(it))), hf);
    let phrase = format!(
        "that {}",
        host_noun.split(' ').next_back().unwrap_or(host_noun)
    );
    let saved = b.it.clone();
    b.it = host_sel.clone();
    b.named.push((phrase.clone(), host_sel.clone()));
    let first_new = b.targets.len();
    let e = parse_clause(x, b);
    b.named.retain(|(p, _)| *p != phrase);
    b.it = saved;
    let Some(e) = e else {
        b.targets.truncate(first_new);
        return false;
    };
    let cond_effect = Effect::If {
        cond: Condition::SelNonEmpty(host_sel),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = match old {
        Effect::Seq(mut v) => {
            v.push(cond_effect);
            Effect::Seq(v)
        }
        other => Effect::seq(vec![other, cond_effect]),
    };
    true
}

inventory::submit! { FollowupPattern { name: "if that equipment was attached to a creature, [effect]", priority: 140, apply: if_that_was_attached } }

/// "an Aura you controlled was attached to it" in a dies or leaves-the-battlefield
/// trigger: Auras (or Equipment) attached to the trigger's object as it last existed on
/// the battlefield.
fn attached_as_it_left(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c.strip_prefix("an ").or_else(|| c.strip_prefix("a "))?;
    let (noun, r) = r.split_once(' ')?;
    let sub = match noun {
        "aura" => "Aura",
        "equipment" => "Equipment",
        _ => return None,
    };
    let (yours, r) = match r.strip_prefix("you controlled ") {
        Some(r) => (true, r),
        None => (false, r),
    };
    if r != "was attached to it" {
        return None;
    }
    // Only about an object that left the battlefield.
    let raw = crate::oracle::raw_text().to_lowercase();
    if !(raw.contains(" dies, ") || raw.contains(" leaves the battlefield, ")) {
        return None;
    }
    let mut f = vec![Filter::Subtype(sub.into())];
    if yours {
        f.push(Filter::ControlledBy(PlayerRel::You));
    }
    Some(Condition::SelNonEmpty(Sel::Matching(
        Box::new(Sel::AttachedAsItLeft(Box::new(Sel::TriggerLki))),
        Filter::and(f),
    )))
}

inventory::submit! { ConditionPattern { name: "an aura you controlled was attached to it (as it left)", priority: 140, parse: attached_as_it_left } }

thread_local! {
    static SOURCE_REWORDING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// "If it had seven or more verse counters on it, [effect]." where "it" is the source
/// (an ability whose cost moved it): read as "If ~ had ...".
fn if_it_had_source(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if SOURCE_REWORDING.with(|r| r.get()) || !matches!(b.it, Sel::This) {
        return false;
    }
    let l = end(l);
    let Some(r) = l.strip_prefix("if it had ") else {
        return false;
    };
    let t = format!("if ~ had {r}");
    SOURCE_REWORDING.with(|r| r.set(true));
    let e = parse_sentence(&t, b);
    SOURCE_REWORDING.with(|r| r.set(false));
    let Some(e) = e else {
        return false;
    };
    let old = std::mem::replace(prev, Effect::Noop);
    *prev = match old {
        Effect::Seq(mut v) => {
            v.push(e);
            Effect::Seq(v)
        }
        other => Effect::seq(vec![other, e]),
    };
    true
}

inventory::submit! { FollowupPattern { name: "if it had [counters] (the source)", priority: 160, apply: if_it_had_source } }

/// "You may discard a card. If you do, draw a card. If a creature card was discarded this
/// way, draw two cards instead." (Lukka, Wayward Bonder): a "this way" condition whose
/// alternative replaces the previous instruction (CR 608.2c).
fn this_way_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(t) = l.strip_suffix(" instead") else {
        return false;
    };
    if !l.starts_with("if ") || !t.contains(" this way, ") {
        return false;
    }
    let mut trial = prev.clone();
    if !super::conditions_this_way::if_this_way(t, &mut trial, b) {
        return false;
    }
    let Effect::Seq(v) = &mut trial else {
        return false;
    };
    let Some(Effect::If { otherwise, .. }) = v.last() else {
        return false;
    };
    if !matches!(**otherwise, Effect::Noop) {
        return false;
    }
    // The instruction the alternative replaces: the last one before the condition, other
    // than the records the condition reads.
    let snapshot = |e: &Effect| match e {
        Effect::Store { var, .. } => *var == super::conditions_this_way::SNAPSHOT,
        Effect::If { then, .. } => matches!(&**then, Effect::Store { var, .. }
            if *var == super::conditions_this_way::SNAPSHOT),
        _ => false,
    };
    let record =
        |e: &Effect| snapshot(e) || matches!(e, Effect::Store { .. } | Effect::StoreValue { .. });
    let n = v.len() - 1;
    let Some(j) = v[..n].iter().rposition(|e| !record(e)) else {
        return false;
    };
    // Not the instruction the condition is about (it must have happened first).
    if v.get(j + 1).is_some_and(snapshot) {
        return false;
    }
    let replaced = v.remove(j);
    let Some(Effect::If { otherwise, .. }) = v.last_mut() else {
        return false;
    };
    **otherwise = replaced;
    *prev = trial;
    true
}

inventory::submit! { FollowupPattern { name: "if [objects] [verb] this way, [effect] instead", priority: 140, apply: this_way_instead } }

/// [`this_way_instead`] for the look-back over earlier sentences
/// (`referents_values_4::lookback`).
pub fn this_way_instead_pub(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    this_way_instead(l, prev, b)
}

/// "Clash with an opponent. If you win, creatures that player controls don't untap during
/// the player's next untap step." (Pollen Lullaby): the group the player an earlier part
/// named controls, through that player's next untap step (CR 502.3).
fn their_creatures_dont_untap(l: &str, b: &mut Builder) -> Option<Effect> {
    use super::oracle_hardening_referents::is_no_player_referent;
    let l = end(l);
    let (subject, whose) = l.split_once(" don't untap during ")?;
    if !matches!(
        whose,
        "the player's next untap step" | "that player's next untap step" | "their next untap step"
    ) || is_no_player_referent(&b.it_player)
    {
        return None;
    }
    let noun = subject.strip_suffix(" that player controls")?;
    let (f, true, tail) = parse_object_phrase(noun)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::AddRestriction {
        restriction: Restriction::DoesntUntap(Filter::and(vec![
            f,
            Filter::ControlledByPlayer(Box::new(b.it_player.clone())),
        ])),
        duration: Duration::ThroughNextUntapStep,
    })
}

inventory::submit! { super::EffectPattern { name: "creatures that player controls don't untap during the player's next untap step", priority: 105, parse: their_creatures_dont_untap } }

/// Whether the clash with an opponent the effect performed earlier was won (1) or not
/// (0), recorded right after it (CR 701.30d), for an "If you win" after other
/// instructions.
pub const CLASH_WON: Var = vars::USER + 14_010;

/// The condition [`CLASH_WON`] reads, in the internal wording [`if_you_win_later`] gives
/// it.
const WON_THE_CLASH: &str = "you won the clash this way";

fn won_the_clash(c: &str) -> Option<Condition> {
    (end(c) == WON_THE_CLASH)
        .then(|| Condition::Compare(Value::Var(CLASH_WON), Cmp::Eq, Value::c(1)))
}

inventory::submit! { ConditionPattern { name: "you won the clash (recorded)", priority: 10, parse: won_the_clash } }

fn is_clash(e: &Effect) -> bool {
    matches!(
        e,
        Effect::KeywordAction {
            action: KeywordAction::Clash,
            ..
        }
    )
}

/// "Clash with an opponent, then return target creature to its owner's hand. If you win,
/// you may put that creature on top of its owner's library instead." (Whirlpool Whelm):
/// the clash's result, recorded right after it, since a later instruction has an outcome
/// of its own.
fn if_you_win_later(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(x) = l.strip_prefix("if you win, ") else {
        return false;
    };
    let mut parts: Vec<Effect> = match &*prev {
        Effect::Seq(v) => v.clone(),
        other => vec![other.clone()],
    };
    let Some(i) = parts.iter().rposition(is_clash) else {
        return false;
    };
    if i + 1 == parts.len() {
        // Right after the clash: its outcome is the previous instruction's.
        return false;
    }
    parts.insert(
        i + 1,
        Effect::StoreValue {
            var: CLASH_WON,
            value: Value::If(
                Box::new(Condition::PrevHappened),
                Box::new(Value::c(1)),
                Box::new(Value::c(0)),
            ),
        },
    );
    let t = format!("if {WON_THE_CLASH}, {x}");
    let mut trial = Effect::Seq(parts);
    if crate::oracle_ext::apply_followup_ext(&t, &mut trial, b) {
        *prev = trial;
        return true;
    }
    let Some(e) = parse_sentence(&t, b) else {
        return false;
    };
    let Effect::Seq(mut v) = trial else {
        return false;
    };
    v.push(e);
    *prev = Effect::Seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "if you win (the clash before other instructions), [effect]", priority: 50, apply: if_you_win_later } }

/// The card a "reveal cards until you reveal a [kind] card" instruction found, recorded
/// right after it ("the revealed land card").
pub const FOUND_CARD: Var = vars::USER + 14_011;

/// The index of the "reveal cards until you reveal [a card]" instruction among `v` (it
/// leaves the card it found in `vars::IT`).
fn reveal_until_at(v: &[Effect]) -> Option<usize> {
    v.iter()
        .rposition(|e| matches!(e, Effect::RevealUntil { .. }))
}

/// "If the revealed land card was a Mountain, ~ deals double that damage instead."
/// (Goblin Charbelcher): the previous damage instruction, with twice its amount, if the
/// condition holds. "The revealed [kind] card" is the card the "reveal cards until you
/// reveal a [kind] card" instruction found.
fn double_that_damage_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", ") else {
        return false;
    };
    let Some(subject) = x
        .strip_suffix(" deals double that damage instead")
        .or_else(|| x.strip_suffix(" deals twice that much damage instead"))
    else {
        return false;
    };
    let Effect::Seq(v) = &*prev else {
        return false;
    };
    let Some(Effect::DealDamage { source, amount, to }) = v.last() else {
        return false;
    };
    if !(subject == "~" && matches!(source, Sel::This | Sel::None)) {
        return false;
    }
    let (source, amount, to) = (source.clone(), amount.clone(), to.clone());
    // "the revealed land card": the card found.
    let found_at = reveal_until_at(v);
    let mut pushed = false;
    if let Some(noun) = c
        .strip_prefix("the revealed ")
        .and_then(|r| r.split_once(" was ").map(|(n, _)| n))
    {
        if found_at.is_some() && noun.ends_with(" card") {
            b.named
                .push((format!("the revealed {noun}"), Sel::Var(FOUND_CARD)));
            pushed = true;
        }
    }
    let cond = super::conditions_referents::parse_condition_with(c, b);
    if pushed {
        b.named.pop();
    }
    let Some(cond) = cond else {
        return false;
    };
    let Effect::Seq(v) = prev else {
        return false;
    };
    let single = v.pop().unwrap_or(Effect::Noop);
    if let (true, Some(i)) = (pushed, found_at) {
        v.insert(
            i + 1,
            Effect::Store {
                var: FOUND_CARD,
                sel: Sel::Var(vars::IT),
            },
        );
    }
    v.push(Effect::If {
        cond,
        then: Box::new(Effect::DealDamage {
            source,
            amount: Value::Mul(Box::new(Value::c(2)), Box::new(amount)),
            to,
        }),
        otherwise: Box::new(single),
    });
    true
}

inventory::submit! { FollowupPattern { name: "if [condition], ~ deals double that damage instead", priority: 140, apply: double_that_damage_instead } }

/// "~ deals 4 damage to each of those permanents and/or players instead if there are
/// seven or more cards in your graveyard." (Shower of Coals): the previous damage
/// instruction, to the same recipients, with another amount if the condition holds.
fn damage_to_those_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (x, c) = match l.strip_prefix("if ") {
        Some(r) => match r.split_once(", ") {
            Some((c, x)) => match x.strip_suffix(" instead") {
                Some(x) => (x, c),
                None => return false,
            },
            None => return false,
        },
        None => match l.rsplit_once(" instead if ") {
            Some((x, c)) => (x, c),
            None => return false,
        },
    };
    let Some(r) = x.strip_prefix("~ deals ") else {
        return false;
    };
    let Some((n, r)) = r.split_once(" damage to ") else {
        return false;
    };
    if !matches!(
        r,
        "each of those permanents and/or players" | "each of them" | "them" | "those targets"
    ) {
        return false;
    }
    let Some((Value::Const(n), tail)) = crate::oracle::phrases::parse_number(n) else {
        return false;
    };
    if !tail.trim().is_empty() {
        return false;
    }
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    let Some(Effect::DealDamage { source, to, .. }) = last else {
        return false;
    };
    // The recipients are the targets of one instance of "target" (several of them).
    let Sel::Target(k) = to else {
        return false;
    };
    if b.targets
        .get(*k as usize)
        .is_none_or(|t| matches!(t.max, Value::Const(m) if m < 2))
    {
        return false;
    }
    let (source, to) = (source.clone(), to.clone());
    let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) else {
        return false;
    };
    let alt = Effect::DealDamage {
        source,
        amount: Value::c(n),
        to,
    };
    let replace = |single: Effect| Effect::If {
        cond: cond.clone(),
        then: Box::new(alt.clone()),
        otherwise: Box::new(single),
    };
    match prev {
        Effect::Seq(v) => {
            let single = v.pop().unwrap_or(Effect::Noop);
            v.push(replace(single));
        }
        other => {
            let single = std::mem::replace(other, Effect::Noop);
            *other = replace(single);
        }
    }
    true
}

inventory::submit! { FollowupPattern { name: "~ deals N damage to each of those targets instead if [condition]", priority: 140, apply: damage_to_those_instead } }

/// "Flip a coin. If you win the flip, exile the chosen creature. Otherwise, ~ deals 1
/// damage to it." (Plasma Caster): what happens if you lose the flip (CR 705.2). "It" is
/// what the winning instruction would have acted on.
fn otherwise_after_flip(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(x) = l.strip_prefix("otherwise, ") else {
        return false;
    };
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    let Some(Effect::FlipCoins(flip)) = last else {
        return false;
    };
    if !flip.call
        || flip.until_lose
        || !matches!(flip.count, Value::Const(1))
        || matches!(flip.on_win, Effect::Noop)
        || !matches!(flip.on_lose, Effect::Noop)
    {
        return false;
    }
    let subject = match &flip.on_win {
        Effect::Exile { what, .. }
        | Effect::Destroy { what, .. }
        | Effect::Move { what, .. }
        | Effect::Tap { what } => Some(what.clone()),
        _ => None,
    };
    let saved = b.it.clone();
    if let Some(s @ Sel::Target(_)) = subject {
        b.it = s;
    }
    let first_new = b.targets.len();
    let e = match x.strip_prefix("you may ") {
        Some(_) => parse_sentence(x, b),
        None => parse_clause(x, b),
    };
    b.it = saved;
    let Some(e) = e else {
        b.targets.truncate(first_new);
        return false;
    };
    let last = match prev {
        Effect::Seq(v) => v.last_mut(),
        other => Some(other),
    };
    let Some(Effect::FlipCoins(flip)) = last else {
        return false;
    };
    flip.on_lose = e;
    true
}

inventory::submit! { FollowupPattern { name: "otherwise, [effect] after if you win the flip", priority: 140, apply: otherwise_after_flip } }

/// "As an additional cost to cast this spell, sacrifice a creature or sacrifice a modified
/// creature. ... If the modified creature was sacrificed, draw a card." (Lethal
/// Throwdown): whether that option of the additional cost was chosen and paid (recorded
/// as the spell is cast, CR 601.2b, 601.2h).
fn option_sacrificed(c: &str) -> Option<Condition> {
    let c = end(c);
    let noun = c.strip_prefix("the ")?.strip_suffix(" was sacrificed")?;
    let option = format!("sacrifice a {noun}");
    let raw = crate::oracle::raw_text().to_lowercase();
    let options = raw.lines().find_map(|l| {
        l.strip_prefix("as an additional cost to cast this spell, ")
            .or_else(|| l.strip_prefix("as an additional cost to cast ~, "))
    })?;
    let options = options.trim_end_matches('.');
    let (a, b) = options.split_once(" or ")?;
    if a == option || b != option {
        return None;
    }
    Some(Condition::CostPaid(option.into()))
}

inventory::submit! { ConditionPattern { name: "the [option] was sacrificed (additional cost choice)", priority: 140, parse: option_sacrificed } }

/// One of the cards an earlier instruction moved, chosen to go elsewhere instead.
pub const INSTEAD_CHOSEN: Var = vars::USER + 14_012;

/// "Return up to two target creature cards from your graveyard to your hand. If this
/// spell was bargained, you may put one of those cards with mana value 4 or less onto the
/// battlefield instead of putting it into your hand." (Back for Seconds): if the
/// condition holds, one of the cards may go to the battlefield and the others to the
/// hand (CR 608.2c).
fn one_of_those_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", you may put one of those cards ") else {
        return false;
    };
    let Some(q) = x.strip_suffix(" onto the battlefield instead of putting it into your hand")
    else {
        return false;
    };
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    let Some(Effect::Move { what, to }) = last else {
        return false;
    };
    if to.zone != ZoneKind::Hand || !matches!(what, Sel::Target(_)) {
        return false;
    }
    let (what, to) = (what.clone(), to.clone());
    let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) else {
        return false;
    };
    // "with mana value 4 or less": a qualifier after "cards".
    let qualifier = if q.is_empty() {
        Filter::Any
    } else {
        let Some((f, true, tail)) = parse_object_phrase(&format!("cards {q}")) else {
            return false;
        };
        if !end(tail).is_empty() {
            return false;
        }
        f
    };
    let chosen = Sel::Var(INSTEAD_CHOSEN);
    let alt = Effect::seq(vec![
        Effect::Store {
            var: INSTEAD_CHOSEN,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::and(vec![Filter::In(Box::new(what.clone())), qualifier]),
                count: Value::c(1),
                up_to: true,
                store: None,
            },
        },
        Effect::Move {
            what: chosen.clone(),
            to: Destination::battlefield(),
        },
        Effect::Move {
            what: Sel::All(Filter::and(vec![
                Filter::In(Box::new(what.clone())),
                Filter::not(Filter::In(Box::new(chosen))),
            ])),
            to: to.clone(),
        },
    ]);
    let replace = |single: Effect| Effect::If {
        cond: cond.clone(),
        then: Box::new(alt.clone()),
        otherwise: Box::new(single),
    };
    match prev {
        Effect::Seq(v) => {
            let single = v.pop().unwrap_or(Effect::Noop);
            v.push(replace(single));
        }
        other => {
            let single = std::mem::replace(other, Effect::Noop);
            *other = replace(single);
        }
    }
    true
}

inventory::submit! { FollowupPattern { name: "if [condition], you may put one of those cards onto the battlefield instead of putting it into your hand", priority: 140, apply: one_of_those_instead } }

/// "Return target card from your graveyard to your hand. Put up to one other target card
/// from your graveyard on top of your library. Exile ~. If at least three green mana was
/// spent to cast ~, instead return those cards to your hand and exile ~." (Once and
/// Future): an alternative that restates the whole effect (it ends with the same last
/// instruction) replaces all of it (CR 608.2c). "Those cards" are the target cards.
pub fn instead_restates_all(l: &str, effects: &mut Vec<Effect>, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", instead ") else {
        return false;
    };
    if effects.len() < 2 {
        return false;
    }
    let Some(cond) = crate::oracle::statics::parse_condition(c, b.ctx) else {
        return false;
    };
    let cards: Vec<Sel> = (0..b.targets.len() as u8)
        .filter(|i| {
            matches!(
                b.targets[*i as usize].what,
                crate::ability::TargetKind::Object(_)
            )
        })
        .map(Sel::Target)
        .collect();
    if cards.len() < 2 {
        return false;
    }
    let phrase = "those cards".to_string();
    b.named.push((phrase.clone(), Sel::Union(cards)));
    let e = parse_clause(x, b);
    b.named.retain(|(p, _)| *p != phrase);
    let Some(e) = e else {
        return false;
    };
    let last_of = |e: &Effect| match e {
        Effect::Seq(v) => v.last().map(|x| format!("{x:?}")),
        other => Some(format!("{other:?}")),
    };
    if last_of(&e) != effects.last().and_then(last_of) {
        return false;
    }
    let all = Effect::Seq(std::mem::take(effects));
    effects.push(Effect::If {
        cond,
        then: Box::new(e),
        otherwise: Box::new(all),
    });
    true
}

/// "Whenever a land you control enters, you may return target nonland permanent card from
/// your graveyard to your hand. If that land is a Plains, you may return that nonland
/// permanent card to the battlefield instead." (Emeria Shepherd): "that land" is the
/// trigger's land; if the condition holds, you may put the card onto the battlefield, and
/// if you don't, you may still return it to your hand.
fn may_return_to_battlefield_instead(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(r) = l.strip_prefix("if that ") else {
        return false;
    };
    let Some((c, x)) = r.split_once(", you may return ") else {
        return false;
    };
    if !x.ends_with(" to the battlefield instead") {
        return false;
    }
    let Some((noun, _)) = c.split_once(' ') else {
        return false;
    };
    // The trigger's object ("that land" in a landfall ability).
    let raw = crate::oracle::raw_text().to_lowercase();
    if !b.in_trigger || !raw.contains(&format!("whenever a {noun} ")) {
        return false;
    }
    let last = match &*prev {
        Effect::Seq(v) => v.last(),
        other => Some(other),
    };
    let Some(Effect::May { who, effect }) = last else {
        return false;
    };
    let Effect::Move { what, to } = &**effect else {
        return false;
    };
    if to.zone != ZoneKind::Hand || !matches!(what, Sel::Target(_)) {
        return false;
    }
    let (who, what, to) = (who.clone(), what.clone(), to.clone());
    let phrase = format!("that {noun}");
    b.named.push((phrase.clone(), Sel::TriggerObject));
    let cond = super::conditions_referents::parse_condition_with(
        &format!("{phrase} {}", &c[noun.len() + 1..]),
        b,
    );
    b.named.retain(|(p, _)| *p != phrase);
    let Some(cond) = cond else {
        return false;
    };
    let to_hand = Effect::May {
        who: who.clone(),
        effect: Box::new(Effect::Move {
            what: what.clone(),
            to,
        }),
    };
    let alt = Effect::seq(vec![
        Effect::May {
            who,
            effect: Box::new(Effect::Move {
                what,
                to: Destination::battlefield(),
            }),
        },
        Effect::If {
            cond: Condition::Not(Box::new(Condition::PrevHappened)),
            then: Box::new(to_hand.clone()),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    let replaced = Effect::If {
        cond,
        then: Box::new(alt),
        otherwise: Box::new(to_hand),
    };
    match prev {
        Effect::Seq(v) => {
            v.pop();
            v.push(replaced);
        }
        other => *other = replaced,
    }
    true
}

inventory::submit! { FollowupPattern { name: "if that land is a [type], you may return that card to the battlefield instead", priority: 60, apply: may_return_to_battlefield_instead } }
