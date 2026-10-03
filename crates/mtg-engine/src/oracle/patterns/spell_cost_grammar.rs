//! Spell cost modifiers as a grammar (CR 601.2f, 118.7): SUBJECT + "cost(s) AMOUNT less /
//! more to cast" + QUALIFIER.
//!
//! * SUBJECT: which spells — an object phrase for spells ("Cleric, Rogue, Warrior, and
//!   Wizard spells", "each artifact spell", "Green enchantment spells and white enchantment
//!   spells"), the caster ("you cast", "your opponents cast", "a player casts", "enchanted
//!   player casts", or anyone's), and qualifiers: "with mana value 4 or greater", "with the
//!   chosen name", "that target ~ / enchanted creature / enchanted player", "that's a Demon,
//!   Horror, or Nightmare", "that are black and/or red", "but don't own", "from your
//!   graveyard or from exile", "during your turn", and an order within the turn: "the first
//!   [quality] spell you cast each turn" (see `kw/first_spell_each_turn.rs`), "the second
//!   spell you cast each turn" (see `kw/spell_cost_grammar.rs`), "... during each of your
//!   turns".
//! * AMOUNT: generic mana ("{1}"), colored mana ("{U}": CR 118.7b–c), "{X} ..., where X is
//!   [value]", or "an additional 3 life" (an additional cost, CR 601.2f).
//! * QUALIFIER: "for each [thing counted]" — counted as the total cost is determined, with
//!   "it" meaning the spell and "its controller" / "that player" its caster ("for each
//!   creature it targets", "for each artifact its controller controls", "for each other
//!   spell that player has cast this turn"); "if it has mutate"; "if / as long as
//!   [condition]"; "except during its controller's turn"; "during your turn".
//!
//! A leading "During your turn, " / "During turns other than yours, " is a condition of the
//! static ability. The same grammar reads effects: "The next Giant spell you cast this turn
//! costs {2} less to cast." (a continuous effect waiting for that spell, CR 611.2f: the
//! spell gains "This spell costs {2} less to cast" as it's put on the stack, CR 601.2a),
//! and "[spells] cost {1} more to cast until your next turn" / "Spells you cast this turn
//! that are black and/or red cost {X} less to cast, where X is ..." (player effects whose
//! amounts are locked in as the effect begins, CR 611.2c).

use super::{
    AbilityPattern, ConditionPattern, EffectPattern, FilterSuffixPattern, FollowupPattern,
    StaticPattern,
};
use crate::ability::*;
use crate::kw::spell_cost_grammar as rules;
use crate::mana::{ManaCost, ManaSymbol};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;
use crate::types::{CardType, Color};

/// Where a spell is in the order of the spells its caster casts each turn.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Ordinal {
    First,
    Second,
}

/// What a spell cost change applies to.
#[derive(Clone, Debug)]
pub(crate) struct Subject {
    /// Whose spells, relative to the source's controller.
    pub who: PlayerRel,
    /// Conjuncts of the spell filter.
    pub parts: Vec<Filter>,
    /// Conditions of the static ability ("during your turn").
    pub conds: Vec<Condition>,
    pub ordinal: Option<Ordinal>,
    /// "this turn" (an effect's duration).
    pub this_turn: bool,
    /// The subject names a single spell ("each spell", "the next spell").
    pub singular: bool,
}

/// The casters a subject may name, in the order they're looked for.
const CASTERS: &[(&str, PlayerRel, Option<&str>)] = &[
    (" you cast", PlayerRel::You, None),
    (" your opponents cast", PlayerRel::Opponent, None),
    (" a player casts", PlayerRel::Any, None),
    (
        " enchanted player casts",
        PlayerRel::Any,
        Some(rules::CASTER_ENCHANTED),
    ),
];

fn zone(k: ZoneKind) -> Filter {
    Filter::Or(vec![Filter::InZone(k), Filter::CastFrom(k)])
}

/// "from your graveyard", "from graveyards", "from exile", "from anywhere other than your
/// hand", and two of them joined by "or from".
fn from_zones(z: &str) -> Option<Filter> {
    if let Some((a, b)) = z.split_once(" or from ") {
        return Some(Filter::Or(vec![from_zones(a)?, from_zones(b)?]));
    }
    Some(match z {
        "anywhere other than your hand" => Filter::not(zone(ZoneKind::Hand)),
        "exile" => zone(ZoneKind::Exile),
        // A card in your graveyard is yours (CR 404.1).
        "your graveyard" => Filter::and(vec![
            zone(ZoneKind::Graveyard),
            Filter::OwnedBy(PlayerRel::You),
        ]),
        "graveyards" | "a graveyard" => zone(ZoneKind::Graveyard),
        _ => return None,
    })
}

/// "~", "enchanted creature", "enchanted player", "a Merfolk you control": what a spell
/// targets.
fn targeted(s: &str) -> Option<Filter> {
    let objects = |f: Filter| {
        Filter::StackTargets(Box::new(TargetsFilter::Targets {
            objects: Some(f),
            players: None,
        }))
    };
    match s {
        "~" => return Some(objects(Filter::Source)),
        "enchanted creature" | "enchanted permanent" | "equipped creature" => {
            return Some(objects(Filter::In(Box::new(Sel::AttachedTo))))
        }
        "enchanted player" => {
            return Some(Filter::StackTargets(Box::new(TargetsFilter::Targets {
                objects: None,
                players: Some(PlayerFilter::Ref(Box::new(PlayerRef::ControllerOf(
                    Box::new(Sel::AttachedTo),
                )))),
            })))
        }
        _ => {}
    }
    let r = s.strip_prefix("a ").or_else(|| s.strip_prefix("an "))?;
    let (f, _, tail) = parse_object_phrase(r)?;
    end(tail).is_empty().then(|| objects(f))
}

/// Takes the qualifier `q` (with its leading space) out of `s`, if it's there.
fn take(s: &mut String, q: &str) -> bool {
    match s.find(q) {
        Some(i) if s[i + q.len()..].is_empty() || s[i + q.len()..].starts_with(' ') => {
            s.replace_range(i..i + q.len(), "");
            true
        }
        _ => false,
    }
}

