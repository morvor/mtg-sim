//! Referent grammar III: batch and per-player referents (CR 603.2c, 608.2c).
//!
//! - "[verb] one of them" after a "one or more" trigger or a group instruction: one of
//!   the objects "them" refers to, chosen as the instruction is performed, which "it"
//!   then refers to ("Whenever one or more creatures you control deal combat damage to a
//!   player, you may sacrifice one of them.", "Whenever you discard one or more nonland
//!   cards, you may exile one of them from your graveyard. If you do, you may cast it
//!   this turn.");
//! - "exile them from your graveyard", "exile it from their graveyard": the objects the
//!   pronoun refers to that are (still) in that graveyard (CR 400.7: a card that left it
//!   is a new object the pronoun no longer finds).

use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// [`Builder::named`] key for the cards the creatures of a "one or more creatures die"
/// trigger became (`Sel::TriggerObjects`): "one of them" may choose among them.
pub const DIED_BATCH: &str = "\u{1}died batch";

/// The object "one of them" chose ("If you do, you may cast it this turn").
pub const ONE_OF_THEM: Var = vars::USER + 12_001;

/// "you may sacrifice one of them", "return one of those creatures to its owner's hand".
fn verb_one_of_them(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (phrase, i) = [
        " one of them",
        " one of those creatures",
        " one of those permanents",
        " one of those cards",
        " one of the other creatures",
    ]
    .iter()
    .find_map(|p| l.find(p).map(|i| (*p, i)))?;
    let head = &l[..i];
    // A plain instruction: one verb, nothing chosen before it; or a token copy of the
    // chosen object ("create a token that's a copy of one of them", Twilight Diviner).
    if (head.contains(' ') && head != "create a token that's a copy of") || head.is_empty() {
        return None;
    }
    let after = &l[i + phrase.len()..];
    if !(after.is_empty() || after.starts_with([' ', ','])) || after.contains(" one of ") {
        return None;
    }
    if after.contains(" at random") {
        return None;
    }
    // "one of the other creatures" after "one or more other creatures": "them".
    let pronoun = match phrase {
        " one of the other creatures" => "them",
        p => p[" one of ".len()..].trim(),
    };
    let them = match super::pronoun_groups::plural_object_ref(pronoun, b) {
        Some(Some((sel, _))) => sel,
        // "Whenever one or more creatures you control die, return one of them ...": the
        // cards the creatures became in the graveyard (CR 400.7), which an instruction can
        // act on; not a referent for their characteristics (their last known information).
        Some(None) if pronoun == "them" => b
            .named
            .iter()
            .find(|(n, _)| n == DIED_BATCH)
            .map(|(_, s)| s.clone())?,
        _ => return None,
    };
    if matches!(them, Sel::None) || super::oracle_hardening_referents::is_no_referent(&them) {
        return None;
    }
    // Where the chosen object must be for the instruction to act on it.
    let mut filter = vec![Filter::In(Box::new(them))];
    if after.starts_with(" from your graveyard") || after.starts_with(" from their graveyard") {
        filter.push(Filter::InZone(ZoneKind::Graveyard));
    } else if head == "sacrifice" {
        filter.push(Filter::InZone(ZoneKind::Battlefield));
        filter.push(Filter::ControlledBy(PlayerRel::You));
    }
    let choose = Effect::Store {
        var: ONE_OF_THEM,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: Filter::and(filter),
            count: Value::c(1),
            up_to: false,
            store: None,
        },
    };
    // The instruction read with "it" for the chosen object.
    let rewritten = format!("{head} it{after}");
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    b.it = Sel::Var(ONE_OF_THEM);
    let e = crate::oracle::effects::parse_clause(&rewritten, b);
    let Some(e) = e else {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1, saved.2);
        return None;
    };
    // A later "it" is what the instruction made it (the exiled card), or has no
    // antecedent: never the batch.
    if matches!(b.it, Sel::Var(ONE_OF_THEM)) {
        b.it = saved.1;
    }
    Some(Effect::seq(vec![choose, e]))
}

inventory::submit! { super::EffectPattern { name: "referents: [verb] one of them", priority: 1500, parse: verb_one_of_them } }

