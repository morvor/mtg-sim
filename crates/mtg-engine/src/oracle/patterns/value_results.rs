//! Value grammar II: amounts that refer to what happened earlier, in the same ability or
//! earlier in the turn (CR 107, 608.2c, 608.2h).
//!
//! Hooks into the value grammar (`value_grammar::count`, `value_grammar::atom_ext`).

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// Whether `rest` ends a word (so a prefix match is a whole phrase).
fn word_end(rest: &str) -> bool {
    rest.is_empty() || rest.starts_with([' ', ',', '.', ';', '"'])
}

/// After "the number of" / "for each": what's counted. Tried before the value grammar's
/// own readings.
pub fn count_ext(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    // An amount chosen for a cost ("cards exiled this way" by an exile cost, "counters
    // removed this way") is the cost grammar's.
    if super::cost_parts::paid_this_way_prefix(r).is_some() {
        return None;
    }
    if let Some(v) = life_lost_this_way(r) {
        return Some(v);
    }
    if let Some(v) = trigger_spell_value(&format!("the number of {r}"), b) {
        return Some(v);
    }
    if let Some(v) = event_amount_groups(r, b) {
        return Some(v);
    }
    if let Some(v) = this_way_count(r, b) {
        return Some(v);
    }
    history_count(r, b)
}

/// A whole value phrase ("the amount of damage dealt to you this turn"). Tried before the
/// value grammar's own readings.
pub fn atom_ext(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(v) = trigger_spell_value(s, b) {
        return Some(v);
    }
    if let Some(v) = event_amount(s, b) {
        return Some(v);
    }
    if let Some(v) = excess(s, b) {
        return Some(v);
    }
    if let Some(v) = result_value(s, b) {
        return Some(v);
    }
    history_amount(s, b)
}

/// "the number of [s]" / "for each [s]" read as a whole phrase about this turn's history
/// (for readers outside instructions: cost changes, static abilities).
pub fn whole_history_count(s: &str) -> Option<Value> {
    // This grammar's own reading first (the value grammar would read "creatures that
    // attacked this turn" as those still on the battlefield).
    {
        use crate::card::Layout;
        use crate::oracle::CompileContext;
        let tl = crate::types::TypeLine::default();
        let ctx = CompileContext {
            card_name: "",
            full_name: "",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        let mut b = Builder::new(&ctx);
        b.it = Sel::This;
        if let Some((v, rest)) = count_ext(end(s), &mut b) {
            if rest.trim().is_empty() && b.targets.is_empty() {
                return Some(v);
            }
        }
    }
    let v = super::value_grammar::whole_count(s, Some(&Sel::This))?;
    let j = serde_json::to_string(&v).unwrap_or_default();
    // Read by this grammar (not an object phrase with a loosely read qualifier, such as
    // the creatures still on the battlefield that attacked this turn).
    [
        "EventsThisTurn",
        "ThisTurn",
        "PermanentsEnteredThisTurn",
        "SpellsCastThisTurn",
        "Custom",
    ]
    .iter()
    .any(|w| j.contains(w))
    .then_some(v)
}

fn events(cond: TriggerCond, t: Tally) -> Value {
    Value::EventsThisTurn(Box::new(cond), t)
}

/// A player the text names as the subject of a past action, as a relation: "you",
/// "your opponents", "an opponent", "they" / "that player" (the player the ability is
/// about), "target player". Returns the relation and the rest.
fn player_subject<'a>(s: &'a str, b: &Builder) -> Option<(PlayerRel, &'a str)> {
    let s = s.trim_start();
    let about = || -> Option<PlayerRel> {
        match &b.it_player {
            // Only a trigger about a player ("whenever a player casts a spell", "at the
            // beginning of each player's upkeep") has a player "they" can mean.
            PlayerRef::TriggerPlayer if trigger_names_player(b) => Some(PlayerRel::TriggerPlayer),
            PlayerRef::Target(t) => Some(PlayerRel::Target(*t)),
            PlayerRef::Iterated => Some(PlayerRel::Iterated),
            _ => None,
        }
    };
    for (p, rel) in [
        ("you", Some(PlayerRel::You)),
        ("your opponents", Some(PlayerRel::Opponent)),
        ("an opponent", Some(PlayerRel::Opponent)),
        ("opponents", Some(PlayerRel::Opponent)),
        ("they", about()),
        ("that player", about()),
    ] {
        if let Some(r) = s.strip_prefix(p) {
            if word_end(r) || r.starts_with("'ve") {
                return Some((rel?, r));
            }
        }
    }
    None
}

/// Whether the trigger condition of the ability being compiled names a player ("whenever a
/// player casts a spell", "at the beginning of each opponent's upkeep").
fn trigger_names_player(b: &Builder) -> bool {
    if !b.in_trigger {
        return false;
    }
    let raw = crate::oracle::raw_text().to_lowercase();
    raw.split(['"', '\n']).any(|part| {
        let Some(i) = part
            .find("whenever ")
            .or_else(|| part.find("when "))
            .or_else(|| part.find("at the beginning of "))
        else {
            return false;
        };
        let cond = part[i..].split(',').next().unwrap_or("");
        cond.contains("player") || cond.contains("opponent")
    })
}

/// "Each opponent loses life equal to the life that player lost this turn." (Archfiend of
/// Despair): "that player" is each opponent in turn, as "they" would be.
fn each_player_that_player(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !(l.starts_with("each opponent ") || l.starts_with("each player ")) {
        return None;
    }
    if !l.contains(" that player ") || !l.ends_with(" this turn") || l.contains("target") {
        return None;
    }
    crate::oracle::effects::parse_clause(&l.replace(" that player ", " they "), b)
}

inventory::submit! { super::EffectPattern { name: "value results: each opponent ... that player ... this turn", priority: 5, parse: each_player_that_player } }

/// "~ deals damage to each opponent equal to the number of cards that player has drawn
/// this turn", "... equal to the number of tapped creatures that opponent controls": an
/// amount for each player in turn (CR 608.2h), "that player" being that one.
fn damage_each_player_equal(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, v) = l.split_once(" equal to ")?;
    let (source, who) = [
        (
            "~ deals damage to each opponent",
            (Sel::This, PlayerRef::EachOpponent),
        ),
        (
            "~ deals damage to each player",
            (Sel::This, PlayerRef::EachPlayer),
        ),
    ]
    .into_iter()
    .find_map(|(p, x)| (head == p).then_some(x))?;
    if !(v.contains("that player") || v.contains("that opponent")) {
        return None;
    }
    let v = v.replace("that opponent", "that player");
    let saved = b.it_player.clone();
    b.it_player = PlayerRef::Iterated;
    let read = super::r107_numbers::value_phrase(&v, b);
    b.it_player = saved;
    let (amount, tail) = read?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::ForEachPlayer {
        who,
        effect: Box::new(Effect::DealDamage {
            source,
            amount,
            to: Sel::Players(PlayerRef::Iterated),
        }),
    })
}

inventory::submit! { super::EffectPattern { name: "value results: ~ deals damage to each opponent equal to [amount for that player]", priority: 60, parse: damage_each_player_equal } }

/// "[player] [have|has|'ve] [verb]" (or the simple past "[player] [verb]"): the player
/// and the rest after the auxiliary.
fn player_perfect<'a>(s: &'a str, b: &Builder) -> Option<(PlayerRel, &'a str)> {
    let (rel, r) = player_subject(s, b)?;
    let r = r
        .strip_prefix("'ve ")
        .or_else(|| r.strip_prefix(" have "))
        .or_else(|| r.strip_prefix(" has "))
        .or_else(|| r.strip_prefix(" "))?;
    Some((rel, r))
}

/// Strips "this turn" (and returns the rest), requiring a word end after it.
fn this_turn(s: &str) -> Option<&str> {
    let r = s.trim_start().strip_prefix("this turn")?;
    word_end(r).then_some(r)
}

/// History counts: "creatures that died under your control this turn", "nontoken
/// creatures put into your graveyard from the battlefield this turn", "creatures that
/// attacked this turn", "cards you've discarded this turn", "spells your opponents have
/// cast this turn", "permanents you've sacrificed this turn", "tokens you created this
/// turn", "opponents who lost life this turn", "2 life your opponents have lost this turn".
fn history_count(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    if let Some(v) = players_who(r, b) {
        return Some(v);
    }
    if let Some(v) = kinds_among_history(r, b) {
        return Some(v);
    }
    if let Some(v) = more_history_count(r, b) {
        return Some(v);
    }
    if let Some(v) = life_in_groups(r, b) {
        return Some(v);
    }
    // "cards [player] [have] drawn this turn", "cards you've discarded this turn",
    // "spells your opponents have cast this turn": the noun, then the player.
    for (noun, card) in [
        ("cards ", true),
        ("card ", true),
        ("spells ", false),
        ("spell ", false),
    ] {
        if let Some(x) = r.strip_prefix(noun) {
            if let Some(v) = player_action(x, Filter::Any, card, b) {
                return Some(v);
            }
        }
    }
    if let Some(v) = object_history(r) {
        return Some(v);
    }
    // "noncreature spells they've cast this turn".
    let (f, _plural, rest) = parse_object_phrase(r)?;
    if is_spell_filter(&f) {
        return player_action(rest, f, false, b);
    }
    None
}

fn is_spell_filter(f: &Filter) -> bool {
    match f {
        Filter::Spell => true,
        Filter::And(v) => v.iter().any(|x| matches!(x, Filter::Spell)),
        _ => false,
    }
}

