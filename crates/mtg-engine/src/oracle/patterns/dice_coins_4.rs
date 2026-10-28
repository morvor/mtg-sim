//! More grammar for dice (CR 706) and coins (CR 705), round 3:
//!
//! - results-table rows that use loyalty symbols as instructions (Comet, Stellar Pup:
//!   "1 or 2 — [+2], then create ...", "4 or 5 — ..., then [−2]"): [+N] puts N loyalty
//!   counters on the source, [−N] removes N (its rulings);
//! - "~ deals N damage to a creature or player": not targeted, the creature or player is
//!   chosen as the instruction is performed (CR 608.2c; Comet's rulings).
//!
//! The behaviors are in `kw/dice_coins_4.rs`.

use super::{ConditionPattern, EffectPattern};
use crate::ability::*;
use crate::kw::dice_coins_4::{choose_creature_or_player, CHOSEN_CREATURE_OR_PLAYER};
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::*;

/// "[+2]" → "put two loyalty counters on ~", "[−1]" / "[-1]" → "remove a loyalty counter
/// from ~": a loyalty symbol used as an instruction.
fn loyalty_symbol(s: &str) -> Option<(String, &str)> {
    let r = s.strip_prefix('[')?;
    let (sym, rest) = r.split_once(']')?;
    let (add, n) = if let Some(n) = sym.strip_prefix('+') {
        (true, n)
    } else if let Some(n) = sym.strip_prefix('-').or_else(|| sym.strip_prefix('−')) {
        (false, n)
    } else {
        return None;
    };
    let n: u32 = n.parse().ok().filter(|n| *n > 0)?;
    let count = if n == 1 {
        "a loyalty counter".to_string()
    } else {
        format!("{n} loyalty counters")
    };
    Some((
        if add {
            format!("put {count} on ~")
        } else {
            format!("remove {count} from ~")
        },
        rest,
    ))
}

/// A results-table row's text with loyalty symbols written out as instructions: "[+2],
/// then create ..." / "[+1], and you may ..." / "..., then [−2]" (Comet, Stellar Pup).
/// "him" / "her" in such a row is the source. `None` if the row has no loyalty symbol.
pub fn row_with_loyalty_symbols(t: &str) -> Option<String> {
    let t = t.trim();
    if let Some((first, rest)) = loyalty_symbol(t) {
        let rest = rest.trim_start();
        if rest.is_empty() || rest == "." {
            return Some(format!("{first}."));
        }
        let joined = rest
            .strip_prefix(", then ")
            .map(|r| format!("{first}, then {r}"))
            .or_else(|| {
                rest.strip_prefix(", and ")
                    .map(|r| format!("{first}, and {r}"))
            })?;
        return Some(joined);
    }
    let body = t.strip_suffix('.').unwrap_or(t);
    let (head, sym) = body.rsplit_once(", then ")?;
    let (last, tail) = loyalty_symbol(sym)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let head = head
        .replace(" on him ", " on ~ ")
        .replace(" on her ", " on ~ ");
    Some(format!("{head}, then {last}."))
}

/// "~ deals 2 damage to a creature or player" / "~ deals damage equal to [value] to a
/// creature or player": the controller chooses a creature or player as the instruction is
/// performed (not a target, CR 115.1), and it's dealt that much damage.
fn damage_to_a_creature_or_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("~ deals ")?;
    let amount = if let Some(v) = r.strip_prefix("damage equal to ") {
        let (v, rest) = crate::oracle::statics::parse_value_phrase(v, b)?;
        if rest.trim() != "to a creature or player" {
            return None;
        }
        v
    } else {
        let (n, rest) = parse_number(r)?;
        if rest.trim() != "damage to a creature or player" {
            return None;
        }
        n
    };
    Some(Effect::seq(vec![
        choose_creature_or_player(),
        Effect::DealDamage {
            source: Sel::This,
            amount,
            to: Sel::Var(CHOSEN_CREATURE_OR_PLAYER),
        },
    ]))
}

inventory::submit! { EffectPattern { name: "dice coins 4: ~ deals damage to a creature or player", priority: 100, parse: damage_to_a_creature_or_player } }

