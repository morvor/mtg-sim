//! Conditions and qualifiers about what the text named (`conditions_referents.rs`,
//! `conditions_compare.rs`, `filter_qualities.rs`): a spell that left the stack, the land
//! a trigger is about, the trigger's group of attackers, a chosen player, counters put by
//! you this turn, player-relative values, name lists, and a few word orders.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn what_a_countered_spell_was_jaces_defeat() {
    cr!("608.2h");
    compiles("Jace's Defeat");
    // A blue Jace planeswalker spell, and a blue spell that isn't one.
    for (spell, scry) in [("Jace Beleren", true), ("Mahamoti Djinn", false)] {
        let mut t = TestGame::new(2);
        t.lands(P1, "Island", 6);
        let s = t.hand(P1, spell);
        t.set_step(P1, Step::PrecombatMain);
        let target = t.cast(P1, s).go();
        t.lands(P0, "Island", 2);
        let jd = t.hand(P0, "Jace's Defeat");
        t.cast(P0, jd).target(target).go();
        t.resolve();
        let scried = t
            .asked()
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::Scry { .. }));
        assert_eq!(scried, scry, "{spell}");
        assert!(t.in_graveyard(P1, spell));
    }
}

#[test]
fn that_land_is_the_trigger_object_guardian_of_tazeem() {
    ruling!(
        "Guardian of Tazeem",
        "If the land that caused it to trigger was an Island"
    );
    compiles("Guardian of Tazeem");
    for (land, stays_tapped) in [("Island", true), ("Plains", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Guardian of Tazeem");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let l = t.hand(P0, land);
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.play_land(P0, l).unwrap();
        t.resolve_all();
        assert!(t.obj_now(bears).tapped);
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(t.obj_now(bears).tapped, stays_tapped, "{land}");
    }
}

#[test]
fn shares_a_card_type_with_the_entering_permanent_amareth() {
    compiles("Amareth, the Lustrous");
    for (top, taken) in [("Grizzly Bears", true), ("Island", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Amareth, the Lustrous");
        t.library_top(P0, top);
        t.answer_yes(P0, true);
        t.enter(P0, "Hill Giant");
        t.resolve_all();
        assert_eq!(t.in_hand(P0, top), taken, "{top}");
    }
}

#[test]
fn none_of_those_creatures_attacked_you_firemane_commando() {
    ruling!(
        "Firemane Commando",
        "cares only that none of the attacking creatures attacked you"
    );
    compiles("Firemane Commando");
    let run = |attack_me: bool| {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Firemane Commando");
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Grizzly Bears");
        let def = if attack_me { P0 } else { P2 };
        t.advance_to(P1, Step::BeginningOfCombat);
        let hand = t.hand_size(P1);
        t.attack(&[(a, Entity::Player(P2)), (b, Entity::Player(def))], &[]);
        t.hand_size(P1) - hand
    };
    assert_eq!(run(false), 1);
    assert_eq!(run(true), 0);
}

#[test]
fn any_attacker_with_power_or_toughness_equal_to_the_chosen_number_squall() {
    ruling!(
        "Squall, Gunblade Duelist",
        "checks the power and toughness of the attacking creatures at the moment it would trigger"
    );
    compiles("Squall, Gunblade Duelist");
    compiles("Talion, the Kindly Lord");
    for (n, hit) in [(2, true), (5, false)] {
        let mut t = TestGame::new(2);
        let squall = t.battlefield(P0, "Squall, Gunblade Duelist");
        t.g.objects[squall.0 as usize].choices.number = Some(n);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(bears, Entity::Player(P1))], &[]);
        let squall_power = t.pt(squall).0;
        let expected = 20 - 2 - if hit { squall_power } else { 0 };
        assert_eq!(t.life(P1), expected, "chosen {n}");
    }
}