/// The spell phrase left once the qualifiers are taken out: "spells", "instant and
/// sorcery spells", "Cleric, Rogue, Warrior, and Wizard spells", "green enchantment spells
/// and white enchantment spells", "creature spell with flying", "kicked spell".
fn spell_noun(s: &str, singular: bool) -> Option<Vec<Filter>> {
    let s = s.trim();
    let mut parts = Vec::new();
    let mut s = s.to_string();
    // "kicked spell" (CR 702.33d): the spell's controller declared they'd pay a kicker cost.
    if let Some(r) = s.strip_prefix("kicked ") {
        parts.push(Filter::CastWithCost("kicker".into()));
        s = r.to_string();
    }
    if s == "spells" || s == "spell" {
        if (s == "spell") != singular {
            return None;
        }
        parts.push(Filter::Spell);
        return Some(parts);
    }
    // "[A] spells and [B] spells": either kind.
    if let Some((a, b)) = s.split_once(" spells and ") {
        let mut kinds = Vec::new();
        for k in [format!("{a} spells"), b.to_string()] {
            let (f, plural, tail) = super::statics::object_phrase(&k)?;
            if !plural || !end(tail).is_empty() {
                return None;
            }
            kinds.push(f);
        }
        parts.push(Filter::Or(kinds));
        return Some(parts);
    }
    let phrase = super::statics::union_nouns(&s);
    let (f, plural, tail) = super::statics::object_phrase(&phrase)?;
    if !end(tail).is_empty() || plural == singular {
        return None;
    }
    // Only spells (not "creature cards").
    if !mentions_spell(&f) {
        return None;
    }
    parts.push(f);
    Some(parts)
}

fn mentions_spell(f: &Filter) -> bool {
    match f {
        Filter::Spell | Filter::SpellOnStack | Filter::InZone(ZoneKind::Stack) => true,
        Filter::And(v) => v.iter().any(mentions_spell),
        Filter::Or(v) => v.iter().all(mentions_spell),
        _ => false,
    }
}

/// "that's a Demon, Horror, or Nightmare" / "that are black and/or red" / "with {X} in its
/// mana cost": a qualifier the object phrase parser doesn't read after the caster.
fn relative_clause(q: &str) -> Option<Filter> {
    if let Some(r) = q
        .strip_prefix("that's a ")
        .or_else(|| q.strip_prefix("that's an "))
    {
        let probe = format!("{r} spell");
        let (f, _, tail) = super::statics::object_phrase(&probe)?;
        return end(tail).is_empty().then_some(f);
    }
    if let Some(r) = q.strip_prefix("that are ") {
        let probe = format!("{r} spells");
        let (f, _, tail) = super::statics::object_phrase(&probe)?;
        return end(tail).is_empty().then_some(f);
    }
    None
}

/// Parses a spell cost change's subject.
pub(crate) fn parse_subject(s: &str) -> Option<Subject> {
    let s = s.trim();
    let (ordinal, s) = if let Some(r) = s.strip_prefix("the first ") {
        (Some(Ordinal::First), r)
    } else if let Some(r) = s.strip_prefix("the second ") {
        (Some(Ordinal::Second), r)
    } else {
        (None, s)
    };
    let (each, s) = match s.strip_prefix("each ") {
        Some(r) => (true, r),
        None => (false, s),
    };
    let singular = each || ordinal.is_some();
    // The caster: the earliest marker in the phrase.
    let found = CASTERS
        .iter()
        .filter_map(|(m, who, custom)| {
            let i = s.find(m)?;
            let after = &s[i + m.len()..];
            (after.is_empty() || after.starts_with(' ')).then_some((i, *m, *who, *custom))
        })
        .min_by_key(|x| x.0);
    let mut parts = Vec::new();
    let (who, mut rest) = match found {
        Some((i, m, who, custom)) => {
            if let Some(c) = custom {
                parts.push(Filter::Custom(c.into()));
            }
            (who, format!("{} {}", &s[..i], &s[i + m.len()..]))
        }
        None => (PlayerRel::Any, s.to_string()),
    };
    let mut conds = Vec::new();
    let mut this_turn = false;
    // Qualifiers, wherever they are.
    if take(&mut rest, " but don't own") {
        if who != PlayerRel::You {
            return None;
        }
        parts.push(Filter::not(Filter::OwnedBy(PlayerRel::You)));
    }
    if take(&mut rest, " during each of your turns") || take(&mut rest, " during your turn") {
        conds.push(Condition::YourTurn);
    }
    if take(&mut rest, " each turn") && ordinal.is_none() {
        return None;
    }
    if take(&mut rest, " this turn") {
        this_turn = true;
    }
    if take(&mut rest, " with the chosen name") {
        parts.push(Filter::ChosenName);
    }
    if take(&mut rest, " with {x} in its mana cost") {
        parts.push(Filter::HasX);
    }
    // "from [zone] (or from [zone])": up to the next qualifier or the end.
    if let Some(i) = rest.find(" from ") {
        let after = &rest[i + " from ".len()..];
        let stop = [" that ", " with "]
            .iter()
            .filter_map(|w| after.find(w))
            .min()
            .unwrap_or(after.len());
        let f = from_zones(after[..stop].trim())?;
        parts.push(f);
        let tail = after[stop..].to_string();
        rest = format!("{}{tail}", &rest[..i]);
    }
    // "that target [object or player]": the rest of the phrase.
    if let Some(i) = rest.find(" that target ") {
        let what = rest[i + " that target ".len()..].trim().to_string();
        parts.push(targeted(&what)?);
        rest.truncate(i);
    }
    for w in [" that's a", " that are "] {
        if let Some(i) = rest.find(w) {
            parts.push(relative_clause(rest[i + 1..].trim())?);
            rest.truncate(i);
        }
    }
    let noun = spell_noun(&rest, singular)?;
    parts.extend(noun);
    if ordinal.is_some() && (who != PlayerRel::You || !matches!(found, Some((_, _, _, None)))) {
        return None;
    }
    Some(Subject {
        who,
        parts,
        conds,
        ordinal,
        this_turn,
        singular,
    })
}

/// An amount and what follows it: the change and the rest of the text.
pub(crate) struct Amount {
    pub change: CostChange,
    /// Spell qualities the amount's qualifier adds ("if it has mutate").
    pub parts: Vec<Filter>,
    pub conds: Vec<Condition>,
    /// "until your next turn" / "this turn": an effect's duration.
    pub duration: Option<Duration>,
}

/// What "for each [...]" counts as a spell's total cost is determined: "it" is the spell
/// and "its controller" / "that player" its caster.
fn for_each_spell(s: &str) -> Option<Value> {
    let s = end(s);
    let caster = || PlayerRef::TriggerPlayer;
    // "other spell that player has cast this turn": the spell being cast isn't cast yet
    // (CR 601.2i).
    if let Some(r) = s.strip_suffix(" that player has cast this turn") {
        let r = r.strip_prefix("other ").unwrap_or(r);
        let f = if r == "spell" {
            Filter::Any
        } else {
            let (f, _, tail) = parse_object_phrase(r)?;
            if !end(tail).is_empty() {
                return None;
            }
            f
        };
        return Some(Value::SpellsCastThisTurn(caster(), f));
    }
    // "artifact its controller controls".
    for suffix in [" its controller controls", " that player controls"] {
        if let Some(r) = s.strip_suffix(suffix) {
            let (f, _, tail) = parse_object_phrase(r)?;
            if !end(tail).is_empty() {
                return None;
            }
            return Some(Value::Count(Filter::and(vec![
                f,
                Filter::ControlledByPlayer(Box::new(caster())),
            ])));
        }
    }
    // "creature it targets" (each time it's targeted counts, Battlefield Thaumaturge
    // ruling).
    if let Some(r) = s.strip_suffix(" it targets") {
        let t = CardType::from_word(r)?;
        return Some(Value::Custom(rules::targets_of_type(t).into()));
    }
    if let Some(v) = super::value_results::whole_history_count(s) {
        return Some(v);
    }
    if s.contains(" this turn") || s.contains("target") {
        return None;
    }
    super::statics::parse_for_each(s, Some(&Sel::TriggerObject))
}

