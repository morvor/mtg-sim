//! Dice and coins, grammar V (cards5-game-rules-dice):
//!
//! - "{6}, Roll a d8: Monstrosity X, where X is the result." (Clay Golem): a die roll as a
//!   cost (CR 118.1, 602.2b), paid after the other parts of the cost (CR 601.2h); its
//!   result is the ability's "the result" (`dice::note_cost_roll`).
//! - "Roll a d20. [instructions using X]. / 1—9 | X is one. / 10—19 | X is two. / 20 | X is
//!   three." (Wand of Wonder): a results table whose rows only define X for instructions
//!   printed between the roll and the table (CR 706.3a, 706.3b, 107.3c). The table is
//!   applied right after the roll, so X has its value before any instruction uses it.
//! - "If you do, flip a coin. When you win the flip, copy that spell. You may choose new
//!   targets for the copy. When you lose the flip, ~ deals damage ... to any target."
//!   (Breeches, the Blastmaker): triggered abilities for the flip's result, created by the
//!   flip; only the one for its result triggers, and it chooses its own targets as it's
//!   put on the stack (the card's rulings; CR 603.7, 603.12).
//! - "Flip a coin until you lose a flip or choose to stop flipping. If you lose a flip, ~
//!   has no effect. If you win one or more flips, ..." (Fiery Gambit): the flipper may
//!   stop after each flip (`CoinFlip::may_stop`), and the instructions about flips won
//!   happen only if no flip was lost.
//! - "Choose a source you control and flip a coin." (Desperate Gambit): a source of the
//!   player's choice (CR 609.7a), "that source" for the next sentences.

use super::{AbilityPattern, CostPattern, EffectPattern, FollowupPattern};
use crate::ability::*;
use crate::dice::{CoinFlip, DieRoll, ResultRow};
use crate::oracle::effects::{parse_effect_text, parse_sentence, Builder};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// "roll a d8", "roll a six-sided die" as a cost.
fn roll_cost(l: &str) -> Option<CostPart> {
    let r = l.trim().strip_prefix("roll a ")?;
    let sides: u32 = match r.strip_prefix('d') {
        Some(n) => n.parse().ok()?,
        None => {
            let n = r.strip_suffix("-sided die")?;
            match crate::oracle::phrases::parse_number(n)? {
                (Value::Const(n), rest) if rest.trim().is_empty() => n as u32,
                _ => return None,
            }
        }
    };
    if sides < 2 {
        return None;
    }
    Some(CostPart::Effect(Box::new(Effect::RollDice(Box::new(
        DieRoll::new(sides),
    )))))
}

inventory::submit! { CostPattern { name: "dice coins 5: roll a dN as a cost", priority: 60, parse: roll_cost } }

/// "1—9 | X is one." rows at the end of a block: (low, high, value).
fn x_rows(rows: &str) -> Option<Vec<(i64, Option<i64>, i32)>> {
    let mut out = Vec::new();
    let mut rest = rows.trim();
    while !rest.is_empty() {
        let (range, r) = rest.split_once(" | ")?;
        let r = r.strip_prefix("X is ")?;
        let (n, r) = r.split_once('.')?;
        let (lo, hi, _) = super::r705_706_dice::table_row(&format!("{range} | x"))?;
        let (v, tail) = crate::oracle::phrases::parse_number(n)?;
        let (Value::Const(v), true) = (v, tail.trim().is_empty()) else {
            return None;
        };
        out.push((lo, hi, v));
        rest = r.trim_start();
    }
    (!out.is_empty()).then_some(out)
}

/// `text` with the word X replaced by `n` ("cast up to X ... spells" → "cast up to 2 ...").
fn with_x(text: &str, n: i32) -> Option<String> {
    let mut out = String::new();
    let mut found = false;
    for (i, w) in text.split(' ').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        if w == "X" {
            out.push_str(&n.to_string());
            found = true;
        } else if w.trim_matches(|c: char| !c.is_alphanumeric()) == "X" || w.contains("{X}") {
            // ("X's", "{X}": not a plain number word.)
            return None;
        } else {
            out.push_str(w);
        }
    }
    found.then_some(out)
}

