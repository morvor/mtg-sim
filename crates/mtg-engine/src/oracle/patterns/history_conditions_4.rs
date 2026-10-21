//! More history conditions (CR 603.4, 608.2c), read with the history clause grammar
//! (`history_conditions::history_condition`):
//!
//! * "if a player other than you lost life this turn" (Ludevic, Necro-Alchemist): some
//!   player other than you did it ([`PlayerFilter::History`]).
//! * "as long as two or more creatures died under your control this turn" (Gimli,
//!   Mournful Avenger): "two or more creatures you controlled died this turn" (CR 700.4;
//!   whose control is judged as they died, CR 603.10a).

use super::history_conditions::history_condition;
use super::history_filters_4::player_history;
use crate::ability::*;
use crate::oracle::phrases::end;

/// "a player other than you [did something] [window]", "an opponent other than ..."
fn other_than_you(c: &str) -> Option<Condition> {
    let c = end(c);
    let (base, r) = if let Some(r) = c.strip_prefix("a player other than you ") {
        (PlayerFilter::NotYou, r)
    } else {
        return None;
    };
    let did = player_history(r)?;
    Some(Condition::Compare(
        Value::CountPlayers(PlayerFilter::And(vec![base, did])),
        Cmp::Ge,
        Value::Const(1),
    ))
}

inventory::submit! { super::ConditionPattern { name: "history 4: a player other than you [did something] this turn", priority: 900, parse: other_than_you } }

/// "[objects] died under your control [window]" / "[objects] left the battlefield under
/// your control [window]": "[objects] you controlled died [window]".
fn under_your_control(c: &str) -> Option<Condition> {
    let c = end(c);
    for verb in ["died", "left the battlefield"] {
        let pat = format!(" {verb} under your control ");
        let Some(i) = c.find(&pat) else { continue };
        let (np, rest) = (&c[..i], &c[i + pat.len()..]);
        if np.contains(" you ") || np.contains(" control") || np.starts_with("~") {
            return None;
        }
        return history_condition(&format!("{np} you controlled {verb} {rest}"));
    }
    None
}

inventory::submit! { super::ConditionPattern { name: "history 4: [objects] died under your control this turn", priority: 900, parse: under_your_control } }

// ---------------------------------------------------------------------------
// As the turn began (`turn_start.rs`)
// ---------------------------------------------------------------------------

/// A player subject without a builder: "you" only.
fn you_subject(c: &str) -> Option<(PlayerRef, &str)> {
    c.strip_prefix("you ").map(|r| (PlayerRef::You, r))
}

/// "[you] were the monarch as the turn began", "[you] had no cards in hand at the
/// beginning of this turn", "[you] had a card in hand at the beginning of this turn".
fn turn_began_clause(who: PlayerRef, r: &str) -> Option<Condition> {
    let stat = |s| Value::AsTurnBegan(s, who.clone());
    let cmp = |v, c, n| Some(Condition::Compare(v, c, Value::Const(n)));
    match r {
        "were the monarch as the turn began" | "was the monarch as the turn began" => {
            cmp(stat(TurnStartStat::Monarch), Cmp::Ge, 1)
        }
        "weren't the monarch as the turn began" | "wasn't the monarch as the turn began" => {
            cmp(stat(TurnStartStat::Monarch), Cmp::Eq, 0)
        }
        "had no cards in hand at the beginning of this turn" => {
            cmp(stat(TurnStartStat::HandSize), Cmp::Eq, 0)
        }
        "had a card in hand at the beginning of this turn"
        | "had one or more cards in hand at the beginning of this turn" => {
            cmp(stat(TurnStartStat::HandSize), Cmp::Ge, 1)
        }
        // "If you had a card in hand, ..." right after a condition about the beginning of
        // this turn (Mindstorm Crown): the same moment.
        "had a card in hand"
            if crate::oracle::raw_text()
                .to_lowercase()
                .contains("cards in hand at the beginning of this turn. if you had a card in hand,") =>
        {
            cmp(stat(TurnStartStat::HandSize), Cmp::Ge, 1)
        }
        _ => None,
    }
}

fn turn_began(c: &str) -> Option<Condition> {
    let (who, r) = you_subject(end(c))?;
    turn_began_clause(who, r)
}

inventory::submit! { super::ConditionPattern { name: "history 4: you were the monarch as the turn began / had no cards in hand at the beginning of this turn", priority: 900, parse: turn_began } }

