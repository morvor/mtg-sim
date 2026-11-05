//! Rulings batch P026 — a token copy of a permanent copies exactly what was printed on it
//! (its copiable values, CR 707.2) plus the copy effect's own exceptions (CR 707.9b), and
//! nothing else: not whether it's tapped, its counters, what's attached to it, or the
//! non-copy effects that changed its power, toughness, types or color. Each copied
//! permanent is dressed up first (see [`dressed`]: tapped, equipped with Bonesplitter, two
//! +1/+1 counters, +3/+3 and green-blue until end of turn).

use crate::r_p026_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The copied permanent of a case: its name, printed colors and power/toughness.
struct Copied {
    name: &'static str,
    colors: ColorSet,
}

fn hill_giant() -> Copied {
    Copied {
        name: "Hill Giant",
        colors: ColorSet::single(Color::Red),
    }
}

/// One case: the copying card, what it copies, how the token is made (given the game and
/// the dressed-up permanent, returns the new token), the token's power and toughness after
/// the copy's exceptions, and a check of those exceptions.
struct Case {
    card: &'static str,
    copied: Copied,
    make: fn(&mut TestGame, ObjectId) -> ObjectId,
    pt: (i32, i32),
    tapped: bool,
    check: fn(&TestGame, ObjectId) -> bool,
}

fn run(cases: &[Case]) {
    for c in cases {
        supported(c.card);
        let mut t = TestGame::new(2);
        let original = dressed(&mut t, P0, c.copied.name);
        let tok = (c.make)(&mut t, original);
        let o = t.obj_now(tok);
        assert!(o.is_token(), "{}: not a token", c.card);
        assert_eq!(o.chars.name, c.copied.name, "{}", c.card);
        assert_eq!(t.pt(tok), c.pt, "{}: power and toughness", c.card);
        assert!(
            o.counters.values().all(|n| *n == 0),
            "{}: counters {:?}",
            c.card,
            o.counters
        );
        assert!(
            t.g.attachments_of(Entity::Object(o.id)).is_empty(),
            "{}: attachments",
            c.card
        );
        assert_eq!(o.tapped, c.tapped, "{}: tapped", c.card);
        assert!((c.check)(&t, tok), "{}: exceptions", c.card);
        // The original is still dressed up: the copy didn't take anything from it.
        if t.on_battlefield(original) {
            assert_eq!(t.counters(original, counters::PLUS1), 2, "{}", c.card);
        }
    }
}

fn colors_are(t: &TestGame, id: ObjectId, c: ColorSet) -> bool {
    t.obj_now(id).chars.colors == c
}

/// The token is the copied permanent's printed colors (not green-blue).
fn printed_colors(t: &TestGame, id: ObjectId, c: &Copied) -> bool {
    colors_are(t, id, c.colors)
}

fn red_giant(t: &TestGame, id: ObjectId) -> bool {
    printed_colors(t, id, &hill_giant()) && has_subtype(t, id, "Giant")
}

fn spell(
    t: &mut TestGame,
    name: &str,
    cost: &str,
    target: ObjectId,
    modes: Option<&[usize]>,
) -> ObjectId {
    one(cast_for_tokens(
        t,
        P0,
        name,
        cost,
        &[obj(target)],
        modes,
        None,
    ))
}

