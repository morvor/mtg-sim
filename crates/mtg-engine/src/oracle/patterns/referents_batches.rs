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
    // A plain instruction: one verb, nothing chosen before it.
    if head.contains(' ') || head.is_empty() {
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
    let them = match super::pronoun_groups::plural_object_ref(pronoun, b)
    {
        Some(Some((sel, _))) => sel,
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
