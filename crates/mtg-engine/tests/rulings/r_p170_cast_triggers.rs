//! Rulings batch P170 — abilities that trigger on casting a (noncreature) spell go on the
//! stack above that spell and resolve first (CR 601.2i, 603.3), whether or not the spell
//! is still there: they resolve even if it's countered or otherwise leaves the stack.

use crate::r_p170_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// A case: the permanent (set up by `setup`, which returns the objects `check` looks at,
/// the triggering permanent first, and queues the trigger's answers), the spell P0 casts,
/// and the check of the trigger's effect once it alone has resolved.
struct Case {
    card: &'static str,
    spell: &'static str,
    setup: fn(&mut TestGame) -> Vec<ObjectId>,
    check: fn(&TestGame, &[ObjectId]),
}

fn wizard_token(t: &mut TestGame) -> Vec<ObjectId> {
    // Mysidian Elder: "When this creature enters, create a 0/1 black Wizard creature
    // token with "Whenever you cast a noncreature spell, this token deals 1 damage to
    // each opponent.""
    t.enter(P0, "Mysidian Elder");
    t.resolve_all();
    let w = tokens_with_subtype(t, P0, "Wizard");
    assert_eq!(w.len(), 1);
    w
}

fn hero_with(t: &mut TestGame, equipment: &str) -> Vec<ObjectId> {
    t.enter(P0, equipment);
    t.resolve_all();
    let h = tokens_with_subtype(t, P0, "Hero");
    assert_eq!(h.len(), 1);
    h
}

fn run(cases: &[Case]) {
    for c in cases {
        supported(c.card);
        for how in ALL_LEAVES {
            let mut t = TestGame::new(2);
            let objs = (c.setup)(&mut t);
            let spell = cast_card(&mut t, P0, c.spell, &[]);
            assert!(trigger_on_top(&t), "{}: the trigger is on top", c.card);
            assert_eq!(t.g.stack[t.g.stack.len() - 2], spell, "{}", c.card);
            leave(&mut t, spell, P0, how);
            t.resolve();
            (c.check)(&t, &objs);
            if how == Leaves::Stays {
                // The spell is still waiting on the stack.
                assert_eq!(t.zone(spell), Zone::Stack, "{} {how:?}", c.card);
            }
        }
    }
}

