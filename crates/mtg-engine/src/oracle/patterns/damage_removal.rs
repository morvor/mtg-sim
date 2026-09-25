//! Damage, destruction, exile, bounce and other removal effects.
//!
//! Damage clauses are parsed compositionally:
//!
//! ```text
//! [you may have] SOURCE deal(s) PART [and PART]* [, where x is VALUE]
//! PART       := AMOUNT damage to RECIPIENTS
//!             | damage equal to VALUE to RECIPIENTS
//!             | damage to RECIPIENTS equal to VALUE
//!             | AMOUNT damage divided as you choose among COUNTED-TARGETS
//! RECIPIENTS := each of COUNTED-TARGETS | ITEM [and ITEM]*
//! ITEM       := any target | target ... | each OBJECT [they control] | each opponent
//!             | you | that player | that creature's controller | itself | it | ~ ...
//! ```
//!
//! Removal effects: "destroy/exile A and B", "exile X until ~ leaves the battlefield"
//! (CR 610.3), graveyard exile, non-targeted bounce ("return a land you control to its
//! owner's hand"), edicts ("sacrifices a creature of their choice"), damage prevention
//! (CR 615), "if that creature would die this turn, exile it instead" (CR 614), delayed
//! removal ("destroy that creature at end of combat", CR 603.7), and follow-up sentences
//! ("It can't be regenerated.", CR 701.19c).

use super::{EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{bind_target_player, object_ref, player_ref, Builder};
use crate::oracle::phrases::*;
use crate::oracle::statics::parse_value_phrase;

/// Variable holding objects for delayed effects created by these patterns ("destroy that
/// creature at end of combat"): delayed triggers keep the creating ability's variables
/// but not its targets or trigger event.
const DELAYED: Var = vars::USER + 40;

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

/// Strips `p` from the start of `s` if it's followed by a word boundary.
fn word<'a>(s: &'a str, p: &str) -> Option<&'a str> {
    let r = s.strip_prefix(p)?;
    match r.chars().next() {
        None => Some(r),
        Some(c) if c == ' ' || c == ',' || c == '.' => Some(r),
        _ => None,
    }
}

fn damage(source: &Sel, amount: Value, to: Sel) -> Effect {
    Effect::DealDamage {
        source: source.clone(),
        amount,
        to,
    }
}

/// Whether "it"/"that creature" currently names an object other than the source (a
/// target object, the trigger object, a variable), i.e. a tracked antecedent.
fn it_is_object(b: &Builder) -> bool {
    match &b.it {
        Sel::This => false,
        Sel::Target(n) => !matches!(
            b.targets.get(*n as usize).map(|t| &t.what),
            Some(TargetKind::Player(_)) | None
        ),
        _ => true,
    }
}

/// The relation a filter uses for "that player controls"/"they control".
fn player_rel_of(p: &PlayerRef) -> Option<PlayerRel> {
    Some(match p {
        PlayerRef::You => PlayerRel::You,
        PlayerRef::Target(n) => PlayerRel::Target(*n),
        PlayerRef::TriggerPlayer => PlayerRel::TriggerPlayer,
        PlayerRef::DefendingPlayer => PlayerRel::Defending,
        PlayerRef::ActivePlayer => PlayerRel::Active,
        PlayerRef::EachOpponent => PlayerRel::Opponent,
        _ => return None,
    })
}

/// "N", "up to N", "one or two", "one, two, or three", "any number of" before a target
/// noun. Returns (min, max, any-number?, rest).
fn target_count(s: &str) -> Option<(u32, Value, bool, &str)> {
    let s = s.trim_start();
    if let Some(r) = s.strip_prefix("any number of ") {
        return Some((0, Value::c(99), true, r));
    }
    if let Some(r) = s.strip_prefix("one or two ") {
        return Some((1, Value::c(2), false, r));
    }
    if let Some(r) = s.strip_prefix("one, two, or three ") {
        return Some((1, Value::c(3), false, r));
    }
    if let Some(r) = s.strip_prefix("up to ") {
        let (n, r) = parse_number(r)?;
        return Some((0, n, false, r));
    }
    let (n, r) = parse_number(s)?;
    let min = n.as_const()?.max(0) as u32;
    Some((min, n, false, r))
}