#[test]
fn token_copy_spells_copy_only_the_printed_creature() {
    cr!("707.2", "707.9b", "111.4");
    ruling!(
        "Sublime Epiphany",
        "The token copies exactly what is printed on the creature and nothing else (unless that creature is a token or is copying something else; see below)."
    );
    ruling!(
        "Quasiduplicate",
        "The token copies exactly what is printed on the creature and nothing else (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Stolen Identity",
        "The token copies exactly what was printed on the original artifact or creature and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    ruling!(
        "Ember Island Production",
        "The token copies exactly what was printed on the original creature and nothing else (unless that creature is copying something else or is a token; see below), with the listed exceptions."
    );
    ruling!(
        "See Double",
        "The token copies exactly what was printed on the original creature and nothing else (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Cackling Counterpart",
        "The token copies exactly what was printed on the original creature and nothing else (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Mirage Mockery",
        "The token copies exactly what was printed on the original creature and nothing else (unless that permanent is copying something else or it is a token; see below)."
    );
    ruling!(
        "Three Steps Ahead",
        "The token copies exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    ruling!(
        "Electroduplicate",
        "The token created by Electroduplicate copies exactly what was printed on the original creature and nothing else, with the listed exceptions (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Kindle the Inner Flame",
        "The token created by Kindle the Inner Flame copies exactly what was printed on the original creature and nothing else, with the listed exceptions (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Echocasting Symposium",
        "The token created by Echocasting Symposium copies exactly what was printed on the original creature and nothing else (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Season of Weaving",
        "The token from the second mode copies exactly what was printed on the original creature or artifact and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    ruling!(
        "Tempt with Reflections",
        "The tokens copy exactly what was printed on the original creature and nothing else (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Twinflame",
        "The tokens copy exactly what was printed on the original creature and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    ruling!(
        "Aggressive Biomancy",
        "The tokens copy exactly what was printed on the original creature and nothing else, with the listed exception (unless that creature is itself copying something else; see below)."
    );
    ruling!(
        "Devastating Onslaught",
        "The tokens copy exactly what was printed on the original permanent and nothing else (unless that permanent is itself copying something else; see below)."
    );
    ruling!(
        "Supplant Form",
        "The token copies exactly what was printed on the creature and nothing else (unless that creature was copying something else or was a token; see below)."
    );
    let cases = [
        Case {
            card: "Sublime Epiphany",
            copied: hill_giant(),
            make: |t, g| spell(t, "Sublime Epiphany", "{4}{U}{U}", g, Some(&[3])),
            pt: (3, 3),
            tapped: false,
            check: red_giant,
        },
        Case {
            card: "Quasiduplicate",
            copied: hill_giant(),
            make: |t, g| spell(t, "Quasiduplicate", "{1}{U}{U}", g, None),
            pt: (3, 3),
            tapped: false,
            check: red_giant,
        },
        Case {
            card: "Stolen Identity",
            copied: hill_giant(),
            make: |t, g| spell(t, "Stolen Identity", "{4}{U}{U}", g, None),
            pt: (3, 3),
            tapped: false,
            check: red_giant,
        },
        // "...except it's not legendary and it's a 4/4 Hero in addition to its other types."
        Case {
            card: "Ember Island Production",
            copied: hill_giant(),
            make: |t, g| spell(t, "Ember Island Production", "{3}{U}{U}", g, Some(&[0])),
            pt: (4, 4),
            tapped: false,
            check: |t, id| red_giant(t, id) && has_subtype(t, id, "Hero"),
        },
        Case {
            card: "See Double",
            copied: hill_giant(),
            make: |t, g| spell(t, "See Double", "{2}{U}{U}", g, Some(&[1])),
            pt: (3, 3),
            tapped: false,
            check: red_giant,
        },
        Case {
            card: "Cackling Counterpart",
            copied: hill_giant(),
            make: |t, g| spell(t, "Cackling Counterpart", "{1}{U}{U}", g, None),
            pt: (3, 3),
            tapped: false,
            check: red_giant,
        },
        // "• Create a token that's a copy of target nonartifact creature you control."
        Case {
            card: "Mirage Mockery",
            copied: hill_giant(),
            make: |t, g| spell(t, "Mirage Mockery", "{2}{U}", g, Some(&[1])),
            pt: (3, 3),
            tapped: false,
            check: red_giant,
        },
        // Spree: "+ {3} — Create a token that's a copy of target artifact or creature you
        // control."
        Case {
            card: "Three Steps Ahead",
            copied: hill_giant(),
            make: |t, g| spell(t, "Three Steps Ahead", "{3}{U}", g, Some(&[1])),
            pt: (3, 3),
            tapped: false,
            check: red_giant,
        },
        // "...except it has haste and "At the beginning of the end step, sacrifice this
        // token.""
        Case {
            card: "Electroduplicate",
            copied: hill_giant(),
            make: |t, g| spell(t, "Electroduplicate", "{2}{R}", g, None),
            pt: (3, 3),
            tapped: false,
            check: |t, id| red_giant(t, id) && has_kw(t, id, KeywordKind::Haste),
        },
        Case {
            card: "Kindle the Inner Flame",
            copied: hill_giant(),
            make: |t, g| spell(t, "Kindle the Inner Flame", "{3}{R}", g, None),
            pt: (3, 3),
            tapped: false,
            check: |t, id| red_giant(t, id) && has_kw(t, id, KeywordKind::Haste),
        },
        // "Target player creates a token that's a copy of target creature you control."
        Case {
            card: "Echocasting Symposium",
            copied: hill_giant(),
            make: |t, g| {
                one(cast_for_tokens(
                    t,
                    P0,
                    "Echocasting Symposium",
                    "{4}{U}{U}",
                    &[Entity::Player(P0), obj(g)],
                    None,
                    None,
                ))
            },
            pt: (3, 3),
            tapped: false,
            check: red_giant,
        },
        // "{P}{P} — Choose an artifact or creature you control. Create a token that's a
        // copy of it."
        Case {
            card: "Season of Weaving",
            copied: hill_giant(),
            make: |t, g| {
                t.answer_choose(P0, &[obj(g)]);
                one(cast_for_tokens(
                    t,
                    P0,
                    "Season of Weaving",
                    "{4}{U}{U}",
                    &[],
                    Some(&[1]),
                    None,
                ))
            },
            pt: (3, 3),
            tapped: false,
            check: red_giant,
        },
        // The opponent declines the tempting offer.
        Case {
            card: "Tempt with Reflections",
            copied: hill_giant(),
            make: |t, g| {
                t.answer_yes(P1, false);
                spell(t, "Tempt with Reflections", "{3}{U}", g, None)
            },
            pt: (3, 3),
            tapped: false,
            check: red_giant,
        },
        Case {
            card: "Twinflame",
            copied: hill_giant(),
            make: |t, g| spell(t, "Twinflame", "{1}{R}", g, None),
            pt: (3, 3),
            tapped: false,
            check: |t, id| red_giant(t, id) && has_kw(t, id, KeywordKind::Haste),
        },
        // X = 1: one token, with "When this token enters, it fights up to one target
        // creature you don't control."
        Case {
            card: "Aggressive Biomancy",
            copied: hill_giant(),
            make: |t, g| {
                one(cast_for_tokens(
                    t,
                    P0,
                    "Aggressive Biomancy",
                    "{2}{G}{U}",
                    &[obj(g)],
                    None,
                    Some(1),
                ))
            },
            pt: (3, 3),
            tapped: false,
            check: |t, id| red_giant(t, id) && abilities_with(t, id, "fights") == 1,
        },
        // X = 1; "Those tokens gain haste until end of turn."
        Case {
            card: "Devastating Onslaught",
            copied: hill_giant(),
            make: |t, g| {
                one(cast_for_tokens(
                    t,
                    P0,
                    "Devastating Onslaught",
                    "{2}{R}",
                    &[obj(g)],
                    None,
                    Some(1),
                ))
            },
            pt: (3, 3),
            tapped: false,
            check: |t, id| red_giant(t, id) && has_kw(t, id, KeywordKind::Haste),
        },
        // "Return target creature to its owner's hand. You create a token that's a copy of
        // that creature." It copies the creature as it last existed.
        Case {
            card: "Supplant Form",
            copied: hill_giant(),
            make: |t, g| {
                let tok = spell(t, "Supplant Form", "{4}{U}{U}", g, None);
                assert!(t.in_hand(P0, "Hill Giant"));
                tok
            },
            pt: (3, 3),
            tapped: false,
            check: red_giant,
        },
    ];
    run(&cases);
}

