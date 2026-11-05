//! Rulings batch P087 — cards that hate on a mechanic ("hate-set-mechanic"), part 2: total
//! cost (Reidane, Brush Off), Goats, base power and toughness, Valkmira, evasion checked
//! once, "unless" costs, wizardcycling, kicker, modes, counters, Adventures in exile, snow
//! landwalk, protection from artifacts and -1/-1 counter replacement.

use crate::r_p087_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const REIDANE: &str = "Reidane, God of the Worthy // Valkmira, Protector's Shield";

#[test]
fn reidane_increases_come_before_reductions_and_mana_value_ignores_both() {
    cr!("601.2f", "202.3");
    ruling!(
        "Reidane, God of the Worthy // Valkmira, Protector's Shield",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases (such as that of Reidane, God of the Worthy), then apply any cost reductions. The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported(REIDANE);
    supported("Sapphire Medallion");
    // Cryptic Command ({1}{U}{U}{U}, mana value 4) with two Sapphire Medallions ("Blue
    // spells you cast cost {1} less") under Reidane: {1}{U}{U}{U} + {2} - {2} =
    // {1}{U}{U}{U}. Reducing first would waste one reduction: {2}{U}{U}{U}.
    for (extra, ok) in [(0, false), (1, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, REIDANE);
        t.battlefield(P0, "Sapphire Medallion");
        t.battlefield(P0, "Sapphire Medallion");
        t.lands(P0, "Island", 3);
        t.lands(P0, "Wastes", extra);
        let cc = t.hand(P0, "Cryptic Command");
        assert_eq!(castable(&mut t, P0, cc), ok, "{extra}");
        if ok {
            let s = t.cast(P0, cc).modes(&[2, 3]).go();
            assert_eq!(crate::r_s26_common::mv(&mut t, s), 4);
            assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
        }
    }
}

#[test]
fn brush_off_reduction_applies_after_increases_and_mana_value_stays_4() {
    cr!("601.2f", "202.3");
    ruling!(
        "Brush Off",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Brush Off's first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Brush Off");
    // Brush Off ({2}{U}{U}) at an instant under Reidane with two Sapphire Medallions:
    // {4}{U}{U} - {1}{U} - {1} - {1} = {1}{U}. Reducing first: {U} + {2} = {2}{U}.
    for (wastes, ok) in [(0, false), (1, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, REIDANE);
        t.battlefield(P0, "Sapphire Medallion");
        t.battlefield(P0, "Sapphire Medallion");
        let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", wastes);
        let card = t.hand(P0, "Brush Off");
        let r = t.cast_with(P0, card, &[obj(bolt)]);
        assert_eq!(r.is_ok(), ok, "{wastes}: {r:?}");
        if let Ok(s) = r {
            assert_eq!(crate::r_s26_common::mv(&mut t, s), 4);
            t.resolve_all();
            assert_eq!(t.life(P0), 20);
        }
    }
}

#[test]
fn goatnap_checks_for_a_goat_once_as_it_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Goatnap",
        "To determine whether the target creature gets +3/+0, look at its creature types after you've gained control of it, untapped it, and given it haste. If it's not a Goat at this time, it won't get +3/+0, even if it becomes a Goat later in the turn. If it is a Goat, it gets +3/+0, even if it stops being a Goat later in the turn."
    );
    supported("Goatnap");
    supported("Amoeboid Changeling");
    for (name, goat) in [("Mountain Goat", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, name);
        let base = t.pt(c).0;
        cast_new(&mut t, P0, "Goatnap", &[obj(c)]);
        t.resolve_all();
        assert_eq!(t.obj_now(c).controller, P0);
        assert_eq!(t.pt(c).0, base + if goat { 3 } else { 0 }, "{name}");
        // Amoeboid Changeling: "{T}: Target creature gains all creature types until end
        // of turn." / "{T}: Target creature loses all creature types until end of turn."
        let amoeboid = t.battlefield(P0, "Amoeboid Changeling");
        let needle = if goat { "loses all" } else { "gains all" };
        activate(&mut t, P0, amoeboid, needle, &[obj(c)]);
        t.resolve_all();
        t.g.recompute();
        assert_eq!(t.obj_now(c).chars.has_subtype("Goat"), !goat, "{name}");
        assert_eq!(t.pt(c).0, base + if goat { 3 } else { 0 }, "{name}");
    }
}