/// "two target creatures", "up to three targets", "one, two, or three target attacking
/// or blocking creatures", "any number of targets".
fn counted_targets(s: &str) -> Option<(TargetSpec, bool, &str)> {
    let (min, max, any_number, r) = target_count(s)?;
    let r = r.trim_start();
    let (what, rest) = if let Some(r2) = word(r, "targets") {
        (TargetKind::AnyTarget, r2)
    } else {
        let r2 = r.strip_prefix("target ")?;
        if let Some(r3) = word(r2, "players").or_else(|| word(r2, "player")) {
            (TargetKind::Player(PlayerFilter::Any), r3)
        } else if let Some(r3) = word(r2, "opponents").or_else(|| word(r2, "opponent")) {
            (TargetKind::Player(PlayerFilter::Opponent), r3)
        } else {
            let (f, _, r3) = parse_object_phrase(r2)?;
            (TargetKind::Object(f), r3)
        }
    };
    let spec = TargetSpec {
        what,
        min,
        max,
        distinct_from: vec![],
        divide: None,
        chosen_by_opponent: false,
        text: String::new(),
        condition: None,
    };
    Some((spec, any_number, rest))
}

/// Value phrases used in damage amounts, on top of the shared value grammar.
fn value_phrase(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let s = s.trim_start();
    if let Some(r) = s.strip_prefix("twice ") {
        let (v, rest) = value_phrase(r, b)?;
        return Some((Value::Mul(Box::new(Value::c(2)), Box::new(v)), rest));
    }
    let it = b.it.clone();
    // "that card"/"that creature" never names the source itself or a player; if the
    // referent is one of those, the antecedent wasn't tracked (e.g. a discarded card).
    if s.starts_with("that ") && !it_is_object(b) {
        return None;
    }
    let pairs: [(&str, Value); 12] = [
        (
            "that creature's power",
            Value::PowerOf(Box::new(it.clone())),
        ),
        (
            "that creature's toughness",
            Value::ToughnessOf(Box::new(it.clone())),
        ),
        (
            "that card's mana value",
            Value::ManaValueOf(Box::new(it.clone())),
        ),
        (
            "that spell's mana value",
            Value::ManaValueOf(Box::new(it.clone())),
        ),
        ("its mana value", Value::ManaValueOf(Box::new(it.clone()))),
        ("~'s power", Value::PowerOf(Box::new(Sel::This))),
        ("~'s toughness", Value::ToughnessOf(Box::new(Sel::This))),
        ("its power", Value::PowerOf(Box::new(it.clone()))),
        ("its toughness", Value::ToughnessOf(Box::new(it.clone()))),
        (
            "the number of cards in your hand",
            Value::HandSize(PlayerRef::You),
        ),
        (
            "the number of cards in that player's hand",
            Value::HandSize(b.it_player.clone()),
        ),
        (
            "the number of cards in their hand",
            Value::HandSize(b.it_player.clone()),
        ),
    ];
    for (p, v) in pairs {
        if let Some(r) = word(s, p) {
            return Some((v, r.to_string()));
        }
    }
    if let Some(r) = s.strip_prefix("the greatest power among ") {
        let (f, _, rest) = parse_object_phrase(r)?;
        return Some((Value::GreatestPower(f), rest.to_string()));
    }
    let (v, rest) = parse_value_phrase(s, b)?;
    // "the number of X" must be fully understood (no leftover qualifier).
    Some((v, rest))
}

/// An amount before the word "damage": "3", "x", "that much", "twice x".
fn amount_phrase<'a>(s: &'a str, where_x: &Option<Value>) -> Option<(Value, &'a str)> {
    let s = s.trim_start();
    if let Some(r) = s.strip_prefix("that much ") {
        return Some((Value::EventAmount, r));
    }
    if let Some(r) = s.strip_prefix("twice x ") {
        let x = where_x.clone().unwrap_or(Value::X);
        return Some((Value::Mul(Box::new(Value::c(2)), Box::new(x)), r));
    }
    let (n, r) = parse_number(s)?;
    // "a" isn't an amount of damage ("deals a damage" doesn't occur).
    if s.starts_with("a ") || s.starts_with("an ") {
        return None;
    }
    let n = match (n, where_x) {
        (Value::X, Some(v)) => v.clone(),
        (n, _) => n,
    };
    Some((n, r))
}

// ---------------------------------------------------------------------------
// Damage
// ---------------------------------------------------------------------------