/// A Command's first mode copies `copied` (of the Command's kindred type); `other` is the
/// second mode's index and its target.
fn command(
    t: &mut TestGame,
    name: &str,
    cost: &str,
    what: ObjectId,
    other: (usize, Entity),
) -> ObjectId {
    one(cast_for_tokens(
        t,
        P0,
        name,
        cost,
        &[obj(what), other.1],
        Some(&[0, other.0]),
        None,
    ))
}

#[test]
fn command_tokens_copy_only_the_printed_permanent() {
    cr!("707.2", "700.2");
    ruling!(
        "Ashling's Command",
        "The token created by the first mode of Ashling's Command copies exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    ruling!(
        "Brigid's Command",
        "The token created by the first mode of Brigid's Command copies exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    ruling!(
        "Grub's Command",
        "The token created by the first mode of Grub's Command copies exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    ruling!(
        "Sygg's Command",
        "The token created by the first mode of Sygg's Command copies exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    ruling!(
        "Trystan's Command",
        "The token created by the first mode of Trystan's Command copies exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    let blue = ColorSet::single(Color::Blue);
    let cases = [
        // Air Elemental: a blue 4/4 Elemental with flying. "• Target player draws two
        // cards."
        Case {
            card: "Ashling's Command",
            copied: Copied {
                name: "Air Elemental",
                colors: blue,
            },
            make: |t, e| {
                command(
                    t,
                    "Ashling's Command",
                    "{3}{U}{R}",
                    e,
                    (1, Entity::Player(P0)),
                )
            },
            pt: (4, 4),
            tapped: false,
            check: |t, id| colors_are(t, id, ColorSet::single(Color::Blue)),
        },
        // Goldmeadow Harrier: a white 1/1 Kithkin. "• Target player creates a 1/1 green and
        // white Kithkin creature token." (the opponent)
        Case {
            card: "Brigid's Command",
            copied: Copied {
                name: "Goldmeadow Harrier",
                colors: ColorSet::single(Color::White),
            },
            make: |t, k| {
                command(
                    t,
                    "Brigid's Command",
                    "{1}{G}{W}",
                    k,
                    (1, Entity::Player(P1)),
                )
            },
            pt: (1, 1),
            tapped: false,
            check: |t, id| colors_are(t, id, ColorSet::single(Color::White)),
        },
        // Goblin Piker: a red 2/1 Goblin. "• Target player mills five cards, ..."
        Case {
            card: "Grub's Command",
            copied: Copied {
                name: "Goblin Piker",
                colors: ColorSet::single(Color::Red),
            },
            make: |t, g| command(t, "Grub's Command", "{3}{B}{R}", g, (3, Entity::Player(P1))),
            pt: (2, 1),
            tapped: false,
            check: |t, id| colors_are(t, id, ColorSet::single(Color::Red)),
        },
        // Merfolk of the Pearl Trident: a blue 1/1 Merfolk. "• Target player draws a card."
        Case {
            card: "Sygg's Command",
            copied: Copied {
                name: "Merfolk of the Pearl Trident",
                colors: blue,
            },
            make: |t, m| command(t, "Sygg's Command", "{1}{W}{U}", m, (2, Entity::Player(P0))),
            pt: (1, 1),
            tapped: false,
            check: |t, id| colors_are(t, id, ColorSet::single(Color::Blue)),
        },
        // Llanowar Elves: a green 1/1 Elf. "• Creatures target player controls get +3/+3
        // until end of turn. Untap them." (the opponent)
        Case {
            card: "Trystan's Command",
            copied: Copied {
                name: "Llanowar Elves",
                colors: ColorSet::single(Color::Green),
            },
            make: |t, e| {
                command(
                    t,
                    "Trystan's Command",
                    "{4}{B}{G}",
                    e,
                    (3, Entity::Player(P1)),
                )
            },
            pt: (1, 1),
            tapped: false,
            check: |t, id| colors_are(t, id, ColorSet::single(Color::Green)),
        },
    ];
    run(&cases);
}

fn ability(t: &mut TestGame, source: &str, cost: &str, target: ObjectId) -> ObjectId {
    supported(source);
    let s = t.battlefield(P0, source);
    lands_for(t, P0, cost);
    one(activate_for_tokens(
        t,
        P0,
        s,
        "Create a token",
        &[obj(target)],
    ))
}

#[test]
fn token_copy_abilities_copy_only_the_printed_creature() {
    cr!("707.2", "707.9b", "602.2");
    ruling!(
        "Tempestra, Dame of Games",
        "The token copies exactly what was printed on the original creature, with the listed exception (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Saheeli, the Sun's Brilliance",
        "The token copies exactly what was printed on the original permanent, with the listed exceptions (unless that permanent is copying something else or is a token; see below)."
    );
    ruling!(
        "Jaxis, the Troublemaker",
        "The token copies exactly what was printed on the original creature and nothing else (unless it's copying a creature that's a token or that's copying something else; see below)."
    );
    ruling!(
        "Orthion, Hero of Lavabrink",
        "The tokens each copy exactly what was printed on the original creature and nothing else (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki",
        "The token created by Reflection of Kiki-Jiki copies exactly what was printed on the original creature (except that the copy also has haste) and nothing else (unless it's copying a creature that's a token or that's copying something else; see below)."
    );
    let cases = [
        // "{2}{R}, {T}, Sacrifice an artifact: ... except it isn't legendary. It gains
        // haste."
        Case {
            card: "Tempestra, Dame of Games",
            copied: hill_giant(),
            make: |t, g| {
                let thopter = t.battlefield(P0, "Ornithopter");
                t.answer_choose(P0, &[obj(thopter)]);
                let tok = ability(t, "Tempestra, Dame of Games", "{2}{R}", g);
                assert!(!t.on_battlefield(thopter));
                tok
            },
            pt: (3, 3),
            tapped: false,
            check: |t, id| red_giant(t, id) && has_kw(t, id, KeywordKind::Haste),
        },
        // "...except it's an artifact in addition to its other types. It gains haste."
        Case {
            card: "Saheeli, the Sun's Brilliance",
            copied: hill_giant(),
            make: |t, g| ability(t, "Saheeli, the Sun's Brilliance", "{U}{R}", g),
            pt: (3, 3),
            tapped: false,
            check: |t, id| {
                red_giant(t, id)
                    && t.obj_now(id).is(CardType::Artifact)
                    && has_kw(t, id, KeywordKind::Haste)
            },
        },
        // "{R}, {T}, Discard a card: ... It gains haste and "When this token dies, draw a
        // card.""
        Case {
            card: "Jaxis, the Troublemaker",
            copied: hill_giant(),
            make: |t, g| {
                t.hand(P0, "Grizzly Bears");
                ability(t, "Jaxis, the Troublemaker", "{R}", g)
            },
            pt: (3, 3),
            tapped: false,
            check: |t, id| red_giant(t, id) && has_kw(t, id, KeywordKind::Haste),
        },
        Case {
            card: "Orthion, Hero of Lavabrink",
            copied: hill_giant(),
            make: |t, g| ability(t, "Orthion, Hero of Lavabrink", "{1}{R}", g),
            pt: (3, 3),
            tapped: false,
            check: |t, id| red_giant(t, id) && has_kw(t, id, KeywordKind::Haste),
        },
        Case {
            card: "Fable of the Mirror-Breaker // Reflection of Kiki-Jiki",
            copied: hill_giant(),
            make: |t, g| {
                let r = reflection(t);
                lands_for(t, P0, "{1}");
                one(activate_for_tokens(t, P0, r, "Create a token", &[obj(g)]))
            },
            pt: (3, 3),
            tapped: false,
            check: |t, id| red_giant(t, id) && has_kw(t, id, KeywordKind::Haste),
        },
    ];
    run(&cases);
}

/// `name` (on the battlefield for P0) triggers when `entering` enters; P0 pays its
/// optional cost (`cost`), and `entering` is dressed up in response. Returns the token.
fn enters_trigger(t: &mut TestGame, name: &str, cost: &str, entering: &str) -> ObjectId {
    supported(name);
    t.battlefield(P0, name);
    lands_for(t, P0, cost);
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    let e = t.enter(P0, entering);
    t.settle();
    assert!(t.stack_len() >= 1, "{name}: no trigger");
    dress(t, P0, e);
    t.resolve_all();
    one(new_tokens(t, P0, &before)
        .into_iter()
        .filter(|id| name_of(t, *id) == entering)
        .collect())
}

/// Runs `make` on a fresh game (the case's dressed-up permanent is replaced by one made by
/// the scenario itself) and asserts the token copies only the printed `copied`.
fn check_scenario(
    card: &str,
    copied: &Copied,
    pt: (i32, i32),
    tapped: bool,
    tok: ObjectId,
    t: &TestGame,
) {
    let o = t.obj_now(tok);
    assert!(o.is_token(), "{card}");
    assert_eq!(o.chars.name, copied.name, "{card}");
    assert!(
        printed_colors(t, tok, copied),
        "{card}: colors {:?}",
        o.chars.colors
    );
    assert_eq!(t.pt(tok), pt, "{card}: power and toughness");
    assert!(o.counters.values().all(|n| *n == 0), "{card}: counters");
    assert!(
        t.g.attachments_of(Entity::Object(o.id)).is_empty(),
        "{card}: attachments"
    );
    assert_eq!(o.tapped, tapped, "{card}: tapped");
}

#[test]
fn token_copy_triggers_copy_only_the_printed_permanent() {
    cr!("707.2", "603.2", "603.3");
    ruling!(
        "Cadric, Soul Kindler",
        "The token copies exactly what is printed on the permanent and nothing else (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Inalla, Archmage Ritualist",
        "The token copies exactly what was printed on the creature and nothing else (unless that creature was copying something else; see below)."
    );
    ruling!(
        "Flameshadow Conjuring",
        "The token copies exactly what's printed on the original creature and nothing else (unless that creature is copying something else; see below)."
    );
    // Cadric: "Whenever another nontoken legendary permanent you control enters, you may
    // pay {1}. If you do, create a token that's a copy of it."
    let mut t = TestGame::new(2);
    let tok = enters_trigger(
        &mut t,
        "Cadric, Soul Kindler",
        "{1}",
        "Isamaru, Hound of Konda",
    );
    let isamaru = Copied {
        name: "Isamaru, Hound of Konda",
        colors: ColorSet::single(Color::White),
    };
    check_scenario("Cadric, Soul Kindler", &isamaru, (2, 2), false, tok, &t);
    // Inalla (eminence, on the battlefield): another nontoken Wizard enters. Prodigal
    // Sorcerer is a blue 1/1 Human Wizard.
    let mut t = TestGame::new(2);
    let tok = enters_trigger(
        &mut t,
        "Inalla, Archmage Ritualist",
        "{1}",
        "Prodigal Sorcerer",
    );
    let sorcerer = Copied {
        name: "Prodigal Sorcerer",
        colors: ColorSet::single(Color::Blue),
    };
    check_scenario(
        "Inalla, Archmage Ritualist",
        &sorcerer,
        (1, 1),
        false,
        tok,
        &t,
    );
    assert!(has_kw(&t, tok, KeywordKind::Haste));
    // Flameshadow Conjuring: a nontoken creature enters; pay {R}.
    let mut t = TestGame::new(2);
    let tok = enters_trigger(&mut t, "Flameshadow Conjuring", "{R}", "Hill Giant");
    check_scenario(
        "Flameshadow Conjuring",
        &hill_giant(),
        (3, 3),
        false,
        tok,
        &t,
    );
    assert!(has_kw(&t, tok, KeywordKind::Haste));
}

#[test]
fn vesuvan_duplimancy_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b", "603.3");
    ruling!(
        "Vesuvan Duplimancy",
        "The token copies exactly what was printed on the original artifact or creature and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    // Giant Growth targets the dressed-up Giant; the trigger resolves first.
    supported("Vesuvan Duplimancy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vesuvan Duplimancy");
    let giant = dressed(&mut t, P0, "Hill Giant");
    let before = t.g.battlefield.clone();
    crate::r_p021_common::giant_growth(&mut t, P0, giant);
    let tok = one(new_tokens(&t, P0, &before));
    check_scenario("Vesuvan Duplimancy", &hill_giant(), (3, 3), false, tok, &t);
    assert!(!legendary(&t, tok));
    assert_eq!(t.pt(giant), (13, 11));
}

#[test]
fn benthic_anomaly_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b", "707.9d");
    ruling!(
        "Benthic Anomaly",
        "The token copies exactly what was printed on the original card and nothing else, with the listed exceptions (unless that creature is itself copying something else; see below)."
    );
    // "When you cast this spell, for each opponent, choose a creature that player
    // controls. Create a token that's a copy of one of those creatures, except its power is
    // equal to the total power of those creatures, its toughness is equal to the total
    // toughness of those creatures, and it's a colorless Eldrazi creature."
    supported("Benthic Anomaly");
    let mut t = TestGame::new(2);
    let giant = dressed(&mut t, P1, "Hill Giant");
    lands_for(&mut t, P0, "{6}{U}");
    let card = t.hand(P0, "Benthic Anomaly");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_choose(P0, &[obj(giant)]);
    let before = t.g.battlefield.clone();
    t.cast(P0, card).go();
    t.resolve();
    let tok = one(new_tokens(&t, P0, &before));
    let o = t.obj_now(tok);
    assert_eq!(o.chars.name, "Hill Giant");
    assert!(o.chars.colors.is_colorless());
    assert!(has_subtype(&t, tok, "Eldrazi") && !has_subtype(&t, tok, "Giant"));
    // Its power and toughness are the Giant's total (10/8), but no counters or
    // attachments came with it.
    assert_eq!(t.pt(tok), (10, 8));
    assert!(fresh(&t, tok));
}