#[test]
fn unable_to_scream_overwrites_earlier_setting_effects_and_later_ones_overwrite_it() {
    cr!("613.4b", "613.7a");
    ruling!(
        "Unable to Scream",
        "Unable to Scream's second ability overwrites all previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply afterward will overwrite this effect."
    );
    supported("Unable to Scream");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Ovinize: "Target creature loses all abilities and has base power and toughness 0/1
    // until end of turn."
    cast_new(&mut t, P0, "Ovinize", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (0, 1));
    attach_new(&mut t, P0, "Unable to Scream", bears);
    assert_eq!(t.pt(bears), (0, 2));
    assert!(t.obj_now(bears).is(CardType::Artifact));
    // Turn to Frog (later): a 1/1.
    cast_new(&mut t, P0, "Turn to Frog", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn valkmira_prevents_one_damage_from_each_opposing_source_each_time() {
    cr!("615.7", "120.1");
    ruling!(
        "Reidane, God of the Worthy // Valkmira, Protector's Shield",
        "Valkmira's first ability prevents 1 damage from each source an opponent controls each time that source would deal damage to you. It prevents 1 of any kind of damage, not just combat damage."
    );
    let mut t = TestGame::new(2);
    crate::r_p179_common::battlefield_back(&mut t, P0, REIDANE);
    let giant = t.battlefield(P0, "Hill Giant");
    // Noncombat damage to P0 and to P0's creature.
    let shocker = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(shocker, Entity::Player(P0), 2, false);
    t.g.deal_damage(shocker, obj(giant), 2, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P0), 19);
    assert_eq!(damage_marked(&t, giant), 1);
    // The same source again: 1 is prevented again.
    t.g.deal_damage(shocker, Entity::Player(P0), 2, false);
    t.g.flush_events();
    assert_eq!(t.life(P0), 18);
    // P0's own source: not prevented.
    t.g.deal_damage(giant, Entity::Player(P0), 2, false);
    t.g.flush_events();
    assert_eq!(t.life(P0), 16);
    // Combat damage from two attackers: 1 each.
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P0)), (b, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 14);
}

