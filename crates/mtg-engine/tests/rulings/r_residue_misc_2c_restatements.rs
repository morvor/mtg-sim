//! Item `rulings-residue-misc-2`, pass C — the order of Agitator Ant's choices, then the
//! rulings that restate the cards' own text, a compact case each: Neyith's single payment,
//! Slicer's goad, Dynaheir, legendary landwalk, Grindstone, Odric, Dauthi Warlord, Nissa's
//! Pilgrimage, Starving Revenant, and being suspected (Hot Pursuit, Nelly Borca).

use crate::r_residue_misc_2c_common::*;
use mtg_engine::combat::{attack_declaration_legal, attack_options};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kwa::suspect_detain::is_suspected;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Agitator Ant ------------------------------------------------------------------------------

#[test]
fn agitator_ant_players_choose_in_turn_order_then_the_counters_are_put_on() {
    cr!("101.4", "608.2e", "701.15a");
    ruling!(
        "Agitator Ant",
        "As Agitator Ant's ability resolves, first you choose which creature you control, if any, to put two +1/+1 counters on, then each other player in turn order does the same, knowing the choices made before them. Then each chosen creature receives its +1/+1 counters and becomes goaded."
    );
    let mut t = TestGame::new(3);
    let ant = t.battlefield(P0, "Agitator Ant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P0, &[obj(ant)]);
    t.answer_choose(P1, &[obj(b1)]);
    t.answer_choose(P2, &[obj(b2)]);
    // When P2 chooses, nobody's creature has the counters yet.
    let seen = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            g.permanents()
                .map(|o| o.counters.values().sum::<u32>())
                .sum::<u32>()
        },
    );
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let order: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(order, vec![P0, P1, P2], "{}", t.dump_log());
    assert_eq!(*seen.lock().unwrap(), vec![0]);
    for c in [ant, b1, b2] {
        assert_eq!(t.counters(c, counters::PLUS1), 2);
        assert!(goaded(&t, c));
    }
}

// --- Restatements -----------------------------------------------------------------------------

#[test]
fn neyith_pays_once_for_one_creature() {
    cr!("603.5", "608.2c");
    ruling!(
        "Neyith of the Dire Hunt",
        "While resolving Neyith’s last ability, you can’t pay multiple times to double a creature’s power more than once or to double more than one creature’s power."
    );
    let mut t = TestGame::new(2);
    let neyith = t.battlefield(P0, "Neyith of the Dire Hunt");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 9);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    let from = t.asked().len();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let pays = asked_since(&t, from)
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { source: Some(s), .. } if *s == neyith))
        .count();
    assert_eq!(pays, 1, "{}", t.dump_log());
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn slicer_goaded_must_attack_another_player_until_its_goaders_next_turn() {
    cr!("701.15a", "701.15b", "508.1d");
    ruling!(
        "Slicer, Hired Muscle // Slicer, High-Speed Antagonist",
        "When you goad a creature, that creature must attack each combat if able and must attack a player other than you if able. It stays goaded until your next turn."
    );
    let mut t = TestGame::new(3);
    let slicer = t.battlefield(P0, "Slicer, Hired Muscle // Slicer, High-Speed Antagonist");
    t.set_step(P0, Step::End);
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(controller(&t, slicer), P1);
    assert!(t.g.goaders(slicer).contains(&P0));
    t.set_step(P1, Step::BeginningOfCombat);
    let opts = attack_options(&t.g);
    let legal = |d: &[(ObjectId, Entity)]| attack_declaration_legal(&t.g, &opts, d);
    assert!(!legal(&[]), "{opts:?} {}", t.dump_log());
    assert!(!legal(&[(slicer, pl(P0))]));
    assert!(legal(&[(slicer, pl(P2))]));
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.goaders(slicer).is_empty());
}