/// The body of the one ability `text` compiles to.
fn only_body(text: &str, ctx: &CompileContext) -> Option<(Ability, Body)> {
    let abilities = crate::oracle::parse_ability(text, ctx)?;
    let [a] = abilities.as_slice() else {
        return None;
    };
    let body = match &a.kind {
        AbilityKind::Activated(x) => x.body.clone(),
        AbilityKind::Spell(x) => x.body.clone(),
        AbilityKind::Triggered(x) => x.body.clone(),
        _ => return None,
    };
    Some((a.clone(), body))
}

/// "Each opponent exiles cards from the top of their library until they exile an instant
/// or sorcery card, then shuffles the rest into their library. You may cast ... from among
/// cards exiled this way ...": the cards the reveal exiled (`vars::DUG`, those still in
/// exile), not what the shuffle of the rest left as "it".
fn exiled_this_way_after_shuffle(e: &mut Effect) {
    let Effect::Seq(v) = e else {
        return;
    };
    let Some(i) = v.iter().position(|x| matches!(x, Effect::RevealUntil { .. })) else {
        return;
    };
    let shuffles = |x: &Effect| {
        serde_json::to_string(x).is_ok_and(|j| j.contains("\"ShuffleIntoLibrary\""))
    };
    let Some(k) = v[i + 1..].iter().position(|x| shuffles(x)).map(|k| k + i + 1) else {
        return;
    };
    if let Some(Effect::Store { sel, .. }) = v.get_mut(k + 1) {
        if matches!(sel, Sel::Var(vars::IT)) {
            *sel = Sel::Var(vars::DUG);
        }
    }
}

/// "{4}, {T}: Roll a d20. [instructions using X]. 1—9 | X is one. 10—19 | X is two. 20 | X
/// is three." (Wand of Wonder): the roll comes first and its table only defines X, so each
/// row performs the instructions after the roll with its value of X.
fn table_defines_x(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    // The rows start at the first " N | X is " / " N—M | X is ".
    let i = t.find(" | X is ")?;
    let start = t[..i].rfind(". ")? + 1;
    let (head, rows) = (t[..start].trim(), &t[start..]);
    let rows = x_rows(rows)?;
    // "[cost: ]Roll a d20. [instructions]": the roll is the first instruction.
    let (cost, effect) = match head.split_once(": ") {
        Some((c, e)) => (Some(c), e),
        None => (None, head),
    };
    let (roll, after) = effect.split_once(". ")?;
    if cost.is_some_and(super::value_grammar::cost_has_x)
        || !(roll.starts_with("Roll a d") && !roll.contains(','))
        || after.to_lowercase().contains("where x is")
    {
        return None;
    }
    let prefix = cost.map(|c| format!("{c}: ")).unwrap_or_default();
    // The ability with the roll alone (its cost, timing and roll).
    let (mut ability, mut body) = only_body(&format!("{prefix}{roll}."), ctx)?;
    let Effect::RollDice(mut spec) = body.effect.clone() else {
        return None;
    };
    if !body.targets.is_empty() || body.modal.is_some() || !spec.table.is_empty() {
        return None;
    }
    for (lo, hi, n) in rows {
        let (_, b) = only_body(&format!("{prefix}{}", with_x(after, n)?), ctx)?;
        if !b.targets.is_empty() || b.modal.is_some() {
            return None;
        }
        let mut effect = b.effect;
        exiled_this_way_after_shuffle(&mut effect);
        spec.table.push(ResultRow { lo, hi, effect });
    }
    body.effect = Effect::RollDice(spec);
    let def = std::sync::Arc::make_mut(&mut ability);
    match &mut def.kind {
        AbilityKind::Activated(x) => x.body = body,
        AbilityKind::Spell(x) => x.body = body,
        AbilityKind::Triggered(x) => x.body = body,
        _ => return None,
    }
    def.text = block.to_string();
    Some(vec![ability])
}

inventory::submit! { AbilityPattern { name: "dice coins 5: a results table defining X for the instructions before it", priority: 104, parse: table_defines_x } }