/// "a player's life total is less than or equal to half their starting life total" (Game
/// Over), "an opponent's life total is ... half their starting life total": each player's
/// own starting life total (CR 119.1).
fn half_starting_life(c: &str) -> Option<Condition> {
    let c = end(c);
    let (base, r) = if let Some(r) = c.strip_prefix("a player's life total is ") {
        (PlayerFilter::Any, r)
    } else if let Some(r) = c.strip_prefix("an opponent's life total is ") {
        (PlayerFilter::Opponent, r)
    } else {
        return None;
    };
    let cmp = match r.strip_suffix(" half their starting life total")? {
        "less than or equal to" | "at most" => Cmp::Le,
        "less than" => Cmp::Lt,
        "greater than or equal to" | "at least" => Cmp::Ge,
        "greater than" => Cmp::Gt,
        _ => return None,
    };
    Some(Condition::Compare(
        Value::CountPlayers(PlayerFilter::And(vec![
            base,
            PlayerFilter::HalfStartingLife(cmp),
        ])),
        Cmp::Ge,
        Value::Const(1),
    ))
}

inventory::submit! { super::ConditionPattern { name: "history 4: a player's life total is at most half their starting life total", priority: 900, parse: half_starting_life } }

/// A player the text names in a value phrase: "you"/"your", "they"/"their"/"that
/// player('s)" (the player the ability is about).
fn named_player<'a>(
    s: &'a str,
    b: &crate::oracle::effects::Builder,
    possessive: bool,
) -> Option<(PlayerRef, &'a str)> {
    use super::oracle_hardening_referents::is_no_player_referent;
    let words: &[(&str, bool)] = if possessive {
        &[("your ", true), ("their ", false), ("that player's ", false)]
    } else {
        &[("you ", true), ("they ", false), ("that player ", false)]
    };
    for (w, you) in words {
        if let Some(r) = s.strip_prefix(w) {
            if *you {
                return Some((PlayerRef::You, r));
            }
            if matches!(b.it_player, PlayerRef::You) || is_no_player_referent(&b.it_player) {
                return None;
            }
            return Some((b.it_player.clone(), r));
        }
    }
    None
}

/// "the number of untapped lands they controlled at the beginning of this turn" (Power
/// Surge), "that player's life total as the turn began" (Sengir, the Dark Baron).
pub fn value_atom(
    s: &str,
    b: &mut crate::oracle::effects::Builder,
) -> Option<(Value, String)> {
    if let Some(v) = spells_cast_value(s, b) {
        return Some(v);
    }
    // "the greatest amount of damage dealt by a source to a permanent or player this
    // turn" (Impact Resonance).
    if let Some(rest) =
        s.strip_prefix("the greatest amount of damage dealt by a source to a permanent or player this turn")
    {
        return Some((
            Value::Custom(crate::kw::damage_history_4::GREATEST_DAMAGE_BY_A_SOURCE.into()),
            rest.to_string(),
        ));
    }
    if let Some(r) = s.strip_prefix("the number of untapped lands ") {
        let (who, r) = named_player(r, b, false)?;
        let rest = r.strip_prefix("controlled at the beginning of this turn")?;
        return Some((
            Value::AsTurnBegan(TurnStartStat::UntappedLands, who),
            rest.to_string(),
        ));
    }
    let (who, r) = named_player(s, b, true)?;
    let rest = r.strip_prefix("life total as the turn began")?;
    Some((
        Value::AsTurnBegan(TurnStartStat::Life, who),
        rest.to_string(),
    ))
}

// ---------------------------------------------------------------------------
// Conditions joined by "and"
// ---------------------------------------------------------------------------