/// "[player] [have] drawn / discarded / cycled or discarded / cast this turn" after
/// "cards" or "spells".
fn player_action(x: &str, f: Filter, card: bool, b: &Builder) -> Option<(Value, String)> {
    let (who, r) = player_perfect(x, b)?;
    let (cond, r) = if card {
        if let Some(r) = r.strip_prefix("drawn").or_else(|| r.strip_prefix("drew")) {
            (TriggerCond::Draws { who }, r)
        } else if let Some(r) = r
            // Cycling a card discards it (CR 702.29a): a card cycled and discarded is
            // one card.
            .strip_prefix("cycled or discarded")
            .or_else(|| r.strip_prefix("discarded"))
        {
            (TriggerCond::Discards { who, filter: f }, r)
        } else {
            return None;
        }
    } else {
        let r = r.strip_prefix("cast")?;
        let filter = if matches!(f, Filter::Any) {
            Filter::Spell
        } else {
            f
        };
        (TriggerCond::CastSpell { who, filter }, r)
    };
    let rest = this_turn(r)?;
    Some((events(cond, Tally::Events), rest.to_string()))
}

/// "[objects] that died [under your control] this turn", "... put into your graveyard
/// from the battlefield this turn", "... that attacked this turn", "... you attacked with
/// this turn", "... you've sacrificed this turn", "... sacrificed this turn", "... you
/// created this turn": the object phrase before the verb, which must read all of it.
fn object_history(r: &str) -> Option<(Value, String)> {
    let you = || Some(Filter::ControlledBy(PlayerRel::You));
    let yours = || Some(Filter::OwnedBy(PlayerRel::You));
    // CR 700.4: "dies" means is put into a graveyard from the battlefield.
    let markers: [(&str, fn(Filter) -> TriggerCond, Option<Filter>); 12] = [
        (" that died under your control", TriggerCond::Dies, you()),
        (" that died", TriggerCond::Dies, None),
        (
            " that were put into your graveyard from the battlefield",
            TriggerCond::Dies,
            yours(),
        ),
        (
            " put into your graveyard from the battlefield",
            TriggerCond::Dies,
            yours(),
        ),
        (
            " that were put into graveyards from the battlefield",
            TriggerCond::Dies,
            None,
        ),
        (
            " put into graveyards from the battlefield",
            TriggerCond::Dies,
            None,
        ),
        (
            " put into a graveyard from the battlefield",
            TriggerCond::Dies,
            None,
        ),
        (" that attacked", TriggerCond::Attacks, None),
        (" you attacked with", TriggerCond::Attacks, you()),
        (" you've sacrificed", TriggerCond::YouSacrifice, None),
        (" sacrificed", TriggerCond::Sacrificed, None),
        (" you created", TriggerCond::TokenCreated, you()),
    ];
    for (m, mk, extra) in markers {
        let Some(i) = r.find(m) else { continue };
        let Some(rest) = this_turn(&r[i + m.len()..]) else {
            continue;
        };
        let Some((f, _, tail)) = parse_object_phrase(&r[..i]) else {
            continue;
        };
        if !tail.trim().is_empty() {
            continue;
        }
        let f = match extra {
            Some(e) => Filter::and(vec![f, e]),
            None => f,
        };
        return Some((events(mk(f), Tally::Events), rest.to_string()));
    }
    None
}

/// A damage source phrase: "~", "~ or a Dragon", "artifacts", "other sources named ~",
/// "sources they controlled".
fn source_phrase(s: &str) -> Option<Filter> {
    let s = s.trim();
    if s == "~" {
        return Some(Filter::Source);
    }
    if let Some(r) = s.strip_prefix("~ or ") {
        let r = r
            .strip_prefix("a ")
            .or_else(|| r.strip_prefix("an "))
            .unwrap_or(r);
        let (f, _, tail) = parse_object_phrase(r)?;
        if !tail.trim().is_empty() {
            return None;
        }
        return Some(Filter::Or(vec![Filter::Source, f]));
    }
    let (f, _, tail) = parse_object_phrase(s)?;
    tail.trim().is_empty().then_some(f)
}

/// More history counts: "+1/+1 counters you've put on creatures under your control this
/// turn", "[permanents] that entered the battlefield under your control this turn" / "you
/// had enter the battlefield under your control this turn", "cards that were put into
/// [player's] graveyard from [zones] this turn", "times you've cast a commander from the
/// command zone this game", "other spells cast this turn", "creatures you controlled that
/// dealt combat damage to a player this turn".
fn more_history_count(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    // Counters you've put.
    for p in [
        "+1/+1 counters you've put on ",
        "+1/+1 counter you've put on ",
    ] {
        if let Some(x) = r.strip_prefix(p) {
            let i = x.find(" this turn")?;
            let (objs, rest) = (&x[..i], &x[i..]);
            let objs = objs
                .strip_suffix(" under your control")
                .map(|o| (o, true))
                .unwrap_or((objs, false));
            let (f, _, tail) = parse_object_phrase(objs.0)?;
            if !tail.trim().is_empty() {
                return None;
            }
            let f = if objs.1 {
                Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)])
            } else {
                f
            };
            let rest = this_turn(rest)?;
            return Some((
                events(
                    TriggerCond::CountersPutBy {
                        who: PlayerRel::You,
                        on_objects: Some(f),
                        on_players: None,
                        kind: Some("+1/+1".into()),
                        each: false,
                    },
                    Tally::Amount,
                ),
                rest.to_string(),
            ));
        }
    }
    // "the number of times it was kicked" in its own triggered ability ("When ~ enters,
    // it deals damage ... equal to twice the number of times it was kicked").
    for p in [
        "times it was kicked",
        "time it was kicked",
        "times he was kicked",
        "times she was kicked",
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if word_end(rest) && triggers_on(b, &["~ enters", "~ attacks"]) {
                return Some((Value::TimesKicked, rest.to_string()));
            }
        }
    }
    // "for each {S} spent to cast ~" (CR 107.4h).
    for p in ["{s} spent to cast ~", "{S} spent to cast ~"] {
        if let Some(rest) = r.strip_prefix(p) {
            if word_end(rest) {
                return Some((
                    Value::Custom(crate::kw::snow_mana::SNOW_MANA_SPENT.into()),
                    rest.to_string(),
                ));
            }
        }
    }
    // "for each time it has attacked this turn" (the object "it" names: in a static
    // ability, each affected object).
    for p in [
        "times it has attacked this turn",
        "time it has attacked this turn",
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if word_end(rest) && !super::oracle_hardening_referents::is_no_referent(&b.it) {
                let f = Filter::In(Box::new(b.it.clone()));
                return Some((
                    events(TriggerCond::Attacks(f), Tally::Events),
                    rest.to_string(),
                ));
            }
        }
    }
    // Times a commander was cast from the command zone (CR 903.8).
    for p in [
        "times you've cast a commander from the command zone this game",
        "time you've cast a commander from the command zone this game",
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if word_end(rest) {
                return Some((
                    Value::Custom(crate::kw::partner::COMMANDER_CASTS.into()),
                    rest.to_string(),
                ));
            }
        }
    }
    // Spells cast this turn by anyone.
    for (p, other) in [
        ("other spells cast this turn", true),
        ("other spell cast this turn", true),
    ] {
        if let Some(rest) = r.strip_prefix(p) {
            if word_end(rest) {
                let filter = if other {
                    Filter::and(vec![Filter::Other, Filter::Spell])
                } else {
                    Filter::Spell
                };
                return Some((
                    events(
                        TriggerCond::CastSpell {
                            who: PlayerRel::Any,
                            filter,
                        },
                        Tally::Events,
                    ),
                    rest.to_string(),
                ));
            }
        }
    }
    // "[objects] that entered the battlefield under your control this turn".
    for m in [
        " that entered the battlefield under your control",
        " you had enter the battlefield under your control",
        " entered the battlefield under your control",
    ] {
        let Some(i) = r.find(m) else { continue };
        let Some(rest) = this_turn(&r[i + m.len()..]) else {
            continue;
        };
        let noun = &r[..i];
        // "for each other Zombie that entered the battlefield under your control this
        // turn" when a Zombie entering triggered the ability: the others than that one.
        let (noun, other) = match noun.strip_prefix("other ") {
            Some(n) => (n, true),
            None => (noun, false),
        };
        let (f, _, tail) = parse_object_phrase(noun)?;
        if !tail.trim().is_empty() {
            return None;
        }
        let entered = Value::PermanentsEnteredThisTurn(PlayerRef::You, f);
        if other {
            if !(b.in_trigger && triggers_on(b, &["enters"])) {
                return None;
            }
            let v = Value::Max(
                Box::new(Value::Diff(Box::new(entered), Box::new(Value::c(1)))),
                Box::new(Value::c(0)),
            );
            return Some((v, rest.to_string()));
        }
        return Some((entered, rest.to_string()));
    }
    // "cards that were put into target player's graveyard from their library this turn",
    // "cards that were put into your graveyard from your hand or library this turn".
    for p in ["cards that were put into ", "card that was put into "] {
        let Some(x) = r.strip_prefix(p) else { continue };
        let (owner, x) = if let Some(x) = x.strip_prefix("your graveyard from ") {
            (PlayerRel::You, x)
        } else if let Some(y) = x
            .strip_prefix("target player's graveyard from ")
            .or_else(|| x.strip_prefix("target opponent's graveyard from "))
        {
            let pf = if x.starts_with("target player") {
                PlayerFilter::Any
            } else {
                PlayerFilter::Opponent
            };
            let it = b.it.clone();
            let slot = b.add_target(TargetSpec::player(pf, "target player"), "target player");
            b.it = it;
            b.it_player = PlayerRef::Target(slot);
            (PlayerRel::Target(slot), y)
        } else {
            return None;
        };
        let i = x.find(" this turn")?;
        let (zones, rest) = (&x[..i], &x[i..]);
        let zones: Vec<ZoneKind> = match zones {
            "your library" | "their library" => vec![ZoneKind::Library],
            "your hand" | "their hand" => vec![ZoneKind::Hand],
            "your hand or library" | "their hand or library" => {
                vec![ZoneKind::Hand, ZoneKind::Library]
            }
            _ => return None,
        };
        let rest = this_turn(rest)?;
        let conds: Vec<TriggerCond> = zones
            .into_iter()
            .map(|z| TriggerCond::ZoneChange {
                filter: Filter::and(vec![Filter::Card, Filter::OwnedBy(owner)]),
                from: Some(z),
                to: Some(ZoneKind::Graveyard),
            })
            .collect();
        let v = match conds.as_slice() {
            [one] => events(one.clone(), Tally::Events),
            _ => Value::Sum(
                conds
                    .into_iter()
                    .map(|c| events(c, Tally::Events))
                    .collect(),
            ),
        };
        return Some((v, rest.to_string()));
    }
    // "creatures you controlled that dealt combat damage to a player this turn".
    for (m, combat_only) in [
        (" that dealt combat damage to a player", true),
        (" that dealt damage to a player", false),
    ] {
        let Some(i) = r.find(m) else { continue };
        let Some(rest) = this_turn(&r[i + m.len()..]) else {
            continue;
        };
        let noun = &r[..i];
        let (noun, yours) = match noun.strip_suffix(" you controlled") {
            Some(n) => (n, true),
            None => (noun, false),
        };
        let (f, _, tail) = parse_object_phrase(noun)?;
        if !tail.trim().is_empty() {
            return None;
        }
        let source = if yours {
            Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)])
        } else {
            f
        };
        let sel = Sel::ThisTurn(Box::new(TriggerCond::DealsDamage {
            source,
            to: DamageRecipient::Player(PlayerRel::Any),
            combat_only,
        }));
        return Some((Value::CountSel(Box::new(sel)), rest.to_string()));
    }
    None
}