/// "If any of those results was 10 or higher" (after "whenever you roll one or more
/// dice"): one of the results of the dice rolled together that triggered the ability, or
/// of the ability's own roll.
fn any_of_those_results(c: &str) -> Option<Condition> {
    let r = end(c)
        .strip_prefix("any of those results was ")
        .or_else(|| end(c).strip_prefix("any of those results were "))?;
    let n: i64 = r.strip_suffix(" or higher")?.parse().ok()?;
    Some(Condition::Custom(
        format!("{}:{n}", crate::dice::ANY_RESULT_AT_LEAST).into(),
    ))
}

inventory::submit! { ConditionPattern { name: "dice coins 4: any of those results was N or higher", priority: 100, parse: any_of_those_results } }

/// "For each different result, create a 1/1 ... token." (Circuits Act): once for each
/// different result of the roll.
fn for_each_different_result(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let head = l
        .strip_prefix("for each different result, ")
        .map(str::to_string)
        .or_else(|| {
            l.strip_suffix(" for each different result")
                .map(str::to_string)
        })?;
    let e = parse_clause(&head, b)?;
    Some(Effect::Repeat {
        times: Value::Var(crate::dice::DIFFERENT),
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "dice coins 4: for each different result", priority: 100, parse: for_each_different_result } }

/// "Return up to X cards from among cards exiled with ~ to their owners' hands" (Bag of
/// Devouring): the same as "up to X cards exiled with ~".
fn from_among_cards_exiled_with(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" from among cards exiled with ~") {
        return None;
    }
    parse_clause(
        &l.replace(" from among cards exiled with ~", " exiled with ~"),
        b,
    )
}

inventory::submit! { EffectPattern { name: "dice coins 4: cards from among cards exiled with ~", priority: 100, parse: from_among_cards_exiled_with } }

/// "Put a creature card from any graveyard onto the battlefield under your control." (The
/// Deck of Many Things): the same as "from a graveyard" (any player's).
fn from_any_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" card from any graveyard ")?;
    if !l[..i].starts_with("put ") && !l[..i].starts_with("return ") {
        return None;
    }
    parse_clause(
        &l.replacen(" card from any graveyard ", " card from a graveyard ", 1),
        b,
    )
}

inventory::submit! { EffectPattern { name: "dice coins 4: a card from any graveyard", priority: 100, parse: from_any_graveyard } }

/// The greatest result an instruction "If the result is N or less, [effect]." covers.
fn result_at_most(e: &Effect) -> Option<i64> {
    let Effect::If {
        cond: Condition::Compare(Value::Custom(v), cmp, Value::Const(n)),
        otherwise,
        ..
    } = e
    else {
        return None;
    };
    if v.as_str() != crate::dice::THE_RESULT || !matches!(**otherwise, Effect::Noop) {
        return None;
    }
    match cmp {
        Cmp::Le => Some(i64::from(*n)),
        Cmp::Lt => Some(i64::from(*n) - 1),
        _ => None,
    }
}

/// "Roll a d20 and subtract the number of cards in your hand. If the result is 0 or less,
/// discard your hand. / 1—9 | ..." (The Deck of Many Things): a results table after
/// sentences about results it doesn't cover belongs to the roll before them; and a table
/// whose first row only reads as a continuation of the sentence before the roll (see
/// [`row_after_header`]). The table's
/// effects and those sentences' never both happen, so their order doesn't matter; they're
/// joined into one instruction for the table's further rows and sentences.
pub fn table_after_sentences(s: &str, effects: &mut Vec<Effect>, b: &mut Builder) -> bool {
    if repeat_until_all_heads(s, effects) {
        return true;
    }
    let lower = s.to_lowercase();
    let Some((lo, _, _)) = super::r705_706_dice::table_row(&lower) else {
        return false;
    };
    let Some(k) = effects
        .iter_mut()
        .rposition(|e| super::r705_706_dice::last_roll(e).is_some())
    else {
        return false;
    };
    // "{T}: Exile target instant or sorcery card from a graveyard. Roll a d20. / 1—9 | Copy
    // that card. ..." (Wizard's Spellbook): a row about the instruction before the roll,
    // in the sentence before it; the two sentences are joined.
    let from = if k + 1 == effects.len() && k > 0 {
        k - 1
    } else if k + 1 < effects.len()
        && effects[k + 1..]
            .iter()
            .all(|e| result_at_most(e).is_some_and(|n| n < lo))
    {
        k
    } else {
        return false;
    };
    let k = from;
    let mut joined = Effect::seq(effects[k..].to_vec());
    if !crate::oracle_ext::apply_followup_ext(s, &mut joined, b) {
        return false;
    }
    effects.truncate(k);
    effects.push(joined);
    true
}

