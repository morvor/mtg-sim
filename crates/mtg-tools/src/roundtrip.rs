//! `roundtrip [--card NAME] [--filter TEXT] [--limit N] [--write PATH] [--check] [--ast]
//! [--list] [--all] [--clusters N]`: renders every fully supported card's compiled abilities back
//! to Oracle-style text ([`mtg_engine::oracle::render`]) and compares it with the card's
//! Oracle text.
//!
//! * `--card NAME`: only that card (prints both sides).
//! * `--filter TEXT`: only cards whose Oracle text contains TEXT (case-insensitive).
//! * `--limit N`: with `--list`, print at most N mismatching cards (default 20).
//! * `--list`: print mismatching cards with both sides.
//! * `--all`: print every checked card with both sides (to diff the compiler's output
//!   before and after a change).
//! * `--write PATH`: write the Markdown report (and `docs/roundtrip-passing.txt`, the
//!   checked-in list of passing cards that must stay passing).
//! * `--check`: exit 1 if a card in `docs/roundtrip-passing.txt` no longer passes.
//! * `--ast`: print the compiled abilities' AST of `--card`.
//! * `--clusters N`: print the N largest mismatch clusters.

use mtg_engine::card::CardDef;
use mtg_engine::oracle::render::compare::{check_card, normalize_unit, CardCheck};
use std::collections::{BTreeMap, BTreeSet};

pub const PASSING_LIST: &str = "docs/roundtrip-passing.txt";