/// "exile them from your graveyard", "exile it from their graveyard".
fn exile_pronoun_from_graveyard(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("exile ")?;
    let (pron, rest) = ["them", "it", "those cards", "that card"]
        .iter()
        .find_map(|p| {
            r.strip_prefix(p)
                .filter(|x| x.starts_with(" from "))
                .map(|x| (*p, x))
        })?;
    let zone = match rest.trim() {
        "from your graveyard" => Filter::OwnedBy(PlayerRel::You),
        "from their graveyard" | "from its owner's graveyard" | "from a graveyard" => Filter::Any,
        _ => return None,
    };
    let sel = if matches!(pron, "them" | "those cards") {
        match super::pronoun_groups::plural_object_ref(pron, b) {
            Some(Some((sel, _))) => sel,
            _ => return None,
        }
    } else {
        let (sel, tail) = crate::oracle::effects::object_ref(pron, b)?;
        if !tail.trim().is_empty() {
            return None;
        }
        sel
    };
    if matches!(sel, Sel::None | Sel::This)
        || super::oracle_hardening_referents::is_no_referent(&sel)
    {
        return None;
    }
    let in_graveyard = Filter::and(vec![Filter::InZone(ZoneKind::Graveyard), zone]);
    let what = match sel {
        // "exile one of them from your graveyard": one of those in that graveyard.
        Sel::Choose {
            chooser,
            filter,
            count,
            up_to,
            store,
        } => Sel::Choose {
            chooser,
            filter: Filter::and(vec![filter, in_graveyard]),
            count,
            up_to,
            store,
        },
        sel => Sel::Matching(Box::new(sel), in_graveyard),
    };
    Some(Effect::Exile {
        what,
        face_down: false,
        link: false,
    })
}

inventory::submit! { super::EffectPattern { name: "referents: exile them from a graveyard", priority: 1500, parse: exile_pronoun_from_graveyard } }

/// "where X is ... their ...": "their" is the one player the sentence names ("it
/// deals X damage to defending player, where X is the number of cards in their hand.").
fn where_x_their(block: &str, ctx: &crate::oracle::CompileContext) -> Option<Vec<Ability>> {
    if super::referents_results::in_standard() {
        return None;
    }
    let lower = block.to_lowercase();
    let w = lower.find(", where x is ")?;
    let j = w + lower[w..].find(" their ")?;
    // The sentence the "where" clause belongs to.
    let start = lower[..w].rfind(['.', '"', ':']).map_or(0, |i| i + 1);
    let main = &lower[start..w];
    const PLAYERS: [&str; 3] = ["defending player", "that player", "that opponent"];
    let named: Vec<&str> = PLAYERS.iter().copied().filter(|p| main.contains(p)).collect();
    let [p] = named.as_slice() else {
        return None;
    };
    let clause = &lower[w..];
    let clause = &clause[..clause.find(['.', '"']).unwrap_or(clause.len())];
    if clause.matches(" their ").count() != 1
        || super::referents_results::has_word(clause, "they")
        || super::referents_results::has_word(main, "they")
    {
        return None;
    }
    if super::referents_results::standard_ok(block, ctx) {
        return None;
    }
    let rewritten = format!("{} {p}'s {}", &block[..j], &block[j + " their ".len()..]);
    let v = crate::oracle::parse_ability(&rewritten, ctx)?;
    let ok = v.iter().all(|a| {
        !(super::oracle_hardening_referents::has_no_referent(a)
            || crate::repeat_process::has_stray_repeat(a)
            || super::filters_relational::unresolved(a))
    });
    if !ok {
        return None;
    }
    let text = crate::oracle::strip_ability_word(block.trim());
    Some(
        v.into_iter()
            .map(|a| {
                let mut d = (*a).clone();
                d.text = text.to_string();
                std::sync::Arc::new(d)
            })
            .collect(),
    )
}

inventory::submit! { super::AbilityPattern { name: "referents: where X is ... their (the one player named)", priority: 50, parse: where_x_their } }