/// Player recipients that aren't targets.
fn player_recipient<'a>(s: &'a str, b: &Builder) -> Option<(Sel, &'a str)> {
    let it = || Box::new(b.it.clone());
    let pairs: Vec<(&str, PlayerRef)> = vec![
        ("each opponent", PlayerRef::EachOpponent),
        ("each other opponent", PlayerRef::EachOpponent),
        ("each player", PlayerRef::EachPlayer),
        ("each other player", PlayerRef::EachOtherPlayer),
        ("you", PlayerRef::You),
        ("that player", b.it_player.clone()),
        ("defending player", PlayerRef::DefendingPlayer),
        ("its controller", PlayerRef::ControllerOf(it())),
        ("their controller", PlayerRef::ControllerOf(it())),
        ("that creature's controller", PlayerRef::ControllerOf(it())),
        ("that permanent's controller", PlayerRef::ControllerOf(it())),
        ("that land's controller", PlayerRef::ControllerOf(it())),
        ("that artifact's controller", PlayerRef::ControllerOf(it())),
        (
            "that enchantment's controller",
            PlayerRef::ControllerOf(it()),
        ),
        (
            "that planeswalker's controller",
            PlayerRef::ControllerOf(it()),
        ),
        ("that spell's controller", PlayerRef::ControllerOf(it())),
        ("its owner", PlayerRef::OwnerOf(it())),
    ];
    for (p, r) in pairs {
        if let Some(rest) = word(s, p) {
            // "each other opponent" in a two-player game is just "each opponent"; in
            // multiplayer it excludes a player named earlier — not supported.
            if p == "each other opponent" {
                return None;
            }
            // Pronouns whose antecedent wasn't tracked still point at the defaults.
            if p == "that player" && matches!(r, PlayerRef::You) {
                return None;
            }
            if p.starts_with("that ") && p != "that player" && !it_is_object(b) {
                return None;
            }
            return Some((Sel::Players(r), rest));
        }
    }
    None
}

/// The players a preceding recipient names, for "each creature they control" / "each
/// creature that player controls".
fn players_of(prev: Option<&Sel>, b: &Builder) -> Option<PlayerRel> {
    match prev? {
        Sel::Players(PlayerRef::EachOpponent) => Some(PlayerRel::Opponent),
        Sel::Players(PlayerRef::EachPlayer) => Some(PlayerRel::Any),
        Sel::Players(p) => player_rel_of(p),
        Sel::Target(n) => match b.targets.get(*n as usize)?.what {
            TargetKind::Player(_) => Some(PlayerRel::Target(*n)),
            _ => None,
        },
        _ => None,
    }
}

/// One recipient item. `prev` is the preceding item in an "and" list.
fn recipient_item(
    s: &str,
    src: &Sel,
    prev: Option<&Sel>,
    b: &mut Builder,
) -> Option<(Sel, String)> {
    let s = s.trim_start();
    if let Some(r) = word(s, "itself") {
        return Some((src.clone(), r.to_string()));
    }
    if let Some((sel, r)) = player_recipient(s, b) {
        return Some((sel, r.to_string()));
    }
    if let Some(r) = word(s, "any other target") {
        let mut spec = TargetSpec::any_target();
        spec.distinct_from = (0..b.targets.len() as u8).collect();
        let slot = b.add_target(spec, "any other target");
        return Some((Sel::Target(slot), r.to_string()));
    }
    if let Some(r) = s.strip_prefix("each ") {
        let (f, _, rest) = parse_object_phrase(r)?;
        let (f, rest) = bind_target_player(f, rest, b);
        let t = rest.trim_start();
        // "each opponent and each creature they control", "target player and each
        // creature that player controls".
        let (f, rest) =
            if let Some(r2) = word(t, "they control").or_else(|| word(t, "that player controls")) {
                let rel = players_of(prev, b)?;
                (
                    Filter::and(vec![f, Filter::ControlledBy(rel)]),
                    r2.to_string(),
                )
            } else {
                (f, rest)
            };
        return Some((Sel::All(f), rest));
    }
    let (sel, rest) = object_ref(s, b)?;
    // A targeted player becomes "that player".
    if let Sel::Target(n) = sel {
        match b.targets[n as usize].what {
            TargetKind::Player(_) => b.it_player = PlayerRef::Target(n),
            TargetKind::ObjectOrPlayer(..) => {
                b.it_player = PlayerRef::ControllerOf(Box::new(Sel::Target(n)))
            }
            _ => {}
        }
    }
    Some((sel, rest))
}

/// Whether `s` begins another recipient (after "and"), as opposed to another damage part.
fn starts_recipient(s: &str) -> bool {
    let s = s.trim_start();
    if s.starts_with("that much") {
        return false;
    }
    [
        "each ",
        "you",
        "target ",
        "that ",
        "its ",
        "~",
        "defending player",
        "any other target",
    ]
    .iter()
    .any(|p| s.starts_with(p))
}

/// Damage recipients: "each of up to two target creatures", or a list of items joined
/// by "and" ("each creature and each player").
fn recipients(s: &str, src: &Sel, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim_start();
    if let Some(r) = s.strip_prefix("each of ") {
        let (spec, _, rest) = counted_targets(r)?;
        let text = format!("each of {}", &r[..r.len() - rest.len()]);
        let slot = b.add_target(spec, text.trim());
        return Some((Sel::Target(slot), rest.to_string()));
    }
    let mut items: Vec<Sel> = Vec::new();
    let mut rest = s.to_string();
    loop {
        let (sel, r) = recipient_item(&rest, src, items.last(), b)?;
        items.push(sel);
        rest = r;
        let t = rest.trim_start();
        if let Some(r2) = t.strip_prefix("and ") {
            if starts_recipient(r2) {
                rest = r2.to_string();
                continue;
            }
        }
        break;
    }
    let sel = if items.len() == 1 {
        items.pop().unwrap()
    } else {
        Sel::Union(items)
    };
    Some((sel, rest))
}