/// What the round trip found in the compiler, kept with the generated report.
const FINDINGS: &str = "## Compiler bugs found by the round trip\n\n\
Each was a card the compiler accepted but misread; each is fixed and has an in-game test in \
`crates/mtg-engine/tests/cards/oracle_roundtrip.rs`.\n\n\
| Bug | Cards (examples) | Fix |\n|---|---|---|\n\
| A probe noun's \"is a card\" part stayed in object filters, so tokens were left out (\"Creatures your opponents control enter tapped\", \"Colorless creatures you control enter with ... counters\", \"each other creature named ~\") | Kinjalli's Sunwing, Curator Beastie, Seven Dwarves, Thalia, Heretic Cathar, Archon of Emeria (about 20) | `phrases::without_probe_card` (the same fix landed upstream for the \"enters\" patterns) |\n\
| \"[trigger], you may pay [cost]. If you do, return this card from your graveyard ...\" functioned from the battlefield, so it never triggered | Punishing Fire, Akoum Firebird, Asgardian Inspiration (about 150) | `trigger_zone` for may-pay triggers (also fixed upstream the same way) |\n\
| \"... deals damage to any other target\" with no earlier target lost \"other\": the object dealing the damage could be the target | Pain for All, Black Panther, Most Dangerous, Red Hulk | `damage_removal::any_other_than` |\n\
| \"with a single target\" swallowed the rest of the sentence: \"unless its controller pays {2}\" was dropped | Divert | qualifier only at the end; new pattern \"[effect] unless its controller pays [cost]\" (5 more cards supported) |\n\
| \"Equip ... Activate only once each turn.\" wasn't enforced (it was for crew) | Dark Knight's Greatsword and 3 others | `kw/equip.rs` |\n\
| \"[...] doesn't untap during your next untap step\" was compiled as its controller's next untap step, so a permanent another player gained control of stayed tapped in that player's untap step | Mogg Hollows, Arbalest Elite, Rhonas's Last Stand | `Duration::ThroughYourNextUntapStep` |\n\
| \"it\" after the card names itself (\"put a +1/+1 counter on ~. It gains flying\", \"sacrifice ~ and it deals 3 damage\", \"~ gets +1/+1 ... Untap it.\") was the triggering object or spell | Mogg Bombers, Machine Man, Model X-51, Blistercoil Weird, Aria of Flame, Vivi Ornitier (about 20) | `oracle_hardening_referents::note_object_last` |\n\
| \"another target creature\" after an earlier target object also excluded the source, which may be that other target | Itzquinth, Firstborn of Gishath, Rhino, Terrible Trampler (about 30) | `Builder::add_target` |\n\
| \"Each player discards a card. If you discarded a card this way, ...\" counted any player's discarded card (test in `tests/cards/roundtrip_renderer_gaps.rs`) | Fanatic of the Harrowing | `conditions_this_way::if_this_way`: only cards you own |\n\
| \"For each creature card exiled this way, each opponent loses 1 life and you gain 1 life\" scaled only the first half (test in `tests/cards/roundtrip_renderer_gaps.rs`) | Graveyard Trespasser // Graveyard Glutton | `hand_graveyard_grammar::scale` reaches each-player instructions |\n\
| \"Equipped creature gets +X/+X, where X is its mana value\" used the Equipment's mana value | Hedron Matrix | `r107_numbers::enchanted_gets_xy` (tests in `tests/cards/roundtrip_parser_bugs_1.rs`, as below) |\n\
| \"Target creature ... gets +X/+X ..., where X is its power\" used the source's power, not the target's | Winged Temple of Orazca (Fatal Frenzy now compiles) | `r107_numbers::where_x_is_parts` |\n\
| \"Whenever ... this turn, put three +1/+1 counters on it. It gains trample ...\": the second sentence was the creating ability's own instruction (about the source) | The Last Ronin | `patterns/this_turn_trigger_followup.rs` |\n\
| \"~ deals damage to any target equal to that card's mana value\": \"that card\" was the damage's target | Undying Flames | `damage_removal::damage_part` |\n\
| \"Whenever a creature enters from your graveyard\" triggered for creatures entering from any graveyard (also \"from your hand\") | Dredging Claw, Flayer of the Hatebound | `patterns/triggers.rs` (owned by you) |\n\
| \"As long as you have 30 or more life and an opponent has 10 or less life\" was read as one condition, \"you have 30 or less life\" (tests for this and the rows below in `tests/cards/roundtrip_tail_1.rs`) | Blood Baron of Vizkopa | `statics::parse_condition_core` (the whole rest) |\n\
| \"Choose up to one target creature. If it's suspected, exile it.\": \"it\" was the source | Agrus Kos, Spirit of Justice | `a701_action_triggers::designation_condition` (only \"~\"; \"it\" is the referent grammar's) |\n\
| \"Add {B} or {G} for each permanent destroyed this way\", \"Add X {G} or X {W}\": the count was dropped (one mana) | Culling Ritual, Muerra, Trash Tactician, Brigid, Doun's Mind | `effects::p_add_mana` (only symbols), `damage_removal_foreach::multiply`, `mana_production::add_amount` |\n\
| \"Search your library for up to X cards\" (quantity only) had to find exactly X (tests for this and the rows below in `tests/cards/roundtrip_clusters_2.rs`) | Diabolic Revelation | `patterns/card_flow_search.rs`: the search grammar |\n\
| \"Put a creature card exiled with ~ onto the battlefield. It gains haste\": \"it\" was the source | Yggdrasil, Rebirth Engine | `r600_linked.rs` |\n\
| \"Whenever ~ becomes blocked by a creature, it deals 2 damage to that creature\": \"it\" was the blocker | Acolyte of the Inferno | object resolver in `oracle/effects.rs` |\n\
| \"Each player mills cards equal to your Ring-bearer's power\" used each player's own Ring-bearer | One Ring to Rule Them All | `a701_actions::with_action_referent` |\n\
| \"during turns other than yours\" was \"during an opponent's turn\" (a teammate's turn didn't count) | Mesa Lynx | `patterns/statics.rs` `turn_condition` |\n\
| \"spells you cast from your graveyard cost less\" also reduced spells cast from another player's graveyard | Patrician Geist | `r601_cost_by_cast_zone.rs` |\n\
| \"enchanted creature or enchantment creature\" kept only the first phrase | Feast of Dreams | `phrases::two_phrases_same_head` |\n\
| \"Reveal a [card] you own from outside the game and put it into your hand\" put the card into the hand without revealing it (tests for this and the row below in `tests/cards/roundtrip_tail_2.rs`) | Golden Wish, Burning Wish, Living Wish, Fae of Wishes (8 cards) | `r108_cards.rs` |\n\
| \"Whenever a player attacks one of your opponents, that attacking player ...\": \"that attacking player\" was the attacked player (\"one or more of your opponents\" also triggered once per opponent attacked) | Jolene, the Plunder Queen, Combat Calligrapher, Ellie, Brick Master, Breena, the Demagogue | `choice_grammar.rs` (the attacking creatures' controller), `patterns/combat.rs` (once per attack) |\n\n\
Approximation the comparison accepts: \"cycle or discard\" triggers are compiled as discard \
triggers; cycling discards the card (CR 702.29a) and such a trigger triggers once for a \
cycled card (CR 702.29d), so the two are the same.\n\n";