/// The card [`one_of_those_cards`] chose.
const ONE_OF_THOSE: Var = vars::USER + 43001;

/// "onto the battlefield tapped", "into your hand", "into your graveyard".
fn simple_destination(s: &str) -> Option<Destination> {
    Some(match s.trim() {
        "onto the battlefield" => Destination::battlefield(),
        "onto the battlefield tapped" => Destination::battlefield().tapped(),
        "into your hand" => Destination::zone(ZoneKind::Hand),
        "into your graveyard" => Destination::zone(ZoneKind::Graveyard),
        _ => return None,
    })
}

/// "Put one of those cards onto the battlefield tapped and the other into your hand, then
/// shuffle." (Druid of the Emerald Grove's results-table row, after a search): you choose
/// one of the cards the earlier instruction found; it goes to the first place and the
/// rest to the second.
fn one_of_those_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("put one of those cards ")?;
    let (r, shuffle) = match r.strip_suffix(", then shuffle") {
        Some(r) => (r, true),
        None => (r, false),
    };
    let (first, second) = r
        .split_once(" and the other ")
        .or_else(|| r.split_once(" and the rest "))?;
    let (first, second) = (simple_destination(first)?, simple_destination(second)?);
    let those = match &b.it {
        Sel::Var(v) => Sel::Var(*v),
        _ => return None,
    };
    let mut v = vec![
        Effect::Store {
            var: ONE_OF_THOSE,
            sel: Sel::Choose {
                chooser: PlayerRef::You,
                filter: Filter::In(Box::new(those.clone())),
                count: Value::c(1),
                up_to: false,
                store: None,
            },
        },
        Effect::Move {
            what: Sel::Matching(
                Box::new(those),
                Filter::not(Filter::In(Box::new(Sel::Var(ONE_OF_THOSE)))),
            ),
            to: second,
        },
        Effect::Move {
            what: Sel::Var(ONE_OF_THOSE),
            to: first,
        },
    ];
    if shuffle {
        v.push(Effect::Shuffle {
            who: PlayerRef::You,
        });
    }
    Some(Effect::seq(v))
}

inventory::submit! { EffectPattern { name: "dice coins 4: put one of those cards ... and the other ...", priority: 100, parse: one_of_those_cards } }

/// The instruction just before the last die roll of `e` ("exile target spell" in "Exile
/// target spell, then roll a d20").
fn instruction_before_roll(e: &mut Effect) -> Option<&mut Effect> {
    match e {
        Effect::Seq(v) => {
            let i = v.iter().rposition(|x| matches!(x, Effect::RollDice(_)))?;
            if i > 0 {
                Some(&mut v[i - 1])
            } else {
                None
            }
        }
        _ => None,
    }
}

/// `e` as JSON, with an exile's link to its source left out.
fn unlinked_json(e: &Effect) -> Option<String> {
    let mut e = e.clone();
    if let Effect::Exile { link, .. } = &mut e {
        *link = false;
    }
    serde_json::to_string(&e).ok()
}

/// A results-table row whose first sentence continues the instruction before the roll
/// ("Exile target spell, then roll a d20 ... / 1—14 | You may cast the exiled card for as
/// long as it remains exiled ..."): the sentence is read as a follow-up of that
/// instruction, and the row's effect is what it adds. The follow-up may link that
/// instruction's exile to the source ("Copy each card exiled with ~", CR 607.2a), which
/// then applies to the instruction itself. The row's further sentences are read as usual.
pub fn row_after_header(text: &str, whole: &mut Effect, b: &mut Builder) -> Option<Effect> {
    let header = instruction_before_roll(whole)?.clone();
    let sentences = crate::oracle::effects::split_sentences(text);
    let (first, rest) = sentences.split_first()?;
    let mut e = header.clone();
    if !crate::oracle_ext::apply_followup_ext(first, &mut e, b) {
        return None;
    }
    // The follow-up must have kept the header's instruction as it was and added others
    // after it.
    let Effect::Seq(mut v) = e else {
        return None;
    };
    if v.len() < 2 || unlinked_json(&v[0])? != unlinked_json(&header)? {
        return None;
    }
    let new_header = v.remove(0);
    let mut row = Effect::seq(v);
    for s in rest {
        if crate::oracle_ext::apply_followup_ext(s, &mut row, b) {
            continue;
        }
        let next = crate::oracle::effects::parse_sentence(s, b)?;
        row = Effect::seq(vec![row, next]);
    }
    *instruction_before_roll(whole)? = new_header;
    Some(row)
}