/// One "AMOUNT damage to RECIPIENTS" part. Returns the effect and the unparsed tail.
fn damage_part(
    src: &Sel,
    s: &str,
    where_x: &Option<Value>,
    b: &mut Builder,
) -> Option<(Effect, String)> {
    let s = s.trim_start();
    if let Some(r) = s.strip_prefix("damage equal to ") {
        let (v, r2) = value_phrase(r, b)?;
        let r3 = r2.trim_start().strip_prefix("to ")?.to_string();
        let (to, tail) = recipients(&r3, src, b)?;
        return Some((damage(src, v, to), tail));
    }
    if let Some(r) = s.strip_prefix("damage to ") {
        let (to, tail) = recipients(r, src, b)?;
        let r2 = tail.trim_start().strip_prefix("equal to ")?.to_string();
        let (v, tail2) = value_phrase(&r2, b)?;
        return Some((damage(src, v, to), tail2));
    }
    let (amount, r) = amount_phrase(s, where_x)?;
    let r = strip(r, "damage")?;
    if let Some(r2) = strip(r, "divided as you choose among ") {
        // CR 601.2d: the division is chosen as the spell is cast; each target gets at
        // least 1.
        let (mut spec, any_number, tail) = counted_targets(r2)?;
        spec.min = spec.min.max(1);
        if any_number {
            spec.max = amount.clone();
        }
        spec.divide = Some(amount);
        let slot = b.add_target(spec, "targets (divided)");
        return Some((
            Effect::DealDividedDamage {
                source: src.clone(),
                slot,
            },
            tail.to_string(),
        ));
    }
    let r = strip(r, "to")?;
    let (to, tail) = recipients(r, src, b)?;
    Some((damage(src, amount, to), tail))
}

/// The source of a damage clause: "~ deals", "it deals", "target creature deals".
fn damage_source(l: &str, verb: &str, b: &mut Builder) -> Option<(Sel, String)> {
    if let Some(r) = l.strip_prefix(&format!("~{verb}")) {
        return Some((Sel::This, r.to_string()));
    }
    let (sel, rest) = object_ref(l, b)?;
    let r = rest.trim_start().strip_prefix(verb.trim_start())?;
    Some((sel, r.to_string()))
}

fn damage_clause(l: &str, b: &mut Builder, verb: &str) -> Option<Effect> {
    // ", where x is VALUE" defines X for the whole clause.
    let (l, where_text) = match l.split_once(", where x is ") {
        Some((head, v)) => (head, Some(v)),
        None => (l, None),
    };
    // The source comes first (its target, if any, is first in the text).
    let (src, rest) = damage_source(l, verb, b)?;
    // "X ... where X is its power": "its" is the source.
    let where_x = match where_text {
        Some(v) => {
            let saved = b.it.clone();
            b.it = src.clone();
            let (val, tail) = value_phrase(v, b)?;
            b.it = saved;
            if !end(&tail).is_empty() {
                return None;
            }
            Some(val)
        }
        None => None,
    };
    let mut effects = Vec::new();
    let mut r = rest;
    loop {
        let (e, tail) = damage_part(&src, &r, &where_x, b)?;
        effects.push(e);
        let t = end(&tail);
        if t.is_empty() {
            break;
        }
        r = t.strip_prefix("and ")?.to_string();
    }
    Some(Effect::seq(effects))
}

/// Damage clauses the core damage pattern doesn't handle.
fn p_damage(l: &str, b: &mut Builder) -> Option<Effect> {
    // "you may have ~ deal 3 damage to ..." (the controller chooses).
    if let Some(r) = l.strip_prefix("you may have ") {
        let e = damage_clause(r, b, " deal ")?;
        return Some(Effect::May {
            who: PlayerRef::You,
            effect: Box::new(e),
        });
    }
    // "each creature deals damage to itself equal to its power".
    if let Some(r) = l.strip_prefix("each ") {
        let (f, _, rest) = parse_object_phrase(r)?;
        let t = rest.trim();
        if t == "deals damage to itself equal to its power" {
            const EACH: Var = vars::USER + 41;
            let me = Sel::Var(EACH);
            return Some(Effect::ForEach {
                sel: Sel::All(f),
                var: EACH,
                effect: Box::new(damage(
                    &me,
                    Value::PowerOf(Box::new(me.clone())),
                    me.clone(),
                )),
            });
        }
        return None;
    }
    damage_clause(l, b, " deals ")
}

inventory::submit! { EffectPattern { name: "damage_removal: damage", priority: 50, parse: p_damage } }