/// Cards the coverage reports count: playable paper cards legal somewhere.
pub fn counted(c: &mtg_data::scryfall::ScryfallCard) -> bool {
    c.is_playable_card()
        && c.is_legal_somewhere()
        && !(c.games.iter().all(|g| g != "paper") && !c.games.is_empty())
}

/// A short description of how a mismatching card differs, used to group mismatches.
pub fn cluster_key(r: &CardCheck) -> String {
    if let Some(g) = r.gaps.first() {
        let g = g.split(['(', '{', ':']).next().unwrap_or(g).trim();
        return format!("gap: {g}");
    }
    let o: Vec<String> = r
        .unmatched_oracle
        .iter()
        .flat_map(|u| normalize_unit(u))
        .collect();
    let m: Vec<String> = r
        .unmatched_rendered
        .iter()
        .flat_map(|u| normalize_unit(u))
        .collect();
    if o.is_empty() {
        return "extra rendered ability".into();
    }
    if m.is_empty() {
        return "missing rendered ability".into();
    }
    let (a, b) = first_diff(&o, &m);
    let a = if a.is_empty() {
        "∅".to_string()
    } else {
        a.join(" ")
    };
    let b = if b.is_empty() {
        "∅".to_string()
    } else {
        b.join(" ")
    };
    format!("oracle `{a}` vs rendered `{b}`")
}

/// The first differing region of two token sequences (after their common prefix and
/// before their common suffix), each cut to a few tokens.
fn first_diff(a: &[String], b: &[String]) -> (Vec<String>, Vec<String>) {
    use mtg_engine::oracle::render::compare::token_eq;
    let p = a.iter().zip(b).take_while(|(x, y)| token_eq(x, y)).count();
    let (a2, b2) = (&a[p..], &b[p..]);
    let s = a2
        .iter()
        .rev()
        .zip(b2.iter().rev())
        .take_while(|(x, y)| token_eq(x, y))
        .count();
    let a3 = &a2[..a2.len() - s];
    let b3 = &b2[..b2.len() - s];
    let cut = |v: &[String]| v.iter().take(5).cloned().collect::<Vec<_>>();
    (cut(a3), cut(b3))
}

struct Options {
    card: Option<String>,
    filter: Option<String>,
    limit: usize,
    write: Option<String>,
    check: bool,
    ast: bool,
    clusters: usize,
    list: bool,
    all: bool,
}

fn parse(args: &[String]) -> Options {
    let mut o = Options {
        card: None,
        filter: None,
        limit: 20,
        write: None,
        check: false,
        ast: false,
        clusters: 0,
        list: false,
        all: false,
    };
    let mut i = 0;
    while i < args.len() {
        let next = args.get(i + 1).cloned();
        match args[i].as_str() {
            "--card" => {
                o.card = next.map(|s| s.to_lowercase());
                i += 1;
            }
            "--filter" => {
                o.filter = next.map(|s| s.to_lowercase());
                i += 1;
            }
            "--limit" => {
                o.limit = next.and_then(|s| s.parse().ok()).unwrap_or(20);
                i += 1;
            }
            "--write" => {
                o.write = next;
                i += 1;
            }
            "--clusters" => {
                o.clusters = next.and_then(|s| s.parse().ok()).unwrap_or(30);
                i += 1;
            }
            "--check" => o.check = true,
            "--ast" => o.ast = true,
            "--list" => o.list = true,
            "--all" => o.all = true,
            _ => {}
        }
        i += 1;
    }
    o
}

/// The round-trip results for every counted, fully supported card.
pub fn check_all(filter: impl Fn(&mtg_data::scryfall::ScryfallCard) -> bool) -> Vec<CardCheck> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for c in mtg_data::cards().iter() {
        if !counted(c) || !filter(c) || !seen.insert(c.name.clone()) {
            continue;
        }
        let def = CardDef::from_scryfall(c);
        if !def.is_fully_supported() {
            continue;
        }
        out.push(check_card(&def));
    }
    out
}

fn print_card(r: &CardCheck) {
    println!(
        "== {} — {}",
        r.name,
        if r.pass { "PASS" } else { "MISMATCH" }
    );
    for (i, (o, m)) in r.faces.iter().enumerate() {
        if r.faces.len() > 1 {
            println!("  -- face {i}");
        }
        for u in o {
            println!("  oracle:   {}", u.replace('\n', " / "));
        }
        for u in m {
            println!("  rendered: {}", u.replace('\n', " / "));
        }
    }
    for g in &r.gaps {
        println!("  gap: {g}");
    }
    if !r.pass {
        println!("  cluster: {}", cluster_key(r));
        for u in &r.unmatched_oracle {
            println!("  - {}", normalize_unit(u).join(" "));
        }
        for u in &r.unmatched_rendered {
            println!("  + {}", normalize_unit(u).join(" "));
        }
    }
}