/// "Each player loses half their life, then discards half the cards in their hand, then
/// sacrifices half the creatures they control of their choice." (Fraying Omnipotence):
/// each instruction after the first has the same subject, and is performed after the one
/// before it is finished (each player in turn, CR 101.4; Fraying Omnipotence rulings).
fn subject_then_each(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let subject = ["each player ", "each opponent "]
        .into_iter()
        .find(|s| l.starts_with(s))?;
    let parts: Vec<&str> = l.split(", then ").collect();
    if parts.len() < 3 {
        return None;
    }
    // Every later part is a verb phrase without a subject ("discards ...").
    let verb_only = |p: &str| {
        let w = p.split(' ').next().unwrap_or("");
        w.len() > 2 && w.ends_with('s') && !["each", "it", "they", "this", "that"].contains(&w)
    };
    if !parts[1..].iter().all(|p| verb_only(p)) {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let mut out = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        let s = if i == 0 {
            p.to_string()
        } else {
            format!("{subject}{p}")
        };
        match crate::oracle::effects::parse_clause(&s, b) {
            Some(e) => out.push(e),
            None => {
                b.targets.truncate(saved.0);
                (b.it, b.it_player) = (saved.1, saved.2);
                return None;
            }
        }
    }
    Some(Effect::seq(out))
}

inventory::submit! { super::EffectPattern { name: "referents: each player [x], then [y], then [z] (the same subject)", priority: 1500, parse: subject_then_each } }

/// "Until end of turn, target creature gains trample and gets +1/+0 for each card you've
/// drawn this turn.", "Target creature you control gets +1/+1 until end of turn for each
/// card in your hand and can't be blocked this turn.": two instructions with one subject,
/// one of them with a "for each" amount.
fn shared_subject_for_each(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if !l.contains(" for each ") {
        return None;
    }
    let (prefix, r) = match l.strip_prefix("until end of turn, ") {
        Some(r) => ("until end of turn, ", r),
        None => ("", l),
    };
    if !r.starts_with("target ") {
        return None;
    }
    const VERBS: [&str; 4] = ["gets ", "gains ", "can't ", "has "];
    let i = VERBS
        .iter()
        .filter_map(|v| r.find(&format!(" {v}")))
        .min()?;
    let subject = &r[..i];
    let vp = &r[i + 1..];
    let (vp1, vp2) = vp
        .match_indices(" and ")
        .map(|(j, _)| (&vp[..j], &vp[j + " and ".len()..]))
        .find(|(_, y)| VERBS.iter().any(|v| y.starts_with(v)))?;
    if vp2.contains(" and ") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let restore = |b: &mut Builder| {
        b.targets.truncate(saved.0);
        (b.it, b.it_player) = (saved.1.clone(), saved.2.clone());
    };
    let Some(first) = crate::oracle::effects::parse_clause(&format!("{prefix}{subject} {vp1}"), b)
    else {
        restore(b);
        return None;
    };
    if b.targets.len() != saved.0 + 1 {
        restore(b);
        return None;
    }
    // The second instruction is about the same target.
    let slot = saved.0 as u8;
    b.it = Sel::Target(slot);
    let Some(second) = crate::oracle::effects::parse_clause(&format!("{prefix}it {vp2}"), b) else {
        restore(b);
        return None;
    };
    if b.targets.len() != saved.0 + 1 {
        restore(b);
        return None;
    }
    Some(Effect::seq(vec![first, second]))
}

inventory::submit! { super::EffectPattern { name: "referents: [target] [does x] and [does y] (one subject, a for-each amount)", priority: 1500, parse: shared_subject_for_each } }

/// The object "choose ... at random" chose, for "the rest".
pub const CHOSEN_AT_RANDOM: Var = vars::USER + 12_005;