/// The objects of a history phrase as a selection: "spells you've cast this turn",
/// "permanents you've sacrificed this turn".
fn history_objects(s: &str, b: &Builder) -> Option<(Sel, String)> {
    for (p, spells) in [
        ("spells you've cast this turn", true),
        ("permanents you've sacrificed this turn", false),
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if !word_end(rest) {
                return None;
            }
            let cond = if spells {
                TriggerCond::CastSpell {
                    who: PlayerRel::You,
                    filter: Filter::Spell,
                }
            } else {
                TriggerCond::YouSacrifice(Filter::Permanent)
            };
            return Some((Sel::ThisTurn(Box::new(cond)), rest.to_string()));
        }
    }
    let _ = b;
    None
}

/// "card types among spells you've cast this turn", "card types among permanents you've
/// sacrificed this turn", "colors among permanents you control and spells you've cast
/// this turn".
fn kinds_among_history(r: &str, b: &Builder) -> Option<(Value, String)> {
    for (p, among) in [
        ("card types among ", Among::CardTypes),
        ("card type among ", Among::CardTypes),
        ("colors among ", Among::Colors),
        ("color among ", Among::Colors),
    ] {
        let Some(x) = r.strip_prefix(p) else { continue };
        if let Some((sel, rest)) = history_objects(x, b) {
            return Some((Value::DistinctAmong(among, Box::new(sel)), rest));
        }
        // "permanents you control and spells you've cast this turn".
        if let Some((objs, hist)) = x.split_once(" and ") {
            let (f, _, tail) = parse_object_phrase(objs)?;
            if !tail.trim().is_empty() {
                return None;
            }
            let (sel, rest) = history_objects(hist, b)?;
            let sel = Sel::Union(vec![Sel::All(f), sel]);
            return Some((Value::DistinctAmong(among, Box::new(sel)), rest));
        }
    }
    None
}

/// "opponents who lost life this turn", "player who lost life this turn", "opponent who
/// was dealt damage this turn", "opponents who were dealt combat damage this turn".
fn players_who(r: &str, _b: &Builder) -> Option<(Value, String)> {
    let (who, x) = [
        ("opponents who ", PlayerRel::Opponent),
        ("opponent who ", PlayerRel::Opponent),
        ("opponents that ", PlayerRel::Opponent),
        ("opponent that ", PlayerRel::Opponent),
        ("your opponents who ", PlayerRel::Opponent),
        ("your opponents that ", PlayerRel::Opponent),
        ("players who ", PlayerRel::Any),
        ("player who ", PlayerRel::Any),
    ]
    .iter()
    .find_map(|(p, rel)| r.strip_prefix(p).map(|x| (*rel, x)))?;
    // "your opponents who were dealt combat damage by ~ or a Dragon this turn".
    for (p, combat_only) in [
        ("were dealt combat damage by ", true),
        ("was dealt combat damage by ", true),
        ("were dealt damage by ", false),
        ("was dealt damage by ", false),
    ] {
        if let Some(y) = x.strip_prefix(p) {
            let i = y.find(" this turn")?;
            let (src, rest) = (&y[..i], &y[i..]);
            let source = source_phrase(src)?;
            let rest = this_turn(rest)?;
            return Some((
                events(
                    TriggerCond::DealsDamage {
                        source,
                        to: DamageRecipient::Player(who),
                        combat_only,
                    },
                    Tally::Players,
                ),
                rest.to_string(),
            ));
        }
    }
    let (cond, x) = if let Some(x) = x.strip_prefix("discarded a card") {
        (
            TriggerCond::Discards {
                who,
                filter: Filter::Any,
            },
            x,
        )
    } else if let Some(x) = x.strip_prefix("drew a card") {
        (TriggerCond::Draws { who }, x)
    } else if let Some(x) = x.strip_prefix("lost life") {
        (TriggerCond::LosesLife { who }, x)
    } else if let Some(x) = x.strip_prefix("gained life") {
        (TriggerCond::GainsLife { who }, x)
    } else if let Some(x) = x
        .strip_prefix("was dealt combat damage")
        .or_else(|| x.strip_prefix("were dealt combat damage"))
    {
        (
            TriggerCond::PlayerDealtDamage {
                who,
                combat_only: true,
            },
            x,
        )
    } else if let Some(x) = x
        .strip_prefix("was dealt damage")
        .or_else(|| x.strip_prefix("were dealt damage"))
    {
        (
            TriggerCond::PlayerDealtDamage {
                who,
                combat_only: false,
            },
            x,
        )
    } else {
        return None;
    };
    let rest = this_turn(x)?;
    Some((events(cond, Tally::Players), rest.to_string()))
}

/// "2 life your opponents have lost this turn", "1 life you gained this turn" (counted in
/// groups: "for each 2 life ...").
fn life_in_groups(r: &str, b: &Builder) -> Option<(Value, String)> {
    let (n, x) = parse_number(r)?;
    let k = n.as_const().filter(|k| *k > 0)?;
    let x = x.trim_start().strip_prefix("life ")?;
    let (who, x) = player_perfect(x, b)?;
    let (cond, x) = if let Some(x) = x.strip_prefix("lost") {
        (TriggerCond::LosesLife { who }, x)
    } else if let Some(x) = x.strip_prefix("gained") {
        (TriggerCond::GainsLife { who }, x)
    } else {
        return None;
    };
    let rest = this_turn(x)?;
    let total = events(cond, Tally::Amount);
    let v = if k == 1 {
        total
    } else {
        Value::Div(Box::new(total), k, false)
    };
    Some((v, rest.to_string()))
}

/// "the life [player] lost this turn", "the total amount of life your opponents lost this
/// turn", "the total life lost by all players this turn", "the damage dealt to your
/// opponents this turn", "the damage already dealt to that player this turn", "the total
/// amount of noncombat damage dealt to your opponents this turn".
fn history_amount(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    let x = s
        .strip_prefix("the total amount of ")
        .or_else(|| s.strip_prefix("the amount of "))
        .or_else(|| s.strip_prefix("the total "))
        .or_else(|| s.strip_prefix("the "))?;
    // Life.
    if let Some(y) = x.strip_prefix("life ") {
        if let Some(z) = y.strip_prefix("lost by all players") {
            let rest = this_turn(z)?;
            return Some((
                events(
                    TriggerCond::LosesLife {
                        who: PlayerRel::Any,
                    },
                    Tally::Amount,
                ),
                rest.to_string(),
            ));
        }
        let (who, z) = player_perfect(y, b)?;
        let (cond, z) = if let Some(z) = z.strip_prefix("lost") {
            (TriggerCond::LosesLife { who }, z)
        } else if let Some(z) = z.strip_prefix("gained") {
            (TriggerCond::GainsLife { who }, z)
        } else {
            return None;
        };
        let rest = this_turn(z)?;
        return Some((events(cond, Tally::Amount), rest.to_string()));
    }
    // "the greatest number of cards an opponent has drawn this turn".
    for (p, pf) in [
        (
            "greatest number of cards an opponent has drawn this turn",
            PlayerFilter::Opponent,
        ),
        (
            "greatest number of cards a player has drawn this turn",
            PlayerFilter::Any,
        ),
    ] {
        if let Some(rest) = x.strip_prefix(p) {
            if word_end(rest) {
                let v = events(
                    TriggerCond::Draws {
                        who: PlayerRel::Iterated,
                    },
                    Tally::Events,
                );
                return Some((
                    Value::OverPlayers(AggOp::Max, pf, Box::new(v)),
                    rest.to_string(),
                ));
            }
        }
    }
    if let Some(v) = damage_to_objects(x, b) {
        return Some(v);
    }
    // Damage dealt to players.
    let (combat_only, noncombat, y) = if let Some(y) = x.strip_prefix("damage ") {
        (false, false, y)
    } else if let Some(y) = x.strip_prefix("combat damage ") {
        (true, false, y)
    } else if let Some(y) = x.strip_prefix("noncombat damage ") {
        (false, true, y)
    } else {
        return None;
    };
    let y = y.strip_prefix("already ").unwrap_or(y);
    let y = y.strip_prefix("dealt to ")?;
    let (who, z) = match player_subject(y, b) {
        Some(x) => x,
        None => target_player(y, b)?,
    };
    let z = z
        .trim_start()
        .strip_prefix("so far ")
        .unwrap_or(z.trim_start());
    // "the damage dealt to you so far this turn by artifacts".
    if let Some(src) = this_turn(z).and_then(|r| r.strip_prefix(" by ")) {
        if noncombat {
            return None;
        }
        let (src, rest) = match src.find([',', '.']) {
            Some(i) => (&src[..i], &src[i..]),
            None => (src, ""),
        };
        let source = source_phrase(src)?;
        return Some((
            events(
                TriggerCond::DealsDamage {
                    source,
                    to: DamageRecipient::Player(who),
                    combat_only,
                },
                Tally::Amount,
            ),
            rest.to_string(),
        ));
    }
    let rest = this_turn(z)?;
    if noncombat {
        return Some((
            Value::Custom(format!("{NONCOMBAT_DAMAGE_TO}{}", rel_code(who)?).into()),
            rest.to_string(),
        ));
    }
    Some((
        events(
            TriggerCond::PlayerDealtDamage { who, combat_only },
            Tally::Amount,
        ),
        rest.to_string(),
    ))
}