/// The triggered ability a coin flip's result creates ("When you win the flip, ...").
fn result_trigger(e: &mut Effect) -> Option<&mut Body> {
    match e {
        Effect::Reflexive { body } => Some(body),
        _ => None,
    }
}

/// "When you win the flip, [effect]" / "When you lose the flip, [effect]" after a flip
/// (called by `r705_706_dice`'s flip results), and a further sentence of the last one
/// that modifies its instruction ("You may choose new targets for the copy.").
pub(super) fn when_flip_result(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let (win, rest) = if let Some(r) = l.strip_prefix("when you win the flip, ") {
        (true, r)
    } else if let Some(r) = l.strip_prefix("when you lose the flip, ") {
        (false, r)
    } else {
        return continue_result_trigger(l, prev, b);
    };
    let ctx = b.ctx;
    let named = b.named.clone();
    let (it, it_player) = (b.it.clone(), b.it_player.clone());
    let Some(f) = super::r705_706_dice::last_flip(prev) else {
        return false;
    };
    // One coin the controller flips with a call (CR 705.2), no other result yet.
    if !f.call
        || f.until_lose
        || !matches!(f.who, PlayerRef::You)
        || !matches!(f.count, Value::Const(1))
        || !matches!(if win { &f.on_win } else { &f.on_lose }, Effect::Noop)
        || !matches!(f.on_heads, Effect::Noop)
        || !matches!(f.on_tails, Effect::Noop)
    {
        return false;
    }
    let mut sub = Builder::new(ctx);
    sub.in_trigger = true;
    sub.it = it;
    sub.it_player = it_player;
    sub.named = named;
    let Some(effect) = parse_effect_text(rest, &mut sub) else {
        return false;
    };
    let t = Effect::Reflexive {
        body: Box::new(Body {
            targets: sub.targets,
            effect,
            modal: None,
        }),
    };
    if win {
        f.on_win = t;
    } else {
        f.on_lose = t;
    }
    true
}

/// "You may choose new targets for the copy." after "When you win the flip, copy that
/// spell.": part of that triggered ability.
fn continue_result_trigger(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    if l.starts_with("when ") || l.starts_with("if ") {
        return false;
    }
    let ctx = b.ctx;
    let Some(f) = super::r705_706_dice::last_flip(prev) else {
        return false;
    };
    // The one created last: the loss's, else the win's.
    let body = if !matches!(f.on_lose, Effect::Noop) {
        result_trigger(&mut f.on_lose)
    } else {
        result_trigger(&mut f.on_win)
    };
    let Some(body) = body else {
        return false;
    };
    let mut sub = Builder::new(ctx);
    sub.in_trigger = true;
    sub.targets = body.targets.clone();
    if !crate::oracle_ext::apply_followup_ext(l, &mut body.effect, &mut sub) {
        return false;
    }
    body.targets = sub.targets;
    true
}

/// Whether the rest of the ability, after "If you lose a flip, ~ has no effect.", is only
/// instructions about flips won ("If you win one or more flips, ...").
fn only_wins_follow(raw: &str) -> bool {
    let Some(i) = raw.find("has no effect.") else {
        return false;
    };
    let rest = raw[i + "has no effect.".len()..].trim();
    !rest.is_empty()
        && crate::oracle::effects::split_sentences(rest)
            .iter()
            .all(|s| s.trim().to_lowercase().starts_with("if you win "))
}

/// "If you lose a flip, ~ has no effect." after "Flip a coin until you lose a flip or
/// choose to stop flipping." (Fiery Gambit).
fn loss_has_no_effect(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if end(l) != "if you lose a flip, ~ has no effect" {
        return false;
    }
    let raw = crate::oracle::raw_text();
    let Some(f) = super::r705_706_dice::last_flip(prev) else {
        return false;
    };
    if !(f.until_lose && f.may_stop) || f.loss_voids || !only_wins_follow(&raw) {
        return false;
    }
    f.loss_voids = true;
    true
}