/// A condition without referents, read by the whole condition grammar.
fn free_condition(c: &str) -> Option<Condition> {
    let tl = crate::types::TypeLine::default();
    let ctx = crate::oracle::CompileContext {
        card_name: "",
        full_name: "",
        type_line: &tl,
        layout: crate::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    crate::oracle::statics::parse_condition(c, &ctx)
}

/// "[condition] and [condition]", "[condition], [condition], and [condition]" (Frodo,
/// Adventurous Hobbit: "~ is your Ring-bearer and the Ring has tempted you two or more
/// times this game"; Happily Ever After): each part a whole condition of its own.
fn conditions_and(c: &str) -> Option<Condition> {
    let c = end(c);
    let parts: Vec<String> = if let Some((list, last)) = c.rsplit_once(", and ") {
        let mut v: Vec<String> = list.split(", ").map(str::to_string).collect();
        v.push(last.to_string());
        v
    } else {
        // Two parts: each split point in turn.
        for (i, _) in c.match_indices(" and ") {
            let (x, y) = (&c[..i], &c[i + " and ".len()..]);
            if let (Some(a), Some(b)) = (free_condition(x), free_condition(y)) {
                return Some(Condition::And(vec![a, b]));
            }
        }
        return None;
    };
    if parts.len() < 3 {
        return None;
    }
    let conds: Option<Vec<Condition>> = parts.iter().map(|p| free_condition(p)).collect();
    Some(Condition::And(conds?))
}

inventory::submit! { super::ConditionPattern { name: "history 4: [condition] and [condition]", priority: 990, parse: conditions_and } }

// ---------------------------------------------------------------------------
// The first spell you've cast this game
// ---------------------------------------------------------------------------

/// "If ~ is the first spell you've cast this game, you may cast it without paying its
/// mana cost." (Once Upon a Time): an alternative cost of nothing (CR 118.9) while you
/// haven't cast a spell this game (a copy isn't cast, CR 707.10).
fn first_spell_free(
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let lower = text.to_lowercase();
    let l = end(lower.trim());
    let (when, body) = l.split_once(", you may ")?;
    let window = match when {
        "if ~ is the first spell you've cast this game" => HistoryWindow::ThisGame,
        _ => return None,
    };
    if body != "cast it without paying its mana cost" {
        return None;
    }
    let cast = TriggerCond::CastSpell {
        who: PlayerRel::You,
        filter: Filter::Spell,
    };
    let none_yet = Condition::Compare(
        Value::EventsDuring(window, Box::new(cast), Tally::Events),
        Cmp::Eq,
        Value::Const(0),
    );
    Some(vec![super::costs_casting_self::this_spell_cost_ability(
        CostChange::AlternativeCost(Cost::free()),
        Some(none_yet),
        text,
    )])
}

inventory::submit! { super::AbilityPattern { name: "history 4: if ~ is the first spell you've cast this game, you may cast it without paying its mana cost", priority: 80, parse: first_spell_free } }

// ---------------------------------------------------------------------------
// The first time something happened to an object this turn
// ---------------------------------------------------------------------------

/// "it's the first time that creature has become tapped this turn" (Captain America,
/// Living Legend): the event about that object is the only one of its kind this turn.
/// Becoming tapped is one event per tapping (CR 701.26a).
pub fn first_time_referent(
    c: &str,
    b: &mut crate::oracle::effects::Builder,
) -> Option<Condition> {
    let r = c.strip_prefix("it's the first time ")?;
    let (who, verb) = r
        .strip_suffix(" has become tapped this turn")
        .map(|w| (w, "became tapped"))?;
    let before = b.targets.len();
    let saved = b.it.clone();
    let got = crate::oracle::effects::object_ref(who, b);
    b.it = saved;
    let (sel, rest) = got?;
    if b.targets.len() != before
        || !rest.trim().is_empty()
        || super::oracle_hardening_referents::is_no_referent(&sel)
    {
        b.targets.truncate(before);
        return None;
    }
    let cond = history_condition(&format!("~ {verb} this turn"))?;
    let Condition::Compare(v, Cmp::Ge, Value::Const(1)) = cond else {
        return None;
    };
    let once = Condition::Compare(v, Cmp::Eq, Value::Const(1));
    if matches!(sel, Sel::This) {
        return Some(once);
    }
    super::history_conditions::about(once, &sel)
}

// ---------------------------------------------------------------------------
// Damage totals
// ---------------------------------------------------------------------------

/// "red sources you controlled dealt 4 or more noncombat damage this turn" (Ojer Axonil,
/// Deepest Might): the total of the damage all such sources dealt this turn (CR 120).
fn sources_dealt_total(c: &str) -> Option<Condition> {
    let c = end(c);
    let (subj, r) = c.split_once(" dealt ")?;
    if !subj.contains("sources") || subj.starts_with("a ") || subj.starts_with("one ") {
        return None;
    }
    let r = r.strip_suffix(" this turn")?;
    let (n, kind) = crate::oracle::phrases::parse_number(r)?;
    let Value::Const(n) = n else { return None };
    let kind = kind.strip_prefix("or more ")?;
    if !matches!(kind, "damage" | "combat damage" | "noncombat damage") {
        return None;
    }
    let one = subj
        .replacen("sources", "source", 1)
        .replace(" you controlled", " you control");
    let (cond, _, _) =
        crate::oracle::triggers::parse_trigger_condition(&format!("whenever a {one} deals {kind}"))?;
    // Counted event by event, not by batch.
    let cond = match cond {
        TriggerCond::Batched { trigger, .. } => *trigger,
        c => c,
    };
    let json = serde_json::to_string(&cond).ok()?;
    if !json.contains("DealsDamage") || json.contains("Batched") || json.contains("Custom") {
        return None;
    }
    Some(Condition::Compare(
        Value::EventsThisTurn(Box::new(cond), Tally::Amount),
        Cmp::Ge,
        Value::Const(n),
    ))
}

inventory::submit! { super::ConditionPattern { name: "history 4: [sources] dealt N or more damage this turn", priority: 900, parse: sources_dealt_total } }

/// "you were dealt damage this turn by a red instant or sorcery spell" (Suffocation): the
/// window before the source.
fn dealt_damage_this_turn_by(c: &str) -> Option<Condition> {
    let c = end(c);
    let (head, by) = c.split_once(" damage this turn by ")?;
    if !head.contains(" dealt") {
        return None;
    }
    history_condition(&format!("{head} damage by {by} this turn"))
}

inventory::submit! { super::ConditionPattern { name: "history 4: [player] was dealt damage this turn by [source]", priority: 900, parse: dealt_damage_this_turn_by } }

// ---------------------------------------------------------------------------
// Conditions about the opponents of the player the ability is about
// ---------------------------------------------------------------------------

/// "[instruction] unless one of their opponents was dealt damage this turn" (Antagonism):
/// "their" is the player the ability is about (CR 102.3).
fn unless_their_opponent(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    use super::oracle_hardening_referents::is_no_player_referent;
    let l = end(l);
    let (x, c) = l.rsplit_once(" unless one of their opponents ")?;
    let who = b.it_player.clone();
    if matches!(who, PlayerRef::You) || is_no_player_referent(&who) {
        return None;
    }
    let did = player_history(c)?;
    let cond = Condition::Compare(
        Value::CountPlayers(PlayerFilter::And(vec![
            PlayerFilter::OpponentOf(Box::new(who)),
            did,
        ])),
        Cmp::Ge,
        Value::Const(1),
    );
    let e = crate::oracle::effects::parse_clause(x, b)?;
    Some(Effect::If {
        cond: Condition::Not(Box::new(cond)),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { super::EffectPattern { name: "history 4: [instruction] unless one of their opponents [did something] this turn", priority: 390, parse: unless_their_opponent } }

/// "[they] were attacked this turn by an Assassin you controlled" (Ramses, Assassin Lord):
/// a creature matching the phrase (as it was declared, CR 508.1) attacked that player this
/// turn.
pub fn was_attacked_by(r: &str, who: &PlayerRef) -> Option<Condition> {
    let rel = match who {
        PlayerRef::TriggerPlayer => PlayerRel::TriggerPlayer,
        PlayerRef::Target(t) => PlayerRel::Target(*t),
        _ => return None,
    };
    let r = end(r);
    let by = r
        .strip_prefix("were attacked this turn by ")
        .or_else(|| r.strip_prefix("was attacked this turn by "))
        .or_else(|| {
            r.strip_prefix("were attacked by ")
                .or_else(|| r.strip_prefix("was attacked by "))?
                .strip_suffix(" this turn")
        })?;
    let np = by
        .strip_prefix("an ")
        .or_else(|| by.strip_prefix("a "))?
        .replace(" you controlled", " you control");
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(&np)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::EventsThisTurn(
            Box::new(TriggerCond::AttacksRecipient {
                attacker: f,
                recipient: DamageRecipient::Player(rel),
            }),
            Tally::Events,
        ),
        Cmp::Ge,
        Value::Const(1),
    ))
}



/// A spell phrase listing kinds with "and" ("instant and sorcery spells": spells of either
/// kind, CR 205.2b), as a filter of spells.
fn spells_phrase(np: &str) -> Option<Filter> {
    let np = np.replacen(" and ", " or ", 1);
    let (f, _, tail) = crate::oracle::phrases::parse_object_phrase(&np)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let json = serde_json::to_string(&f).ok()?;
    json.contains("\"Spell\"").then_some(f)
}

/// "the greatest mana value among instant and sorcery spells you've cast this turn"
/// (Rootha, Mastering the Moment), "the number of instant and sorcery spells you've cast
/// this turn", "the number of instant and sorcery spells cast before that spell this
/// turn" (Sentinel Tower: the spells cast before the spell the ability triggered on, CR
/// 601.2i).
fn spells_cast_value(
    s: &str,
    b: &mut crate::oracle::effects::Builder,
) -> Option<(Value, String)> {
    let cast = |who: PlayerRel, f: Filter| TriggerCond::CastSpell { who, filter: f };
    if let Some(r) = s.strip_prefix("the greatest mana value among ") {
        let (np, rest) = r.split_once(" you've cast this turn")?;
        let f = spells_phrase(np)?;
        return Some((
            Value::Extreme(
                Box::new(Value::ManaValueOf(Box::new(Sel::Var(vars::TESTED)))),
                Box::new(Sel::ThisTurn(Box::new(cast(PlayerRel::You, f)))),
                true,
            ),
            rest.to_string(),
        ));
    }
    let r = s.strip_prefix("the number of ")?;
    if !r.contains(" and ") {
        return None;
    }
    if let Some((np, rest)) = r.split_once(" cast before that spell this turn") {
        // Only in an ability that triggered on a spell being cast.
        let raw = crate::oracle::raw_text().to_lowercase();
        if !b.in_trigger || !(raw.contains(" is cast") || raw.contains(" casts ")) {
            return None;
        }
        let f = spells_phrase(np)?;
        let through = Filter::and(vec![
            f,
            Filter::Custom(format!("{}that spell", crate::kw::cast_history::CAST_BY_THEN).into()),
        ]);
        return Some((
            Value::Sum(vec![
                Value::EventsThisTurn(Box::new(cast(PlayerRel::Any, through)), Tally::Events),
                Value::Const(-1),
            ]),
            rest.to_string(),
        ));
    }
    let (np, rest) = r.split_once(" you've cast this turn")?;
    let f = spells_phrase(np)?;
    Some((
        Value::EventsThisTurn(Box::new(cast(PlayerRel::You, f)), Tally::Events),
        rest.to_string(),
    ))
}

/// "At the beginning of the next end step, destroy all non-Wall creatures that player
/// controls that didn't attack this turn. Ignore this effect for each creature the player
/// didn't control continuously since the beginning of the turn." (Siren's Call, after
/// "Creatures the active player controls attack this turn if able."): "that player" is
/// the active player, whose turn the next end step is in (the spell is cast before
/// attackers are declared); the creatures are those of theirs that didn't attack and that
/// they've controlled continuously since the turn began (CR 302.6).
fn ignore_for_each_not_controlled(
    block: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    const IGNORE: &str = " ignore this effect for each creature the player didn't control continuously since the beginning of the turn.";
    let t = block.trim();
    let lower = t.to_lowercase();
    let head = lower.strip_suffix(IGNORE)?;
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("creatures the active player controls attack this turn if able") {
        return None;
    }
    let (before, after) = head.split_once(" that player controls that didn't attack this turn.")?;
    if !after.is_empty() || !before.starts_with("at the beginning of the next end step, destroy all ") {
        return None;
    }
    let rewritten = format!(
        "{before} the active player has controlled continuously since the beginning of the turn that didn't attack this turn."
    );
    let v = crate::oracle::parse_ability(&rewritten, ctx)?;
    Some(
        v.into_iter()
            .map(|a| {
                let mut d = (*a).clone();
                d.text = t.to_string();
                std::sync::Arc::new(d)
            })
            .collect(),
    )
}

inventory::submit! { super::AbilityPattern { name: "history 4: ignore this effect for each creature the player didn't control continuously since the beginning of the turn", priority: 90, parse: ignore_for_each_not_controlled } }

/// "an oil counter was removed from a permanent you controlled this turn" (Churning
/// Reservoir): such a counter was removed this turn.
fn counter_removed(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c.strip_prefix("an ").or_else(|| c.strip_prefix("a "))?;
    let (kind, obj) = r.split_once(" counter was removed from ")?;
    let obj = obj.strip_suffix(" this turn")?;
    let obj = obj.replace(" you controlled", " you control");
    let article = if kind.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
    let (cond, _, _) = crate::oracle::triggers::parse_trigger_condition(&format!(
        "whenever {article} {kind} counter is removed from {obj}"
    ))?;
    if !matches!(cond, TriggerCond::CountersRemoved { .. }) {
        return None;
    }
    Some(Condition::Compare(
        Value::EventsThisTurn(Box::new(cond), Tally::Events),
        Cmp::Ge,
        Value::Const(1),
    ))
}

inventory::submit! { super::ConditionPattern { name: "history 4: a [kind] counter was removed from [object] this turn", priority: 900, parse: counter_removed } }

/// "[condition] or [condition]": either, each a whole condition of its own (Churning
/// Reservoir: "an oil counter was removed from a permanent you controlled this turn or a
/// permanent with an oil counter on it was put into a graveyard this turn").
fn conditions_or(c: &str) -> Option<Condition> {
    let c = end(c);
    for (i, _) in c.match_indices(" or ") {
        let (x, y) = (&c[..i], &c[i + " or ".len()..]);
        if let (Some(a), Some(b)) = (free_condition(x), free_condition(y)) {
            return Some(Condition::Or(vec![a, b]));
        }
    }
    None
}

inventory::submit! { super::ConditionPattern { name: "history 4: [condition] or [condition]", priority: 995, parse: conditions_or } }

/// "You may cast a spell from your hand without paying its mana cost if it has the same
/// name as a spell that was cast this turn." (Twinning Glass): only such a card may be
/// chosen (whoever cast the first spell, Twinning Glass rulings).
fn cast_if_same_name_as_cast(l: &str, b: &mut crate::oracle::effects::Builder) -> Option<Effect> {
    let l = end(l);
    let (head, cond) = l.rsplit_once(" if it has the same name as ")?;
    let who = match cond {
        "a spell that was cast this turn" => PlayerRel::Any,
        "a spell you've cast this turn" | "a spell you cast this turn" => PlayerRel::You,
        _ => return None,
    };
    if !head.contains(" cast ") || !head.ends_with("without paying its mana cost") {
        return None;
    }
    let e = crate::oracle::effects::parse_clause(head, b)?;
    let mut json = serde_json::to_value(&e).ok()?;
    let filter = json
        .get_mut("CastCard")?
        .get_mut("what")?
        .get_mut("Choose")?
        .get_mut("filter")?;
    let f: Filter = serde_json::from_value(filter.clone()).ok()?;
    let same = Filter::SameNameAs(Box::new(Sel::ThisTurn(Box::new(TriggerCond::CastSpell {
        who,
        filter: Filter::Spell,
    }))));
    *filter = serde_json::to_value(Filter::and(vec![f, same])).ok()?;
    serde_json::from_value(json).ok()
}

inventory::submit! { super::EffectPattern { name: "history 4: cast a spell ... if it has the same name as a spell that was cast this turn", priority: 10, parse: cast_if_same_name_as_cast } }

/// "if a Giant, Wizard, or spell you controlled dealt damage to it this turn" (Aegar, the
/// Freezing Flame): damage dealt this turn to the object the ability is about by such a
/// source (as it was then controlled, CR 603.10a for sources that left).
pub fn sources_dealt_damage_to_it(
    c: &str,
    b: &mut crate::oracle::effects::Builder,
) -> Option<Condition> {
    if c.starts_with("~ ") || !c.contains(" dealt damage to it this turn") {
        return None;
    }
    let sel = match &b.it {
        Sel::This => return None,
        it if super::oracle_hardening_referents::is_no_referent(it) => return None,
        it => it.clone(),
    };
    let cond = history_condition(&c.replacen(" to it ", " to a permanent ", 1))?;
    let mut json = serde_json::to_value(&cond).ok()?;
    let with = serde_json::to_value(DamageRecipient::Object(Filter::In(Box::new(sel)))).ok()?;
    let mut n = 0;
    fn walk(v: &mut serde_json::Value, with: &serde_json::Value, n: &mut usize) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, x) in m.iter_mut() {
                    if k == "to" && x.get("Object").is_some() {
                        *x = with.clone();
                        *n += 1;
                    } else {
                        walk(x, with, n);
                    }
                }
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, with, n)),
            _ => {}
        }
    }
    walk(&mut json, &with, &mut n);
    if n != 1 {
        return None;
    }
    serde_json::from_value(json).ok()
}