/// "{1} less to cast for each ...", "{U} less to cast", "{X} less to cast, where X is
/// ...", "an additional 3 life to cast".
pub(crate) fn parse_amount(
    r: &str,
    ctx: &CompileContext,
    effect: bool,
    in_trigger: bool,
) -> Option<Amount> {
    let r = end(r);
    let mut parts = Vec::new();
    let mut conds = Vec::new();
    // "an additional 3 life to cast" (Terror of the Peaks): an additional cost.
    if let Some(x) = r.strip_prefix("an additional ") {
        let (n, rest) = parse_number(x)?;
        let Value::Const(_) = n else {
            return None;
        };
        if end(rest) != "life to cast" {
            return None;
        }
        return Some(Amount {
            change: CostChange::AdditionalCost(Cost::free().with(CostPart::PayLife(n))),
            parts,
            conds,
            duration: None,
        });
    }
    if !r.starts_with('{') {
        return None;
    }
    let close = r.rfind("} ")?;
    let mana = ManaCost::parse(&r[..=close].to_uppercase())?;
    let t = r[close + 1..].trim_start();
    let (more, mut t) = if let Some(t) = t.strip_prefix("less to cast") {
        (false, t)
    } else {
        (true, t.strip_prefix("more to cast")?)
    };
    let mut duration = None;
    for (w, d) in [
        (" until your next turn", Duration::UntilYourNextTurn),
        (" this turn", Duration::EndOfTurn),
    ] {
        if let Some(x) = t
            .strip_suffix(w)
            .filter(|_| !t.contains(" for each ") && !t.contains(", where x is "))
        {
            if !effect {
                return None;
            }
            duration = Some(d);
            t = x;
        }
    }
    let mut times: Option<Value> = None;
    let x = matches!(mana.symbols.as_slice(), [ManaSymbol::X]);
    if let Some(v) = t.strip_prefix(", where x is ") {
        if !x {
            return None;
        }
        let mut b = Builder::new(ctx);
        b.in_trigger = in_trigger;
        if !effect {
            b.it = Sel::TriggerObject;
            b.it_player = PlayerRef::TriggerPlayer;
        }
        let (v, rest) = crate::oracle::statics::parse_value_phrase(v, &mut b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        times = Some(v);
    } else if x {
        return None;
    } else if let Some(fe) = t.strip_prefix(" for each ") {
        times = Some(for_each_spell(fe)?);
    } else if let Some(kw) = t.strip_prefix(" if it has ") {
        // "if it has mutate": a quality of the spell.
        let probe = format!("spell with {kw}");
        let (f, _, tail) = super::statics::object_phrase(&probe)?;
        if !end(tail).is_empty() {
            return None;
        }
        parts.push(f);
    } else if t == " except during its controller's turn" {
        parts.push(Filter::Custom(rules::CASTER_NOT_ACTIVE.into()));
    } else if !t.is_empty() {
        let c = super::costs_casting_self::cost_condition(t.trim(), ctx)?;
        conds.push(c);
    }
    let symbols = &mana.symbols;
    let generic = symbols.iter().all(|s| matches!(s, ManaSymbol::Generic(_)));
    let scaled = |n: u32| -> Value {
        match (&times, n) {
            (None, n) => Value::c(n as i32),
            (Some(v), 1) => v.clone(),
            (Some(v), n) => Value::Mul(Box::new(Value::c(n as i32)), Box::new(v.clone())),
        }
    };
    let change = if x {
        let v = times?;
        if more {
            CostChange::IncreaseGeneric(v)
        } else {
            CostChange::ReduceGeneric(v)
        }
    } else if generic {
        let v = scaled(mana.generic_amount());
        if more {
            CostChange::IncreaseGeneric(v)
        } else {
            CostChange::ReduceGeneric(v)
        }
    } else if let [ManaSymbol::Colored(c)] = symbols.as_slice() {
        let c: Color = *c;
        if more {
            if times.is_some() {
                return None;
            }
            CostChange::IncreaseMana(mana.clone())
        } else {
            CostChange::ReduceColored(c, scaled(1))
        }
    } else {
        if times.is_some() {
            return None;
        }
        if more {
            CostChange::IncreaseMana(mana.clone())
        } else {
            CostChange::ReduceMana {
                mana: mana.clone(),
                colored_only: false,
            }
        }
    };
    Some(Amount {
        change,
        parts,
        conds,
        duration,
    })
}

/// Splits "[subject] cost(s) [amount]" at the verb.
fn split_verb(l: &str) -> Option<(&str, &str)> {
    let mut best: Option<(usize, usize)> = None;
    for v in [" costs {", " cost {", " costs an additional ", " cost an additional "] {
        if let Some(i) = l.find(v) {
            // The verb and the space after it.
            let verb_len = v[1..].find(' ')? + 2;
            if best.is_none_or(|(b, _)| i < b) {
                best = Some((i, verb_len));
            }
        }
    }
    let (i, n) = best?;
    Some((&l[..i], &l[i + n..]))
}

fn condition_of(conds: Vec<Condition>) -> Option<Condition> {
    match conds.len() {
        0 => None,
        1 => conds.into_iter().next(),
        _ => Some(Condition::And(conds)),
    }
}

/// "[During your turn, ]SUBJECT cost(s) AMOUNT[ QUALIFIER]" as a static ability.
fn spell_cost_static(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut l = end(l);
    let mut conds = Vec::new();
    for (p, c) in [
        ("during your turn, ", Condition::YourTurn),
        ("during turns other than yours, ", Condition::NotYourTurn),
    ] {
        if let Some(r) = l.strip_prefix(p) {
            conds.push(c);
            l = r;
        }
    }
    let (subject, rest) = split_verb(l)?;
    let subj = parse_subject(subject)?;
    if subj.this_turn {
        return None;
    }
    // "each spell" / "the first spell ..." take "costs"; plural subjects "cost".
    let verb_s = l[subject.len()..].starts_with(" costs ");
    if verb_s != subj.singular {
        return None;
    }
    let amount = parse_amount(rest, ctx, false, false)?;
    let mut parts = subj.parts;
    parts.extend(amount.parts);
    match subj.ordinal {
        Some(Ordinal::First) => parts.push(Filter::Custom(
            crate::kw::first_spell_each_turn::FIRST_THIS_TURN.into(),
        )),
        Some(Ordinal::Second) => parts.push(Filter::Custom(rules::SECOND_THIS_TURN.into())),
        None => {}
    }
    conds.extend(subj.conds);
    conds.extend(amount.conds);
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::Spells(Filter::And(parts)),
        who: subj.who,
        change: amount.change,
    }));
    s.condition = condition_of(conds);
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "spell cost grammar: [spells] cost {N} less/more to cast", priority: 120, parse: spell_cost_static } }