inventory::submit! { FollowupPattern { name: "dice coins 5: if you lose a flip, ~ has no effect", priority: 9, apply: loss_has_no_effect } }

/// "If you win two or more flips, [effect]" after a flip whose loss voids the rest: it
/// happens if that many flips were won and none was lost.
fn if_you_win_n_flips(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("if you win ") else {
        return false;
    };
    let Some((n, rest)) = r.split_once(" or more flips, ") else {
        return false;
    };
    let Some((Value::Const(n), tail)) = crate::oracle::phrases::parse_number(n) else {
        return false;
    };
    if !tail.trim().is_empty() || n < 1 {
        return false;
    }
    // The flip is the first instruction; each "If you win ..." sentence follows it.
    let flip_voids = match &mut *prev {
        Effect::Seq(v) => v.first_mut().and_then(super::r705_706_dice::last_flip),
        e => super::r705_706_dice::last_flip(e),
    }
    .is_some_and(|f| f.loss_voids);
    if !flip_voids {
        return false;
    }
    let Some(e) = parse_sentence(rest, b) else {
        return false;
    };
    let cond = Condition::And(vec![
        Condition::Compare(Value::Var(crate::dice::WINS), Cmp::Ge, Value::c(n)),
        Condition::Compare(Value::Var(crate::dice::LOSSES), Cmp::Eq, Value::c(0)),
    ]);
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![
        old,
        Effect::If {
            cond,
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "dice coins 5: if you win N or more flips (none lost)", priority: 9, apply: if_you_win_n_flips } }

/// Where "Choose a source you control" keeps the source.
const CHOSEN_SOURCE: Var = vars::USER + 43202;

/// "choose a source you control and flip a coin" (Desperate Gambit): a source of damage
/// of the player's choice (CR 609.7a: a permanent, a spell on the stack, or an object
/// something on the stack refers to), which "that source" and "it" name afterwards.
fn choose_a_source(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("choose a source you control")?;
    let then = match r {
        "" => None,
        r => Some(r.strip_prefix(" and ")?),
    };
    let mut v = vec![Effect::ChooseSource {
        who: PlayerRef::You,
        filter: Filter::ControlledBy(PlayerRel::You),
        var: CHOSEN_SOURCE,
    }];
    if let Some(t) = then {
        if t != "flip a coin" {
            return None;
        }
        v.push(Effect::FlipCoins(Box::new(CoinFlip::new())));
    }
    b.named.push(("that source".into(), Sel::Var(CHOSEN_SOURCE)));
    b.it = Sel::Var(CHOSEN_SOURCE);
    Some(Effect::seq(v))
}

inventory::submit! { EffectPattern { name: "dice coins 5: choose a source you control [and flip a coin]", priority: 100, parse: choose_a_source } }

/// The results of "roll a six-sided die twice" as it enters.
const FIRST_RESULT: Var = vars::USER + 43203;
const SECOND_RESULT: Var = vars::USER + 43204;

/// "roll a six-sided die", "roll a d6", optionally "twice": the kind of die and how many
/// times it's rolled.
fn as_enters_roll(s: &str) -> Option<(u32, u32)> {
    let r = s.strip_prefix("roll a ")?;
    let (r, times) = match r.strip_suffix(" twice") {
        Some(r) => (r, 2),
        None => (r, 1),
    };
    let sides = match r.strip_prefix('d') {
        Some(n) => n.parse().ok()?,
        None => match crate::oracle::phrases::parse_number(r.strip_suffix("-sided die")?)? {
            (Value::Const(n), rest) if rest.trim().is_empty() && n > 1 => n as u32,
            _ => return None,
        },
    };
    Some((sides, times))
}

/// "its base power becomes the first result and its base toughness becomes the second
/// result": the base power and toughness set by the rolls (`times` of them).
fn base_pt_from_results(s: &str, times: u32) -> Option<Modification> {
    let result = |w: &str| -> Option<Value> {
        Some(Value::Var(match (w, times) {
            ("the result", 1) => FIRST_RESULT,
            ("the first result", 2) => FIRST_RESULT,
            ("the second result", 2) => SECOND_RESULT,
            _ => return None,
        }))
    };
    let (mut p, mut t) = (None, None);
    for part in s.split(" and ") {
        let part = part.trim();
        if let Some(r) = part.strip_prefix("its base power becomes ") {
            if p.replace(result(r)?).is_some() {
                return None;
            }
        } else if let Some(r) = part.strip_prefix("its base toughness becomes ") {
            if t.replace(result(r)?).is_some() {
                return None;
            }
        } else {
            return None;
        }
    }
    // Every result is used.
    let used = [&p, &t]
        .iter()
        .filter_map(|v| match v {
            Some(Value::Var(x)) => Some(*x),
            _ => None,
        })
        .collect::<Vec<_>>();
    if (times == 2 && !(used.contains(&FIRST_RESULT) && used.contains(&SECOND_RESULT)))
        || (p.is_none() && t.is_none())
    {
        return None;
    }
    Some(Modification::SetPT(p, t))
}

/// "As ~ enters, roll a six-sided die twice. Its base power becomes the first result and
/// its base toughness becomes the second result." (Vedalken Squirrel-Whacker), "As ~
/// enters, flip a coin. If the coin comes up heads, ~ enters as a 5/2 creature with haste.
/// If it comes up tails, ~ enters as a 2/5 creature with defender." (Molten Sentry): die
/// rolls and coin flips performed as the permanent enters (CR 614.1c, 705, 706), whose
/// results set characteristics it enters with; like other "as ... enters" abilities that
/// set power and toughness, these are part of its copiable values (CR 707.2; Molten
/// Sentry's rulings).
fn as_enters_roll_or_flip(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if !ctx.is_permanent() || !ctx.type_line.card_types.contains(crate::types::CardType::Creature)
    {
        return None;
    }
    let lower = block
        .trim()
        .to_lowercase()
        .replace("enters the battlefield", "enters");
    let r = lower.strip_prefix("as ~ enters, ")?;
    let sentences: Vec<String> = crate::oracle::effects::split_sentences(r)
        .into_iter()
        .map(|s| end(&s.to_lowercase()).to_string())
        .collect();
    let [first, rest @ ..] = sentences.as_slice() else {
        return None;
    };
    let effect = if first == "flip a coin" {
        // "If the coin comes up heads, ~ enters as ... If it comes up tails, ~ enters as ...".
        let mut flip = CoinFlip::new();
        flip.call = false;
        let [heads, tails] = rest else {
            return None;
        };
        let form = |s: &str, prefixes: &[&str]| -> Option<Effect> {
            let r = prefixes.iter().find_map(|p| s.strip_prefix(p))?;
            let r = r.strip_prefix("~ enters as ")?;
            Some(Effect::EnterAs(
                super::etb_choices::creature_form(r, false, ctx)?,
            ))
        };
        flip.on_heads = form(heads, &["if the coin comes up heads, "])?;
        flip.on_tails = form(tails, &["if it comes up tails, ", "if the coin comes up tails, "])?;
        Effect::FlipCoins(Box::new(flip))
    } else {
        let (sides, times) = as_enters_roll(first)?;
        let [pt] = rest else {
            return None;
        };
        let m = base_pt_from_results(pt, times)?;
        let mut v = Vec::new();
        for var in [FIRST_RESULT, SECOND_RESULT].into_iter().take(times as usize) {
            v.push(Effect::RollDice(Box::new(DieRoll::new(sides))));
            v.push(Effect::StoreValue {
                var,
                value: Value::Custom(crate::dice::THE_RESULT.into()),
            });
        }
        v.push(Effect::EnterAs(vec![m]));
        Effect::seq(v)
    };
    let def = StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        action: ReplacementAction::AsEnters(Box::new(effect)),
        self_replacement: false,
        optional: false,
    });
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(def)),
        block,
    )])
}