// ---------------------------------------------------------------------------
// Prevention (CR 615)
// ---------------------------------------------------------------------------

/// Recipients of prevention: targets, "~", "you", pronouns.
fn prevent_recipient(s: &str, b: &mut Builder) -> Option<(Sel, String)> {
    let s = s.trim_start();
    if let Some(r) = word(s, "you") {
        return Some((Sel::Players(PlayerRef::You), r.to_string()));
    }
    for p in ["him", "her"] {
        if let Some(r) = word(s, p) {
            return Some((Sel::This, r.to_string()));
        }
    }
    object_ref(s, b)
}

/// "prevent the next N damage that would be dealt to X this turn", "prevent all [combat]
/// damage that would be dealt to X this turn".
fn p_prevent(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("prevent ")?;
    let (amount, r) = if let Some(r) = r.strip_prefix("the next ") {
        let (n, r) = parse_number(r)?;
        (Some(n), r)
    } else if let Some(r) = r.strip_prefix("all ") {
        (None, r)
    } else {
        return None;
    };
    let r = r.trim_start();
    let (combat_only, r) = match r.strip_prefix("combat ") {
        Some(x) => (true, x),
        None => (false, r),
    };
    let r = r.strip_prefix("damage that would be dealt ")?;
    // "... to X this turn" or "... this turn to X"
    let (to, tail) = if let Some(x) = r.strip_prefix("this turn to ") {
        let (to, tail) = prevent_recipient(x, b)?;
        (to, tail)
    } else {
        let x = r.strip_prefix("to ")?;
        let (to, tail) = prevent_recipient(x, b)?;
        let tail = tail.trim().strip_prefix("this turn")?.to_string();
        (to, tail)
    };
    if !end(&tail).is_empty() {
        return None;
    }
    // A "next N damage" shield on a combat-only basis isn't printed; keep it simple.
    if amount.is_some() && combat_only {
        return None;
    }
    Some(Effect::PreventDamage {
        to,
        amount,
        duration: Duration::EndOfTurn,
        combat_only,
    })
}

inventory::submit! { EffectPattern { name: "damage_removal: prevent", priority: 50, parse: p_prevent } }

/// "damage can't be prevented this turn".
fn p_cant_prevent(l: &str, _b: &mut Builder) -> Option<Effect> {
    match l {
        "damage can't be prevented this turn" | "all damage can't be prevented this turn" => {
            Some(Effect::AddRestriction {
                restriction: Restriction::DamageCantBePrevented,
                duration: Duration::EndOfTurn,
            })
        }
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "damage_removal: can't be prevented", priority: 50, parse: p_cant_prevent } }

// ---------------------------------------------------------------------------
// "If that creature would die this turn, exile it instead" (CR 614.1a, 614.6)
// ---------------------------------------------------------------------------

fn p_exile_instead(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("if ")?;
    let (who, rest) = r.split_once(" would die this turn, ")?;
    if rest != "exile it instead" {
        return None;
    }
    let filter = match who {
        "that creature" | "that creature or planeswalker" | "that permanent" | "it" => {
            Filter::In(Box::new(b.it.clone()))
        }
        "a creature dealt damage this way" => Filter::and(vec![
            Filter::In(Box::new(Sel::Var(vars::DAMAGED))),
            Filter::creature(),
        ]),
        "a permanent dealt damage this way"
        | "a creature or planeswalker dealt damage this way" => {
            Filter::In(Box::new(Sel::Var(vars::DAMAGED)))
        }
        _ => return None,
    };
    // "Dies" means put into a graveyard from the battlefield (CR 700.4).
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::ZoneChange {
                filter,
                from: Some(ZoneKind::Battlefield),
                to: Some(ZoneKind::Graveyard),
            },
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfTurn,
        uses: None,
    })
}

inventory::submit! { EffectPattern { name: "damage_removal: exile instead", priority: 50, parse: p_exile_instead } }

// ---------------------------------------------------------------------------
// Destroy / exile / bounce
// ---------------------------------------------------------------------------

/// Two object references joined by "and": "target creature and target land".
fn pair_refs(s: &str, b: &mut Builder) -> Option<Sel> {
    let (a, rest) = object_ref(s, b)?;
    let r = rest.trim_start().strip_prefix("and ")?;
    let (c, tail) = object_ref(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Sel::Union(vec![a, c]))
}

/// "destroy target creature and target land", "exile target creature and target land".
fn p_pair(l: &str, b: &mut Builder) -> Option<Effect> {
    if let Some(r) = l.strip_prefix("destroy ") {
        let what = pair_refs(r, b)?;
        return Some(Effect::Destroy {
            what,
            no_regen: false,
        });
    }
    if let Some(r) = l.strip_prefix("exile ") {
        let what = pair_refs(r, b)?;
        return Some(Effect::Exile {
            what,
            face_down: false,
            link: false,
        });
    }
    None
}