/// "The next [quality] spell you cast this turn costs {N} less to cast." (CR 611.2f): the
/// spell gains "This spell costs {N} less to cast" as it's put on the stack (CR 601.2a).
fn next_spell_costs_less(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("the next ")?;
    let (subject, rest) = r.split_once(" costs {")?;
    let rest = format!("{{{rest}");
    let subject = subject.strip_suffix(" you cast this turn")?;
    let parts = spell_noun(subject, true)?;
    let amount = parse_amount(&rest, b.ctx, true, b.in_trigger)?;
    if !amount.parts.is_empty() || !amount.conds.is_empty() || amount.duration.is_some() {
        return None;
    }
    // The amount is determined as the effect begins (CR 611.2c; see
    // `kw/spell_cost_grammar.rs`).
    if !matches!(
        amount.change,
        CostChange::ReduceGeneric(_) | CostChange::ReduceColored(..)
    ) {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change: amount.change,
    }));
    s.zone = FunctionZone::Anywhere;
    let granted = AbilityDef::new(
        AbilityKind::Static(s),
        &format!(
            "This spell costs {} less to cast.",
            &rest[..rest.find('}')? + 1].to_uppercase()
        ),
    );
    Some(Effect::NextSpell {
        filter: Filter::and(parts),
        mods: vec![Modification::AddAbility(granted)],
        expires: Duration::ThisTurn,
    })
}

inventory::submit! { EffectPattern { name: "spell cost grammar: the next spell you cast this turn costs {N} less", priority: 100, parse: next_spell_costs_less } }

/// "[spells] cost {N} more to cast until your next turn", "Spells you cast this turn that
/// are black and/or red cost {X} less to cast, where X is ...": a player effect on the
/// casters (CR 611.2a), its amount locked in now (CR 611.2c).
fn spells_cost_for_a_while(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (subject, rest) = split_verb(l)?;
    let subj = parse_subject(subject)?;
    if subj.ordinal.is_some() || subj.singular || !subj.conds.is_empty() {
        return None;
    }
    let amount = parse_amount(rest, b.ctx, true, b.in_trigger)?;
    if !amount.conds.is_empty() {
        return None;
    }
    let duration = match (subj.this_turn, amount.duration) {
        (true, None) => Duration::EndOfTurn,
        (false, Some(d)) => d,
        _ => return None,
    };
    let who = match subj.who {
        PlayerRel::You => PlayerRef::You,
        PlayerRel::Opponent => PlayerRef::EachOpponent,
        PlayerRel::Any => PlayerRef::EachPlayer,
        _ => return None,
    };
    if subj
        .parts
        .iter()
        .any(|f| matches!(f, Filter::Custom(_)) || matches!(f, Filter::Not(x) if matches!(**x, Filter::OwnedBy(_))))
    {
        return None;
    }
    let mut parts = subj.parts;
    parts.extend(amount.parts);
    Some(Effect::AddPlayerEffect {
        who,
        effect: PlayerModification::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(Filter::And(parts)),
            who: PlayerRel::You,
            change: amount.change,
        }),
        duration,
    })
}

inventory::submit! { EffectPattern { name: "spell cost grammar: spells cost more for a duration", priority: 100, parse: spells_cost_for_a_while } }

#[cfg(test)]
mod tests {
    use super::*;

    fn subject(s: &str) -> Subject {
        parse_subject(s).unwrap_or_else(|| panic!("no subject: {s}"))
    }

    #[test]
    fn subjects() {
        let s = subject("cleric, rogue, warrior, and wizard spells you cast");
        assert_eq!(s.who, PlayerRel::You);
        let s = subject("spells you cast but don't own");
        assert!(s.parts.len() == 2);
        let s = subject("each spell a player casts");
        assert!(s.singular && s.who == PlayerRel::Any);
        let s = subject("the second spell you cast each turn");
        assert_eq!(s.ordinal, Some(Ordinal::Second));
        let s = subject("the first non-lemur creature spell with flying you cast during each of your turns");
        assert_eq!(s.ordinal, Some(Ordinal::First));
        assert_eq!(s.conds.len(), 1);
        subject("spells your opponents cast from graveyards or from exile");
        subject("spells you cast from your graveyard or from exile");
        subject("aura spells you cast that target enchanted creature");
        subject("spells that target ~");
        subject("spells with the chosen name enchanted player casts");
        subject("each spell you cast that's a demon, horror, or nightmare");
        subject("green enchantment spells and white enchantment spells");
        subject("instant and sorcery spells you cast with mana value 5 or greater");
        subject("the first spell you cast with {x} in its mana cost each turn");
        subject("spells you cast this turn that are black and/or red");
        assert!(parse_subject("creature cards you own").is_none());
    }

    #[test]
    fn verbs() {
        assert_eq!(
            split_verb("each spell costs {1} more to cast"),
            Some(("each spell", "{1} more to cast"))
        );
        assert_eq!(
            split_verb("spells your opponents cast that target ~ cost an additional 3 life to cast"),
            Some(("spells your opponents cast that target ~", "an additional 3 life to cast"))
        );
        assert_eq!(
            split_verb("the first spell you cast with {x} in its mana cost each turn costs {1} less to cast"),
            Some(("the first spell you cast with {x} in its mana cost each turn", "{1} less to cast"))
        );
    }
}

// ---------------------------------------------------------------------------
// A spell's own cost changes
// ---------------------------------------------------------------------------

/// "card you own in exile and in your graveyard that's an instant card, a sorcery card, or
/// a card that has an Adventure" (Sailors' Bane): the cards of those kinds in either zone.
pub(crate) fn cards_in_exile_and_graveyard(s: &str) -> Option<Value> {
    let list = s
        .strip_prefix("card you own in exile and in your graveyard that's ")
        .or_else(|| s.strip_prefix("cards you own in exile and in your graveyard that are "))?;
    let mut kinds = Vec::new();
    for item in list.replace(", or ", ", ").replace(" or ", ", ").split(", ") {
        let item = item.trim();
        let item = item
            .strip_prefix("a ")
            .or_else(|| item.strip_prefix("an "))
            .unwrap_or(item);
        let (f, _, tail) = parse_object_phrase(item)?;
        if !end(tail).is_empty() {
            return None;
        }
        kinds.push(f);
    }
    let kinds = Filter::Or(kinds);
    let count = |z: ZoneKind| {
        Value::Count(Filter::and(vec![
            kinds.clone(),
            Filter::Card,
            Filter::InZone(z),
            Filter::OwnedBy(PlayerRel::You),
        ]))
    };
    Some(Value::Sum(vec![
        count(ZoneKind::Exile),
        count(ZoneKind::Graveyard),
    ]))
}