inventory::submit! { AbilityPattern { name: "dice coins 5: as ~ enters, roll / flip, and what it enters as", priority: 104, parse: as_enters_roll_or_flip } }

/// "each player rolls a d20" (Chaos Dragon): every player rolls, in turn order; each
/// player's result is kept (`dice::PLAYER_RESULT`). No results table follows (only the
/// comparison of [`opponents_had_highest`]).
fn each_player_rolls(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("each player rolls a d")?;
    let sides: u32 = r.parse().ok()?;
    if sides < 2 {
        return None;
    }
    // Only with the comparison that reads each player's result next: "the result" and a
    // results table would be one player's.
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains(&format!(
        "each player rolls a d{sides}. if one or more opponents had the highest result, "
    )) {
        return None;
    }
    let mut spec = DieRoll::new(sides);
    spec.who = PlayerRef::EachPlayer;
    Some(Effect::RollDice(Box::new(spec)))
}

inventory::submit! { EffectPattern { name: "dice coins 5: each player rolls a dN", priority: 100, parse: each_player_rolls } }

/// The last instruction of an effect, if it's each player's roll.
fn each_player_roll(e: &Effect) -> bool {
    match e {
        Effect::RollDice(r) => matches!(r.who, PlayerRef::EachPlayer) && r.table.is_empty(),
        Effect::Seq(v) => v.last().is_some_and(each_player_roll),
        _ => false,
    }
}