#[test]
fn noncreature_cast_triggers_resolve_first_even_if_the_spell_leaves() {
    cr!("603.3", "601.2i", "608.2", "701.6a");
    ruling!(
        "Primeval Bounty",
        "Primeval Bounty's first and second abilities will each resolve before the spell that caused them to trigger. They will resolve even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Prompto Argentum",
        "Prompto's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Ral, Crackling Wit",
        "Ral's first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Sahagin",
        "Sahagin's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Shambling Cie'th",
        "Shambling Cie'th's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Shantotto, Tactician Magician",
        "Shantotto's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Sky Theater Strix",
        "Sky Theater Strix’s last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Thancred Waters",
        "Thancred Waters's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "The Prima Vista",
        "The Prima Vista's second ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Mysidian Elder",
        "The Wizard token's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Red Mage's Rapier",
        "The ability granted by Red Mage's Rapier resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Black Mage's Rod",
        "The granted ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Vivi Ornitier",
        "Vivi Ornitier's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Stormchaser's Talent",
        "The level 3 class ability of Stormchaser's Talent resolves before the spell or ability that caused it to trigger. It resolves even if that spell is countered."
    );
    let cases = [
        Case {
            // "Whenever you cast a noncreature spell, put three +1/+1 counters on target
            // creature you control."
            card: "Primeval Bounty",
            spell: "Divination",
            setup: |t| {
                t.battlefield(P0, "Primeval Bounty");
                let bears = t.battlefield(P0, "Grizzly Bears");
                t.answer_targets(P0, &[obj(bears)]);
                vec![bears]
            },
            check: |t, o| assert_eq!(t.counters(o[0], "+1/+1"), 3),
        },
        Case {
            // "Whenever you cast a creature spell, create a 3/3 green Beast creature token."
            card: "Primeval Bounty",
            spell: "Grizzly Bears",
            setup: |t| vec![t.battlefield(P0, "Primeval Bounty")],
            check: |t, _| assert_eq!(tokens_with_subtype(t, P0, "Beast").len(), 1),
        },
        Case {
            // "... if at least four mana was spent to cast it, create a Treasure token."
            card: "Prompto Argentum",
            spell: "Concentrate",
            setup: |t| vec![t.battlefield(P0, "Prompto Argentum")],
            check: |t, _| assert_eq!(tokens_with_subtype(t, P0, "Treasure").len(), 1),
        },
        Case {
            card: "Ral, Crackling Wit",
            spell: "Divination",
            setup: |t| {
                let ral = t.battlefield(P0, "Ral, Crackling Wit");
                assert_eq!(t.counters(ral, "loyalty"), 4);
                vec![ral]
            },
            check: |t, o| assert_eq!(t.counters(o[0], "loyalty"), 5),
        },
        Case {
            card: "Sahagin",
            spell: "Concentrate",
            setup: |t| vec![t.battlefield(P0, "Sahagin")],
            check: |t, o| assert_eq!(t.counters(o[0], "+1/+1"), 1),
        },
        Case {
            // It triggers from the graveyard; P0 pays {B} and gets it back.
            card: "Shambling Cie'th",
            // {U} only, so the Swamp is left for {B}.
            spell: "Opt",
            setup: |t| {
                let c = t.graveyard(P0, "Shambling Cie'th");
                t.lands(P0, "Swamp", 1);
                t.answer_yes(P0, true);
                vec![c]
            },
            check: |t, _| assert!(t.in_hand(P0, "Shambling Cie'th")),
        },
        Case {
            // +X/+0 with X = 4 mana spent, and X >= 4: draw a card.
            card: "Shantotto, Tactician Magician",
            spell: "Concentrate",
            setup: |t| vec![t.battlefield(P0, "Shantotto, Tactician Magician")],
            check: |t, o| {
                assert_eq!(t.pt(o[0]), (4, 4));
                assert_eq!(t.library_size(P0), 29);
            },
        },
        Case {
            card: "Sky Theater Strix",
            spell: "Divination",
            setup: |t| vec![t.battlefield(P0, "Sky Theater Strix")],
            check: |t, o| assert_eq!(t.pt(o[0]), (2, 2)),
        },
        Case {
            card: "Thancred Waters",
            spell: "Divination",
            setup: |t| vec![t.battlefield(P0, "Thancred Waters")],
            check: |t, o| assert!(t.obj_now(o[0]).has_keyword(KeywordKind::Indestructible)),
        },
        Case {
            card: "The Prima Vista",
            spell: "Concentrate",
            setup: |t| {
                let v = t.battlefield(P0, "The Prima Vista");
                assert!(!t.obj_now(v).is_creature());
                vec![v]
            },
            check: |t, o| {
                assert!(t.obj_now(o[0]).is_creature());
                assert_eq!(t.pt(o[0]), (5, 3));
            },
        },
        Case {
            card: "Mysidian Elder",
            spell: "Divination",
            setup: wizard_token,
            check: |t, _| assert_eq!(t.life(P1), 19),
        },
        Case {
            // The Hero token has "... this creature gets +2/+0 until end of turn".
            card: "Red Mage's Rapier",
            spell: "Divination",
            setup: |t| hero_with(t, "Red Mage's Rapier"),
            check: |t, o| assert_eq!(t.pt(o[0]), (3, 1)),
        },
        Case {
            card: "Black Mage's Rod",
            spell: "Divination",
            setup: |t| hero_with(t, "Black Mage's Rod"),
            check: |t, _| assert_eq!(t.life(P1), 19),
        },
        Case {
            card: "Vivi Ornitier",
            spell: "Divination",
            setup: |t| vec![t.battlefield(P0, "Vivi Ornitier")],
            check: |t, o| {
                assert_eq!(t.counters(o[0], "+1/+1"), 1);
                assert_eq!(t.life(P1), 19);
            },
        },
        Case {
            // Level 3: "Whenever you cast an instant or sorcery spell, create a 1/1 blue
            // and red Otter creature token with prowess."
            card: "Stormchaser's Talent",
            spell: "Opt",
            setup: |t| {
                let c = t.battlefield(P0, "Stormchaser's Talent");
                mtg_engine::classes::set_level(&mut t.g, c, 3);
                t.g.flush_events();
                t.settle();
                assert!(tokens_with_subtype(t, P0, "Otter").is_empty());
                vec![c]
            },
            check: |t, _| assert_eq!(tokens_with_subtype(t, P0, "Otter").len(), 1),
        },
    ];
    run(&cases);
}