#[test]
fn valkmira_counters_opposing_spells_and_abilities_that_target_you_or_another_permanent() {
    cr!("603.2", "118.12", "115.1");
    ruling!(
        "Reidane, God of the Worthy // Valkmira, Protector's Shield",
        "Valkmira's last ability affects each spell (including Aura spells), activated ability, and triggered ability controlled by an opponent that targets you or another permanent you control."
    );
    let setup = || {
        let mut t = TestGame::new(2);
        let valk = crate::r_p179_common::battlefield_back(&mut t, P0, REIDANE);
        let giant = t.battlefield(P0, "Hill Giant");
        (t, valk, giant)
    };
    // An Aura spell at P0's creature: countered (P1 can't pay {1}).
    let (mut t, _, giant) = setup();
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Pacifism", &[obj(giant)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Pacifism"));
    // A spell at P0: countered (P1 can't pay {1}).
    let (mut t, _, _) = setup();
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // An activated ability at P0's creature (Icy Manipulator): countered; paid for: not.
    for pay in [false, true] {
        let (mut t, _, giant) = setup();
        let icy = t.battlefield(P1, "Icy Manipulator");
        t.lands(P1, "Wastes", if pay { 2 } else { 1 });
        activate(&mut t, P1, icy, "Tap target", &[obj(giant)]);
        assert_eq!(t.stack_len(), 2);
        t.answer_yes(P1, true);
        t.resolve_all();
        assert_eq!(is_tapped(&t, giant), pay);
    }
    // A triggered ability at P0's creature: countered.
    let (mut t, _, giant) = setup();
    t.answer_targets(P1, &[obj(giant)]);
    t.enter(P1, "Flametongue Kavu");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    // Targeting Valkmira itself, or targeting by P0: no trigger.
    let (mut t, valk, giant) = setup();
    cast_new(&mut t, P1, "Shatter", &[obj(valk)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(valk));
    cast_new(&mut t, P0, "Shock", &[obj(giant)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn scrapdiver_serpent_stays_blocked_when_the_defender_gains_an_artifact() {
    cr!("509.1b", "509.1h");
    ruling!(
        "Scrapdiver Serpent",
        "Whether Scrapdiver Serpent can be blocked is relevant only as the declare blockers step begins. If the defending player gains control of an artifact after Scrapdiver Serpent has become blocked, the Serpent will remain blocked."
    );
    supported("Scrapdiver Serpent");
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Scrapdiver Serpent");
    let wall = t.battlefield(P1, "Wall of Stone");
    to_blockers(&mut t, &[(serpent, Entity::Player(P1))], &[(wall, serpent)]);
    assert_eq!(blocks_now(&t), vec![(wall, serpent)]);
    t.battlefield(P1, "Ornithopter");
    t.g.recompute();
    assert_eq!(blocks_now(&t), vec![(wall, serpent)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage_marked(&t, wall), 5);
    // With an artifact as blockers are declared, it can't be blocked.
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Scrapdiver Serpent");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.battlefield(P1, "Ornithopter");
    attack_with(&mut t, &[(serpent, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(wall, serpent)]));
}

#[test]
fn godhunter_octopus_keeps_attacking_when_the_enchantment_goes() {
    cr!("508.1c", "506.4");
    ruling!(
        "Godhunter Octopus",
        "Whether the defending player controls an enchantment or enchanted permanent is checked only as attackers are declared. Once Godhunter Octopus has been declared as an attacker, it will continue to attack even if the defending player loses control of each enchantment or enchanted permanent they controlled."
    );
    supported("Godhunter Octopus");
    let mut t = TestGame::new(2);
    let octo = t.battlefield(P0, "Godhunter Octopus");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t, octo));
    let anthem = t.battlefield(P1, "Glorious Anthem");
    assert!(crate::r_s02_common::can_attack(&mut t, octo));
    attack_with(&mut t, &[(octo, Entity::Player(P1))]);
    destroy(&mut t, anthem);
    assert!(t.g.combat.as_ref().unwrap().attacker(octo).is_some());
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn goatnapper_can_target_a_changeling() {
    cr!("702.73a", "205.3m");
    ruling!(
        "Goatnapper",
        "Yes, target Goat. Goat is a creature type. Keep in mind that each permanent with changeling is a Goat, as is any permanent affected by a spell or ability that gives it every creature type."
    );
    supported("Goatnapper");
    let mut t = TestGame::new(2);
    let outcast = t.battlefield(P1, "Changeling Outcast");
    let goat = t.battlefield(P1, "Mountain Goat");
    let bears = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, outcast);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(outcast)]);
    t.enter(P0, "Goatnapper");
    t.g.flush_events();
    t.settle();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&obj(outcast)) && offered[0].contains(&obj(goat)));
    assert!(!offered[0].contains(&obj(bears)));
    t.resolve_all();
    assert_eq!(t.obj_now(outcast).controller, P0);
    assert!(!is_tapped(&t, outcast));
}