pub fn run(args: &[String]) {
    // "--norm TEXT...": the normalized tokens of each text, as the comparison sees them.
    if args.first().is_some_and(|a| a == "--norm") {
        for t in &args[1..] {
            println!("{}", normalize_unit(t).join(" "));
        }
        return;
    }
    let o = parse(args);
    if o.ast {
        for c in mtg_data::cards().iter() {
            if o.card.as_ref().is_some_and(|n| c.name.to_lowercase() != *n) {
                continue;
            }
            let def = CardDef::from_scryfall(c);
            for f in &def.faces {
                println!("== {} ==\n{}", f.chars.name, f.chars.rules_text);
                for a in &f.chars.abilities {
                    println!("{:#?}", a.kind);
                }
            }
            break;
        }
        return;
    }
    let card = o.card.clone();
    let filter = o.filter.clone();
    let results = check_all(|c| {
        if card.as_ref().is_some_and(|n| c.name.to_lowercase() != *n) {
            return false;
        }
        if let Some(f) = &filter {
            let text = c
                .faces()
                .iter()
                .map(|f| f.oracle_text.clone().unwrap_or_default())
                .collect::<Vec<_>>()
                .join("\n")
                .to_lowercase();
            if !text.contains(f.as_str()) {
                return false;
            }
        }
        true
    });
    if o.card.is_some() {
        for r in &results {
            print_card(r);
        }
        if results.is_empty() {
            println!("no fully supported card by that name");
        }
        return;
    }
    let total = results.len();
    let passing: Vec<&CardCheck> = results.iter().filter(|r| r.pass).collect();
    // Cards with hand-written abilities are counted apart: what those abilities do is in
    // code, not in the ability language the renderer reads.
    let failing: Vec<&CardCheck> = results
        .iter()
        .filter(|r| !r.pass && !r.hand_written)
        .collect();
    let hand_written = results.iter().filter(|r| !r.pass && r.hand_written).count();
    let mut clusters: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for r in &failing {
        clusters.entry(cluster_key(r)).or_default().push(&r.name);
    }
    let mut cl: Vec<(&String, &Vec<&str>)> = clusters.iter().collect();
    cl.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));
    println!(
        "Round trip: {} of {total} fully supported cards pass ({:.1}%); {} mismatch; {hand_written} with hand-written abilities not checked",
        passing.len(),
        100.0 * passing.len() as f64 / total.max(1) as f64,
        failing.len()
    );
    if o.list {
        for r in failing.iter().take(o.limit) {
            print_card(r);
        }
    }
    if o.all {
        for r in &results {
            print_card(r);
        }
    }
    if o.clusters > 0 {
        for (k, v) in cl.iter().take(o.clusters) {
            println!(
                "{:5}  {k}  [{}]",
                v.len(),
                v.iter().take(3).cloned().collect::<Vec<_>>().join("; ")
            );
        }
    }
    if o.check {
        let path = super::repo_root().join(PASSING_LIST);
        let list = std::fs::read_to_string(&path).unwrap_or_default();
        let now: BTreeSet<&str> = passing.iter().map(|r| r.name.as_str()).collect();
        let checked: BTreeSet<&str> = results.iter().map(|r| r.name.as_str()).collect();
        let mut regressions = Vec::new();
        for name in list
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        {
            if !now.contains(name) && (o.filter.is_none() || checked.contains(name)) {
                regressions.push(name);
            }
        }
        if !regressions.is_empty() {
            println!("{} cards no longer pass the round trip:", regressions.len());
            for r in &regressions {
                println!("  {r}");
            }
            std::process::exit(1);
        }
        println!("all cards in {PASSING_LIST} still pass");
    }
    if let Some(path) = &o.write {
        let md = report(total, &passing, &failing, hand_written, &cl);
        std::fs::write(super::repo_root().join(path), md).expect("write report");
        // The list is by name: leave out a name the card database resolves to another card
        // (two cards with the same name, e.g. a digital-only one), which the engine test
        // would check instead.
        let db = mtg_engine::card::CardDb::global();
        let mut names: Vec<&str> = passing
            .iter()
            .map(|r| r.name.as_str())
            .filter(|n| db.get(n).is_some_and(|d| check_card(&d).pass))
            .collect();
        names.sort();
        let mut list = String::from(
            "# Cards whose compiled abilities render back to their Oracle text.\n# Generated by `mtg-tools roundtrip --write docs/ROUNDTRIP.md`; `roundtrip --check` and\n# the engine test `oracle_roundtrip` fail if one of them stops passing.\n",
        );
        for n in names {
            list.push_str(n);
            list.push('\n');
        }
        std::fs::write(super::repo_root().join(PASSING_LIST), list).expect("write list");
        println!("wrote {path} and {PASSING_LIST}");
    }
}