/// "If one or more opponents had the highest result, ~ can't attack those players or
/// planeswalkers they control this combat." after each player's roll (Chaos Dragon): the
/// opponents whose result no player beat (ties included: the card's rulings say it can
/// attack freely only if your result was higher than all of your opponents').
fn opponents_had_highest(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !each_player_roll(prev) {
        return false;
    }
    let Some(r) = end(l).strip_prefix("if one or more opponents had the highest result, ~ can't attack those players") else {
        return false;
    };
    let planeswalkers = match r {
        " or planeswalkers they control this combat" => true,
        " this combat" => false,
        _ => return false,
    };
    let highest = PlayerFilter::And(vec![
        PlayerFilter::Opponent,
        PlayerFilter::History(Box::new(Condition::Custom(crate::dice::HAD_HIGHEST.into()))),
    ]);
    let e = Effect::AddRestriction {
        restriction: Restriction::CantAttackPlayer {
            attackers: Filter::Source,
            defender: PlayerFilter::Ref(Box::new(PlayerRef::Each(highest))),
            planeswalkers,
            battles: false,
        },
        duration: Duration::EndOfCombat,
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "dice coins 5: if one or more opponents had the highest result, ~ can't attack those players", priority: 9, apply: opponents_had_highest } }

/// "The next time target player would roll one or more dice this turn, instead they roll
/// that many dice plus one and you choose one of those rolls to ignore." (Bamboozling
/// Beeble): a replacement of that player's next die roll this turn (CR 614.1a, 706.6; see
/// `dice::NEXT_ROLL_EXTRA`).
fn next_roll_one_more(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("the next time ")?;
    let (who_s, r) = r.split_once(" would roll one or more dice this turn, instead ")?;
    let (pronoun, r) = match r.split_once(' ') {
        Some(("they" | "that player", r)) => ("they", r),
        Some(("you", r)) => ("you", r),
        _ => return None,
    };
    let rest = match pronoun {
        "they" => r.strip_prefix("roll that many dice plus one and ")?,
        _ => r.strip_prefix("roll that many dice plus one and ")?,
    };
    if rest != "you choose one of those rolls to ignore" {
        return None;
    }
    let who = match who_s {
        "you" if pronoun == "you" => PlayerRef::You,
        s if pronoun == "they" => {
            let (spec, tail) = crate::oracle::phrases::parse_target(s)?;
            if !tail.trim().is_empty() || !matches!(spec.what, TargetKind::Player(_)) {
                return None;
            }
            PlayerRef::Target(b.add_target(spec, s))
        }
        _ => return None,
    };
    Some(Effect::AddPlayerEffect {
        who,
        effect: PlayerModification::Custom(crate::dice::NEXT_ROLL_EXTRA.into()),
        duration: Duration::EndOfTurn,
    })
}