#[test]
fn dynaheir_doesnt_grant_haste() {
    cr!("302.6", "702.10c");
    ruling!(
        "Dynaheir, Invoker Adept",
        "Dynaheir, Invoker Adept doesn't actually grant haste to creatures you control, nor does it let you attack with them as though they had haste."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dynaheir, Invoker Adept");
    let elves = t.battlefield_sick(P0, "Llanowar Elves");
    assert!(!t.obj_now(elves).has_keyword(KeywordKind::Haste));
    assert!(!can_attack(&mut t, elves));
    // Its {T} ability can be activated all the same.
    t.activate(P0, elves, 0, &[]).expect("{T}: Add {G}");
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn livonya_silone_legendary_landwalk() {
    cr!("702.14c", "509.1b");
    ruling!(
        "Livonya Silone",
        "“Legendary landwalk” means “This creature is can’t be blocked as long as defending player controls a legendary land.”"
    );
    for legendary in [true, false] {
        let mut t = TestGame::new(2);
        let liv = t.battlefield(P0, "Livonya Silone");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, if legendary { "Gaea's Cradle" } else { "Forest" });
        attack_with(&mut t, &[(liv, pl(P1))]);
        assert_eq!(
            legal_blocks(&mut t, P1, &[(bears, liv)]),
            !legendary,
            "legendary {legendary}"
        );
    }
}

#[test]
fn grindstone_repeats_while_the_two_cards_share_a_color() {
    cr!("701.17a", "105.4");
    ruling!(
        "Grindstone",
        "The process can repeat any number of times, as long as two cards that share a color are milled each time."
    );
    supported("Grindstone");
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Grindstone");
    stack_library(
        &mut t,
        P1,
        &[
            "Lightning Bolt",
            "Shock",
            "Hill Giant",
            "Goblin Piker",
            "Grizzly Bears",
            "Forest",
            "Storm Crow",
        ],
    );
    t.lands(P0, "Wastes", 3);
    t.activate(P0, gs, 0, &[pl(P1)]).expect("Grindstone");
    t.resolve_all();
    // Red-red, red-red, then green and colorless: six cards.
    assert_eq!(t.graveyard_size(P1), 6, "{}", t.dump_log());
}

#[test]
fn odric_counts_abilities_not_creatures() {
    cr!("111.10g", "603.2");
    ruling!(
        "Odric, Blood-Cursed",
        "Odric, Blood-Cursed counts the number of the listed abilities found, not the number of creatures with those abilities. If you control a single creature with both reach and vigilance as Odric enters the battlefield, you create two Blood tokens. On the other hand, if you control two creatures that each have flying, you create a single Blood token."
    );
    // (creatures, Blood tokens)
    let cases: [(&[&str], usize); 2] = [
        (&["Sentinel Spider"], 2),          // vigilance, reach
        (&["Storm Crow", "Wind Drake"], 1), // flying, flying
    ];
    for (creatures, blood) in cases {
        let mut t = TestGame::new(2);
        for c in creatures {
            supported(c);
            t.battlefield(P0, c);
        }
        t.enter(P0, "Odric, Blood-Cursed");
        t.resolve_all();
        assert_eq!(tokens_with(&t, P0, "Blood").len(), blood, "{creatures:?}");
    }
}

#[test]
fn dauthi_warlord_counts_every_players_shadow_creatures() {
    cr!("702.28a", "604.3");
    ruling!(
        "Dauthi Warlord",
        "Dauthi Warlord counts creatures with the Shadow ability controlled by all players."
    );
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Dauthi Warlord");
    t.battlefield(P1, "Dauthi Slayer");
    t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(w), (2, 1));
}