#[test]
fn felhide_spiritbinder_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b");
    ruling!(
        "Felhide Spiritbinder",
        "The token copies exactly what was printed on the original creature (except that the copy is also an enchantment) and nothing else (unless that permanent is copying something else or is a token; see below)."
    );
    // Inspired — "Whenever this creature becomes untapped, you may pay {1}{R}. If you do,
    // create a token that's a copy of another target creature, except it's an enchantment
    // in addition to its other types."
    supported("Felhide Spiritbinder");
    let mut t = TestGame::new(2);
    let binder = t.battlefield(P0, "Felhide Spiritbinder");
    let giant = dressed(&mut t, P1, "Hill Giant");
    lands_for(&mut t, P0, "{1}{R}");
    t.g.tap(binder);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    let before = t.g.battlefield.clone();
    t.g.untap(binder);
    t.g.flush_events();
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    check_scenario(
        "Felhide Spiritbinder",
        &hill_giant(),
        (3, 3),
        false,
        tok,
        &t,
    );
    assert!(t.obj_now(tok).is(CardType::Enchantment) && is_creature(&t, tok));
}

#[test]
fn hofri_token_copies_only_the_printed_creature() {
    cr!("707.2", "707.9b", "608.2h");
    ruling!(
        "Hofri Ghostforge",
        "The token copies exactly what was printed on the original creature and nothing else (unless the creature was copying something else; see below)."
    );
    // "...create a token that's a copy of that creature, except it's a Spirit in addition
    // to its other types..." Hofri gives Spirits +1/+1, trample and haste.
    supported("Hofri Ghostforge");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hofri Ghostforge");
    let giant = dressed(&mut t, P0, "Hill Giant");
    let before = t.g.battlefield.clone();
    kill(&mut t, giant);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    check_scenario("Hofri Ghostforge", &hill_giant(), (4, 4), false, tok, &t);
    assert!(has_subtype(&t, tok, "Spirit") && has_subtype(&t, tok, "Giant"));
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn ashling_the_limitless_token_copies_only_the_printed_elemental() {
    cr!("707.2", "608.2h");
    ruling!(
        "Ashling, the Limitless",
        "The token created by Ashling's last ability copies exactly what was printed on the original permanent and nothing else (unless that creature was copying something else; see below)."
    );
    // "Whenever you sacrifice a nontoken Elemental, create a token that's a copy of it."
    // Village Rites sacrifices a dressed-up Air Elemental (a blue 4/4).
    supported("Ashling, the Limitless");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashling, the Limitless");
    let elemental = dressed(&mut t, P0, "Air Elemental");
    lands_for(&mut t, P0, "{B}");
    let rites = t.hand(P0, "Village Rites");
    t.answer_choose(P0, &[obj(elemental)]);
    let before = t.g.battlefield.clone();
    t.cast(P0, rites).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Air Elemental"));
    let tok = one(new_tokens(&t, P0, &before));
    let air = Copied {
        name: "Air Elemental",
        colors: ColorSet::single(Color::Blue),
    };
    check_scenario("Ashling, the Limitless", &air, (4, 4), false, tok, &t);
}

#[test]
fn twilight_diviner_token_copies_only_the_printed_permanent() {
    cr!("707.2", "603.4");
    ruling!(
        "Twilight Diviner",
        "The token created by Twilight Diviner's last ability copies exactly what was printed on the original permanent and nothing else (unless that permanent is copying something else; see below)."
    );
    // Zombify returns the Giant from P0's graveyard; it's dressed up in response to the
    // trigger.
    supported("Twilight Diviner");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twilight Diviner");
    let card = t.graveyard(P0, "Hill Giant");
    lands_for(&mut t, P0, "{3}{B}");
    let zombify = t.hand(P0, "Zombify");
    t.cast(P0, zombify).target(obj(card)).go();
    t.resolve();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(t.stack_len(), 1);
    let before = t.g.battlefield.clone();
    dress(&mut t, P0, giant);
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    check_scenario("Twilight Diviner", &hill_giant(), (3, 3), false, tok, &t);
}

#[test]
fn attacking_token_copies_copy_only_the_printed_creature() {
    cr!("707.2", "508.4", "701.43a");
    ruling!(
        "Grub, Storied Matriarch // Grub, Notorious Auntie",
        "The token created by Grub, Notorious Auntie's second ability copies exactly what was printed on the original creature and nothing else, with the listed exception (unless that creature is copying something else or is a token; see below)."
    );
    ruling!(
        "Sandstorm Crasher",
        "The token created by the reflexive triggered ability copies exactly what was printed on the original creature and nothing else (unless that creature is copying something else or is a token; see below)."
    );
    // Grub, Notorious Auntie: "Whenever Grub attacks, you may blight 1. If you do, create
    // a tapped and attacking token that's a copy of the blighted creature". The Giant gets
    // the -1/-1 counter.
    supported("Grub, Storied Matriarch // Grub, Notorious Auntie");
    let mut t = TestGame::new(2);
    let grub = t.battlefield(P0, "Grub, Storied Matriarch // Grub, Notorious Auntie");
    mtg_engine::dfc::transform(&mut t.g, grub);
    t.g.recompute();
    let giant = dressed(&mut t, P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    let before = t.g.battlefield.clone();
    t.attack(&[(grub, Entity::Player(P1))], &[]);
    let tok = one(new_tokens(&t, P0, &before));
    check_scenario(
        "Grub, Storied Matriarch // Grub, Notorious Auntie",
        &hill_giant(),
        (3, 3),
        true,
        tok,
        &t,
    );
    // Sandstorm Crasher: exerted as it attacks, it copies the dressed-up Giant.
    supported("Sandstorm Crasher");
    let mut t = TestGame::new(2);
    let crasher = t.battlefield(P0, "Sandstorm Crasher");
    let giant = dressed(&mut t, P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    let before = t.g.battlefield.clone();
    t.attack(&[(crasher, Entity::Player(P1))], &[]);
    let tok = one(new_tokens(&t, P0, &before));
    check_scenario("Sandstorm Crasher", &hill_giant(), (3, 3), true, tok, &t);
}

#[test]
fn rionya_tokens_copy_only_the_printed_creature() {
    cr!("707.2", "107.3a");
    ruling!(
        "Rionya, Fire Dancer",
        "The tokens copy exactly what is printed on the creature and nothing else (unless that creature is a token or is copying something else; see below)."
    );
    // "At the beginning of combat on your turn, create X tokens that are copies of another
    // target creature you control, where X is one plus the number of instant and sorcery
    // spells you've cast this turn."
    supported("Rionya, Fire Dancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rionya, Fire Dancer");
    let giant = dressed(&mut t, P0, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(giant)]);
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    check_scenario("Rionya, Fire Dancer", &hill_giant(), (3, 3), false, tok, &t);
    assert!(has_kw(&t, tok, KeywordKind::Haste));
}