inventory::submit! { EffectPattern { name: "dice coins 5: the next time [player] would roll dice this turn, one more and you ignore one", priority: 100, parse: next_roll_one_more } }

/// "exile them again" / "exile it again" (Lae'zel's Acrobatics: "Return those cards to
/// the battlefield under their owner's control, then exile them again."): the objects the
/// previous instruction acted on, which are the new objects it put onto the battlefield
/// (CR 400.7).
fn exile_them_again(l: &str, b: &mut Builder) -> Option<Effect> {
    if !matches!(end(l), "exile them again" | "exile it again") {
        return None;
    }
    // Only right after the return, which leaves "it" naming the new permanents.
    if !matches!(b.it, Sel::Var(vars::IT)) {
        return None;
    }
    Some(Effect::Exile {
        what: Sel::Var(vars::IT),
        face_down: false,
        link: false,
    })
}

inventory::submit! { EffectPattern { name: "dice coins 5: exile them again", priority: 100, parse: exile_them_again } }

/// "Return those cards to the battlefield under their owner's control, then exile them
/// again." after an instruction that exiled them: the return (read as the follow-up it is
/// on its own), then the exile of the new permanents.
fn then_exile_them_again(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let l = end(l);
    let Some(head) = l
        .strip_suffix(", then exile them again")
        .or_else(|| l.strip_suffix(", then exile it again"))
    else {
        return false;
    };
    if !head.starts_with("return ") || !head.contains(" to the battlefield") {
        return false;
    }
    fn last(e: &Effect) -> &Effect {
        match e {
            Effect::Seq(v) => v.last().map_or(e, last),
            e => e,
        }
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let mut e = prev.clone();
    if !crate::oracle_ext::apply_followup_ext(head, &mut e, b)
        || !matches!(last(&e), Effect::Move { to, .. } if to.zone == ZoneKind::Battlefield)
    {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return false;
    }
    b.it = Sel::Var(vars::IT);
    *prev = Effect::seq(vec![
        e,
        Effect::Exile {
            what: Sel::Var(vars::IT),
            face_down: false,
            link: false,
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "dice coins 5: return those cards ..., then exile them again", priority: 9, apply: then_exile_them_again } }

/// "... until end of turn, or until any player rolls a 1, whichever comes first" (Drop
/// Tower): the duration and the text before it.
pub(crate) fn end_of_turn_or_roll(s: &str) -> Option<(Duration, &str)> {
    let r = s.strip_suffix(", whichever comes first")?;
    let (head, n) = r.rsplit_once(" until end of turn, or until any player rolls a ")?;
    let n: u32 = n.parse().ok()?;
    Some((Duration::UntilEndOfTurnOrRoll(n), head))
}

/// "[...] until you roll a 3 or less while rolling to visit your Attractions" (Ferris
/// Wheel): the text before it and the event.
pub(crate) fn until_roll_to_visit(s: &str) -> Option<(&str, UntilEvent)> {
    let r = s.strip_suffix(" or less while rolling to visit your attractions")?;
    let (head, n) = r.rsplit_once(" until you roll a ")?;
    let n: u32 = n.parse().ok()?;
    Some((head, UntilEvent::YouRollToVisitAtMost(n)))
}

/// `Filter::Custom`: an object that hasn't been phased out with the ability's source
/// ("target creature that hasn't been phased out with this Attraction", Ferris Wheel; see
/// `dice::DiceState::phased_out_by`).
pub const NOT_PHASED_OUT_WITH_THIS: &str = "attraction:not phased out with this";

fn not_phased_out_with_this<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let rest = r
        .strip_prefix("that hasn't been phased out with this attraction")
        .or_else(|| r.strip_prefix("that hasn't been phased out with ~"))?;
    Some((Filter::Custom(NOT_PHASED_OUT_WITH_THIS.into()), rest))
}

inventory::submit! { super::FilterSuffixPattern { name: "dice coins 5: that hasn't been phased out with this Attraction", priority: 100, parse: not_phased_out_with_this } }

/// "Roll a six-sided die plus an additional six-sided die for each mana from Treasures
/// spent to activate this ability." (Mr. House, President and CEO): one die, and one more
/// for each mana of that kind spent on the ability (`kw::cast_history`'s mana provenance).
fn roll_plus_one_for_each_mana(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("roll a ")?;
    let (die, r) = r.split_once(" plus an additional ")?;
    let (die2, r) = r.split_once(" for each mana from ")?;
    if die != die2 {
        return None;
    }
    let (sides, times) = as_enters_roll(&format!("roll a {die}"))?;
    if times != 1 {
        return None;
    }
    let kind = r.strip_suffix(" spent to activate this ability")?;
    let what = super::mana_4::mana_from_kind(kind)?;
    let mut spec = DieRoll::new(sides);
    spec.count = Value::Sum(vec![
        Value::c(1),
        super::history_conditions::spent_value("this ability", &what),
    ]);
    Some(Effect::RollDice(Box::new(spec)))
}

inventory::submit! { EffectPattern { name: "dice coins 5: roll a die plus an additional die for each mana from [kind] spent to activate this ability", priority: 100, parse: roll_plus_one_for_each_mana } }

#[cfg(test)]
mod tests {
    use super::super::followup_grammar::tests::compiled;

    #[test]
    fn wordings_compile() {
        for (ty, text) in [
            // A roll as a cost, with a d6 or "a six-sided die".
            ("Artifact", "{2}, Roll a d6: You gain life equal to the result."),
            ("Artifact", "{1}, Roll a six-sided die: Scry X, where X is the result."),
            // A table defining X, with a cost or as a spell.
            ("Sorcery", "Roll a d20. Draw X cards.\n1—9 | X is one.\n10—20 | X is two."),
            // One roll as it enters.
            ("Creature", "As ~ enters, roll a d6. Its base power becomes the result."),
            ("Creature", "As ~ enters, roll a six-sided die twice. Its base toughness becomes the first result and its base power becomes the second result."),
            // Coin flips as it enters, won or lost.
            ("Creature", "As ~ enters, flip a coin. If the coin comes up heads, ~ enters as a 4/4 creature with trample. If the coin comes up tails, ~ enters as a 1/1 creature with flying."),
            // Results of a flip as triggered abilities.
            ("Instant", "Flip a coin. When you win the flip, draw two cards. When you lose the flip, ~ deals 2 damage to any target."),
            ("Sorcery", "Flip a coin until you lose a flip or choose to stop flipping. If you lose a flip, ~ has no effect. If you win one or more flips, draw a card. If you win two or more flips, draw a card."),
            ("Instant", "Choose a source you control and flip a coin. If you lose the flip, the next time that source would deal damage this turn, prevent that damage."),
            ("Instant", "The next time target player would roll one or more dice this turn, instead they roll that many dice plus one and you choose one of those rolls to ignore."),
            ("Artifact — Attraction", "Visit — Target creature gets +2/+0 until end of turn, or until any player rolls a 6, whichever comes first."),
            ("Artifact — Attraction", "Visit — Target creature an opponent controls phases out until you roll a 2 or less while rolling to visit your Attractions."),
        ] {
            assert!(compiled(ty, text).is_some(), "{text}");
        }
    }

    #[test]
    fn wordings_not_accepted() {
        for (ty, text) in [
            // A table row defining X with nothing before it that uses X.
            ("Sorcery", "Roll a d20.\n1—9 | X is one.\n10—20 | X is two."),
            // Each player's roll without the comparison of their results.
            ("Creature", "At the beginning of your upkeep, each player rolls a d20. If the result is 10 or less, draw a card."),
            // A result the rolls don't give.
            ("Creature", "As ~ enters, roll a d6. Its base power becomes the second result."),
            // "Has no effect" with an instruction after it that isn't about flips won.
            ("Sorcery", "Flip a coin until you lose a flip or choose to stop flipping. If you lose a flip, ~ has no effect. Draw a card."),
            // Two coins: "When you win the flip" is about one flip.
            ("Instant", "Flip two coins. When you win the flip, draw a card."),
        ] {
            assert!(compiled(ty, text).is_none(), "{text}");
        }
    }
}
