//! Smaller basic-effect constructs:
//!
//! - "shuffle target nontoken permanent you control into its owner's library" (CR 701.24):
//!   the object is shuffled into its owner's library.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, Builder};
use crate::oracle::phrases::*;

/// "shuffle [object] into its owner's library", "shuffle [objects] into their owners'
/// libraries".
fn shuffle_into_owners_library(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("shuffle ")?;
    let (what, tail) = object_ref(r, b)?;
    match tail.trim() {
        "into its owner's library" | "into their owners' libraries" => {
            Some(Effect::ShuffleInto { what })
        }
        _ => None,
    }
}

inventory::submit! { EffectPattern { name: "basic effects: shuffle [object] into its owner's library", priority: 60, parse: shuffle_into_owners_library } }

/// "It gets an additional -1/-1 until end of turn for each Desert you control.", "Zombie
/// creatures you control get an additional +2/+2 until end of turn": "additional" only
/// says the change adds to an earlier one; it's the same change.
fn gets_additional(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" an additional ")?;
    let head = &l[..i];
    if !(head.ends_with(" gets") || head.ends_with(" get")) {
        return None;
    }
    let text = format!("{head} {}", &l[i + " an additional ".len()..]);
    crate::oracle::effects::parse_clause(&text, b)
}

inventory::submit! { EffectPattern { name: "basic effects: gets an additional +N/+N", priority: 60, parse: gets_additional } }