/// "target player" / "target opponent" as the player something was done to: adds the
/// target.
fn target_player<'a>(s: &'a str, b: &mut Builder) -> Option<(PlayerRel, &'a str)> {
    for (p, pf) in [
        ("target opponent", PlayerFilter::Opponent),
        ("target player", PlayerFilter::Any),
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) {
                let it = b.it.clone();
                let slot = b.add_target(TargetSpec::player(pf, p), p);
                b.it = it;
                b.it_player = PlayerRef::Target(slot);
                return Some((PlayerRel::Target(slot), rest));
            }
        }
    }
    None
}

/// "the damage already dealt to it this turn" (the object "it" names), "the amount of
/// damage dealt to ~ this turn by other sources named ~": damage dealt this turn to an
/// object, optionally by sources of a kind.
fn damage_to_objects(x: &str, b: &Builder) -> Option<(Value, String)> {
    let y = x
        .strip_prefix("damage already dealt to ")
        .or_else(|| x.strip_prefix("damage dealt to "))?;
    let (obj, z) = if let Some(z) = y.strip_prefix("~") {
        (Filter::Source, z)
    } else if let Some(z) = y.strip_prefix("it") {
        if super::oracle_hardening_referents::is_no_referent(&b.it) || matches!(b.it, Sel::This) {
            return None;
        }
        (Filter::In(Box::new(b.it.clone())), z)
    } else {
        return None;
    };
    if !word_end(z) {
        return None;
    }
    let after = this_turn(z)?;
    let (source, rest) = match after.strip_prefix(" by ") {
        Some(src) => {
            let (src, rest) = match src.find([',', '.']) {
                Some(i) => (&src[..i], &src[i..]),
                None => (src, ""),
            };
            let src = src.replace("other sources named ~", "other permanents named ~");
            let f = source_phrase(&src).or_else(|| {
                // "sources they controlled": each player in turn.
                matches!(
                    src.as_str(),
                    "sources they controlled" | "sources they control"
                )
                .then_some(Filter::ControlledBy(PlayerRel::Iterated))
            })?;
            (f, rest.to_string())
        }
        None => (Filter::Any, after.to_string()),
    };
    Some((
        events(
            TriggerCond::DealsDamage {
                source,
                to: DamageRecipient::Object(obj),
                combat_only: false,
            },
            Tally::Amount,
        ),
        rest,
    ))
}

/// `Value::Custom` prefix: noncombat damage dealt this turn to the players of a relation
/// (see `kw/value_results.rs`).
pub const NONCOMBAT_DAMAGE_TO: &str = "noncombat damage dealt this turn to:";

fn rel_code(r: PlayerRel) -> Option<&'static str> {
    Some(match r {
        PlayerRel::You => "you",
        PlayerRel::Opponent => "opponents",
        _ => return None,
    })
}

/// The raw text of the face being compiled, lowercased, with its name (or the part of it
/// before a comma, "Shanna" for "Shanna, Purifying Blade") and "this creature" as "~".
fn raw_text_named() -> String {
    let mut raw = crate::oracle::raw_text().to_lowercase();
    let name = crate::oracle::card_name().to_lowercase();
    if !name.is_empty() {
        raw = raw.replace(&name, "~");
        if let Some((short, _)) = name.split_once(", ") {
            raw = raw.replace(short, "~");
        }
    }
    raw.replace("this creature", "~")
}

/// Whether the ability being compiled triggers on an event of this kind ("damage",
/// "gain life", "lose life"), judged by its text.
fn triggers_on(b: &Builder, what: &[&str]) -> bool {
    if !b.in_trigger {
        return false;
    }
    let raw = raw_text_named();
    // The trigger condition: the text before the first comma of a "when"/"whenever"
    // ability (granted abilities in quotes included).
    raw.split(['"', '\n'])
        .filter_map(|part| {
            let i = part.find("whenever ").or_else(|| part.find("when "))?;
            Some(part[i..].split(',').next().unwrap_or(""))
        })
        .any(|cond| what.iter().any(|w| cond.contains(w)))
}

/// The amount of the event that triggered the ability: "the amount of life you gained"
/// in a "whenever you gain life" trigger, "the amount of damage it dealt to that player"
/// in a "whenever ~ deals combat damage to a player" trigger (for "one or more" triggers,
/// the total of the batch, CR 603.2c).
fn event_amount(s: &str, b: &Builder) -> Option<(Value, String)> {
    let life: [(&str, &[&str]); 4] = [
        (
            "the amount of life you gained",
            &["gain life", "gains life"],
        ),
        ("the amount of life you lost", &["lose life", "loses life"]),
        (
            "the amount of life they gained",
            &["gain life", "gains life"],
        ),
        ("the amount of life they lost", &["lose life", "loses life"]),
    ];
    for (p, on) in life {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) && !rest.trim_start().starts_with("this turn") && triggers_on(b, on) {
                return Some((Value::EventAmount, rest.to_string()));
            }
        }
    }
    let x = s
        .strip_prefix("the amount of damage")
        .or_else(|| s.strip_prefix("the damage"))?;
    let damage = ["damage"];
    if !triggers_on(b, &damage) {
        return None;
    }
    let x = x.strip_prefix(" dealt").or_else(|| {
        [
            " it dealt",
            " ~ dealt",
            " he dealt",
            " she dealt",
            " those creatures dealt",
            " that creature dealt",
        ]
        .iter()
        .find_map(|p| x.strip_prefix(p))
    })?;
    let x = [
        " to that player",
        " to them",
        " to it",
        " to that creature",
        " to ~",
    ]
    .iter()
    .find_map(|p| x.strip_prefix(p))
    .unwrap_or(x);
    if !word_end(x) || x.trim_start().starts_with("this turn") {
        return None;
    }
    Some((Value::EventAmount, x.to_string()))
}

/// "for each 2 damage dealt to them" in a damage trigger: the event's amount in groups.
fn event_amount_groups(r: &str, b: &Builder) -> Option<(Value, String)> {
    let (n, x) = parse_number(r)?;
    let k = n.as_const().filter(|k| *k > 0)?;
    let x = x.trim_start().strip_prefix("damage dealt")?;
    let x = [" to that player", " to them"]
        .iter()
        .find_map(|p| x.strip_prefix(p))
        .unwrap_or(x);
    if !word_end(x) || x.trim_start().starts_with("this turn") || !triggers_on(b, &["damage"]) {
        return None;
    }
    let v = if k == 1 {
        Value::EventAmount
    } else {
        Value::Div(Box::new(Value::EventAmount), k, false)
    };
    Some((v, x.to_string()))
}

/// In a "whenever you cast a spell" trigger, amounts about that spell: "the amount of mana
/// spent to cast that spell", "the number of colors of mana spent to cast it" (also "for
/// each color of mana spent to cast that spell"), "the number of times that spell was
/// kicked" (from its last known information if it has left the stack).
fn trigger_spell_value(s: &str, b: &Builder) -> Option<(Value, String)> {
    use crate::kw::value_results::{COLORS_SPENT_ON_THAT_SPELL, TIMES_THAT_SPELL_WAS_KICKED};
    if !triggers_on(b, &[" cast"]) {
        return None;
    }
    let spell_ref = |r: &str| -> Option<String> {
        let rest = r.strip_prefix("that spell").or_else(|| {
            r.strip_prefix("it")
                .filter(|_| matches!(b.it, Sel::TriggerSpell | Sel::TriggerObject))
        })?;
        word_end(rest).then(|| rest.to_string())
    };
    if let Some(r) = s.strip_prefix("the amount of mana spent to cast ") {
        let rest = spell_ref(r)?;
        return Some((
            Value::Custom(crate::kw::opus::MANA_SPENT_ON_THAT_SPELL.into()),
            rest,
        ));
    }
    if let Some(r) = s
        .strip_prefix("the number of colors of mana spent to cast ")
        .or_else(|| s.strip_prefix("the number of color of mana spent to cast "))
    {
        let rest = spell_ref(r)?;
        return Some((Value::Custom(COLORS_SPENT_ON_THAT_SPELL.into()), rest));
    }
    for p in [
        "the number of times that spell was kicked",
        "the number of time that spell was kicked",
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) {
                return Some((
                    Value::Custom(TIMES_THAT_SPELL_WAS_KICKED.into()),
                    rest.to_string(),
                ));
            }
        }
    }
    None
}

/// "the excess damage dealt this way", "the amount of excess damage dealt to that
/// creature this way" (CR 120.10: the excess damage the latest damage instruction dealt,
/// to the one permanent it damaged), and in a "whenever [a permanent] is dealt excess
/// damage" trigger "that excess damage" (the triggering event's).
fn excess(s: &str, b: &Builder) -> Option<(Value, String)> {
    if let Some(rest) = s.strip_prefix("that excess damage") {
        if word_end(rest) && b.in_trigger {
            return Some((Value::EventAmount, rest.to_string()));
        }
        return None;
    }
    let x = s
        .strip_prefix("the amount of excess damage dealt")
        .or_else(|| s.strip_prefix("the excess damage dealt"))?;
    let x = [" to that creature", " to that permanent", " to it"]
        .iter()
        .find_map(|p| x.strip_prefix(p))
        .unwrap_or(x);
    let rest = x.strip_prefix(" this way")?;
    word_end(rest).then(|| (Value::Var(vars::EXCESS), rest.to_string()))
}