/// Whether "it" is a target spell or ability ("Choose target spell or ability with one or
/// more targets.").
fn it_is_target_stack_object(b: &Builder) -> Option<u8> {
    let Sel::Target(i) = b.it else {
        return None;
    };
    matches!(
        b.targets.get(i as usize)?.what,
        TargetKind::Spell(_) | TargetKind::Ability(_) | TargetKind::SpellOrAbility(_)
    )
    .then_some(i)
}

/// "You may choose new targets for that spell or ability. Then copy it." (Wyll's
/// Reversal), after "Choose target spell or ability ...": that target.
fn that_target_spell_or_ability(l: &str, b: &mut Builder) -> Option<Effect> {
    let i = it_is_target_stack_object(b)?;
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    for that in ["that spell or ability", "that spell", "that ability"] {
        if let Some(head) = l.strip_suffix(&format!(" for {that}") as &str) {
            return parse_clause(&format!("{head} for it"), b);
        }
    }
    if matches!(
        l,
        "copy it" | "copy that spell or ability" | "copy that spell"
    ) {
        return Some(Effect::CopySpell {
            what: Sel::Target(i),
            count: Value::c(1),
            new_targets: false,
        });
    }
    None
}

inventory::submit! { EffectPattern { name: "dice coins 4: that target spell or ability", priority: 100, parse: that_target_spell_or_ability } }

/// "If you lose the flip, destroy target creature you control unless you pay {3} and
/// repeat this process." (Crooked Scales): paying instead repeats the whole process —
/// another flip, with the same targets (its rulings) — and the process repeats as long as
/// the player keeps paying after losing (CR 608.2c).
fn f_unless_pay_and_repeat(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(head) = end(l).strip_suffix(" and repeat this process") else {
        return false;
    };
    if !head.starts_with("if you lose the flip, ") || !head.contains(" unless you pay ") {
        return false;
    }
    let mut e = prev.clone();
    if !crate::oracle_ext::apply_followup_ext(head, &mut e, b) {
        return false;
    }
    let Some(flip) = super::r705_706_dice::last_flip(&mut e) else {
        return false;
    };
    let Effect::PayOptional { then, .. } = &mut flip.on_lose else {
        return false;
    };
    if !matches!(**then, Effect::Noop) {
        return false;
    }
    **then = Effect::RepeatThisProcess;
    *prev = Effect::RepeatProcess { body: Box::new(e) };
    true
}

inventory::submit! { super::FollowupPattern { name: "dice coins 4: unless you pay and repeat this process", priority: 5, apply: f_unless_pay_and_repeat } }

/// "Repeat this process until both players' coins come up heads on the same flip." (Mana
/// Clash): the instructions so far are a process, performed again after each pass in
/// which some flipper's coin came up tails (CR 608.2c).
fn repeat_until_all_heads(s: &str, effects: &mut Vec<Effect>) -> bool {
    let lower = s.to_lowercase();
    if !matches!(
        end(&lower),
        "repeat this process until both players' coins come up heads on the same flip"
            | "repeat this process until each player's coin comes up heads on the same flip"
    ) {
        return false;
    }
    let mut body = Effect::seq(std::mem::take(effects));
    let Some(flip) = super::r705_706_dice::last_flip(&mut body) else {
        *effects = vec![body];
        return false;
    };
    if flip.call {
        *effects = vec![body];
        return false;
    }
    let again = Effect::If {
        cond: Condition::Not(Box::new(Condition::Compare(
            Value::Var(crate::dice::PLAYERS_ALL_HEADS),
            Cmp::Eq,
            Value::Var(crate::dice::PLAYERS_FLIPPED),
        ))),
        then: Box::new(Effect::RepeatThisProcess),
        otherwise: Box::new(Effect::Noop),
    };
    *effects = vec![Effect::RepeatProcess {
        body: Box::new(Effect::seq(vec![body, again])),
    }];
    true
}