#[test]
fn counters_you_put_this_turn_kid_loki() {
    compiles("Kid Loki");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kid Loki");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(!t.obj_now(a).has_keyword(KeywordKind::Hexproof));
    // P0 puts a counter on one; the other gets one an opponent's effect put.
    let theirs = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(a), "+1/+1", 1, None);
    t.g.add_counters(Entity::Object(b), "+1/+1", 1, Some(theirs));
    t.g.flush_events();
    t.g.recompute();
    assert!(t.obj_now(a).has_keyword(KeywordKind::Hexproof));
    assert!(!t.obj_now(b).has_keyword(KeywordKind::Hexproof));
    // Next turn, it no longer has hexproof.
    t.advance_to(P1, Step::Upkeep);
    t.g.recompute();
    assert!(!t.obj_now(a).has_keyword(KeywordKind::Hexproof));
}

#[test]
fn the_chosen_player_controls_a_permanent_of_the_chosen_color_jihad() {
    cr!("607.2d", "603.8");
    ruling!("Jihad", "it tracks only the chosen player");
    compiles("Jihad");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jihad");
    t.g.objects[j.0 as usize].choices.color = Some(Color::Green);
    t.g.objects[j.0 as usize].choices.player = Some(P1);
    let knight = t.battlefield(P0, "White Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // A green permanent another player controls doesn't count.
    t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.pt(knight), (4, 3));
    // No more green nontoken permanents: Jihad is sacrificed.
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(!t.on_battlefield(j));
}

#[test]
fn equipment_with_any_of_the_names_kaldra() {
    cr!("201.2");
    ruling!(
        "Helm of Kaldra",
        "The token isn\u{2019}t put onto the battlefield unless you control all three Equipment"
    );
    compiles("Shield of Kaldra");
    compiles("Helm of Kaldra");
    let mut t = TestGame::new(2);
    let helm = t.battlefield(P0, "Helm of Kaldra");
    let sword = t.battlefield(P0, "Sword of Kaldra");
    t.lands(P0, "Plains", 2);
    t.activate(P0, helm, 0, &[]).unwrap();
    t.resolve();
    assert!(t.named_on_battlefield("Kaldra").is_empty());
    let shield = t.battlefield(P0, "Shield of Kaldra");
    t.g.recompute();
    // Shield of Kaldra: the three are indestructible.
    for e in [helm, sword, shield] {
        assert!(t.obj_now(e).has_keyword(KeywordKind::Indestructible));
    }
    t.activate(P0, helm, 0, &[]).unwrap();
    t.resolve();
    let kaldra = t.named_on_battlefield("Kaldra");
    assert_eq!(kaldra.len(), 1);
    for e in [helm, sword, shield] {
        assert_eq!(t.obj_now(e).attached_to, Some(Entity::Object(kaldra[0])));
    }
}

#[test]
fn mana_value_compared_with_the_casters_lands_lavinia() {
    compiles("Lavinia, Azorius Renegade");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lavinia, Azorius Renegade");
    t.lands(P1, "Island", 2);
    let div = t.hand(P1, "Divination"); // mana value 3
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P1, div).try_go().is_err());
    t.lands(P1, "Island", 1);
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P1, div).try_go().is_ok());
}

#[test]
fn power_compared_with_the_enchanted_creatures_toughness_ironclaw_curse() {
    compiles("Ironclaw Curse");
    compiles("Death's Oasis");
    compiles("Catapult Fodder // Catapult Captain");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Stone"); // 0/8 -> 0/7
    let curse = t.battlefield(P0, "Ironclaw Curse");
    t.g.attach(curse, Entity::Object(wall));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    // A 2-power attacker is less than 7: the Wall can block it.
    t.attack(&[(bears, Entity::Player(P1))], &[(wall, bears)]);
    assert_eq!(t.life(P1), 20);
    // Enchanting a 2/2 (now 2/1): it can't block a 3-power attacker.
    let mut t = TestGame::new(2);
    let blocker = t.battlefield(P1, "Grizzly Bears");
    let curse = t.battlefield(P0, "Ironclaw Curse");
    t.g.attach(curse, Entity::Object(blocker));
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(blocker), (2, 1));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[(blocker, giant)]);
    assert_eq!(t.life(P1), 17);
    assert!(t.on_battlefield(blocker));
}