/// "for each 1 life lost this way" after a life-loss instruction: the life it made players
/// lose in total (CR 119.3), counted in groups.
fn life_lost_this_way(r: &str) -> Option<(Value, String)> {
    let (n, x) = parse_number(r)?;
    let k = n.as_const().filter(|k| *k > 0)?;
    let rest = x.trim_start().strip_prefix("life lost this way")?;
    if !word_end(rest) {
        return None;
    }
    let v = if k == 1 {
        Value::Prev
    } else {
        Value::Div(Box::new(Value::Prev), k, false)
    };
    Some((v, rest.to_string()))
}

/// "If no life is lost this way, ..." after a life-loss instruction.
fn no_life_lost_this_way(c: &str) -> Option<Condition> {
    matches!(
        end(c),
        "no life is lost this way" | "no life was lost this way"
    )
    .then(|| Condition::Compare(Value::Prev, Cmp::Eq, Value::c(0)))
}

inventory::submit! { super::ConditionPattern { name: "value results: no life is lost this way", priority: 100, parse: no_life_lost_this_way } }

// ---------------------------------------------------------------------------
// Results of earlier instructions ("this way", "the sacrificed creature")
// ---------------------------------------------------------------------------

/// The relation "they"/"their"/"that player" names, if any.
fn their_rel(b: &Builder) -> Option<PlayerRel> {
    match &b.it_player {
        PlayerRef::TriggerPlayer => Some(PlayerRel::TriggerPlayer),
        PlayerRef::Target(t) => Some(PlayerRel::Target(*t)),
        PlayerRef::Iterated => Some(PlayerRel::Iterated),
        PlayerRef::You => Some(PlayerRel::You),
        _ => None,
    }
}

/// "[objects] [verb] this way": the objects an earlier instruction of the ability acted
/// on (CR 608.2c), as the instruction left them: the cards exiled, discarded, milled,
/// returned or countered (as the new objects they became, CR 400.7), the permanents
/// destroyed, sacrificed or tapped (as they last existed on the battlefield). Qualifiers:
/// "creatures you controlled that were destroyed this way", "cards discarded this way",
/// "land cards put into their graveyard this way".
pub fn this_way_sel(r: &str, b: &Builder) -> Option<(Sel, String)> {
    use crate::discard_rules::DISCARDED;
    use crate::kw::value_results::DESTROYED;
    let i = r.find(" this way")?;
    let rest = &r[i + " this way".len()..];
    if !word_end(rest) {
        return None;
    }
    let head = &r[..i];
    let yours = Some(Filter::OwnedBy(PlayerRel::You));
    let theirs = their_rel(b).map(Filter::OwnedBy);
    let verbs: [(&str, Var, Option<Filter>); 18] = [
        (" destroyed", DESTROYED, None),
        (" sacrificed", vars::SACRIFICED, None),
        (" exiled", vars::IT, None),
        (" discarded", DISCARDED, None),
        (" milled", vars::IT, None),
        (" put into your graveyard", vars::IT, yours.clone()),
        (" put into their graveyard", vars::IT, theirs),
        (" put into a graveyard", vars::IT, None),
        (" put into graveyards", vars::IT, None),
        (" returned to your hand", vars::IT, yours),
        (" returned to its owner's hand", vars::IT, None),
        (" returned to their owner's hand", vars::IT, None),
        (" returned to their owners' hands", vars::IT, None),
        (" returned", vars::IT, None),
        (" countered", vars::IT, None),
        (" tapped", vars::TAPPED, None),
        (" drawn", vars::REVEALED, None),
        (" revealed", vars::REVEALED, None),
    ];
    // "for each card discarded this way" in a "whenever you discard one or more cards"
    // trigger, "the number of nonland cards milled this way" in a "whenever one or more
    // nonland cards are milled" trigger: the objects of the triggering batch.
    let batch_verb = [
        (" discarded", "discard"),
        (" milled", "mill"),
        (" exiled", "exile"),
        (" sacrificed", "sacrifice"),
    ]
    .into_iter()
    .find(|(v, w)| head.ends_with(v) && triggers_on(b, &[w]));
    if let Some((v, _)) = batch_verb {
        let noun = head.strip_suffix(v)?;
        let (f, _, tail) = parse_object_phrase(noun)?;
        if !tail.trim().is_empty() {
            return None;
        }
        return Some((
            Sel::Matching(Box::new(Sel::TriggerObjects), f),
            rest.to_string(),
        ));
    }
    let (noun, mut var, extra) = verbs
        .iter()
        .find_map(|(v, var, extra)| head.strip_suffix(v).map(|n| (n, *var, extra.clone())))?;
    let noun = noun
        .strip_suffix(" that were")
        .or_else(|| noun.strip_suffix(" that was"))
        .unwrap_or(noun);
    let mut parts = vec![];
    // "creatures you controlled that were destroyed this way", "artifacts they controlled
    // that were put into a graveyard this way": who controlled them as they last existed
    // on the battlefield.
    let mut noun = noun;
    for (p, rel) in [
        (" you controlled", Some(PlayerRel::You)),
        (" they controlled", their_rel(b)),
        (" that player controlled", their_rel(b)),
    ] {
        if let Some(n) = noun.strip_suffix(p) {
            parts.push(Filter::ControlledBy(rel?));
            noun = n;
            if var == vars::IT && head.contains(" put into ") {
                var = DESTROYED;
            }
            if var != DESTROYED && var != vars::SACRIFICED && var != vars::TAPPED {
                return None;
            }
        }
    }
    let (f, _, tail) = parse_object_phrase(noun)?;
    if !tail.trim().is_empty() {
        return None;
    }
    parts.insert(0, f);
    parts.extend(extra);
    let f = Filter::and(parts);
    // The cards exiled, discarded, milled, returned or countered are new objects (CR
    // 400.7); the noun says what they were ("the permanent exiled this way", "spells
    // countered this way"), and their characteristics are those they last had (CR
    // 608.2h). The destroyed, sacrificed or tapped permanents are kept as they were.
    let from = if [vars::IT, DISCARDED].contains(&var) {
        Sel::Before(Box::new(Sel::Var(var)))
    } else if head.ends_with(" revealed") {
        // The cards a reveal instruction revealed: a whole hand, or the cards revealed
        // until one was found (that one included).
        Sel::Union(vec![Sel::Var(vars::REVEALED), Sel::Var(vars::DUG_FOUND)])
    } else {
        Sel::Var(var)
    };
    Some((Sel::Matching(Box::new(from), f), rest.to_string()))
}

/// Whether "for each [s]" counts objects an earlier instruction acted on, read by
/// [`this_way_sel`] as a whole.
pub fn reads_this_way(s: &str, b: &Builder) -> bool {
    let s = end(s);
    if life_lost_this_way(s).is_some_and(|(_, rest)| rest.trim().is_empty()) {
        return true;
    }
    let s = s
        .strip_prefix("card types among ")
        .or_else(|| s.strip_prefix("card type among "))
        .unwrap_or(s);
    this_way_sel(s, b).is_some_and(|(_, rest)| rest.trim().is_empty())
}

/// Whether a whole value phrase is about the results of earlier instructions ("the
/// greatest number of cards a player discarded this way", "the number of creatures
/// destroyed this way"), as this grammar reads it.
pub fn reads_result_value(s: &str, b: &mut Builder) -> bool {
    let s = end(s);
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let v = match s.strip_prefix("the number of ") {
        Some(r) => this_way_count(r, b),
        None => result_value(s, b),
    };
    b.targets.truncate(saved.0);
    b.it = saved.1;
    b.it_player = saved.2;
    v.is_some_and(|(_, rest)| rest.trim().is_empty())
}

/// "the number of [objects] [verb] this way", "card types among cards discarded this way",
/// "the greatest number of cards a player discarded this way".
fn this_way_count(r: &str, b: &mut Builder) -> Option<(Value, String)> {
    for p in ["card types among ", "card type among "] {
        if let Some(x) = r.strip_prefix(p) {
            let (sel, rest) = this_way_sel(x, b).or_else(|| result_ref(x, b))?;
            return Some((Value::DistinctAmong(Among::CardTypes, Box::new(sel)), rest));
        }
    }
    let (sel, rest) = this_way_sel(r, b)?;
    Some((Value::CountSel(Box::new(sel)), rest))
}

