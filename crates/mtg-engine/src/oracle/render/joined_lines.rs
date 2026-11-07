//! Consecutive rendered abilities that cards print as one sentence.

/// "Prevent all damage that would be dealt to X." + "Prevent all damage that would be
/// dealt by X." = "Prevent all damage that would be dealt to and dealt by X." (two
/// prevention effects, one for each direction, CR 615.1).
pub(super) fn join_lines(a: &str, b: &str) -> Option<String> {
    let a = a.trim_end_matches('.');
    let b = b.trim_end_matches('.');
    // "The first creature spell you cast each turn costs {2} less to cast." + "You may cast
    // the first creature spell you cast each turn as though it had flash." = "... costs
    // {2} less to cast and can be cast as though it had flash." (the same spells).
    let split = match a.split_once(" costs ") {
        Some((x, y)) => Some((x, y.to_string())),
        None => a.split_once(" cost {").map(|(x, y)| (x, format!("{{{y}"))),
    };
    if let Some((spells, cost)) = split {
        let spells_b = b
            .strip_prefix("You may cast ")
            .or_else(|| b.strip_prefix("you may cast "))
            .and_then(|r| r.strip_suffix(" as though it had flash"));
        if let Some(spells_b) = spells_b {
            if spells.to_lowercase() == spells_b.to_lowercase() && spells.starts_with("The first ")
            {
                return Some(format!(
                    "{spells} costs {cost} and can be cast as though it had flash."
                ));
            }
        }
    }
    if let Some(j) = join_same_subject(a, b) {
        return Some(j);
    }
    let (head, to) = a.split_once(" that would be dealt to ")?;
    if !head.starts_with("Prevent all ") {
        return None;
    }
    let rest = b
        .strip_prefix(head)?
        .strip_prefix(" that would be dealt ")?;
    let by = rest
        .strip_prefix("{opt:to a permanent or player} by ")
        .or_else(|| rest.strip_prefix("by "))?;
    (by == to).then(|| format!("{head} that would be dealt to and dealt by {to}."))
}

/// Two rules about the same things, printed as one sentence: "Spells and abilities your
/// opponents control can't cause you to discard cards or sacrifice permanents.", "You may
/// look at the top card of your library and at face-down creatures you don't control any
/// time.", "Creatures with flying can't attack or block, and their activated abilities
/// with {T} in their costs can't be activated."
fn join_same_subject(a: &str, b: &str) -> Option<String> {
    let cause = "Spells and abilities your opponents control can't cause you to ";
    if let (Some(x), Some(y)) = (a.strip_prefix(cause), b.strip_prefix(cause)) {
        return Some(format!("{cause}{x} or {y}."));
    }
    // "You may spend white mana as though it were mana of any color. You may spend other
    // mana only as though it were colorless mana." (one ability on the card: "other" is
    // the mana the first sentence names, checked: the second rule lists every other
    // color).
    if let Some(m) = b
        .strip_prefix("You may spend mana other than ")
        .and_then(|r| r.strip_suffix(" mana only as though it were colorless mana"))
    {
        if a.starts_with(&format!("You may spend {m} mana as though it were ")) {
            return Some(format!(
                "{a}. You may spend other mana only as though it were colorless mana."
            ));
        }
    }
    // "~ costs {1} less to cast if you control a Spirit. It also costs {1} less to cast if
    // you control an enchantment." (two reductions of the spell's own cost, each with its
    // own condition, CR 601.2f).
    if let (Some(x), Some(y)) = (a.strip_prefix("~ costs "), b.strip_prefix("~ costs ")) {
        if x.contains(" less to cast if ") && y.contains(" less to cast if ") {
            return Some(format!("{a}. It also costs {y}."));
        }
    }
    // "During combat, players can't cast spells or activate abilities that aren't mana
    // abilities." (two restrictions with the same duration).
    let combat = " during combat";
    if let (Some(x), Some(y)) = (a.strip_suffix(combat), b.strip_suffix(combat)) {
        if x == "Players can't activate abilities that aren't mana abilities"
            // (Exactly these two: a restriction with any other quality isn't joined.)
            && y == "{alt:players can't cast spells|spells can't be cast}"
        {
            return Some(
                "During combat, players can't cast spells or activate abilities that aren't mana abilities."
                    .into(),
            );
        }
    }
    // "You and Humans you control have hexproof.", "You, planeswalkers you control, and
    // other creatures you control have hexproof." (a player's ability and the objects',
    // the same keyword).
    if let Some(kw) = a.strip_prefix("You have ") {
        if let Some(s) = b.strip_suffix(&format!(" have {kw}")) {
            if !s.contains('{') && !s.starts_with("You") {
                let s = lower_start(s);
                return Some(match s.split_once(" and/or ") {
                    Some((x, y)) => format!("You, {x}, and {y} have {kw}."),
                    None => format!("You and {s} have {kw}."),
                });
            }
        }
    }
    let look = "You may look at ";
    if let (Some(x), Some(y)) = (
        a.strip_prefix(look)
            .and_then(|x| x.strip_suffix(" any time")),
        b.strip_prefix(look)
            .and_then(|y| y.strip_suffix(" any time")),
    ) {
        return Some(format!("{look}{x} and at {y} any time."));
    }
    // "Equipped creature gets +1/+1. If it's a Warrior, it gets +2/+1 instead." (one
    // bonus for each kind of creature, the two never applying together).
    let gets = |s: &'_ str| -> Option<(String, String)> {
        s.split_once(" gets ")
            .or_else(|| s.split_once(" get "))
            .map(|(x, y)| (x.to_string(), y.to_string()))
    };
    if let (Some((sa, xa)), Some((sb, xb))) = (gets(a), gets(b)) {
        if let (Some((head, ta)), Some((head2, tb))) =
            (sa.split_once(" that isn't "), sb.split_once(" that's "))
        {
            if head == head2 && ta == tb {
                return Some(format!(
                    "{head} gets {xa}. If it's {ta}, it gets {xb} instead."
                ));
            }
        }
    }
    if let Some(subj) = a.strip_suffix(" can't attack or block") {
        let t = format!("Activated abilities of {} with ", subj.to_lowercase());
        let rest = b.strip_prefix(&t).or_else(|| {
            b.strip_prefix(&format!(
                "activated abilities of {} with ",
                subj.to_lowercase()
            ))
        })?;
        let rest = rest.strip_suffix(" in their costs can't be activated")?;
        return Some(format!(
            "{subj} can't attack or block, and their activated abilities with {rest} in their costs can't be activated."
        ));
    }
    None
}

/// `s` with its first letter in lower case.
fn lower_start(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().chain(c).collect(),
        None => String::new(),
    }
}