/// "You and target opponent each flip a coin." (Mana Clash): both flip; the effect cares
/// only about heads and tails (no call, CR 705.2).
fn you_and_player_each_flip(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("you and ")?
        .strip_suffix(" each flip a coin")?;
    let saved = b.targets.len();
    let (other, rest) = crate::oracle::effects::player_ref(r, b)?;
    if !rest.trim().is_empty() || !matches!(other, PlayerRef::Target(_)) {
        b.targets.truncate(saved);
        return None;
    }
    let mut flip = crate::dice::CoinFlip::new();
    flip.call = false;
    flip.who = PlayerRef::Each(PlayerFilter::Or(vec![
        PlayerFilter::You,
        PlayerFilter::Ref(Box::new(other)),
    ]));
    Some(Effect::FlipCoins(Box::new(flip)))
}

inventory::submit! { EffectPattern { name: "dice coins 4: you and target opponent each flip a coin", priority: 100, parse: you_and_player_each_flip } }

/// "~ deals 1 damage to each player whose coin comes up tails." after flips by several
/// players: what happens to each flipper whose coin came up tails (heads).
fn f_to_each_player_whose_coin(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (heads, phrase) = if l.contains("each player whose coin comes up tails") {
        (false, "each player whose coin comes up tails")
    } else if l.contains("each player whose coin comes up heads") {
        (true, "each player whose coin comes up heads")
    } else {
        return false;
    };
    if l.starts_with(phrase) {
        return false;
    }
    {
        let Some(f) = super::r705_706_dice::last_flip(prev) else {
            return false;
        };
        if !matches!(f.who, PlayerRef::Each(_)) || f.call {
            return false;
        }
    }
    let old = std::mem::replace(&mut b.it_player, PlayerRef::Iterated);
    let e = parse_clause(&l.replace(phrase, "that player"), b);
    b.it_player = old;
    let Some(e) = e else {
        return false;
    };
    let Some(f) = super::r705_706_dice::last_flip(prev) else {
        return false;
    };
    let branch = if heads {
        &mut f.on_heads
    } else {
        &mut f.on_tails
    };
    if !matches!(branch, Effect::Noop) {
        return false;
    }
    *branch = e;
    true
}

inventory::submit! { super::FollowupPattern { name: "dice coins 4: [effect] to each player whose coin comes up", priority: 10, apply: f_to_each_player_whose_coin } }

/// "If you would roll one or more planar dice, instead roll that many planar dice plus one
/// and ignore one." (Ichor Elixir).
fn extra_planar_die(
    l: &str,
    text: &str,
    _ctx: &crate::oracle::CompileContext,
) -> Option<Vec<Ability>> {
    if end(l)
        != "if you would roll one or more planar dice, instead roll that many planar dice plus one and ignore one"
    {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Dice(
            crate::dice::DiceStatic::ExtraPlanarDie {
                who: PlayerRel::You,
            },
        ))),
        text,
    )])
}

inventory::submit! { super::StaticPattern { name: "dice coins 4: extra planar die", priority: 100, parse: extra_planar_die } }

/// "Roll a d20 and add the number of creature cards in your graveyard that were put there
/// from the battlefield this turn. / 15+ | Return those cards from your graveyard to the
/// battlefield." (Revivify): "those cards" are the cards the roll's modifier counted.
pub fn row_about_counted(text: &str, bonus: Option<&Value>) -> Option<Effect> {
    let Some(Value::Count(f)) = bonus else {
        return None;
    };
    let t = end(text.trim()).to_lowercase();
    let to = match t.as_str() {
        "return those cards from your graveyard to the battlefield"
        | "return those cards to the battlefield" => Destination::battlefield(),
        "return those cards from your graveyard to your hand"
        | "return those cards to your hand" => Destination::zone(ZoneKind::Hand),
        _ => return None,
    };
    if f.zone() != Some(ZoneKind::Graveyard) {
        return None;
    }
    Some(Effect::Move {
        what: Sel::All(f.clone()),
        to,
    })
}