// ---------------------------------------------------------------------------
// "Do the same for [keywords]."
// ---------------------------------------------------------------------------

/// "If that creature has haste and ~ doesn't, put a haste counter on ~. Do the same for
/// flying, first strike, ..., and vigilance." (Super-Adaptoid): the sentence again for
/// each listed keyword in place of the first (each a keyword counter, CR 122.1b), in that
/// order.
fn do_the_same_for(block: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let i = lower.find(". do the same for ")?;
    let list = lower[i + ". do the same for ".len()..].strip_suffix('.')?;
    if list.contains(". ") {
        return None;
    }
    let before = &t[..i];
    let start = before.rfind(". ").map_or(0, |j| j + 2);
    let sentence = &t[start..i];
    let s_lower = sentence.to_lowercase();
    let word = |hay: &str, w: &str| {
        hay.match_indices(w).any(|(k, _)| {
            let pre = hay[..k].chars().last();
            let post = hay[k + w.len()..].chars().next();
            pre.is_none_or(|c| c == ' ') && post.is_none_or(|c| c == ' ' || c == ',')
        })
    };
    let kws: Vec<String> = crate::keywords::KeywordKind::ALL
        .iter()
        .map(|k| k.name().to_lowercase())
        .filter(|n| word(&s_lower, n))
        .collect();
    let [kw] = kws.as_slice() else { return None };
    let items: Vec<&str> = list
        .split(", and ")
        .flat_map(|p| p.split(", "))
        .flat_map(|p| p.split(" and "))
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    // "If that creature has haste and ~ doesn't, ...": "and ~ doesn't have haste".
    let full = s_lower.replace(" and ~ doesn't,", &format!(" and ~ doesn't have {kw},"));
    let mut sentences = vec![full.clone()];
    for item in items {
        crate::keywords::KeywordKind::from_name(item)?;
        sentences.push(full.replace(kw.as_str(), item));
    }
    let rewritten = format!("{}{}.", &t[..start], sentences.join(". "));
    let v = crate::oracle::parse_ability(&rewritten, ctx)?;
    Some(
        v.into_iter()
            .map(|a| {
                let mut d = (*a).clone();
                d.text = t.to_string();
                std::sync::Arc::new(d)
            })
            .collect(),
    )
}