#[test]
fn good_king_mogs_delayed_trigger_resolves_first_even_if_the_spell_leaves() {
    cr!("603.3", "603.7a", "601.2i", "714.2b");
    ruling!(
        "Summon: Good King Mog XII",
        "The delayed triggered ability set up by Summon: Good King Mog XII's second and third chapter abilities resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Summon: Good King Mog XII");
    for how in ALL_LEAVES {
        let mut t = TestGame::new(2);
        let mog = t.enter(P0, "Summon: Good King Mog XII");
        t.resolve_all();
        // Chapter I: two Moogles.
        assert_eq!(tokens_with_subtype(&t, P0, "Moogle").len(), 2);
        add_counters(&mut t, mog, "lore", 1);
        t.resolve_all();
        let spell = cast_card(&mut t, P0, "Divination", &[]);
        assert!(trigger_on_top(&t));
        leave(&mut t, spell, P0, how);
        t.resolve();
        assert_eq!(tokens_with_subtype(&t, P0, "Moogle").len(), 3, "{how:?}");
        if how == Leaves::Stays {
            assert_eq!(t.zone(spell), Zone::Stack);
        }
    }
}

#[test]
fn riddleform_becomes_a_creature_after_the_spells_targets_are_chosen() {
    cr!("603.3", "601.2c", "601.2i");
    ruling!(
        "Riddleform",
        "Riddleform's first ability resolves before the spell that caused it to trigger but after targets for that spell are chosen. It will resolve even if that spell is countered."
    );
    supported("Riddleform");
    for how in ALL_LEAVES {
        let mut t = TestGame::new(2);
        let rf = t.battlefield(P0, "Riddleform");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        let spell = cast_card(&mut t, P0, "Giant Growth", &[obj(bears)]);
        // As Giant Growth's target was chosen, Riddleform wasn't a creature.
        let candidates: Vec<Entity> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => {
                    Some(candidates.clone())
                }
                _ => None,
            })
            .flatten()
            .collect();
        assert!(candidates.contains(&obj(bears)));
        assert!(!candidates.contains(&obj(rf)));
        assert!(trigger_on_top(&t));
        leave(&mut t, spell, P0, how);
        t.answer_yes(P0, true);
        t.resolve();
        assert!(t.obj_now(rf).is_creature(), "{how:?}");
        assert_eq!(t.pt(rf), (3, 3));
        assert!(t.obj_now(rf).has_keyword(KeywordKind::Flying));
        if how == Leaves::Stays {
            assert_eq!(t.zone(spell), Zone::Stack);
            t.resolve_all();
            assert_eq!(t.pt(bears), (5, 5));
        }
    }
}

#[test]
fn kindlespark_duo_can_be_activated_between_its_untap_trigger_and_the_spell() {
    cr!("603.3", "117.3b", "601.2i");
    ruling!(
        "Kindlespark Duo",
        "Players can cast spells and activate abilities (such as Kindlespark Duo's own activated ability) after the triggered ability resolves but before the spell that caused it to trigger does."
    );
    supported("Kindlespark Duo");
    let mut t = TestGame::new(2);
    let duo = t.battlefield(P0, "Kindlespark Duo");
    t.activate(P0, duo, 0, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.obj_now(duo).tapped);
    let spell = cast_card(&mut t, P0, "Divination", &[]);
    assert!(trigger_on_top(&t));
    t.resolve();
    assert!(!t.obj_now(duo).tapped);
    assert_eq!(t.zone(spell), Zone::Stack);
    // Activated again with Divination still on the stack: the ability resolves first.
    t.activate(P0, duo, 0, &[pl(P1)]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 28);
}