#[test]
fn each_creature_without_mana_value_of_the_chosen_quality_enters_tapped() {
    ruling!(
        "Ashling's Prerogative",
        "creatures with odd mana values enter tapped and creatures with even mana values have haste"
    );
    compiles("Ashling's Prerogative");
    let mut t = TestGame::new(2);
    let ap = t.enter(P0, "Ashling's Prerogative");
    let chosen = t
        .obj_now(ap)
        .linked_choices
        .values()
        .find_map(|c| c.text.clone())
        .expect("odd or even chosen");
    let even = t.enter(P1, "Grizzly Bears"); // 2
    let odd = t.enter(P1, "Llanowar Elves"); // 1
    if chosen == "odd" {
        assert!(t.obj_now(even).tapped && !t.obj_now(odd).tapped);
    } else {
        assert!(t.obj_now(odd).tapped && !t.obj_now(even).tapped);
    }
}

#[test]
fn different_mana_values_x_or_less_agadeems_awakening() {
    cr!("601.2c");
    compiles("Agadeem's Awakening // Agadeem, the Undercrypt");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let a = t.graveyard(P0, "Grizzly Bears"); // 2
    let b = t.graveyard(P0, "Hill Giant"); // 4
    let c = t.graveyard(P0, "Llanowar Elves"); // 1
    let aw = t.hand(P0, "Agadeem's Awakening // Agadeem, the Undercrypt");
    t.cast(P0, aw)
        .x(2)
        .targets(&[Entity::Object(a), Entity::Object(c)])
        .go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    let _ = b;
    // Mana value 4 is greater than X = 2: Hill Giant can't be a target.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    let aw = t.hand(P0, "Agadeem's Awakening // Agadeem, the Undercrypt");
    let _ = t
        .cast(P0, aw)
        .x(2)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .try_go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // Two cards with the same mana value can't both be targets.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let c = t.graveyard(P0, "Llanowar Elves");
    let d = t.graveyard(P0, "Llanowar Elves");
    let aw = t.hand(P0, "Agadeem's Awakening // Agadeem, the Undercrypt");
    let _ = t
        .cast(P0, aw)
        .x(2)
        .targets(&[Entity::Object(c), Entity::Object(d)])
        .try_go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Llanowar Elves").len() <= 1);
}

#[test]
fn the_greatest_power_among_that_players_creatures_kraven() {
    compiles("Kraven the Hunter");
    for (dying, other, draws) in [
        ("Hill Giant", "Grizzly Bears", 1),
        ("Grizzly Bears", "Hill Giant", 0),
        // Tied for the greatest: it has the greatest power.
        ("Grizzly Bears", "Grizzly Bears", 1),
    ] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Kraven the Hunter");
        let victim = t.battlefield(P1, dying);
        t.battlefield(P1, other);
        // Another opponent's bigger creature doesn't matter.
        t.battlefield(P2, "Craw Wurm");
        let hand = t.hand_size(P0);
        t.g.destroy(victim, None);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws, "{dying} with {other}");
    }
}