/// An object (or objects) a result phrase names: "the sacrificed creature(s)", "the
/// discarded card(s)", "[the] [objects] [verb] this way".
fn result_ref(s: &str, b: &Builder) -> Option<(Sel, String)> {
    for (p, var) in [
        ("the sacrificed creatures", vars::SACRIFICED),
        ("the sacrificed creature", vars::SACRIFICED),
        ("the sacrificed artifacts", vars::SACRIFICED),
        ("the sacrificed artifact", vars::SACRIFICED),
        ("the sacrificed permanents", vars::SACRIFICED),
        ("the sacrificed permanent", vars::SACRIFICED),
        ("the sacrificed land", vars::SACRIFICED),
        ("the sacrificed enchantment", vars::SACRIFICED),
        ("the discarded cards", crate::discard_rules::DISCARDED),
        ("the discarded card", crate::discard_rules::DISCARDED),
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) || rest.starts_with('\'') {
                return Some((Sel::Var(var), rest.to_string()));
            }
        }
    }
    // "those creatures" in a "whenever one or more creatures ..." trigger: the batch's
    // objects (CR 603.2c).
    for p in ["those creatures", "those cards"] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest)
                && b.in_trigger
                && crate::oracle::raw_text()
                    .to_lowercase()
                    .contains("one or more")
            {
                return Some((Sel::TriggerObjects, rest.to_string()));
            }
        }
    }
    // "the creature that died" in a dies trigger: as it last existed (CR 603.10a).
    if let Some(rest) = s.strip_prefix("the creature that died") {
        if word_end(rest) && triggers_on(b, &["dies", "die"]) {
            return Some((Sel::TriggerLki, rest.to_string()));
        }
    }
    // "the exiled card", "the card you exiled": the card the cost exiled (CR 400.7j), or
    // that an earlier instruction of the ability exiled, or else the card exiled with the
    // source by its linked ability (CR 607.2a).
    for p in [
        "the exiled card",
        "the card you exiled",
        "the exiled creature card",
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) || rest.starts_with('\'') {
                let sel = Sel::Union(vec![
                    Sel::Var(crate::zones::COST_EXILED),
                    Sel::Matching(
                        Box::new(Sel::Var(vars::IT)),
                        Filter::InZone(ZoneKind::Exile),
                    ),
                    Sel::All(Filter::and(vec![
                        Filter::In(Box::new(Sel::Linked)),
                        Filter::InZone(ZoneKind::Exile),
                    ])),
                ]);
                return Some((sel, rest.to_string()));
            }
        }
    }
    // "the revealed card" of a spell whose additional cost reveals a card from your hand.
    if let Some(rest) = s.strip_prefix("the revealed card") {
        let raw = crate::oracle::raw_text().to_lowercase();
        if (word_end(rest) || rest.starts_with('\''))
            && raw.contains("as an additional cost to cast this spell, reveal ")
        {
            return Some((Sel::Var(crate::zones::COST_REVEALED), rest.to_string()));
        }
    }
    // "the tapped creature": the creature tapped to pay the cost.
    if let Some(rest) = s.strip_prefix("the tapped creature") {
        if word_end(rest) || rest.starts_with('\'') {
            return Some((Sel::Var(vars::TAPPED), rest.to_string()));
        }
    }
    let x = s.strip_prefix("the ").unwrap_or(s);
    this_way_sel(x, b)
}

/// "~'s loyalty" (as it last existed, if it has left the battlefield), "three times ~'s
/// power", "the difference between its power and toughness" (the smaller subtracted from
/// the larger), "the number of creatures you control in excess of the number of
/// creatures target opponent controls".
fn stat_extras(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    // "the amount of {E} you have" (CR 107.14, 122.1).
    for p in ["the amount of {e} you have", "the amount of {E} you have"] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) {
                return Some((
                    Value::PlayerCounters(PlayerRef::You, "energy".into()),
                    rest.to_string(),
                ));
            }
        }
    }
    // "the amount of mana spent to cast her" (a character's pronoun: the card itself).
    for p in [
        "the amount of mana spent to cast her",
        "the amount of mana spent to cast him",
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            if word_end(rest) && !b.ctx.is_spell() {
                return Some((Value::ManaSpent, rest.to_string()));
            }
        }
    }
    // "the mana value of a commander you own on the battlefield or in the command zone"
    // (CR 903.3): the greatest, if there are several.
    if let Some(rest) = s.strip_prefix(
        "the mana value of a commander you own on the battlefield or in the command zone",
    ) {
        if word_end(rest) {
            return super::value_grammar::parse_value(
                &format!(
                    "the greatest mana value of a commander you own on the battlefield or in the command zone{rest}"
                ),
                b,
            );
        }
    }
    if let Some(rest) = s.strip_prefix("~'s loyalty") {
        if word_end(rest) {
            return Some((Value::LoyaltyOf(Box::new(Sel::This)), rest.to_string()));
        }
    }
    if let Some(r) = s.strip_prefix("three times ") {
        let (v, rest) = super::value_grammar::parse_value(r, b)?;
        return Some((Value::Mul(Box::new(Value::c(3)), Box::new(v)), rest));
    }
    if let Some(r) = s.strip_prefix("the difference between ") {
        let (pw, tail) = r.split_once(" and ")?;
        let p = super::value_grammar::parse_value(&format!("{pw}"), b)?;
        if !p.1.trim().is_empty() {
            return None;
        }
        // "its toughness", or just "toughness" (of the same object).
        let (t, rest) = match tail.strip_prefix("toughness") {
            Some(rest) => {
                let Value::PowerOf(sel) = &p.0 else {
                    return None;
                };
                (Value::ToughnessOf(sel.clone()), rest.to_string())
            }
            None => super::value_grammar::parse_value(tail, b)?,
        };
        if !matches!((&p.0, &t), (Value::PowerOf(_), Value::ToughnessOf(_))) {
            return None;
        }
        let d = |a: &Value, b: &Value| Value::Diff(Box::new(a.clone()), Box::new(b.clone()));
        return Some((
            Value::Max(Box::new(d(&p.0, &t)), Box::new(d(&t, &p.0))),
            rest,
        ));
    }
    if let Some(r) = s.strip_prefix("the number of ") {
        if let Some((a, c)) = r.split_once(" in excess of ") {
            let (va, ta) = super::value_grammar::parse_value(&format!("the number of {a}"), b)?;
            if !ta.trim().is_empty() {
                return None;
            }
            let (vc, rest) = super::value_grammar::parse_value(c, b)?;
            let v = Value::Max(
                Box::new(Value::Diff(Box::new(va), Box::new(vc))),
                Box::new(Value::c(0)),
            );
            return Some((v, rest));
        }
    }
    None
}

/// Amounts about the results of earlier instructions: "the sacrificed creature's power",
/// "the total power of the creatures sacrificed this way", "the greatest mana value among
/// cards discarded this way", "the mana value of the permanent exiled this way", "the
/// number of red mana symbols in the sacrificed creature's mana cost", "the number of
/// card types the discarded card has", "the greatest number of cards a player discarded
/// this way".
fn result_value(s: &str, b: &mut Builder) -> Option<(Value, String)> {
    use super::value_grammar::{of_referent, stat_word};
    // "the greatest number of cards a player discarded this way" (Windfall): the most
    // any one player discarded.
    for (p, pf) in [
        (
            "the greatest number of cards a player discarded this way",
            PlayerFilter::Any,
        ),
        (
            "the greatest number of cards an opponent discarded this way",
            PlayerFilter::Opponent,
        ),
    ] {
        if let Some(rest) = s.strip_prefix(p) {
            // The discarded cards are in their owners' graveyards (or wherever a
            // replacement effect put them): each player's own.
            let theirs = Sel::Matching(
                Box::new(Sel::Var(crate::discard_rules::DISCARDED)),
                Filter::OwnedBy(PlayerRel::Iterated),
            );
            return Some((
                Value::OverPlayers(AggOp::Max, pf, Box::new(Value::CountSel(Box::new(theirs)))),
                rest.to_string(),
            ));
        }
    }
    for (p, op) in [
        ("the total ", AggOp::Sum),
        ("the greatest ", AggOp::Max),
        ("the highest ", AggOp::Max),
        ("the least ", AggOp::Min),
        ("the lowest ", AggOp::Min),
    ] {
        if let Some(x) = s.strip_prefix(p) {
            let Some((stat, y)) = stat_word(x) else {
                continue;
            };
            let link = if op == AggOp::Sum { " of " } else { " among " };
            let Some(y) = y.strip_prefix(link) else {
                continue;
            };
            let Some((sel, rest)) = result_ref(y, b) else {
                continue;
            };
            return Some((Value::Aggregate(op, stat, Box::new(sel)), rest));
        }
    }
    // "the power of the card returned this way", "the mana value of the sacrificed
    // artifact".
    if let Some(x) = s.strip_prefix("the ") {
        if let Some((stat, y)) = stat_word(x) {
            if let Some(y) = y.strip_prefix(" of ") {
                if let Some((sel, rest)) = result_ref(y, b) {
                    return Some((of_referent(stat, sel), rest));
                }
            }
        }
    }
    // "the number of red mana symbols in the sacrificed creature's mana cost" (CR 107.4e:
    // hybrid symbols of the color count).
    if let Some(x) = s.strip_prefix("the number of ") {
        let (w, y) = split_word(x);
        if let Some(color) = crate::types::Color::from_word(w) {
            if let Some(y) = y.strip_prefix("mana symbols in ") {
                if let Some((sel, r)) = result_ref(y, b) {
                    if let Some(rest) = r.strip_prefix("'s mana cost") {
                        return Some((
                            Value::Aggregate(AggOp::Sum, Stat::ManaSymbols(color), Box::new(sel)),
                            rest.to_string(),
                        ));
                    }
                }
            }
        }
        // "the number of card types the discarded card has".
        if let Some(y) = x.strip_prefix("card types ") {
            if let Some((sel, r)) = result_ref(y, b) {
                if let Some(rest) = r.strip_prefix(" has").filter(|r| word_end(r)) {
                    return Some((
                        Value::DistinctAmong(Among::CardTypes, Box::new(sel)),
                        rest.to_string(),
                    ));
                }
            }
        }
    }
    if let Some(v) = stat_extras(s, b) {
        return Some(v);
    }
    // "the sacrificed creature's power", "the discarded card's mana value".
    let (sel, r) = result_ref(s, b)?;
    let r = r.strip_prefix("'s ")?;
    let (stat, rest) = stat_word(r)?;
    if !word_end(rest) {
        return None;
    }
    // "... equal to the sacrificed creature's power, then ... equal to its toughness".
    if matches!(sel, Sel::Var(v) if v == vars::SACRIFICED) {
        b.it = sel.clone();
    }
    Some((of_referent(stat, sel), rest.to_string()))
}

/// An activated ability whose cost sacrifices one permanent other than its source:
/// "that creature" ("that artifact", ...) in its effect is the sacrificed permanent (its
/// last known information, CR 608.2h), which the effect calls "the sacrificed [noun]".
pub fn cost_sacrificed_text(cost: &Cost, effect: &str) -> Option<String> {
    let sacs: Vec<&CostPart> = cost
        .parts
        .iter()
        .filter(|p| matches!(p, CostPart::Sacrifice { .. }))
        .collect();
    let [CostPart::Sacrifice {
        count: Value::Const(1),
        ..
    }] = sacs.as_slice()
    else {
        return None;
    };
    let mut text = effect.to_string();
    let mut changed = false;
    for noun in ["creature", "artifact", "permanent", "land", "enchantment"] {
        for (that, the) in [("that", "the"), ("That", "The")] {
            let from = format!("{that} {noun}");
            if text.contains(&format!("{from}'s")) {
                text = text.replace(&format!("{from}'s"), &format!("{the} sacrificed {noun}'s"));
                changed = true;
            }
        }
    }
    changed.then_some(text)
}