inventory::submit! { super::AbilityPattern { name: "history 4: [sentence about a keyword]. Do the same for [keywords].", priority: 90, parse: do_the_same_for } }

// ---------------------------------------------------------------------------
// "it's the first A, the first B, or the first C you've cast this turn"
// ---------------------------------------------------------------------------

/// "if it's the first instant spell, the first sorcery spell, or the first Otter spell
/// other than Alania you've cast this turn" (Alania, Divergent Storm): any of the ordinal
/// conditions. "Other than ~" is neither the source nor the spell it was (CR 400.7).
pub fn nth_spell_list(c: &str, b: &mut crate::oracle::effects::Builder) -> Option<Condition> {
    let r = c.strip_prefix("it's ")?;
    let body = r.strip_suffix(" you've cast this turn")?;
    let (list, last) = body.rsplit_once(", or ")?;
    let mut items: Vec<&str> = list.split(", ").collect();
    items.push(last);
    if items.len() < 2 || items.iter().any(|i| !i.starts_with("the ")) {
        return None;
    }
    let other = serde_json::to_value(Filter::Other).ok()?;
    let not_self = serde_json::to_value(Filter::not(Filter::Custom(
        crate::kw::damage_history_4::SOURCE_OR_ITS_SPELL.into(),
    )))
    .ok()?;
    let mut conds = Vec::new();
    for item in items {
        let cond = super::history_conditions::nth_spell(&format!("it's {item} you've cast this turn"), b)?;
        let mut json = serde_json::to_value(&cond).ok()?;
        fn walk(v: &mut serde_json::Value, from: &serde_json::Value, to: &serde_json::Value) {
            if *v == *from {
                *v = to.clone();
                return;
            }
            match v {
                serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, from, to)),
                serde_json::Value::Object(m) => m.values_mut().for_each(|x| walk(x, from, to)),
                _ => {}
            }
        }
        walk(&mut json, &other, &not_self);
        conds.push(serde_json::from_value(json).ok()?);
    }
    Some(Condition::Or(conds))
}