#[test]
fn grip_of_amnesia_can_be_paid_with_an_empty_graveyard() {
    cr!("118.12", "118.3");
    ruling!(
        "Grip of Amnesia",
        "You can choose to exile all cards from your graveyard even if it has no cards in it."
    );
    supported("Grip of Amnesia");
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    assert_eq!(t.graveyard_size(P1), 0);
    t.answer_yes(P1, true);
    lands_for_cost(&mut t, P0, "Grip of Amnesia");
    let grip = t.hand(P0, "Grip of Amnesia");
    let hand = t.hand_size(P0);
    t.cast_with(P0, grip, &[obj(bolt)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    // Grip of Amnesia left the hand and drew a card.
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Grip of Amnesia"));
}

#[test]
fn wizardcycling_finds_any_wizard_card_or_nothing() {
    cr!("702.29e", "205.3m", "701.23b");
    ruling!(
        "Vedalken Aethermage",
        "You can choose to find any card with the Wizard creature type, even if it isn't a creature card. This includes, for example, kindred cards with the Changeling ability. You can also choose not to find a card, even if there is a Wizard card in your library."
    );
    supported("Vedalken Aethermage");
    for find in [true, false] {
        let mut t = TestGame::new(2);
        // Nameless Inversion: Kindred Instant — Shapeshifter, with changeling.
        let inv = t.library_top(P0, "Nameless Inversion");
        t.library_top(P0, "Grizzly Bears");
        let card = t.hand(P0, "Vedalken Aethermage");
        t.lands(P0, "Wastes", 3);
        let from = t.asked().len();
        t.answer_choose(P0, if find { vec![obj(inv)] } else { vec![] }.as_slice());
        crate::r_s04_common::activate_named(&mut t, P0, card, "Cycling", 0).unwrap();
        t.resolve_all();
        let offered = entity_candidates(&t, from);
        assert!(offered.contains(&obj(inv)), "{offered:?}");
        assert_eq!(t.in_hand(P0, "Nameless Inversion"), find);
        assert!(!t.in_hand(P0, "Grizzly Bears"));
    }
}

#[test]
fn ertais_trickery_targets_any_spell_but_counters_only_a_kicked_one() {
    cr!("702.33d", "608.2c");
    ruling!(
        "Ertai's Trickery",
        "You can target any spell, but that spell will be countered only if it was kicked."
    );
    supported("Ertai's Trickery");
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P1, "Mountain", 1);
        t.lands(P1, "Wastes", if kicked { 4 } else { 0 });
        let burst = t.hand(P1, "Burst Lightning");
        let s = t.cast(P1, burst).target(P0).kicked(kicked).go();
        cast_new(&mut t, P0, "Ertai's Trickery", &[obj(s)]);
        t.resolve_all();
        assert_eq!(t.life(P0), if kicked { 20 } else { 18 });
    }
}

#[test]
fn aether_shockwave_mode_is_chosen_on_announcement() {
    cr!("601.2b", "700.2a");
    ruling!("Aether Shockwave", "You choose the mode on announcement.");
    supported("Aether Shockwave");
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P1, "Kami of Ancient Law");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Aether Shockwave");
    let card = t.hand(P0, "Aether Shockwave");
    let s = t.cast(P0, card).modes(&[1]).go();
    assert_eq!(crate::r_s07_common::chosen_modes(&t, s), vec![1]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
    assert!(is_tapped(&t, bears) && !is_tapped(&t, spirit));
}

#[test]
fn dramatists_puppet_chooses_per_kind_on_resolution() {
    cr!("608.2c", "122.1");
    ruling!(
        "Dramatist's Puppet",
        "You choose whether to add or remove counters as the ability resolves. If the permanent has multiple kinds of counters, you don't have to make the same choice for each kind."
    );
    supported("Dramatist's Puppet");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 2);
    put_counters(&mut t, bears, counters::TIME, 2);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Dramatist's Puppet");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Nothing was chosen yet; on resolution, one choice per kind: put for the first, remove
    // for the second.
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    let asked = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseOption { .. }))
        .count();
    assert_eq!(asked, 2);
    let mut n = [
        t.counters(bears, counters::PLUS1),
        t.counters(bears, counters::TIME),
    ];
    n.sort();
    assert_eq!(n, [1, 3]);
}

#[test]
fn memory_theft_adventure_choice_after_seeing_the_hand_even_if_empty() {
    cr!("608.2c", "715.4");
    ruling!(
        "Memory Theft",
        "You choose which exiled adventurer card, if any, to put into that player's graveyard after seeing their hand. You may do this even if that player has no nonland cards in hand, or even no cards at all."
    );
    supported("Memory Theft");
    for hand in [0, 1] {
        let mut t = TestGame::new(2);
        let giant = t.exile(P1, "Bonecrusher Giant // Stomp");
        let bolt = (hand > 0).then(|| t.hand(P1, "Lightning Bolt"));
        if let Some(b) = bolt {
            t.answer_choose(P0, &[obj(b)]);
        }
        t.answer_choose(P0, &[obj(giant)]);
        t.answer_yes(P0, true);
        cast_new(&mut t, P0, "Memory Theft", &[Entity::Player(P1)]);
        t.resolve_all();
        assert_eq!(t.zone(t.g.current(giant)), Zone::Graveyard(P1), "{hand}");
        assert_eq!(t.in_graveyard(P1, "Lightning Bolt"), hand > 0);
    }
}