/// "~ costs 3 life more to cast for each target." (Phyrexian Purge): the life is paid
/// once for each target chosen (CR 601.2c, 601.2f).
fn life_more_for_each_target(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("~ costs ")?;
    let (n, r) = parse_number(r)?;
    let Value::Const(_) = n else {
        return None;
    };
    if end(r) != "life more to cast for each target" {
        return None;
    }
    let repeated = CostPart::Repeated {
        cost: Box::new(Cost::free().with(CostPart::PayLife(n))),
        times: Value::Custom(rules::OWN_TARGETS.into()),
    };
    Some(vec![super::costs_casting_self::this_spell_cost_ability(
        CostChange::AdditionalCost(Cost::free().with(repeated)),
        None,
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "spell cost grammar: ~ costs N life more for each target", priority: 80, parse: life_more_for_each_target } }

/// "~ costs {1} less to cast if you control a Spirit. It also costs {1} less to cast if you
/// control an enchantment." (Geistlight Snare): two cost changes.
fn two_own_cost_changes(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let (a, b) = text.split_once(". It also costs ")?;
    let first = format!("{a}.");
    let second = format!("~ costs {b}");
    let mut out = super::costs_casting_self::own_cost_change(&first, ctx)?;
    out.extend(super::costs_casting_self::own_cost_change(&second, ctx)?);
    for x in out.iter_mut() {
        *x = AbilityDef::new(x.kind.clone(), text);
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "spell cost grammar: ~ costs less ... It also costs less", priority: 80, parse: two_own_cost_changes } }

// ---------------------------------------------------------------------------
// Additional costs paid any number of times
// ---------------------------------------------------------------------------

/// "sacrifice any number of creatures", "exile any number of black cards from your hand",
/// "tap any number of untapped creatures you control", "remove any number of +1/+1
/// counters from among creatures you control", "pay {1}{G} any number of times": one
/// instance of the cost, and the name the number of times it's paid is announced under.
fn repeatable_part(s: &str) -> Option<Cost> {
    if let Some(m) = s
        .strip_prefix("pay ")
        .and_then(|r| r.strip_suffix(" any number of times"))
    {
        let mana = ManaCost::parse(&m.to_uppercase())?;
        if mana.has_x() || format!("{mana}").to_lowercase() != m {
            return None;
        }
        return Some(Cost::mana(mana));
    }
    let (verb, rest) = s.split_once(" any number of ")?;
    if !matches!(verb, "sacrifice" | "exile" | "tap" | "remove") {
        return None;
    }
    let part = crate::oracle::costs::parse_cost_part(&format!("{verb} x {rest}"))?;
    let one = Value::c(1);
    let part = match part {
        CostPart::Sacrifice {
            filter,
            count: Value::X,
        } => CostPart::Sacrifice { filter, count: one },
        CostPart::Exile {
            filter,
            zone,
            count: Value::X,
        } => CostPart::Exile {
            filter,
            zone,
            count: one,
        },
        CostPart::TapUntapped {
            filter,
            count: Value::X,
        } => CostPart::TapUntapped { filter, count: one },
        CostPart::RemoveCountersFromAmong {
            kind,
            filter,
            count: Value::X,
        } => CostPart::RemoveCountersFromAmong {
            kind,
            filter,
            count: one,
        },
        _ => return None,
    };
    Some(Cost::free().with(part))
}

/// "As an additional cost to cast ~, [you may] sacrifice any number of creatures.[ ~ costs
/// {2} less to cast for each creature sacrificed this way[ and {2} less to cast for each
/// other artifact or creature you've sacrificed this turn].]": the number of times is
/// announced as the spell is cast (CR 601.2b) and the cost paid with the rest of the total
/// cost (CR 601.2f–h); a reduction for each one applies to that total (see
/// `kw/spell_cost_grammar.rs`).
fn any_number_additional_cost(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let l = end(&lower);
    let (first, second) = match l.split_once(". ") {
        Some((a, b)) => (a, Some(b)),
        None => (l, None),
    };
    let r = first.strip_prefix("as an additional cost to cast ~, ")?;
    let r = r.strip_prefix("you may ").unwrap_or(r);
    let one = repeatable_part(r)?;
    let name = smol_str::SmolStr::new(r);
    let times = Value::Custom(rules::paid_times(&name).into());
    let cost = Cost::free().with(CostPart::Repeated {
        cost: Box::new(one),
        times: times.clone(),
    });
    let mut out = vec![super::costs_casting_self::this_spell_cost_ability(
        CostChange::AdditionalCost(cost),
        None,
        text,
    )];
    if let Some(s) = second {
        let s = s.strip_prefix("~ costs ")?;
        let mut changes = Vec::new();
        for (i, piece) in s.split(" and ").enumerate() {
            let (mana, rest) = super::costs_casting_self::leading_mana(piece)?;
            if !mana.symbols.iter().all(|s| matches!(s, ManaSymbol::Generic(_))) {
                return None;
            }
            let fe = rest.trim_start().strip_prefix("less to cast for each ")?;
            let counted = if i == 0 && fe.ends_with(" this way") {
                // What the cost paid: as many as were announced.
                times.clone()
            } else {
                super::costs_casting_self::for_each_value(fe)?
            };
            let n = mana.generic_amount() as i32;
            changes.push(CostChange::ReduceGeneric(match n {
                1 => counted,
                n => Value::Mul(Box::new(Value::c(n)), Box::new(counted)),
            }));
        }
        let _ = ctx;
        for c in changes {
            out.push(super::costs_casting_self::this_spell_cost_ability(
                c, None, text,
            ));
        }
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "spell cost grammar: additional cost paid any number of times", priority: 79, parse: any_number_additional_cost } }

// ---------------------------------------------------------------------------
// Other additional costs
// ---------------------------------------------------------------------------

/// "As an additional cost to cast ~, you may sacrifice an artifact." / "... you may pay
/// {2}{R}." / "... you may exile a creature card from your graveyard.": an optional
/// additional cost announced as the spell is cast (CR 601.2b) and paid with the rest of
/// the total cost (CR 601.2f–h); "if this spell's additional cost was paid" checks it.
fn optional_plain_additional_cost(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("as an additional cost to cast ~, you may ")?;
    if r.contains(". ") || r.contains(" or ") {
        return None;
    }
    let cost = super::costs_casting_alt::plain_cost(r)?;
    Some(vec![super::costs_casting_self::this_spell_cost_ability(
        CostChange::OptionalAdditionalCost {
            name: smol_str::SmolStr::new(r),
            cost,
        },
        None,
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "spell cost grammar: you may [cost] as an additional cost", priority: 95, parse: optional_plain_additional_cost } }

/// "As an additional cost to cast ~, sacrifice half the lands you control, rounded up."
/// (Tectonic Split): as many as half of them, rounded up, counted as the cost is paid.
fn sacrifice_half(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("as an additional cost to cast ~, sacrifice half the ")?;
    let (what, up) = if let Some(w) = r.strip_suffix(", rounded up") {
        (w, true)
    } else {
        (r.strip_suffix(", rounded down")?, false)
    };
    let (f, true, tail) = parse_object_phrase(what)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    let count = Value::Div(Box::new(Value::Count(f.clone())), 2, up);
    Some(vec![super::costs_casting_self::this_spell_cost_ability(
        CostChange::AdditionalCost(Cost::free().with(CostPart::Sacrifice { filter: f, count })),
        None,
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "spell cost grammar: sacrifice half as an additional cost", priority: 80, parse: sacrifice_half } }

/// "As an additional cost to cast ~, you may collect evidence 6. ~ costs {2} less to cast
/// if evidence was collected.": two sentences, each a cost rule of the spell.
fn additional_cost_then_own_change(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') || !text.to_lowercase().starts_with("as an additional cost to cast ~") {
        return None;
    }
    let (a, b) = text.split_once(". ~ costs ")?;
    let first = format!("{a}.");
    let second = format!("~ costs {b}");
    let mut out = crate::oracle::parse_ability(&first, ctx)?;
    out.extend(super::costs_casting_self::own_cost_change(&second, ctx)?);
    let own_cost = |x: &Ability| {
        matches!(
            &x.kind,
            AbilityKind::Static(StaticAbility {
                effect: StaticEffect::CostModifier(CostModifier {
                    applies_to: CostTarget::ThisSpell,
                    ..
                }),
                ..
            })
        )
    };
    if !out.iter().all(own_cost) {
        return None;
    }
    for x in out.iter_mut() {
        *x = AbilityDef::new(x.kind.clone(), text);
    }
    Some(out)
}

inventory::submit! { AbilityPattern { name: "spell cost grammar: additional cost, then a cost change", priority: 81, parse: additional_cost_then_own_change } }

/// "You may cast ~ as though it had flash if you behold a Dragon as an additional cost to
/// cast it." (Molten Exhale, CR 601.3c, 701.4a).
fn flash_if_additional_cost(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = text.to_lowercase();
    let r = end(&lower)
        .strip_prefix("you may cast ~ as though it had flash if you ")?
        .strip_suffix(" as an additional cost to cast it")?;
    let cost = match super::a701_behold::behold_cost_part(r) {
        Some(p) => Cost::free().with(p),
        None => super::costs_casting_alt::plain_cost(r)?,
    };
    Some(vec![super::costs_casting_self::this_spell_cost_ability(
        CostChange::FlashForAdditionalCost(cost),
        None,
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "spell cost grammar: flash if you [cost] as an additional cost", priority: 80, parse: flash_if_additional_cost } }

/// "As an additional cost to cast ~, you may reveal a Dragon card from your hand or choose a
/// Dragon you control.": beholding a Dragon, spelled out (CR 701.4a).
fn spelled_out_behold(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("as an additional cost to cast ~, you may reveal ")?;
    let (card, rest) = r.split_once(" card from your hand or choose ")?;
    let quality = card.strip_prefix("a ").or_else(|| card.strip_prefix("an "))?;
    let chosen = rest.strip_suffix(" you control")?;
    let chosen = chosen
        .strip_prefix("a ")
        .or_else(|| chosen.strip_prefix("an "))?;
    if chosen != quality {
        return None;
    }
    let article = if card.starts_with("an ") { "an" } else { "a" };
    let as_behold = format!("As an additional cost to cast ~, you may behold {article} {quality}.");
    let out = crate::oracle::parse_ability(&as_behold, ctx)?;
    Some(
        out.into_iter()
            .map(|a| AbilityDef::new(a.kind.clone(), text))
            .collect(),
    )
}

inventory::submit! { AbilityPattern { name: "spell cost grammar: reveal a [quality] card or choose a [quality] (behold)", priority: 80, parse: spelled_out_behold } }

// ---------------------------------------------------------------------------
// Payments while an ability resolves
// ---------------------------------------------------------------------------

/// One resource of a payment: "{1}", "2 life", "life equal to its power", "{1} for each
/// artifact they control" (counted as it's paid, CR 702.24a-style repetition).
fn payment_part(s: &str, b: &mut Builder) -> Option<Cost> {
    if let Some(c) = super::counters_resources_pay::resolution_cost(s) {
        return Some(c);
    }
    if let Some((m, fe)) = s.split_once(" for each ") {
        let one = super::counters_resources_pay::resolution_cost(m)?;
        // The paying player's own permanents ("they control" after "that player may pay").
        let fe = fe.replace(" they control", " you control");
        let times = super::statics::parse_for_each(&fe, None)?;
        return Some(Cost::free().with(CostPart::Repeated {
            cost: Box::new(one),
            times,
        }));
    }
    let v = s.strip_prefix("life equal to ")?;
    let (v, rest) = crate::oracle::statics::parse_value_phrase(v, b)?;
    if !end(&rest).is_empty() {
        return None;
    }
    Some(Cost::free().with(CostPart::PayLife(v)))
}

/// "you may pay {1} and 1 life", "you may pay {W}{B} and 2 life", "you may pay life equal
/// to its power": an optional payment of all of them (CR 118.12); "If you do" / "When you
/// do" reads whether it was paid.
fn may_pay_compound(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you may pay ")?;
    let mut cost = Cost::free();
    let pieces: Vec<&str> = r.split(" and ").collect();
    if pieces.len() == 1 && !r.starts_with("life equal to ") && !r.contains(" for each ") {
        return None;
    }
    for piece in pieces {
        let c = payment_part(piece, b)?;
        if let Some(m) = c.mana {
            match cost.mana.as_mut() {
                Some(t) => t.add(&m),
                None => cost.mana = Some(m),
            }
        }
        cost.parts.extend(c.parts);
    }
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "spell cost grammar: you may pay [cost] and [cost]", priority: 99, parse: may_pay_compound } }

/// "This cost is reduced by {2} for each basic land type among lands you control." after
/// "[effect] unless you pay {10}" (Draco): the payment is {10} less {2} for each, never
/// less than nothing (CR 118.7).
fn this_cost_is_reduced(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("this cost is reduced by ") else {
        return false;
    };
    let Some((mana, rest)) = super::costs_casting_self::leading_mana(r) else {
        return false;
    };
    let Some(fe) = rest.trim_start().strip_prefix("for each ") else {
        return false;
    };
    if !mana.symbols.iter().all(|s| matches!(s, ManaSymbol::Generic(_))) {
        return false;
    }
    let Some(count) = super::statics::parse_for_each(fe, None) else {
        return false;
    };
    fn payment(e: &mut Effect) -> Option<&mut Cost> {
        match e {
            Effect::PayOptional { cost, .. } => Some(cost),
            Effect::Seq(v) => v.last_mut().and_then(payment),
            Effect::AsPlayer { effect, .. } => payment(effect),
            _ => None,
        }
    }
    let Some(cost) = payment(prev) else {
        return false;
    };
    let Some(m) = &cost.mana else {
        return false;
    };
    if !cost.parts.is_empty() || !m.symbols.iter().all(|s| matches!(s, ManaSymbol::Generic(_))) {
        return false;
    }
    let total = m.generic_amount() as i32;
    let less = Value::Mul(
        Box::new(Value::c(mana.generic_amount() as i32)),
        Box::new(count),
    );
    let times = Value::Max(
        Box::new(Value::c(0)),
        Box::new(Value::Diff(Box::new(Value::c(total)), Box::new(less))),
    );
    *cost = Cost::free().with(CostPart::Repeated {
        cost: Box::new(Cost::mana(ManaCost::generic(1))),
        times,
    });
    true
}

inventory::submit! { FollowupPattern { name: "spell cost grammar: this cost is reduced by {N} for each", priority: 100, apply: this_cost_is_reduced } }

/// "Each player starting with you may pay any amount of mana." (Mana-Charged Dragon): join
/// forces, worded the other way around.
fn join_forces_reworded(l: &str, b: &mut Builder) -> Option<Effect> {
    if end(l) != "each player starting with you may pay any amount of mana" {
        return None;
    }
    crate::oracle::effects::parse_clause(
        "starting with you, each player may pay any amount of mana",
        b,
    )
}

inventory::submit! { EffectPattern { name: "spell cost grammar: each player starting with you may pay any amount of mana", priority: 100, parse: join_forces_reworded } }

/// "with mana cost {0} or {1}" (Urza's Saga): exactly that mana cost (CR 202.1); an object
/// with no mana cost has none of them.
fn with_mana_cost<'a>(s: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = s.strip_prefix("with mana cost ")?;
    let mut options = Vec::new();
    let mut rest = r;
    loop {
        let close = rest.find('}')?;
        let mut j = close + 1;
        while rest[j..].starts_with('{') {
            j += rest[j..].find('}')? + 1;
        }
        let m = ManaCost::parse(&rest[..j].to_uppercase())?;
        options.push(Filter::Custom(rules::mana_cost_is(&m).into()));
        rest = &rest[j..];
        match rest.strip_prefix(" or ").filter(|x| x.starts_with('{')) {
            Some(x) => rest = x,
            None => break,
        }
    }
    let f = match options.len() {
        1 => options.pop()?,
        _ => Filter::Or(options),
    };
    Some((f, rest))
}

inventory::submit! { FilterSuffixPattern { name: "spell cost grammar: with mana cost {N} or {N}", priority: 100, parse: with_mana_cost } }

/// "the creature tapped to pay ~'s additional cost" (Swallow Whole): the creature the
/// spell's cost tapped.
fn tapped_to_pay(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" the creature tapped to pay ~'s additional cost") {
        return None;
    }
    let reworded = l.replace(
        " the creature tapped to pay ~'s additional cost",
        " the tapped creature",
    );
    crate::oracle::effects::parse_clause(&reworded, b)
}

inventory::submit! { EffectPattern { name: "spell cost grammar: the creature tapped to pay ~'s additional cost", priority: 100, parse: tapped_to_pay } }

/// "you have no land cards in hand": none of those cards in your hand.
fn no_cards_in_hand(c: &str) -> Option<Condition> {
    let r = end(c)
        .strip_prefix("you have no ")?
        .strip_suffix(" in hand")?;
    let (f, true, tail) = parse_object_phrase(r)? else {
        return None;
    };
    if !end(tail).is_empty() {
        return None;
    }
    Some(Condition::Compare(
        Value::Count(Filter::and(vec![
            f,
            Filter::InZone(ZoneKind::Hand),
            Filter::OwnedBy(PlayerRel::You),
        ])),
        Cmp::Eq,
        Value::c(0),
    ))
}

inventory::submit! { ConditionPattern { name: "spell cost grammar: you have no [cards] in hand", priority: 900, parse: no_cards_in_hand } }

/// "you've discarded a card this turn", "you've sacrificed an artifact this turn": one or
/// more such events this turn (read by the history grammar).
fn youve_done_this_turn(c: &str) -> Option<Condition> {
    let r = end(c).strip_prefix("you've ")?.strip_suffix(" this turn")?;
    let (verb, noun) = r.split_once(' ')?;
    let noun = noun
        .strip_prefix("a ")
        .or_else(|| noun.strip_prefix("an "))?;
    let v = super::value_results::whole_history_count(&format!("{noun} you've {verb} this turn"))?;
    Some(Condition::Compare(v, Cmp::Ge, Value::c(1)))
}

inventory::submit! { ConditionPattern { name: "spell cost grammar: you've [done something] this turn", priority: 900, parse: youve_done_this_turn } }

/// "you may pay any amount of life", "pay any amount of mana", "you may pay any amount of
/// {R}": the player chooses an amount and pays it (CR 107.1b, 119.4); "that many" / "that
/// much" in the instructions that follow is the amount paid (see
/// `kw/spell_cost_grammar.rs`).
fn pay_any_amount(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (may, r) = match l.strip_prefix("you may pay any amount of ") {
        Some(r) => (true, r),
        None => (false, l.strip_prefix("pay any amount of ")?),
    };
    let kind = match r {
        "life" => "life",
        "mana" => "mana",
        "{r}" | "{w}" | "{u}" | "{b}" | "{g}" | "{c}" => r,
        _ => return None,
    };
    Some(Effect::Custom(rules::pay_any_amount(kind, may).into()))
}

inventory::submit! { EffectPattern { name: "spell cost grammar: pay any amount of life/mana", priority: 100, parse: pay_any_amount } }

/// "If you do, draw that many cards." / "Look at that many cards ..." / "When you do, it
/// deals that much damage to any target." after [`pay_any_amount`]: X is the amount paid.
fn that_many_paid(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    fn ends_with_pay(e: &Effect) -> bool {
        match e {
            Effect::Custom(n) => rules::is_pay_any_amount(n),
            Effect::Seq(v) => v.last().is_some_and(ends_with_pay),
            Effect::AsPlayer { effect, .. } => ends_with_pay(effect),
            _ => false,
        }
    }
    // "you may pay life equal to its power. If you do, put that many +1/+1 counters on
    // it": the life paid.
    fn life_paid(e: &Effect) -> Option<Value> {
        match e {
            Effect::PayOptional { cost, .. } if cost.mana.is_none() => match cost.parts.as_slice() {
                [CostPart::PayLife(v)] if !matches!(v, Value::Const(_)) => Some(v.clone()),
                _ => None,
            },
            Effect::Seq(v) => v.last().and_then(life_paid),
            Effect::AsPlayer { effect, .. } => life_paid(effect),
            _ => None,
        }
    }
    let life = life_paid(prev);
    if !(ends_with_pay(prev) || life.is_some())
        || !(l.contains("that many") || l.contains("that much"))
    {
        return false;
    }
    if let Some(v) = life {
        let reworded = l.replace("that many", "x").replace("that much", "x");
        let Some(e) = super::value_grammar::with_x_defined(true, || {
            crate::oracle::effects::parse_sentence(&reworded, b)
        }) else {
            return false;
        };
        let p = std::mem::replace(prev, Effect::Noop);
        *prev = Effect::seq(vec![p, Effect::SetX { value: v }, e]);
        return true;
    }
    let reworded = l.replace("that many", "x").replace("that much", "x");
    // "When you do, it deals that much damage to any target": a reflexive triggered
    // ability (CR 603.12) whose "it" keeps its meaning (nothing was acted on), and whose
    // X is the amount paid, kept for it.
    let e = if let Some(r) = reworded.strip_prefix("when you do, ") {
        let mut sub = Builder::new(b.ctx);
        sub.in_trigger = true;
        sub.it = b.it.clone();
        sub.it_player = b.it_player.clone();
        let Some(effect) = super::value_grammar::with_x_defined(true, || {
            crate::oracle::effects::parse_effect_text(r, &mut sub)
        }) else {
            return false;
        };
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(Effect::Reflexive {
                body: Box::new(Body {
                    targets: sub.targets,
                    effect: Effect::seq(vec![
                        Effect::SetX {
                            value: Value::Var(rules::AMOUNT_PAID),
                        },
                        effect,
                    ]),
                    modal: None,
                }),
            }),
            otherwise: Box::new(Effect::Noop),
        }
    } else {
        let Some(e) = super::value_grammar::with_x_defined(true, || {
            crate::oracle::effects::parse_sentence(&reworded, b)
        }) else {
            return false;
        };
        e
    };
    let p = std::mem::replace(prev, Effect::Noop);
    *prev = Effect::seq(vec![p, e]);
    true
}

inventory::submit! { FollowupPattern { name: "spell cost grammar: that many (the amount paid)", priority: 100, apply: that_many_paid } }

// ---------------------------------------------------------------------------
// Alternative costs for activated abilities
// ---------------------------------------------------------------------------

/// "You may pay {0} rather than pay the cycling cost of the first card you cycle each
/// turn." (Gavi), "You may remove a loyalty counter from a planeswalker you control rather
/// than pay ~'s crew cost." (Heart of Kiran): an alternative cost for a keyword ability's
/// cost (CR 118.9, 601.2b via 602.2b), announced as the ability is activated; the rest of
/// the keyword ability's cost is still paid (see `activation_costs.rs`).
fn keyword_cost_alternative(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("you may ")?;
    let (pay, r) = r.split_once(" rather than pay ")?;
    let cost = match super::costs_casting_alt::plain_cost(pay) {
        Some(c) => c,
        None => {
            // "remove a loyalty counter from a planeswalker you control".
            let (c, false) = crate::oracle::costs::parse_cost(pay)? else {
                return None;
            };
            let ok = c.mana.is_none()
                && matches!(
                    c.parts.as_slice(),
                    [CostPart::RemoveCountersFromAmong {
                        kind: Some(_),
                        count: Value::Const(1),
                        ..
                    }]
                );
            if !ok {
                return None;
            }
            c
        }
    };
    let (kind, sources, first) = if let Some(k) = r
        .strip_prefix("the ")
        .and_then(|x| x.strip_suffix(" cost of the first card you cycle each turn"))
    {
        // Cycling a card is activating its cycling ability (CR 702.29a).
        if k != "cycling" {
            return None;
        }
        (k, Filter::Any, true)
    } else {
        let k = r.strip_prefix("~'s ")?.strip_suffix(" cost")?;
        (k, Filter::Source, false)
    };
    let kind = crate::keywords::KeywordKind::from_name(kind)?;
    let mut scope = AbilityScope::new(sources, AbilityClass::Keyword(kind));
    scope.first_each_turn = first;
    let s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ActivatedAbilities(Box::new(scope)),
        who: PlayerRel::You,
        change: CostChange::AlternativeCost(cost),
    }));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "spell cost grammar: pay rather than pay a keyword ability's cost", priority: 100, parse: keyword_cost_alternative } }