#[test]
fn same_mana_value_as_the_revealed_card_counterbalance() {
    ruling!(
        "Counterbalance",
        "the mana value of that spell takes the value of X into account"
    );
    compiles("Counterbalance");
    for (top, countered) in [("Hill Giant", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Counterbalance");
        t.library_top(P0, top);
        t.answer_yes(P0, true);
        t.lands(P1, "Mountain", 4);
        let wurm = t.hand(P1, "Hill Giant"); // mana value 4
        t.set_step(P1, Step::PrecombatMain);
        t.cast(P1, wurm).go();
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Hill Giant").is_empty(),
            countered,
            "{top}"
        );
    }
    // A spell with X: Blaze with X = 3 has mana value 4.
    for (top, countered) in [("Hill Giant", true), ("Grizzly Bears", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Counterbalance");
        t.library_top(P0, top);
        t.answer_yes(P0, true);
        t.lands(P1, "Mountain", 4);
        let blaze = t.hand(P1, "Blaze");
        t.set_step(P1, Step::PrecombatMain);
        t.cast(P1, blaze).x(3).target(Entity::Player(P0)).go();
        t.resolve_all();
        assert_eq!(t.life(P0) == 20, countered, "Blaze, {top}");
    }
}

#[test]
fn cards_of_each_of_the_sacrificed_creatures_colors_mind_extraction() {
    ruling!(
        "Mind Extraction",
        "all cards that share at least one color with the creature are discarded"
    );
    compiles("Mind Extraction");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let sac = t.battlefield(P0, "Grizzly Bears"); // green
    t.hand(P1, "Llanowar Elves");
    t.hand(P1, "Lightning Bolt");
    let me = t.hand(P0, "Mind Extraction");
    t.answer_choose(P0, &[Entity::Object(sac)]);
    t.cast(P0, me).target(P1).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(t.in_hand(P1, "Lightning Bolt"));
}

#[test]
fn each_spell_and_ability_your_opponents_control_whirlwind_denial() {
    cr!("701.6b");
    ruling!(
        "Whirlwind Denial",
        "only activated and triggered abilities use the stack"
    );
    compiles("Whirlwind Denial");
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Island", 3);
    let wd = t.hand(P0, "Whirlwind Denial");
    t.cast(P0, wd).go();
    // P1 can't pay {4}.
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // An activated ability is countered too.
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, sorcerer, 0, &[Entity::Player(P0)]).unwrap();
    t.lands(P0, "Island", 3);
    let wd = t.hand(P0, "Whirlwind Denial");
    t.cast(P0, wd).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // (Without it, the ability deals 1 damage.)
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, sorcerer, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn lesser_mana_value_than_the_sacrificed_creature_kethek() {
    compiles("Kethek, Crucible Goliath");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kethek, Crucible Goliath"); // mana value 4
    t.battlefield(P0, "Hill Giant"); // mana value 4
                                     // From the top: a 6-drop (not lesser), then a 2-drop.
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    // Compared with the sacrificed creature, not Kethek: sacrificing Craw Wurm (6), a
    // Hill Giant (4, as Kethek) is lesser.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kethek, Crucible Goliath");
    t.battlefield(P0, "Craw Wurm");
    t.library_top(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn target_artifact_that_opponent_controls_parapet_thrasher() {
    compiles("Parapet Thrasher");
    let mut t = TestGame::new(3);
    let pt = t.battlefield(P0, "Parapet Thrasher");
    let theirs = t.battlefield(P1, "Ornithopter");
    let other = t.battlefield(P2, "Ornithopter");
    t.answer(
        P0,
        mtg_engine::testing::DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![0]),
    );
    // Only P1's artifact can be the target.
    t.answer_targets(P0, &[Entity::Object(other)]);
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(pt, Entity::Player(P1))], &[]);
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(other));
}