inventory::submit! { EffectPattern { name: "damage_removal: destroy/exile pair", priority: 50, parse: p_pair } }

/// "exile target nonland permanent an opponent controls until ~ leaves the battlefield"
/// (CR 610.3).
fn p_exile_until(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("exile ")?;
    let r = r.strip_suffix(" until ~ leaves the battlefield")?;
    let (what, tail) = object_ref(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    // Only permanents: the return puts them back onto the battlefield.
    let on_battlefield = match &what {
        Sel::Target(slot) => match &b.targets[*slot as usize].what {
            TargetKind::Object(f) => f.zone().is_none_or(|z| z == ZoneKind::Battlefield),
            _ => false,
        },
        Sel::All(f) => f.zone().is_none_or(|z| z == ZoneKind::Battlefield),
        _ => false,
    };
    if !on_battlefield {
        return None;
    }
    Some(Effect::ExileUntil {
        what,
        until: UntilEvent::SourceLeavesBattlefield,
    })
}

inventory::submit! { EffectPattern { name: "damage_removal: exile until", priority: 50, parse: p_exile_until } }

/// Graveyard exile: "exile target player's graveyard", "exile all graveyards", "exile
/// each opponent's graveyard", "exile your graveyard", "exile all cards from target
/// player's graveyard".
fn p_exile_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("exile ")?;
    let r = r.strip_prefix("all cards from ").unwrap_or(r);
    let gy = |rel: Option<PlayerRel>| {
        let mut v = vec![Filter::InZone(ZoneKind::Graveyard)];
        if let Some(rel) = rel {
            v.push(Filter::OwnedBy(rel));
        }
        Filter::and(v)
    };
    let filter = match r {
        "all graveyards" | "each graveyard" => gy(None),
        "your graveyard" => gy(Some(PlayerRel::You)),
        "each opponent's graveyard" | "all opponents' graveyards" => gy(Some(PlayerRel::Opponent)),
        "that player's graveyard" => gy(Some(player_rel_of(&b.it_player)?)),
        "target player's graveyard" | "target opponent's graveyard" => {
            let (pf, text) = if r.starts_with("target player") {
                (PlayerFilter::Any, "target player")
            } else {
                (PlayerFilter::Opponent, "target opponent")
            };
            let slot = b.add_target(TargetSpec::player(pf, text), text);
            b.it_player = PlayerRef::Target(slot);
            gy(Some(PlayerRel::Target(slot)))
        }
        _ => return None,
    };
    Some(Effect::Exile {
        what: Sel::All(filter),
        face_down: false,
        link: false,
    })
}

inventory::submit! { EffectPattern { name: "damage_removal: exile graveyard", priority: 50, parse: p_exile_graveyard } }

/// Non-targeted bounce: "return a land you control to its owner's hand", "return two
/// creatures you control to their owner's hand".
fn p_bounce_choose(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = l.strip_prefix("return ")?;
    let (n, r) = parse_number(r)?;
    let n = n.as_const()?;
    let (f, _, tail) = parse_object_phrase(r)?;
    let tail = tail.trim();
    // Only permanents the controller controls are chosen this way.
    let yours = match &f {
        Filter::And(v) => v
            .iter()
            .any(|x| matches!(x, Filter::ControlledBy(PlayerRel::You))),
        _ => false,
    };
    if !yours {
        return None;
    }
    if !matches!(
        tail,
        "to its owner's hand" | "to their owner's hand" | "to their owners' hands"
    ) {
        return None;
    }
    Some(Effect::Move {
        what: Sel::Choose {
            chooser: PlayerRef::You,
            filter: f,
            count: Value::c(n),
            up_to: false,
            store: None,
        },
        to: Destination::zone(ZoneKind::Hand),
    })
}

inventory::submit! { EffectPattern { name: "damage_removal: bounce (choose)", priority: 50, parse: p_bounce_choose } }

/// Edicts: "each opponent sacrifices a creature of their choice", "target player
/// sacrifices an artifact or creature of their choice".
fn p_edict(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = l.strip_suffix(" of their choice")?;
    let (who, rest) = player_ref(r, b)?;
    let rest = rest.trim().strip_prefix("sacrifices ")?;
    let (n, r2) = parse_number(rest)?;
    let (f, _, tail) = parse_object_phrase(r2)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(Effect::Sacrifice {
        who,
        filter: f,
        count: n,
    })
}

inventory::submit! { EffectPattern { name: "damage_removal: edict", priority: 50, parse: p_edict } }