/// "Sacrifice a creature. You gain life equal to that creature's toughness.", "you may
/// sacrifice another creature. If you do, ... where X is that creature's power": after one
/// player sacrifices one permanent, "that creature" (as it last existed) is the sacrificed
/// one, until an instruction names a target (which "that creature" would mean then).
pub fn note_sacrificed(e: &Effect, b: &mut Builder) {
    let ours = |sel: &Sel| matches!(sel, Sel::Var(v) if *v == vars::SACRIFICED);
    let single = |who: &PlayerRef| {
        !matches!(
            who,
            PlayerRef::EachOpponent | PlayerRef::EachPlayer | PlayerRef::EachOtherPlayer
        )
    };
    let sacrifices_one = match e {
        Effect::Sacrifice { who, count, .. } => single(who) && matches!(count, Value::Const(1)),
        _ => false,
    };
    if sacrifices_one {
        b.named
            .retain(|(n, sel)| !(n == "that creature" && ours(sel)));
        b.named
            .push(("that creature".to_string(), Sel::Var(vars::SACRIFICED)));
        return;
    }
    let names_target = serde_json::to_string(e).is_ok_and(|j| j.contains("\"Target\""));
    if names_target {
        b.named
            .retain(|(n, sel)| !(n == "that creature" && ours(sel)));
    }
}

/// "Target player discards a card. ~ deals damage to that player equal to that card's
/// mana value.": after one player discards one card, "that card" / "it" is the discarded
/// card (the new object it became, CR 400.7j).
pub fn note_discarded(e: &Effect, b: &mut Builder) {
    let single = |who: &PlayerRef| {
        !matches!(
            who,
            PlayerRef::EachOpponent | PlayerRef::EachPlayer | PlayerRef::EachOtherPlayer
        )
    };
    let one = match e {
        Effect::Discard { who, n, .. } => single(who) && matches!(n, Value::Const(1)),
        Effect::AsPlayer { who, effect } => {
            single(who)
                && matches!(
                    &**effect,
                    Effect::Discard {
                        who: PlayerRef::You,
                        n: Value::Const(1),
                        ..
                    }
                )
        }
        _ => false,
    };
    // "that card" (not "it", which may well be the source: "unless her additional cost
    // was paid"), until the next instruction has been read.
    let discarded = |sel: &Sel| matches!(sel, Sel::Var(v) if *v == crate::discard_rules::DISCARDED);
    b.named
        .retain(|(n, sel)| !(n == "that card" && discarded(sel)));
    if one {
        b.named.retain(|(n, _)| n != "that card");
        b.named.push((
            "that card".to_string(),
            Sel::Var(crate::discard_rules::DISCARDED),
        ));
    }
}

/// "Target player discards a number of cards equal to [value]", "mill a number of cards
/// equal to [value]": the same as "cards equal to [value]".
fn a_number_of_cards(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" a number of cards equal to ")?;
    let (head, tail) = (&l[..i], &l[i + " a number of cards equal to ".len()..]);
    let verb = head.rsplit(' ').next()?;
    if !matches!(
        verb,
        "discard" | "discards" | "mill" | "mills" | "draw" | "draws"
    ) {
        return None;
    }
    crate::oracle::effects::parse_clause(&format!("{head} cards equal to {tail}"), b)
}

inventory::submit! { super::EffectPattern { name: "value results: [verb] a number of cards equal to [value]", priority: 400, parse: a_number_of_cards } }

/// "Each player discards their hand, then draws cards equal to the greatest number of
/// cards a player discarded this way." (Windfall): every player discards, and then every
/// player draws the same number of cards, determined once all the discarding is done (CR
/// 608.2c, 101.4); not each player in turn.
fn each_discards_then_draws(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = [(
        "each player discards their hand, then draws cards equal to ",
        PlayerRef::EachPlayer,
    )]
    .into_iter()
    .find_map(|(p, w)| l.strip_prefix(p).map(|r| (w, r)))?;
    if !reads_result_value(r, b) {
        return None;
    }
    let (v, rest) = result_value(r, b)?;
    if !rest.trim().is_empty() {
        return None;
    }
    Some(Effect::seq(vec![
        Effect::DiscardHand { who: who.clone() },
        Effect::Draw { who, n: v },
    ]))
}

inventory::submit! { super::EffectPattern { name: "value results: each player discards their hand, then draws cards equal to [result]", priority: 5, parse: each_discards_then_draws } }

// ---------------------------------------------------------------------------
// "that many" after an instruction with an amount
// ---------------------------------------------------------------------------

/// The numeric variable holding the amount an instruction named, for a later "that many".
const NAMED_AMOUNT: Var = vars::USER + 7523;

/// The amount the last instruction of `e` names, if it's one that names an amount of
/// cards ("draw two cards", "draws cards equal to ...", "mill X cards").
fn named_amount(e: &mut Effect) -> Option<&mut Value> {
    match e {
        Effect::Draw { n, .. } | Effect::Mill { n, .. } | Effect::Discard { n, .. } => Some(n),
        Effect::AsPlayer { effect, .. } | Effect::May { effect, .. } => named_amount(effect),
        Effect::Seq(v) => v.last_mut().and_then(named_amount),
        _ => None,
    }
}

/// Whether the last instruction of `e` moves objects ("exile all creatures you control",
/// "return up to three target land cards ..."), recording them as `vars::IT`.
fn ends_with_move(e: &Effect) -> bool {
    match e {
        Effect::Exile { .. } | Effect::Move { .. } | Effect::Search { .. } => true,
        Effect::Seq(v) => v
            .iter()
            .rev()
            .find(|x| !matches!(x, Effect::Store { .. }))
            .is_some_and(ends_with_move),
        _ => false,
    }
}

/// `e`'s named amount kept in [`NAMED_AMOUNT`] (determined once, CR 608.2h), and `then`
/// read with "that many" as that amount. After a move, "that many" is the number of
/// objects it moved.
fn with_that_many(mut e: Effect, then_text: &str, b: &mut Builder) -> Option<(Effect, Effect)> {
    if then_text
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| w == "x")
    {
        return None;
    }
    if ends_with_move(&e) {
        let moved = Value::CountSel(Box::new(Sel::Var(vars::IT)));
        let then =
            crate::oracle::effects::parse_clause(&then_text.replacen("that many", "x", 1), b)?;
        let then = super::r107_numbers::substitute_x(&then, &Value::Var(NAMED_AMOUNT))?;
        let e = Effect::seq(vec![
            e,
            Effect::StoreValue {
                var: NAMED_AMOUNT,
                value: moved,
            },
        ]);
        return Some((e, then));
    }
    let n = named_amount(&mut e)?;
    let amount = std::mem::replace(n, Value::Var(NAMED_AMOUNT));
    let then = crate::oracle::effects::parse_clause(&then_text.replacen("that many", "x", 1), b)?;
    let then = super::r107_numbers::substitute_x(&then, &Value::Var(NAMED_AMOUNT))?;
    let e = Effect::seq(vec![
        Effect::StoreValue {
            var: NAMED_AMOUNT,
            value: amount,
        },
        e,
    ]);
    Some((e, then))
}

/// "Target player draws two cards, then discards that many cards.", "draw cards equal to
/// the number of cards in your hand, then discard that many cards": "that many" is the
/// amount the first instruction named (Horrid Shadowspinner's ruling: not the number of
/// cards actually drawn).
fn then_that_many(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (first, second) = l.split_once(", then ")?;
    if !second.contains("that many") || first.contains("that many") {
        return None;
    }
    // The subject of the first clause does the second ("target player draws ..., then
    // discards ...").
    let subject = [
        "target player ",
        "target opponent ",
        "each player ",
        "each opponent ",
    ]
    .iter()
    .find(|p| first.starts_with(**p));
    let e = crate::oracle::effects::parse_clause(first, b)?;
    if named_amount(&mut e.clone()).is_none() && !ends_with_move(&e) {
        return None;
    }
    let second = match subject {
        Some(p) if p.starts_with("target") => format!("that player {second}"),
        Some(_) => return None,
        None => second.to_string(),
    };
    let (e, then) = with_that_many(e, &second, b)?;
    Some(Effect::seq(vec![e, then]))
}

inventory::submit! { super::EffectPattern { name: "value results: [draw N], then [instruction with that many]", priority: 60, parse: then_that_many } }

/// "You may draw cards equal to its power. If you do, discard that many cards.", "Draw two
/// cards. Then discard that many cards.": "that many" is the amount the previous
/// instruction named.
fn that_many_followup(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let (r, conditional) = if let Some(r) = l.strip_prefix("if you do, ") {
        (r, true)
    } else if let Some(r) = l.strip_prefix("then ") {
        (r, false)
    } else if ends_with_move(prev) {
        // "Create that many ...", "That player may search their library for that many
        // basic land cards ...".
        (l, false)
    } else {
        return false;
    };
    if !r.contains("that many") {
        return false;
    }
    let optional = matches!(last_of(prev), Effect::May { .. });
    if conditional != optional {
        return false;
    }
    let old = std::mem::take(prev);
    let Some((e, then)) = with_that_many(old.clone(), r, b) else {
        *prev = old;
        return false;
    };
    let then = if conditional {
        Effect::If {
            cond: Condition::PrevHappened,
            then: Box::new(then),
            otherwise: Box::new(Effect::Noop),
        }
    } else {
        then
    };
    *prev = Effect::seq(vec![e, then]);
    true
}

fn last_of(e: &Effect) -> &Effect {
    match e {
        Effect::Seq(v) => v.last().map_or(e, last_of),
        _ => e,
    }
}

inventory::submit! { super::FollowupPattern { name: "value results: if you do / then, [instruction with that many]", priority: 45, apply: that_many_followup } }