#[test]
fn nissas_pilgrimage_with_spell_mastery_three_forests() {
    cr!("701.23a", "207.2c");
    ruling!(
        "Nissa's Pilgrimage",
        "If the spell mastery ability applies, and you find three basic Forest cards, one of them will be put onto the battlefield tapped and two of them will be put into your hand."
    );
    supported("Nissa's Pilgrimage");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Opt");
    t.graveyard(P0, "Lightning Bolt");
    let forests: Vec<ObjectId> = (0..3).map(|_| t.library_top(P0, "Forest")).collect();
    let es: Vec<Entity> = forests.iter().map(|f| obj(*f)).collect();
    t.answer_choose(P0, &es);
    t.answer_choose_moved(P0, &es[..1]);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Nissa's Pilgrimage", &[]);
    t.resolve_all();
    let on_bf: Vec<ObjectId> = forests
        .iter()
        .map(|f| t.g.current(*f))
        .filter(|f| t.zone(*f) == Zone::Battlefield)
        .collect();
    assert_eq!(on_bf.len(), 1, "{}", t.dump_log());
    assert!(t.obj_now(on_bf[0]).tapped);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn starving_revenant_draws_the_cards_it_kept_on_top() {
    cr!("701.25a", "121.1");
    ruling!(
        "Starving Revenant",
        "Except in some unusual cases, the cards you put on top of your library while surveilling will be the cards you draw."
    );
    supported("Starving Revenant");
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Hill Giant", "Craw Wurm"]);
    t.answer(
        P0,
        DecisionKind::Surveil,
        Answer::Split(vec![cards[1], cards[0]], vec![]),
    );
    t.enter(P0, "Starving Revenant");
    t.resolve_all();
    assert!(
        t.in_hand(P0, "Hill Giant") && t.in_hand(P0, "Craw Wurm"),
        "{}",
        t.dump_log()
    );
    assert_eq!(t.life(P0), 14);
}

// --- Being suspected (Hot Pursuit, Nelly Borca) -----------------------------------------------

/// P0's Hot Pursuit enters suspecting `target`.
fn pursue(t: &mut TestGame, target: ObjectId) {
    t.answer_targets(P0, &[obj(target)]);
    t.enter(P0, "Hot Pursuit");
    t.resolve_all();
    assert!(is_suspected(&t.g, target));
}

fn menace_cant_block(t: &TestGame, id: ObjectId) -> (bool, bool) {
    (
        t.obj_now(id).has_keyword(KeywordKind::Menace),
        !t.g.can_block_at_all(t.g.current(id)),
    )
}

#[test]
fn suspected_creatures_have_menace_and_cant_block_until_they_leave() {
    cr!("701.60a", "701.60b", "701.60c", "613.1f");
    ruling!(
        "Hot Pursuit",
        "When an effect suspects a creature, it becomes suspected. It gains menace and \"This creature can't block\" for as long as it’s suspected. It stays suspected until it leaves the battlefield or another effect causes it to no longer be suspected."
    );
    ruling!(
        "Hot Pursuit",
        "If a suspected creature loses all abilities, it will lose menace and \"This creature can't block\", but it won’t stop being suspected."
    );
    ruling!(
        "Hot Pursuit",
        "There’s no limit to the number of creatures that can be suspected simultaneously. Suspecting a new creature doesn’t cause other creatures to stop being suspected."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    pursue(&mut t, bears);
    assert_eq!(menace_cant_block(&t, bears), (true, true));
    // A second one: both suspected.
    pursue(&mut t, giant);
    assert!(is_suspected(&t.g, bears) && is_suspected(&t.g, giant));
    // Turn to Frog on the Bears: no menace, can block, still suspected.
    cast_new(&mut t, P0, "Turn to Frog", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(menace_cant_block(&t, bears), (false, false));
    assert!(is_suspected(&t.g, bears));
    // Next turn (the frog effect over), and until it leaves the battlefield.
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(menace_cant_block(&t, bears), (true, true));
    let gone = mv(&mut t, bears, Zone::Hand(P1));
    let back = mv(&mut t, gone, Zone::Battlefield);
    assert!(!is_suspected(&t.g, back));
}

#[test]
fn suspecting_a_suspected_creature_again_changes_nothing() {
    cr!("701.60d", "115.1");
    ruling!(
        "Hot Pursuit",
        "If a creature is already suspected, suspecting it again won’t have any effect."
    );
    ruling!(
        "Nelly Borca, Impulsive Accuser",
        "You can target a creature that's already suspected with Nelly Borca, Impulsive Accuser's second ability."
    );
    supported("Nelly Borca, Impulsive Accuser");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    pursue(&mut t, bears);
    let nelly = t.battlefield(P0, "Nelly Borca, Impulsive Accuser");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(nelly, pl(P1))]);
    t.resolve_all();
    let offered = asked_since(&t, from).iter().any(|(_, d)| {
        matches!(d, Decision::ChooseTargets { candidates, .. } if candidates.contains(&obj(bears)))
    });
    assert!(offered, "{}", t.dump_log());
    assert!(is_suspected(&t.g, bears));
    let menace = t
        .obj_now(bears)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.to_lowercase().contains("menace"))
        .count();
    assert_eq!(menace, 1);
}