/// "Choose a creature at random, then destroy the rest." (Last One Standing): one of the
/// objects chosen at random (CR 701.9b's randomness: each equally likely), then the
/// instruction for all the others.
fn choose_at_random_then_the_rest(l: &str, _b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("choose ")?;
    let (what, verb) = r.split_once(" at random, then ")?;
    let what = what.strip_prefix("a ").or_else(|| what.strip_prefix("an "))?;
    let verb = verb.strip_suffix(" the rest")?;
    let (f, plural, tail) = crate::oracle::phrases::parse_object_phrase(what)?;
    if plural || !tail.trim().is_empty() || f.zone().is_some_and(|z| z != ZoneKind::Battlefield) {
        return None;
    }
    let f = Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)]);
    let rest = Sel::All(Filter::and(vec![
        f.clone(),
        Filter::not(Filter::In(Box::new(Sel::Var(CHOSEN_AT_RANDOM)))),
    ]));
    let act = match verb {
        "destroy" => Effect::Destroy {
            what: rest,
            no_regen: false,
        },
        "exile" => Effect::Exile {
            what: rest,
            face_down: false,
            link: false,
        },
        _ => return None,
    };
    Some(Effect::seq(vec![
        Effect::Store {
            var: CHOSEN_AT_RANDOM,
            sel: Sel::AtRandom {
                filter: f,
                count: Value::c(1),
                store: None,
            },
        },
        act,
    ]))
}

inventory::submit! { super::EffectPattern { name: "referents: choose [an object] at random, then [destroy] the rest", priority: 1500, parse: choose_at_random_then_the_rest } }

/// "Put a stun counter on each of those creatures you don't control." (Lost in the Maze):
/// those of the objects "those creatures" refers to that you (don't) control.
fn those_you_control(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (phrase, mine) = [
        ("each of those creatures you don't control", false),
        ("each of those creatures you control", true),
        ("those creatures you don't control", false),
        ("those creatures you control", true),
    ]
    .into_iter()
    .find(|(p, _)| l.contains(p))?;
    let them = match super::pronoun_groups::plural_object_ref("those creatures", b) {
        Some(Some((sel, _))) => sel,
        _ => return None,
    };
    if matches!(them, Sel::None) || super::oracle_hardening_referents::is_no_referent(&them) {
        return None;
    }
    let control = Filter::ControlledBy(PlayerRel::You);
    let sel = Sel::All(Filter::and(vec![
        Filter::In(Box::new(them)),
        if mine { control } else { Filter::not(control) },
    ]));
    let key = "the creatures of those named here".to_string();
    let rewritten = l.replacen(phrase, &key, 1);
    b.named.push((key.clone(), sel));
    let e = crate::oracle::effects::parse_clause(&rewritten, b);
    b.named.retain(|(p, _)| *p != key);
    e
}

inventory::submit! { super::EffectPattern { name: "referents: [each of] those creatures you (don't) control", priority: 1500, parse: those_you_control } }

/// "that player loses 1 life for each card fewer than three in their hand" (Nezumi
/// Shortfang's Stabwhisker): three minus the number of cards in their hand, if positive.
fn for_each_card_fewer_than(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (head, k) = l.split_once(" 1 life for each card fewer than ")?;
    let k = k.strip_suffix(" in their hand")?;
    if k.contains(' ') {
        return None;
    }
    crate::oracle::effects::parse_sentence(
        &format!("{head} x life, where x is {k} minus the number of cards in their hand"),
        b,
    )
}

inventory::submit! { super::EffectPattern { name: "referents: N life for each card fewer than K in their hand", priority: 1500, parse: for_each_card_fewer_than } }

/// "~ deals damage to that land's controller equal to the number of land cards in that
/// player's graveyard" (Roiling Terrain): "that player" is the controller just named.
fn that_player_the_controller_named(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" that player's ")?;
    let before = &l[..i];
    let named: Vec<&str> = ["that land's controller", "that creature's controller", "its controller"]
        .into_iter()
        .filter(|p| before.contains(p))
        .collect();
    let [p] = named.as_slice() else {
        return None;
    };
    if l[i + 1..].contains(" that player") && l[i + 1..].matches("that player").count() > 1 {
        return None;
    }
    let rewritten = format!("{before} {p}'s {}", &l[i + " that player's ".len()..]);
    crate::oracle::effects::parse_sentence(&rewritten, b)
}

inventory::submit! { super::EffectPattern { name: "referents: that player (the controller the sentence named)", priority: 1500, parse: that_player_the_controller_named } }