// ---------------------------------------------------------------------------
// Counters in amounts that depend on each object or player
// ---------------------------------------------------------------------------

/// The object each instruction of a [`Effect::ForEach`] is about.
const EACH_OBJECT: Var = vars::USER + 7524;

/// "Put a number of +1/+1 counters on each other creature you control equal to that
/// creature's toughness." (Canopy Gargantuan): each object gets its own number (CR
/// 608.2h), all at once (CR 608.2f).
fn counters_each_equal_to_its(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("put a number of ")?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r.trim_start().strip_prefix("counters on each ")?;
    let (objs, stat) = r.split_once(" equal to ")?;
    let (f, _, tail) = parse_object_phrase(objs)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let st = ["that creature's ", "that permanent's ", "its "]
        .iter()
        .find_map(|p| stat.strip_prefix(p))?;
    let (stat, rest) = super::value_grammar::stat_word(st)?;
    if !rest.trim().is_empty() {
        return None;
    }
    let each = Sel::Var(EACH_OBJECT);
    Some(Effect::ForEach {
        sel: Sel::All(f),
        var: EACH_OBJECT,
        effect: Box::new(Effect::AddCounters {
            what: each.clone(),
            kind,
            n: super::value_grammar::of_referent(stat, each),
        }),
    })
}

inventory::submit! { super::EffectPattern { name: "value results: put a number of counters on each [object] equal to its [stat]", priority: 60, parse: counters_each_equal_to_its } }

/// "Each opponent gets a number of rad counters equal to its power." (Feral Ghoul):
/// players get counters (CR 122.1).
fn players_get_counters_equal_to(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = [
        ("each opponent gets a number of ", PlayerRef::EachOpponent),
        ("each player gets a number of ", PlayerRef::EachPlayer),
        ("you get a number of ", PlayerRef::You),
    ]
    .into_iter()
    .find_map(|(p, w)| l.strip_prefix(p).map(|r| (w, r)))?;
    let (kind, r) = crate::oracle::costs::counter_kind(r)?;
    let r = r.trim_start().strip_prefix("counters equal to ")?;
    let (n, tail) = super::r107_numbers::value_phrase(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::AddPlayerCounters { who, kind, n })
}

inventory::submit! { super::EffectPattern { name: "value results: players get a number of counters equal to [value]", priority: 60, parse: players_get_counters_equal_to } }

// ---------------------------------------------------------------------------
// X fixed by a payment ("you may pay {X}")
// ---------------------------------------------------------------------------

/// The cap on the X a player chooses as they pay a cost with {X} while an ability
/// resolves ("X can't be greater than the amount of life you gained this turn"), read by
/// `mana_abilities::bind_x_for_payment`.
pub const X_MAX: Var = vars::USER + 7521;

/// A cost of mana symbols with {X} in it ("{X}", "{X}{X}", "{X}{R}").
fn x_mana_cost(s: &str) -> Option<Cost> {
    let s = end(s);
    let only_symbols = s.starts_with('{')
        && s.ends_with('}')
        && s.split('}')
            .all(|p| p.is_empty() || (p.starts_with('{') && !p[1..].contains('{')));
    if !only_symbols || !s.contains("{x}") {
        return None;
    }
    let m = crate::mana::ManaCost::parse(&s.to_uppercase())?;
    m.has_x().then(|| Cost::mana(m))
}

/// "You may pay {X}." (CR 107.3f: nothing defines X, so the player chooses it as they
/// pay, and that's the value of X for the rest of the resolution, including a reflexive
/// triggered ability it causes, CR 603.12); "you may pay {X}, where X is less than or
/// equal to the amount of life you gained" (the choice is capped).
fn may_pay_x(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you may pay ")?;
    let (cost_s, cap) = match r.split_once(", where x is less than or equal to ") {
        Some((c, v)) => (c, Some(v)),
        None => (r, None),
    };
    let cost = x_mana_cost(cost_s)?;
    let cap = match cap {
        Some(v) => {
            let (v, tail) = super::r107_numbers::value_phrase(v, b)?;
            if !end(&tail).is_empty() {
                return None;
            }
            Some(v)
        }
        None => None,
    };
    // The following sentences' X is this X.
    b.named
        .push((super::tokens_x_x::X_DEFINED.to_string(), Sel::None));
    let pay = Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::Noop),
    };
    Some(match cap {
        Some(v) => Effect::seq(vec![
            Effect::StoreValue {
                var: X_MAX,
                value: v,
            },
            pay,
        ]),
        None => pay,
    })
}

inventory::submit! { super::EffectPattern { name: "value results: you may pay {X}", priority: 99, parse: may_pay_x } }

/// The payment with {X} an effect ends with, if it does.
fn ends_with_x_payment(e: &Effect) -> bool {
    match e {
        Effect::PayOptional { cost, .. } => cost.mana.as_ref().is_some_and(|m| m.has_x()),
        Effect::Seq(v) => v.last().is_some_and(ends_with_x_payment),
        _ => false,
    }
}

/// "You may pay {X}. When you do, put X +1/+1 counters on that creature.": the reflexive
/// triggered ability's X is the X paid, and its "it" / "that creature" is what the
/// triggered ability was about (a payment acts on no object).
fn when_you_do_after_x_payment(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(r) = end(l).strip_prefix("when you do, ") else {
        return false;
    };
    if !ends_with_x_payment(prev) {
        return false;
    }
    let body = super::value_grammar::with_x_defined(true, || {
        super::r600_triggers::reflexive_body_about(r, b, b.it.clone())
    });
    let Some(body) = body else {
        return false;
    };
    let e = Effect::If {
        cond: Condition::PrevHappened,
        then: Box::new(Effect::Reflexive {
            body: Box::new(body),
        }),
        otherwise: Box::new(Effect::Noop),
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { super::FollowupPattern { name: "value results: when you do, after paying {X}", priority: 40, apply: when_you_do_after_x_payment } }

/// "[trigger], you may pay {X}. If you do, draw X cards. X can't be greater than the
/// amount of life you gained this turn.": the player can't choose a greater X as they pay.
fn x_cant_be_greater(block: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    let lower = block.trim().to_lowercase();
    let (head, cap) = lower.rsplit_once(". x can't be greater than ")?;
    if !head.contains("you may pay {x}") {
        return None;
    }
    let cap = end(cap);
    let mut abilities = crate::oracle::parse_ability(&format!("{head}."), ctx)?;
    let [a] = abilities.as_mut_slice() else {
        return None;
    };
    let a = std::sync::Arc::make_mut(a);
    let AbilityKind::Triggered(t) = &mut a.kind else {
        return None;
    };
    let mut b = Builder::new(ctx);
    b.in_trigger = true;
    let (v, tail) = super::r107_numbers::value_phrase(cap, &mut b)?;
    if !end(&tail).is_empty() || !b.targets.is_empty() {
        return None;
    }
    if !cap_payment(&mut t.body.effect, &v) {
        return None;
    }
    a.text = block.to_string();
    Some(abilities)
}

/// Puts the cap on X right before the payment with {X} in `e`.
fn cap_payment(e: &mut Effect, v: &Value) -> bool {
    match e {
        Effect::PayOptional { cost, .. } if cost.mana.as_ref().is_some_and(|m| m.has_x()) => {
            let pay = std::mem::take(e);
            *e = Effect::seq(vec![
                Effect::StoreValue {
                    var: X_MAX,
                    value: v.clone(),
                },
                pay,
            ]);
            true
        }
        Effect::Seq(xs) => xs.iter_mut().any(|x| cap_payment(x, v)),
        _ => false,
    }
}

inventory::submit! { super::AbilityPattern { name: "value results: X can't be greater than [value]", priority: 10, parse: x_cant_be_greater } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Layout;
    use crate::oracle::CompileContext;
    use crate::types::TypeLine;

    /// Developer probe: `VR_PROBE=<file>` with lines `V: <value phrase>`, `E: <effect
    /// text>` or `C: <card name>`; prints what each compiles to.
    #[test]
    fn probe() {
        let Ok(path) = std::env::var("VR_PROBE") else {
            return;
        };
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let tl = TypeLine::parse("Creature — Test");
        let stl = TypeLine::parse("Sorcery");
        let ctx = CompileContext {
            card_name: "Probe",
            full_name: "Probe",
            type_line: &tl,
            layout: Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        };
        for line in text.lines() {
            if let Some(v) = line.strip_prefix("V: ") {
                let mut b = Builder::new(&ctx);
                let r = super::super::value_grammar::parse_value(&v.to_lowercase(), &mut b);
                println!("V {v}\n  => {r:?}");
            } else if let Some(c) = line.strip_prefix("U: ") {
                let def = crate::card::card(c);
                for u in def.unsupported_text() {
                    println!("U {c}: {u}");
                }
            } else if let Some(c) = line.strip_prefix("D: ") {
                let def = crate::card::card(c);
                for f in &def.faces {
                    for a in &f.chars.abilities {
                        println!("D {c} | {}\n    {:?}", a.text, a.kind);
                    }
                }
            } else if let Some(c) = line.strip_prefix("C: ") {
                let def = crate::card::card(c);
                for f in &def.faces {
                    for a in &f.chars.abilities {
                        println!("C {c}: {:#?}", a.kind);
                    }
                }
            } else if let Some((t, tl)) = line
                .strip_prefix("T: ")
                .map(|t| (t, &stl))
                .or_else(|| line.strip_prefix("A: ").map(|t| (t, &tl)))
            {
                let c2 = CompileContext {
                    card_name: "Probe",
                    full_name: "Probe",
                    type_line: tl,
                    layout: Layout::Normal,
                    face_index: 0,
                    keywords: &[],
                    power: None,
                    toughness: None,
                };
                let comp = crate::oracle::compile(t, &c2);
                println!("T {t}\n  unsupported={:?}", comp.unsupported);
                for a in &comp.abilities {
                    println!("  {:?}", a.kind);
                }
            }
        }
    }
}