#[test]
fn lesser_mana_value_than_the_triggering_spell_rashmi() {
    compiles("Rashmi, Eternities Crafter");
    for (top, cast) in [("Grizzly Bears", true), ("Craw Wurm", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Rashmi, Eternities Crafter");
        t.library_top(P0, top);
        t.lands(P0, "Forest", 3);
        let giant = t.hand(P0, "Hill Giant"); // mana value 4
        t.lands(P0, "Mountain", 1);
        t.answer_yes(P0, true);
        t.cast(P0, giant).go();
        t.resolve_all();
        if cast {
            assert_eq!(t.named_on_battlefield(top).len(), 1, "{top}");
        } else {
            assert!(t.in_hand(P0, top), "{top}");
        }
    }
}

#[test]
fn if_the_token_is_an_aura_yenna() {
    compiles("Yenna, Redtooth Regent");
    let mut t = TestGame::new(2);
    let yenna = t.battlefield(P0, "Yenna, Redtooth Regent");
    t.lands(P0, "Forest", 2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    t.activate(P0, yenna, 0, &[Entity::Object(anthem)]).unwrap();
    t.resolve();
    // A copy of a non-Aura enchantment: Yenna stays tapped.
    assert_eq!(t.named_on_battlefield("Glorious Anthem").len(), 2);
    assert!(t.obj_now(yenna).tapped);
    // Now each Glorious Anthem has the same name as another permanent: no legal target.
    t.g.objects[yenna.0 as usize].tapped = false;
    t.lands(P0, "Forest", 2);
    assert!(t.activate(P0, yenna, 0, &[Entity::Object(anthem)]).is_err());
}

#[test]
fn delayed_enters_trigger_of_a_returned_permanent_isnt_read_jailbreak() {
    // "Return target permanent card ... to the battlefield under their control. When that
    // permanent enters, ...": the permanent that enters is a new object (CR 400.7), which
    // the delayed trigger's captured referent can't follow. Rejected rather than compiled
    // into a trigger that never fires.
    assert!(!card("Jailbreak").unsupported_text().is_empty());
}

#[test]
fn equal_or_lesser_than_the_triggering_spell_kellan() {
    ruling!(
        "Kellan, the Kid",
        "X is the value chosen as that spell was cast for the purpose of determining its mana value"
    );
    compiles("Kellan, the Kid");
    // A spell cast from the graveyard (Think Twice by flashback, mana value 2): a
    // permanent spell with mana value 2 or less from your hand, not one with Kellan's own
    // 3.
    for (in_hand, cast) in [("Grizzly Bears", true), ("Centaur Courser", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Kellan, the Kid");
        t.lands(P0, "Island", 3);
        let tt = t.graveyard(P0, "Think Twice");
        let c = t.hand(P0, in_hand);
        t.set_step(P0, Step::PrecombatMain);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(c)]);
        t.cast(P0, tt)
            .method(mtg_engine::object::CastMethod::Keyword(
                KeywordKind::Flashback,
            ))
            .go();
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield(in_hand).len(),
            cast as usize,
            "{in_hand}"
        );
    }
    // An X spell cast from the top of the library (Future Sight): its X counts. Blaze
    // with X = 2 has mana value 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kellan, the Kid");
    t.battlefield(P0, "Future Sight");
    t.lands(P0, "Mountain", 3);
    let blaze = t.library_top(P0, "Blaze");
    let c = t.hand(P0, "Centaur Courser");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(c)]);
    t.cast(P0, blaze).x(2).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Centaur Courser").len(), 1);
}