/// "a permanent entered the battlefield face down under your control this turn"
/// (Oblivious Bookworm, CR 708.2a).
fn entered_face_down(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c.strip_suffix(" entered the battlefield face down under your control this turn")?;
    let np = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(np)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    let f = Filter::and(vec![
        f,
        Filter::ControlledBy(PlayerRel::You),
        Filter::Custom(crate::kw::damage_history_4::ENTERED_FACE_DOWN.into()),
    ]);
    Some(Condition::Compare(
        Value::EventsThisTurn(Box::new(TriggerCond::EntersBattlefield(f)), Tally::Events),
        Cmp::Ge,
        Value::Const(1),
    ))
}

inventory::submit! { super::ConditionPattern { name: "history 4: a permanent entered the battlefield face down under your control this turn", priority: 900, parse: entered_face_down } }

/// "..., if that player lost less than 8 life this turn, they lose life equal to the
/// difference." (Lolth, Spider Queen's emblem): 8 minus the life they lost this turn.
fn lost_less_than_the_difference(
    block: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let i = lower.find(", if that player lost less than ")?;
    let r = &lower[i + ", if that player lost less than ".len()..];
    let (n, rest) = r.split_once(" life this turn, ")?;
    let (Value::Const(_), "") = crate::oracle::phrases::parse_number(n)? else {
        return None;
    };
    if rest != "they lose life equal to the difference." {
        return None;
    }
    let rewritten = format!(
        "{}, if that player lost less than {n} life this turn, they lose life equal to {n} minus the amount of life they lost this turn.",
        &t[..i]
    );
    let v = crate::oracle::parse_ability(&rewritten, ctx)?;
    Some(
        v.into_iter()
            .map(|a| {
                let mut d = (*a).clone();
                d.text = t.to_string();
                std::sync::Arc::new(d)
            })
            .collect(),
    )
}