/// "Unlock costs you pay cost {1} less." (Inquisitive Glimmer): the costs of unlocking
/// doors (CR 709.5e) that you pay.
fn unlock_costs_less(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("unlock costs you pay cost {")?;
    let n: u32 = r.strip_suffix("} less")?.parse().ok()?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
            rules::unlock_costs_less(n).into(),
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "spell cost grammar: unlock costs you pay cost {N} less", priority: 100, parse: unlock_costs_less } }

/// "you may pay {R}{R} or 2 life", "you may pay {G}, {W}, or {U}": the player may pay one
/// of the costs (CR 118.12); each is offered in turn until one is paid, and "If you do"
/// reads whether one was.
fn may_pay_one_of(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you may pay ")?;
    let list = r.replace(", or ", ", ").replace(" or ", ", ");
    let options: Vec<&str> = list.split(", ").collect();
    if options.len() < 2 {
        return None;
    }
    let mut costs = Vec::new();
    for o in options {
        costs.push(payment_part(o.trim(), b)?);
    }
    let pay = |cost: Cost| Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::Noop),
    };
    let mut it = costs.into_iter().rev();
    let mut e = pay(it.next()?);
    for c in it {
        e = Effect::seq(vec![
            pay(c),
            Effect::If {
                cond: Condition::Not(Box::new(Condition::PrevHappened)),
                then: Box::new(e),
                // (Not `Noop`: when a cost was paid, the record that it was stays.)
                otherwise: Box::new(Effect::Seq(vec![])),
            },
        ]);
    }
    Some(e)
}

inventory::submit! { EffectPattern { name: "spell cost grammar: you may pay [cost] or [cost]", priority: 99, parse: may_pay_one_of } }

/// "it's at least one of the chosen colors" (Tablet of the Guilds): the spell or
/// triggering object is one or more of the source's chosen colors.
fn at_least_one_chosen_color(c: &str) -> Option<Condition> {
    (end(c) == "it's at least one of the chosen colors").then(|| {
        Condition::Compare(
            Value::Custom(rules::CHOSEN_COLORS_IT_IS.into()),
            Cmp::Ge,
            Value::c(1),
        )
    })
}

inventory::submit! { ConditionPattern { name: "spell cost grammar: it's at least one of the chosen colors", priority: 100, parse: at_least_one_chosen_color } }