/// "Put a double strike counter on target creature, then goad each creature that had a
/// double strike counter put on it this way." (Spectacular Showdown), "Goad each creature
/// that had counters put on it this way." (Agitator Ant): the objects the counter
/// instruction just before put counters on (recorded as its result, CR 608.2c).
fn had_counters_put_on_it(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" that had ")?;
    let j = i + l[i..].find(" put on it this way")?;
    let what = &l[i + " that had ".len()..j];
    let counters = what == "counters"
        || what.ends_with(" counter") && (what.starts_with("a ") || what.starts_with("an "));
    if !counters {
        return None;
    }
    // "each creature" / "each permanent" just before.
    let k = l[..i].rfind("each ")?;
    let noun = &l[k + "each ".len()..i];
    let (f, plural, tail) = crate::oracle::phrases::parse_object_phrase(noun)?;
    if plural || !tail.trim().is_empty() {
        return None;
    }
    // Only right after an instruction that puts counters ("..., then goad ...", or the
    // sentence before).
    let raw = crate::oracle::raw_text().to_lowercase();
    let before = &raw[..raw.find("put on it this way")?];
    let last = before.rfind(['.', '\n']).map_or(before, |x| &before[..x]);
    let recent = before[last.len()..].contains(" counter") && before[last.len()..].contains("put ")
        || last.rsplit(['.', '\n']).next().is_some_and(|s| s.contains("put ") && s.contains(" counter"));
    if !recent {
        return None;
    }
    let key = "the objects that got counters this way".to_string();
    let rewritten = format!("{}{key}{}", &l[..k], &l[j + " put on it this way".len()..]);
    b.named.push((
        key.clone(),
        Sel::Matching(Box::new(Sel::Var(vars::IT)), f),
    ));
    let e = crate::oracle::effects::parse_clause(&rewritten, b);
    b.named.retain(|(p, _)| *p != key);
    e
}

inventory::submit! { super::EffectPattern { name: "referents: each creature that had counters put on it this way", priority: 1500, parse: had_counters_put_on_it } }

/// The permanents chosen by [`choose_those_that_got_counters`].
pub const CHOSEN_GOT_COUNTERS: Var = vars::USER + 12_007;

/// "Proliferate, then choose any number of permanents you control that had a counter put
/// on them this way. Those permanents phase out." (Ripples of Potential): a choice among
/// the objects the counter instruction just put counters on, which "those permanents"
/// then refers to.
fn choose_those_that_got_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("choose any number of ")?;
    let noun = r
        .strip_suffix(" you control that had a counter put on them this way")
        .or_else(|| r.strip_suffix(" you control that had counters put on them this way"))?;
    let (f, true, tail) = crate::oracle::phrases::parse_object_phrase(noun)? else {
        return None;
    };
    if !tail.trim().is_empty() {
        return None;
    }
    let filter = Filter::and(vec![
        f,
        Filter::In(Box::new(Sel::Var(vars::IT))),
        Filter::InZone(ZoneKind::Battlefield),
        Filter::ControlledBy(PlayerRel::You),
    ]);
    b.it = Sel::Var(CHOSEN_GOT_COUNTERS);
    Some(Effect::Store {
        var: CHOSEN_GOT_COUNTERS,
        sel: Sel::Choose {
            chooser: PlayerRef::You,
            filter: filter.clone(),
            count: Value::Count(filter),
            up_to: true,
            store: None,
        },
    })
}

inventory::submit! { super::EffectPattern { name: "referents: choose any number of permanents that had a counter put on them this way", priority: 1500, parse: choose_those_that_got_counters } }