inventory::submit! { super::AbilityPattern { name: "history 4: if that player lost less than N life this turn, they lose life equal to the difference", priority: 90, parse: lost_less_than_the_difference } }

/// "if any of that damage was dealt by a Warrior" (Mindblade Render, after "Whenever your
/// opponents are dealt combat damage"): a source of the damage the ability triggered on
/// (the batch's sources) is one, as it last existed if it has left (CR 603.10a).
fn any_of_that_damage_by(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c.strip_prefix("any of that damage was dealt by ")?;
    let raw = crate::oracle::raw_text().to_lowercase();
    if !(raw.starts_with("whenever ") && raw.contains(" dealt combat damage, if any of that damage"))
    {
        return None;
    }
    let np = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, false, tail) = crate::oracle::phrases::parse_object_phrase(np)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::CountSel(Box::new(Sel::Matching(Box::new(Sel::TriggerObjects), f))),
        Cmp::Ge,
        Value::Const(1),
    ))
}

inventory::submit! { super::ConditionPattern { name: "history 4: any of that damage was dealt by [a source]", priority: 900, parse: any_of_that_damage_by } }

/// "Whenever you waterbend, earthbend, firebend, or airbend, draw a card. Then if you've
/// done all four this turn, transform ~." (Avatar Aang): each of the trigger's actions
/// happened this turn.
fn done_all_of_them(block: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let r = lower.strip_prefix("whenever you ")?;
    // "A, B, C, or D, [effect]" / "A or B, [effect]".
    let (head, rest) = r.split_once(" or ")?;
    let last = rest.split_once(", ")?.0;
    let mut items: Vec<&str> = head.trim_end_matches(',').split(", ").collect();
    items.push(last);
    if items.iter().any(|w| w.contains(' ')) {
        return None;
    }
    let n = items.len();
    let word = match n {
        2 => "both",
        3 => "all three",
        4 => "all four",
        _ => return None,
    };
    let marker = format!(" then if you've done {word} this turn, ");
    let i = lower.find(&marker)?;
    let pasts: Option<Vec<String>> = items
        .iter()
        .map(|v| {
            super::history_conditions::VERBS
                .iter()
                .find(|(_, base, _)| base == v)
                .map(|(past, _, _)| format!("you've {past} this turn"))
        })
        .collect();
    let cond = pasts?.join(" and ");
    let rewritten = format!(
        "{} Then if {cond}, {}",
        &t[..i],
        &t[i + marker.len()..]
    );
    let v = crate::oracle::parse_ability(&rewritten, ctx)?;
    Some(
        v.into_iter()
            .map(|a| {
                let mut d = (*a).clone();
                d.text = t.to_string();
                std::sync::Arc::new(d)
            })
            .collect(),
    )
}