/// "Each creature gets twice -X/-X until end of turn." (Nuclear Fallout): twice the amount.
fn gets_twice(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let i = l.find(" twice ")?;
    let head = &l[..i];
    if !(head.ends_with(" gets") || head.ends_with(" get")) {
        return None;
    }
    let text = format!("{head} {}", &l[i + " twice ".len()..]);
    let e = crate::oracle::effects::parse_clause(&text, b)?;
    let Effect::Modify {
        what,
        mods,
        duration,
    } = e
    else {
        return None;
    };
    let twice = |v: Value| Value::Mul(Box::new(Value::Const(2)), Box::new(v));
    let mods = mods
        .into_iter()
        .map(|m| match m {
            Modification::ModifyPT(p, t) => Some(Modification::ModifyPT(twice(p), twice(t))),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Effect::Modify {
        what,
        mods,
        duration,
    })
}

inventory::submit! { EffectPattern { name: "basic effects: gets twice -X/-X", priority: 60, parse: gets_twice } }

/// "Until end of turn, double target creature's power X times." (Exponential Growth): the
/// power is doubled, then doubled again, X times in all (each doubling gives it +N/+0
/// where N is its power then, CR 701.10d).
fn double_n_times(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "Until end of turn, double ...": the duration belongs to the doubling.
    let moved;
    let l = match l.strip_prefix("until end of turn, ") {
        Some(r) => {
            let i = r.rfind(" times")?;
            let j = r[..i].rfind(' ')?;
            moved = format!("{} until end of turn{}", &r[..j], &r[j..]);
            moved.as_str()
        }
        None => l,
    };
    let i = l.rfind(" times")?;
    if !l[i..].trim_end().eq(" times") {
        return None;
    }
    let head = &l[..i];
    let j = head.rfind(' ')?;
    let (n, rest) = parse_number(&head[j + 1..])?;
    if !rest.is_empty() {
        return None;
    }
    let inner = &head[..j];
    if !(inner.contains("double ")) {
        return None;
    }
    let e = crate::oracle::effects::parse_clause(inner, b)?;
    Some(Effect::Repeat {
        times: n,
        effect: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "basic effects: double ... N times", priority: 60, parse: double_n_times } }

/// "That creature can block up to two additional creatures this turn", "target creature
/// can block an additional creature this turn", "... can block any number of creatures
/// this turn" (CR 509.1b).
fn can_block_additional(l: &str, b: &mut Builder) -> Option<Effect> {
    let (dur, l) = crate::oracle::effects::duration_suffix(end(l));
    if !matches!(dur, Duration::EndOfTurn) {
        return None;
    }
    let i = l.find(" can block ")?;
    let (what, tail) = object_ref(&l[..i], b)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let r = &l[i + " can block ".len()..];
    let n = match r {
        "an additional creature" => Some(1),
        "any number of creatures" => None,
        _ => {
            let r = r.strip_prefix("up to ")?.strip_suffix(" additional creatures")?;
            let (n, rest) = parse_number(r)?;
            if !rest.is_empty() {
                return None;
            }
            Some(n.as_const()? as u32)
        }
    };
    Some(Effect::AddRestriction {
        restriction: Restriction::ExtraBlocks {
            blocker: Filter::In(Box::new(what)),
            n,
        },
        duration: dur,
    })
}

inventory::submit! { EffectPattern { name: "basic effects: can block additional creatures this turn", priority: 60, parse: can_block_additional } }

/// "That creature gets an additional +4/+4 until end of turn unless any player pays {2}."
/// (Wild Might): any player may pay to stop it (CR 118.12a).
fn unless_any_player_pays(l: &str, b: &mut Builder) -> Option<Effect> {
    let (head, cost) = end(l).split_once(" unless any player pays ")?;
    let cost = crate::oracle::keywords::parse_keyword_cost(cost)?;
    let e = crate::oracle::effects::parse_clause(head, b)?;
    Some(Effect::PayOptional {
        who: PlayerRef::EachPlayer,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(e),
    })
}

inventory::submit! { EffectPattern { name: "basic effects: unless any player pays", priority: 60, parse: unless_any_player_pays } }

/// "look at the top three cards of target player's library, then put them back in any
/// order", "look at the top two cards of target opponent's library, then exile one of
/// them", "that player looks at the top three cards of your library, then puts them back
/// in any order": the player looking makes the choices, whoever owns the library.
fn look_at_top_of_library(l: &str, b: &mut Builder) -> Option<Effect> {
    use crate::kw::basic_effects::{look_at_top, LookAtTop};
    let l = end(l);
    let (looker, r, third) = if let Some(r) = l.strip_prefix("look at the top ") {
        (PlayerRef::You, r, false)
    } else if let Some(r) = l.strip_prefix("that player looks at the top ") {
        if super::oracle_hardening_referents::is_no_player_referent(&b.it_player) {
            return None;
        }
        (b.it_player.clone(), r, true)
    } else {
        return None;
    };
    let (n, r) = parse_number(r)?;
    n.as_const()?;
    let r = r.trim_start().strip_prefix("cards of ")?;
    let (lib, r) = r.split_once(" library, then ")?;
    let (exile, reorder) = match (r, third) {
        ("put them back in any order", false) | ("puts them back in any order", true) => {
            (0, true)
        }
        ("exile one of them", false) => (1, false),
        _ => return None,
    };
    let library = match lib {
        "your" if third => PlayerRef::You,
        "target player's" | "target opponent's" if !third => {
            let (pf, text) = if lib.starts_with("target player") {
                (PlayerFilter::Any, "target player")
            } else {
                (PlayerFilter::Opponent, "target opponent")
            };
            let slot = b.add_target(TargetSpec::player(pf, text), text);
            b.it_player = PlayerRef::Target(slot);
            PlayerRef::Target(slot)
        }
        _ => return None,
    };
    Some(look_at_top(&LookAtTop {
        library,
        looker,
        n,
        exile,
        reorder,
    }))
}

inventory::submit! { EffectPattern { name: "basic effects: look at the top of another player's library", priority: 80, parse: look_at_top_of_library } }

/// Keywords an object loses: plain keywords ("flying, first strike"), a landwalk
/// ("forestwalk": only that one), "all \"bands with other\" abilities".
fn lost_mods(s: &str) -> Option<Vec<Modification>> {
    use crate::kw::basic_effects::{remove_keyword, RemoveKeyword};
    if s == "all \"bands with other\" abilities" {
        return Some(vec![remove_keyword(&RemoveKeyword {
            kind: crate::keywords::KeywordKind::Banding,
            filter: None,
        })]);
    }
    let mut out = Vec::new();
    for m in crate::oracle::effects::keyword_mods(s)? {
        let Modification::AddKeyword(k) = m else {
            return None;
        };
        if k.cost.is_some() || k.n.is_some() {
            return None;
        }
        out.push(match (&k.kind, &k.filter) {
            (crate::keywords::KeywordKind::Landwalk, Some(f)) => remove_keyword(&RemoveKeyword {
                kind: k.kind,
                filter: Some(f.clone()),
            }),
            (_, None) => Modification::RemoveKeyword(k.kind),
            _ => return None,
        });
    }
    Some(out)
}

/// "target creature loses forestwalk until end of turn", "~ gains flying and loses trample
/// until end of turn", "until end of turn, ~ becomes a 3/3 Construct artifact creature and
/// loses flying", "target creature loses all \"bands with other\" abilities until end of
/// turn", "target creature loses your choice of flying, first strike, or trample until end
/// of turn".
fn and_loses(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let moved;
    let l = match l.strip_prefix("until end of turn, ") {
        Some(r) => {
            moved = format!("{r} until end of turn");
            moved.as_str()
        }
        None => l,
    };
    let (dur, main) = crate::oracle::effects::duration_suffix(l);
    let dur_s = match dur {
        Duration::EndOfTurn => " until end of turn",
        Duration::Permanent => "",
        _ => return None,
    };
    // "[subject] loses your choice of A, B, or C": one of them, chosen as it resolves.
    if let Some((subject, list)) = main.split_once(" loses your choice of ") {
        let (what, tail) = object_ref(subject, b)?;
        if !tail.trim().is_empty() {
            return None;
        }
        let items: Vec<String> = list
            .replace(", or ", ", ")
            .replace(" or ", ", ")
            .split(", ")
            .map(str::to_string)
            .collect();
        let mut options = Vec::new();
        for i in items {
            let mods = lost_mods(&i)?;
            options.push((
                i.clone(),
                Effect::Modify {
                    what: what.clone(),
                    mods,
                    duration: dur.clone(),
                },
            ));
        }
        return Some(Effect::ChooseOne {
            who: PlayerRef::You,
            options,
        });
    }
    let (head, lost) = match main.rsplit_once(" and loses ") {
        Some((h, k)) => (Some(h), k),
        None => (None, main.split_once(" loses ")?.1),
    };
    let lost = lost_mods(lost)?;
    let base = match head {
        Some(h) => crate::oracle::effects::parse_clause(&format!("{h}{dur_s}"), b)?,
        None => {
            let subject = main.split_once(" loses ")?.0;
            let (what, tail) = object_ref(subject, b)?;
            if !tail.trim().is_empty() {
                return None;
            }
            Effect::Modify {
                what,
                mods: vec![],
                duration: dur.clone(),
            }
        }
    };
    let Effect::Modify {
        what,
        mut mods,
        duration,
    } = base
    else {
        return None;
    };
    mods.extend(lost);
    Some(Effect::Modify {
        what,
        mods,
        duration,
    })
}

inventory::submit! { EffectPattern { name: "basic effects: loses keywords", priority: 120, parse: and_loses } }

/// "behold a Goblin and exile it" (Champion of the Weird and the other Lorwyn champions):
/// behold (CR 701.4a), then exile what was beheld; the exiled card is exiled to pay the
/// cost (CR 607.2q).
fn behold_and_exile(p: &str) -> Option<CostPart> {
    let r = end(p).strip_prefix("behold ")?.strip_suffix(" and exile it")?;
    let (n, r) = parse_number(r)?;
    if n.as_const() != Some(1) {
        return None;
    }
    let (f, _, tail) = parse_object_phrase(r)?;
    if !end(tail).is_empty() {
        return None;
    }
    Some(CostPart::Effect(Box::new(Effect::seq(vec![
        Effect::KeywordAction {
            action: KeywordAction::Behold,
            who: PlayerRef::You,
            what: Sel::All(f),
            n,
        },
        Effect::Exile {
            what: Sel::Var(vars::IT),
            face_down: false,
            link: false,
        },
    ]))))
}

inventory::submit! { super::CostPattern { name: "basic effects: behold and exile it", priority: 100, parse: behold_and_exile } }

/// "sacrifice a legendary artifact or legendary creature" (a cost, e.g. for ward): one
/// permanent you control described by alternatives.
fn sacrifice_alternatives(p: &str) -> Option<CostPart> {
    let r = end(p).strip_prefix("sacrifice ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, plural, rest) = super::basic_effects_targets::object_alternatives(r)?;
    if plural || !end(rest).is_empty() {
        return None;
    }
    Some(CostPart::Sacrifice {
        filter: Filter::and(vec![f, Filter::ControlledBy(PlayerRel::You)]),
        count: Value::Const(1),
    })
}

inventory::submit! { super::CostPattern { name: "basic effects: sacrifice one of alternatives", priority: 100, parse: sacrifice_alternatives } }