/// "Choose target creature you control and target creature an opponent controls. Each of
/// those creatures deals damage equal to its toughness to the other." (Grim Contest): the
/// two chosen creatures, each dealing damage to the other.
fn each_deals_damage_to_the_other(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("each of those creatures deals damage equal to its ")?;
    let (stat, rest) = super::value_grammar::stat_word(r)?;
    if rest != " to the other" {
        return None;
    }
    let mut slots: Vec<u8> = Vec::new();
    for (_, sel) in &b.named {
        if let Sel::Target(i) = sel {
            if !slots.contains(i) {
                slots.push(*i);
            }
        }
    }
    let [a, c] = slots.as_slice() else {
        return None;
    };
    let (a, c) = (Sel::Target(*a), Sel::Target(*c));
    let (a, c) = (&a, &c);
    let hit = |from: &Sel, to: &Sel| Effect::DealDamage {
        source: from.clone(),
        amount: super::value_grammar::of_referent(stat.clone(), from.clone()),
        to: to.clone(),
    };
    Some(Effect::seq(vec![hit(a, c), hit(c, a)]))
}

inventory::submit! { super::EffectPattern { name: "referents: each of those creatures deals damage equal to its [stat] to the other", priority: 1500, parse: each_deals_damage_to_the_other } }

/// "~ deals 2 damage to each creature for each Aura attached to that creature." (Baki's
/// Curse): an amount for each of the objects, measured for that object.
fn damage_to_each_for_each_attached(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("~ deals ")?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    let r = r.trim_start().strip_prefix("damage to each ")?;
    let (noun, r) = r.split_once(" for each ")?;
    let (counted, that) = r.split_once(" attached to that ")?;
    if that != noun {
        return None;
    }
    let (f, plural, tail) = crate::oracle::phrases::parse_object_phrase(noun)?;
    if plural || !tail.trim().is_empty() {
        return None;
    }
    let (cf, cplural, ctail) = crate::oracle::phrases::parse_object_phrase(counted)?;
    if cplural || !ctail.trim().is_empty() {
        return None;
    }
    let _ = b;
    let each = vars::USER + 12_008;
    let attached = Value::Count(Filter::and(vec![
        cf,
        Filter::AttachedToAnyOf(Box::new(Sel::Var(each))),
    ]));
    Some(Effect::ForEach {
        sel: Sel::All(Filter::and(vec![f, Filter::InZone(ZoneKind::Battlefield)])),
        var: each,
        effect: Box::new(Effect::DealDamage {
            source: Sel::This,
            amount: Value::Mul(Box::new(n), Box::new(attached)),
            to: Sel::Var(each),
        }),
    })
}

inventory::submit! { super::EffectPattern { name: "referents: N damage to each [object] for each [thing] attached to that [object]", priority: 1500, parse: damage_to_each_for_each_attached } }

/// The cards "N of them at random" chose.
pub const RETURNED_AT_RANDOM: Var = vars::USER + 12_009;

/// "Choose three target creature cards in your graveyard. Return two of them at random to
/// the battlefield and put the other on the bottom of your library." (Sinister Waltz).
fn return_n_of_them_at_random(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("return ")?;
    let (n, r) = crate::oracle::phrases::parse_number(r)?;
    let r = r
        .trim_start()
        .strip_prefix("of them at random to the battlefield and put the other on the bottom of your library")?;
    if !r.is_empty() {
        return None;
    }
    let them = match super::pronoun_groups::plural_object_ref("them", b) {
        Some(Some((sel, _))) if matches!(sel, Sel::Target(_)) => sel,
        _ => return None,
    };
    let rest = Sel::All(Filter::and(vec![
        Filter::In(Box::new(them.clone())),
        Filter::not(Filter::In(Box::new(Sel::Var(RETURNED_AT_RANDOM)))),
    ]));
    let mut bottom = Destination::zone(ZoneKind::Library);
    bottom.position = LibraryPosition::Bottom;
    Some(Effect::seq(vec![
        Effect::Store {
            var: RETURNED_AT_RANDOM,
            sel: Sel::AtRandom {
                filter: Filter::In(Box::new(them)),
                count: n,
                store: None,
            },
        },
        Effect::Move {
            what: Sel::Var(RETURNED_AT_RANDOM),
            to: Destination::battlefield(),
        },
        Effect::Move {
            what: rest,
            to: bottom,
        },
    ]))
}

inventory::submit! { super::EffectPattern { name: "referents: return N of them at random and put the other on the bottom", priority: 1500, parse: return_n_of_them_at_random } }