#[test]
fn exchange_with_equal_or_lesser_mana_value_pucas_mischief() {
    ruling!(
        "Puca's Mischief",
        "You choose two targets such that as a set, all of the targeting conditions are met"
    );
    compiles("Puca's Mischief");
    // Only a 6-drop opposite your 2- and 4-drops: no legal pair.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Puca's Mischief");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).controller, P1);
    assert_eq!(t.obj_now(bears).controller, P0);
    // An Elves (1) is lesser than the Bears (2).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Puca's Mischief");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Craw Wurm");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(elves).controller, P0);
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn lesser_mana_value_than_the_creature_that_died_deaths_oasis() {
    ruling!(
        "Death's Oasis",
        "Use the mana value of the creature as it last existed on the battlefield"
    );
    compiles("Death's Oasis");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Death's Oasis"); // mana value 3
    let giant = t.battlefield(P0, "Hill Giant"); // mana value 4
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.graveyard(P0, "Craw Wurm"); // 6
    t.graveyard(P0, "Centaur Courser"); // 3
    t.g.destroy(giant, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Centaur Courser"));
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn creatures_that_each_have_toughness_greater_than_their_power_catapult_fodder() {
    ruling!(
        "Catapult Fodder // Catapult Captain",
        "each of the three creatures must have toughness greater than its own power"
    );
    let run = |second_wall: bool| {
        let mut t = TestGame::new(2);
        let f = t.battlefield(P0, "Catapult Fodder // Catapult Captain"); // 1/5
        t.battlefield(P0, "Wall of Stone"); // 0/8
                                            // 3/3: toughness isn't greater than its power (the total toughness of the three
                                            // is).
        t.battlefield(P0, "Hill Giant");
        if second_wall {
            t.battlefield(P0, "Wall of Stone");
        }
        t.set_step(P0, Step::PrecombatMain);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        t.obj_now(f).chars.name.to_string()
    };
    assert_eq!(run(false), "Catapult Fodder");
    assert_eq!(run(true), "Catapult Captain");
}

#[test]
fn mana_value_power_or_toughness_equal_to_the_chosen_number_talion() {
    ruling!(
        "Talion, the Kindly Lord",
        "use the value chosen for X to determine if the spell's mana value is the chosen number"
    );
    compiles("Talion, the Kindly Lord");
    let setup = |n: i32| {
        let mut t = TestGame::new(2);
        let tal = t.battlefield(P0, "Talion, the Kindly Lord");
        t.g.objects[tal.0 as usize].choices.number = Some(n);
        t.lands(P1, "Mountain", 4);
        t.set_step(P1, Step::PrecombatMain);
        t
    };
    // Hill Giant: mana value 4, power and toughness 3.
    for (n, hit) in [(3, true), (4, true), (2, false)] {
        let mut t = setup(n);
        let hand = t.hand_size(P0);
        let g = t.hand(P1, "Hill Giant");
        t.cast(P1, g).go();
        t.resolve_all();
        assert_eq!(t.life(P1), if hit { 18 } else { 20 }, "chosen {n}");
        assert_eq!(t.hand_size(P0), hand + hit as usize, "chosen {n}");
    }
    // Blaze with X = 3: mana value 4.
    let mut t = setup(4);
    let blaze = t.hand(P1, "Blaze");
    t.cast(P1, blaze).x(3).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn more_creatures_of_the_chosen_type_than_each_other_player_peer_pressure() {
    compiles("Peer Pressure");
    let idx = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Bear")
        .unwrap();
    for (theirs, gain) in [(1, true), (2, false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 4);
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Grizzly Bears");
        // Creatures of another type don't count.
        t.battlefield(P1, "Hill Giant");
        t.battlefield(P1, "Hill Giant");
        let bears: Vec<ObjectId> = (0..theirs)
            .map(|_| t.battlefield(P1, "Grizzly Bears"))
            .collect();
        let pp = t.hand(P0, "Peer Pressure");
        t.answer(
            P0,
            DecisionKind::Option,
            mtg_engine::decision::Answer::Index(idx),
        );
        t.cast(P0, pp).go();
        t.resolve();
        let ctl = if gain { P0 } else { P1 };
        assert!(
            bears.iter().all(|b| t.obj_now(*b).controller == ctl),
            "{theirs}"
        );
        assert!(t
            .named_on_battlefield("Hill Giant")
            .iter()
            .all(|g| t.obj_now(*g).controller == P1));
    }
}

#[test]
fn if_the_token_is_an_aura_untap_yenna() {
    compiles("Yenna, Redtooth Regent");
    let mut t = TestGame::new(2);
    let yenna = t.battlefield(P0, "Yenna, Redtooth Regent");
    t.lands(P0, "Forest", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Rancor");
    t.g.attach(aura, Entity::Object(bears));
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, yenna, 0, &[Entity::Object(aura)]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Rancor").len(), 2);
    assert!(!t.obj_now(yenna).tapped);
}