/// Parses "destroy/exile/return X [to its owner's hand] at end of combat|at the beginning
/// of the next end step". Returns (verb, object text, tail, step).
fn delayed_parts(l: &str) -> Option<(&'static str, &str, TriggerStep)> {
    let (body, step) = if let Some(x) = l.strip_suffix(" at end of combat") {
        (x, TriggerStep::EndOfCombat)
    } else if let Some(x) = l.strip_suffix(" at the beginning of the next end step") {
        (x, TriggerStep::End)
    } else {
        return None;
    };
    let (verb, r) = if let Some(r) = body.strip_prefix("destroy ") {
        ("destroy", r)
    } else if let Some(r) = body.strip_prefix("exile ") {
        ("exile", r)
    } else if let Some(r) = body.strip_prefix("return ") {
        ("return", r)
    } else {
        return None;
    };
    Some((verb, r, step))
}

/// Builds "store the object now; at the step, destroy/exile/bounce it" (CR 603.7).
/// Delayed triggers keep the creating ability's variables, not its targets or event.
fn delayed_removal(verb: &str, what: Sel, tail: &str, step: TriggerStep) -> Option<Effect> {
    let delayed = Sel::Var(DELAYED);
    let effect = match (verb, tail) {
        ("destroy", "") => Effect::Destroy {
            what: delayed,
            no_regen: false,
        },
        ("exile", "") => Effect::Exile {
            what: delayed,
            face_down: false,
            link: false,
        },
        ("return", "to its owner's hand" | "to their owners' hands" | "to their owner's hand") => {
            Effect::Move {
                what: delayed,
                to: Destination::zone(ZoneKind::Hand),
            }
        }
        _ => return None,
    };
    Some(Effect::seq(vec![
        Effect::Store {
            var: DELAYED,
            sel: what,
        },
        Effect::AtNext {
            step,
            effect: Box::new(effect),
        },
    ]))
}

/// Delayed removal of an object named earlier: "destroy that creature at end of combat",
/// "return that creature to its owner's hand at end of combat", "destroy it at the
/// beginning of the next end step" (CR 603.7).
fn p_delayed_removal(l: &str, b: &mut Builder) -> Option<Effect> {
    let (verb, r, step) = delayed_parts(l)?;
    let n_targets = b.targets.len();
    let (what, tail) = object_ref(r, b)?;
    // Only an object named earlier (no new targets for a delayed effect), and only one
    // that can't have changed zones since: the source or trigger object in the first
    // sentence of a triggered ability, or a targeted permanent. (A pronoun after "create
    // a token" or "return ... to the battlefield" is handled by `f_delayed_after`.)
    if b.targets.len() != n_targets {
        return None;
    }
    let safe = match &what {
        Sel::This | Sel::TriggerObject | Sel::TriggerLki => b.in_trigger && b.sentences == 0,
        Sel::AttachedTo => true,
        Sel::Target(n) => matches!(
            &b.targets[*n as usize].what,
            TargetKind::Object(f) if f.zone().is_none_or(|z| z == ZoneKind::Battlefield)
        ),
        _ => false,
    };
    if !safe {
        return None;
    }
    delayed_removal(verb, what, tail.trim(), step)
}

inventory::submit! { EffectPattern { name: "damage_removal: delayed removal", priority: 50, parse: p_delayed_removal } }

/// "Create a token ... Exile it at the beginning of the next end step", "Return target
/// creature card ... to the battlefield. Exile it at ...": the pronoun names the new
/// objects the previous effect created or put onto the battlefield.
fn f_delayed_after(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let var = match last_effect(prev) {
        Effect::CreateToken { .. } | Effect::CreateTokenCopy { .. } => vars::CREATED,
        Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield => vars::IT,
        _ => return false,
    };
    let Some((verb, r, step)) = delayed_parts(l) else {
        return false;
    };
    let mut tail = None;
    for p in [
        "it",
        "them",
        "that token",
        "those tokens",
        "the token",
        "the tokens",
        "that creature",
        "those creatures",
    ] {
        if let Some(x) = word(r, p) {
            tail = Some(x);
            break;
        }
    }
    let Some(tail) = tail else {
        return false;
    };
    let Some(e) = delayed_removal(verb, Sel::Var(var), tail.trim(), step) else {
        return false;
    };
    // Later pronouns name the same new objects.
    b.it = Sel::Var(var);
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "damage_removal: delayed after create/return", priority: 50, apply: f_delayed_after } }

// ---------------------------------------------------------------------------
// Follow-up sentences
// ---------------------------------------------------------------------------

/// Marks the last destroy effect (in sequence order) as "can't be regenerated".
fn set_no_regen(e: &mut Effect) -> bool {
    match e {
        Effect::Destroy { no_regen, .. } => {
            *no_regen = true;
            true
        }
        Effect::Seq(v) => v.iter_mut().rev().any(set_no_regen),
        Effect::May { effect, .. } => set_no_regen(effect),
        Effect::If { then, .. } => set_no_regen(then),
        Effect::ForEach { effect, .. } | Effect::ForEachPlayer { effect, .. } => {
            set_no_regen(effect)
        }
        _ => false,
    }
}