#[test]
fn zombie_musher_snow_landwalk() {
    cr!("702.14c", "509.1b");
    ruling!(
        "Zombie Musher",
        "Zombie Musher can’t be blocked as long as defending player controls a snow land."
    );
    supported("Zombie Musher");
    for snow in [false, true] {
        let mut t = TestGame::new(2);
        let musher = t.battlefield(P0, "Zombie Musher");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, if snow { "Snow-Covered Forest" } else { "Forest" });
        attack_with(&mut t, &[(musher, Entity::Player(P1))]);
        assert_eq!(legal_blocks(&mut t, P1, &[(bears, musher)]), !snow);
    }
}

#[test]
fn tel_jilad_defiance_protection_from_artifacts() {
    cr!("702.16b", "702.16c", "702.16d", "702.16e", "702.16f", "704.5m", "704.5n");
    ruling!(
        "Tel-Jilad Defiance",
        "“Protection from artifacts” means the following: -- The affected creature can't be blocked by artifact creatures. -- The affected creature can't be equipped. If any Equipment are attached to it, they become unattached. The affected creature also can't be enchanted by Auras that have somehow become artifacts in addition to enchantments. -- The affected creature can't be targeted by abilities from artifact sources. -- All damage that would be dealt to the affected creature by artifact sources is prevented."
    );
    supported("Tel-Jilad Defiance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let eq = attach_new(&mut t, P0, "Bonesplitter", bears);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Tel-Jilad Defiance", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // The Equipment becomes unattached.
    assert!(t.on_battlefield(eq));
    assert_eq!(attached_to(&t, eq), None);
    assert!(!t.g.attach(eq, obj(bears)));
    // Abilities of artifact sources can't target it (Rod of Ruin).
    let rod = t.battlefield(P1, "Rod of Ruin");
    let c = ability_targets(&mut t, rod, 0);
    assert!(!c.contains(&obj(bears)) && c.contains(&Entity::Player(P0)));
    // Damage from artifact sources is prevented.
    let thopter = t.battlefield(P1, "Ornithopter");
    t.g.deal_damage(thopter, obj(bears), 1, false);
    t.g.flush_events();
    assert_eq!(damage_marked(&t, bears), 0);
    // An Aura that has become an artifact (Mycosynth Lattice) falls off.
    let strength = attach_new(&mut t, P0, "Holy Strength", bears);
    t.settle();
    assert!(t.on_battlefield(strength));
    t.battlefield(P1, "Mycosynth Lattice");
    t.settle();
    assert!(!t.on_battlefield(strength));
    // Artifact creatures can't block it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let thopter = t.battlefield(P1, "Ornithopter");
    let elf = t.battlefield(P1, "Llanowar Elves");
    cast_new(&mut t, P0, "Tel-Jilad Defiance", &[obj(bears)]);
    t.resolve_all();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(thopter, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(elf, bears)]));
}

#[test]
fn vizier_of_remedies_reduces_counters_a_creature_enters_with() {
    cr!("614.1c", "122.6", "702.79a");
    ruling!(
        "Vizier of Remedies",
        "“Put on a creature you control” includes that creature entering the battlefield with -1/-1 counters on it. If a creature would enter the battlefield under your control with a number of -1/-1 counters on it while you control Vizier of Remedies, it enters with that many counters minus one."
    );
    supported("Vizier of Remedies");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of Remedies");
    // Grim Poppet enters with three -1/-1 counters: two.
    let poppet = t.enter(P0, "Grim Poppet");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.counters(poppet, counters::MINUS1), 2);
    // Kitchen Finks returns with persist's -1/-1 counter: none.
    let finks = t.battlefield(P0, "Kitchen Finks");
    destroy(&mut t, finks);
    t.resolve_all();
    let back = t.named_on_battlefield("Kitchen Finks");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], counters::MINUS1), 0);
    // An opponent's creature isn't affected.
    let theirs = t.enter(P1, "Grim Poppet");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.counters(theirs, counters::MINUS1), 3);
}
