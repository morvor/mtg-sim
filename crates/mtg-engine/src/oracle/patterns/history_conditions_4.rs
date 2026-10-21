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

// ---------------------------------------------------------------------------
// Exerted this turn
// ---------------------------------------------------------------------------

/// "~ hasn't been exerted this turn" (Combat Celebrant): no player exerted it this turn
/// (CR 701.43a).
fn not_exerted(c: &str) -> Option<Condition> {
    let c = end(c);
    let subj = c
        .strip_suffix(" hasn't been exerted this turn")
        .or_else(|| c.strip_suffix(" wasn't exerted this turn"))?;
    if subj != "~" {
        return None;
    }
    // Whoever exerted it (its controller then).
    let exerted = TriggerCond::Where {
        trigger: Box::new(TriggerCond::PlayerAction {
            name: crate::kwa::exert::EXERTED_EVENT.into(),
            who: PlayerRel::Any,
        }),
        cond: Condition::SelMatches(Sel::TriggerObject, Filter::Source),
    };
    Some(Condition::Compare(
        Value::EventsThisTurn(Box::new(exerted), Tally::Events),
        Cmp::Eq,
        Value::Const(0),
    ))
}

inventory::submit! { super::ConditionPattern { name: "history 4: ~ hasn't been exerted this turn", priority: 900, parse: not_exerted } }

/// "If ~ hasn't been exerted this turn, you may exert it as it attacks. When you do, ..."
/// (Combat Celebrant): the optional attack cost (CR 508.1g) while the condition holds.
fn conditional_exert(
    block: &str,
    ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    let r = lower.strip_prefix("if ")?;
    let (c, rest) = r.split_once(", ")?;
    let rest_lower = rest.strip_prefix("you may exert it as it attacks.")?;
    let cond = crate::oracle::statics::parse_condition(c, ctx)?;
    let orig_rest = &t[t.len() - rest_lower.len()..];
    let rewritten = format!("You may exert ~ as it attacks.{orig_rest}");
    let v = crate::oracle::parse_ability(&rewritten, ctx)?;
    let mut out = Vec::new();
    for a in v {
        let mut d = (*a).clone();
        if let AbilityKind::Static(s) = &mut d.kind {
            let exert = match &s.effect {
                StaticEffect::OptionalAttackCost { .. } => true,
                StaticEffect::Custom(n) => n.as_str() == crate::kw::exert::EXERT_AS_ATTACKS,
                _ => false,
            };
            if !exert || s.condition.is_some() {
                return None;
            }
            s.condition = Some(cond.clone());
        }
        d.text = t.to_string();
        out.push(std::sync::Arc::new(d));
    }
    Some(out)
}

inventory::submit! { super::AbilityPattern { name: "history 4: if [condition], you may exert it as it attacks", priority: 90, parse: conditional_exert } }