/// "It can't be regenerated." / "They can't be regenerated." / "A creature destroyed this
/// way can't be regenerated." (CR 701.19c).
fn f_no_regen(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let ok = matches!(
        l,
        "it can't be regenerated"
            | "they can't be regenerated"
            | "that creature can't be regenerated"
            | "a creature destroyed this way can't be regenerated"
            | "creatures destroyed this way can't be regenerated"
            | "permanents destroyed this way can't be regenerated"
            | "artifacts destroyed this way can't be regenerated"
            | "lands destroyed this way can't be regenerated"
    );
    ok && set_no_regen(prev)
}

inventory::submit! { FollowupPattern { name: "damage_removal: can't be regenerated", priority: 50, apply: f_no_regen } }

/// The last effect of a sequence (in execution order).
fn last_effect(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_effect),
        other => other,
    }
}

/// "to the battlefield [tapped] [transformed] under its owner's control" etc. `owned`
/// is the selection whose owners control the returned objects.
fn battlefield_destination(s: &str, owned: Sel) -> Option<Destination> {
    let mut r = s.strip_prefix("to the battlefield")?.trim_start();
    let mut d = Destination::battlefield();
    let mut owner = false;
    loop {
        if let Some(x) = r.strip_prefix("tapped") {
            d.tapped = true;
            r = x.trim_start();
        } else if let Some(x) = r.strip_prefix("transformed") {
            d.transformed = true;
            r = x.trim_start();
        } else if let Some(x) = r
            .strip_prefix("under its owner's control")
            .or_else(|| r.strip_prefix("under their owner's control"))
            .or_else(|| r.strip_prefix("under their owners' control"))
        {
            owner = true;
            r = x.trim_start();
        } else if let Some(x) = r.strip_prefix("under your control") {
            r = x.trim_start();
        } else {
            break;
        }
    }
    if !r.is_empty() {
        return None;
    }
    d.controller = Some(if owner {
        // Each card returns under its own owner's control (see `move_to_destination`).
        PlayerRef::OwnerOf(Box::new(owned))
    } else {
        PlayerRef::You
    });
    Some(d)
}

/// "[Exile X], then return it to the battlefield under its owner's control" and "Return
/// that card to the battlefield under its owner's control at the beginning of the next
/// end step" after an exile: the returned object is the card in exile — a new object
/// (CR 400.7), not the original target.
fn f_return_exiled(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Effect::Exile { what, .. } = last_effect(prev) else {
        return false;
    };
    let what = what.clone();
    let Some(r) = l.strip_prefix("return ") else {
        return false;
    };
    let mut pronoun_it = false;
    let mut rest = None;
    for p in [
        "it ",
        "that card ",
        "them ",
        "those cards ",
        "the exiled card ",
        "the exiled cards ",
        "that creature ",
    ] {
        if let Some(x) = r.strip_prefix(p) {
            pronoun_it = p == "it ";
            rest = Some(x);
            break;
        }
    }
    let Some(rest) = rest else {
        return false;
    };
    // The pronoun must name what was just exiled.
    let same =
        format!("{:?}", b.it) == format!("{what:?}") || (pronoun_it && matches!(what, Sel::This));
    if !same {
        return false;
    }
    let (dest_s, step) = match rest.strip_suffix(" at the beginning of the next end step") {
        Some(x) => (x, Some(TriggerStep::End)),
        None => (rest, None),
    };
    let effects = match step {
        None => {
            let Some(dest) = battlefield_destination(dest_s, Sel::Var(vars::IT)) else {
                return false;
            };
            // "It gains first strike until end of turn": the returned permanent.
            b.it = Sel::Var(vars::IT);
            vec![Effect::Move {
                what: Sel::Var(vars::IT),
                to: dest,
            }]
        }
        Some(step) => {
            // The delayed trigger keeps this ability's variables (CR 603.7c).
            let Some(dest) = battlefield_destination(dest_s, Sel::Var(DELAYED)) else {
                return false;
            };
            vec![
                Effect::Store {
                    var: DELAYED,
                    sel: Sel::Var(vars::IT),
                },
                Effect::AtNext {
                    step,
                    effect: Box::new(Effect::Move {
                        what: Sel::Var(DELAYED),
                        to: dest,
                    }),
                },
            ]
        }
    };
    let old = std::mem::take(prev);
    let mut v = vec![old];
    v.extend(effects);
    *prev = Effect::seq(v);
    true
}

inventory::submit! { FollowupPattern { name: "damage_removal: return exiled", priority: 40, apply: f_return_exiled } }