fn report(
    total: usize,
    passing: &[&CardCheck],
    failing: &[&CardCheck],
    hand_written: usize,
    clusters: &[(&String, &Vec<&str>)],
) -> String {
    use mtg_engine::oracle::render::compare::{EQUIVALENCES, IGNORED_WORDS};
    let squash = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut md = String::new();
    md.push_str("# Oracle round trip\n\n");
    md.push_str("Generated by `cargo run --release -p mtg-tools -- roundtrip --write docs/ROUNDTRIP.md`.\n\n");
    md.push_str("Every fully supported card's compiled abilities are rendered back into Oracle-style text (`crates/mtg-engine/src/oracle/render/`, written independently of the parser) and compared with the card's Oracle text after the same normalization on both sides (reminder text, self-references, case, punctuation, number words, grammatical number). A card *passes* when every ability matches. `docs/roundtrip-passing.txt` lists the passing cards; they must stay passing (`roundtrip --check`, engine test `oracle_roundtrip`).\n\n");
    md.push_str(&format!(
        "- Fully supported cards checked: **{total}**\n- Round-trip verified: **{}** ({:.1}%)\n- Mismatches: **{}**\n- With hand-written abilities (`crates/mtg-engine/src/cards/`), not checked: **{hand_written}**\n\n",
        passing.len(),
        100.0 * passing.len() as f64 / total.max(1) as f64,
        failing.len()
    ));
    md.push_str(FINDINGS);
    md.push_str("## Allowed equivalences\n\nApplied to both sides (`compare.rs`, `EQUIVALENCES`):\n\n| Pattern | Replacement | Why |\n|---|---|---|\n");
    for e in EQUIVALENCES {
        md.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            e.pattern.replace('|', "\\|"),
            e.replacement,
            squash(e.why).replace('|', "\\|")
        ));
    }
    md.push_str("\nSentence forms made uniform (`SENTENCE_FORMS`, `sentence_rewrites`):\n\n");
    for (form, why) in mtg_engine::oracle::render::compare::SENTENCE_FORMS {
        md.push_str(&format!("- {form}: {why}\n"));
    }
    md.push_str("\nIgnored words (`IGNORED_WORDS`):\n\n");
    for (w, why) in IGNORED_WORDS {
        md.push_str(&format!("- `{w}`: {}\n", squash(why)));
    }
    md.push_str(
        "\nQuantifiers that may be left out but never stand for one another \
         (`OPTIONAL_QUANTIFIERS`):\n\n",
    );
    for (w, why) in mtg_engine::oracle::render::compare::OPTIONAL_QUANTIFIERS {
        md.push_str(&format!("- `{w}`: {}\n", squash(why)));
    }
    md.push_str("\n## Mismatch clusters\n\n| Cards | Difference | Examples |\n|---:|---|---|\n");
    for (k, v) in clusters.iter().take(200) {
        md.push_str(&format!(
            "| {} | {} | {} |\n",
            v.len(),
            k.replace('|', "\\|"),
            v.iter().take(3).cloned().collect::<Vec<_>>().join("; ")
        ));
    }
    md.push_str("\n## Sample mismatches\n\n");
    for r in failing.iter().take(80) {
        md.push_str(&format!("### {}\n\n", r.name));
        for u in &r.unmatched_oracle {
            md.push_str(&format!("- Oracle: {}\n", u.replace('\n', " / ")));
        }
        for u in &r.unmatched_rendered {
            md.push_str(&format!("- Rendered: {}\n", u.replace('\n', " / ")));
        }
        for g in &r.gaps {
            md.push_str(&format!("- Gap: {g}\n"));
        }
        md.push('\n');
    }
    md
}