inventory::submit! { super::AbilityPattern { name: "history 4: then if you've done all four this turn", priority: 90, parse: done_all_of_them } }

/// "if none of them were cast", "if no mana was spent to cast them" (Satoru, the
/// Infiltrator: "if none of them were cast or no mana was spent to cast them", after "one
/// or more ... creatures you control enter"): about the objects the ability triggered on.
/// Mana spent for additional costs or cost increases counts (Satoru rulings).
fn none_of_them_cast(c: &str) -> Option<Condition> {
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains(" enter, if ") || !raw.contains(" of them") && !raw.contains(" cast them") {
        return None;
    }
    let f = match end(c) {
        "none of them were cast" => Filter::Custom(crate::kw::cast_history::WAS_CAST.into()),
        "no mana was spent to cast them" => {
            Filter::Custom(crate::kw::damage_history_4::CAST_WITH_MANA.into())
        }
        _ => return None,
    };
    Some(Condition::Not(Box::new(Condition::SelNonEmpty(Sel::Matching(
        Box::new(Sel::TriggerObjects),
        f,
    )))))
}

inventory::submit! { super::ConditionPattern { name: "history 4: none of them were cast / no mana was spent to cast them", priority: 900, parse: none_of_them_cast } }

/// "you have seven or more land cards in your hand" (Sasaya, Orochi Ascendant): cards of a
/// kind in your hand.
fn cards_of_a_kind_in_hand(c: &str) -> Option<Condition> {
    let c = end(c);
    let r = c.strip_prefix("you have ")?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    let Value::Const(_) = n else { return None };
    let (cmp, np) = if let Some(x) = r.strip_prefix("or more ") {
        (Cmp::Ge, x)
    } else if let Some(x) = r.strip_prefix("or fewer ") {
        (Cmp::Le, x)
    } else {
        return None;
    };
    let np = np.strip_suffix(" in your hand")?;
    if !np.ends_with(" cards") || np == "cards" {
        return None;
    }
    let (f, true, tail) = crate::oracle::phrases::parse_object_phrase(np)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::Count(Filter::and(vec![
            f,
            Filter::InZone(ZoneKind::Hand),
            Filter::OwnedBy(PlayerRel::You),
        ])),
        cmp,
        n,
    ))
}

inventory::submit! { super::ConditionPattern { name: "history 4: you have N or more [kind] cards in your hand", priority: 900, parse: cards_of_a_kind_in_hand } }
