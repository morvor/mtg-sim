//! Characteristic-changing grammar (`oracle/patterns/becomes_grammar.rs`): one subject
//! with several predicates that change what it is ("gets +1/+0, becomes black, and gains
//! shadow"), "becomes [type words]" for groups and Vehicles, base power and toughness,
//! "is [characteristic]" statics (in several zones too), and "it's a [type]" after an
//! instruction that puts a permanent onto the battlefield (CR 205.1, 611.2, 613).

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn chars(t: &TestGame, id: ObjectId) -> &object::Characteristics {
    &t.obj_now(id).chars
}

#[test]
fn traitors_clutch_pumps_colors_and_grants_shadow_together() {
    cr!("611.2a", "613.1e", "613.1f", "613.4c");
    assert_supported("Traitor's Clutch");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let clutch = t.hand(P0, "Traitor's Clutch");
    t.lands(P0, "Swamp", 5);
    t.cast(P0, clutch).target(bear).go();
    t.resolve();
    assert_eq!(t.pt(bear), (3, 2));
    assert_eq!(chars(&t, bear).colors, ColorSet::single(Color::Black));
    assert!(chars(&t, bear).has_keyword(KeywordKind::Shadow));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bear), (2, 2));
    assert_eq!(chars(&t, bear).colors, ColorSet::single(Color::Green));
    assert!(!chars(&t, bear).has_keyword(KeywordKind::Shadow));
}

#[test]
fn curious_colossus_shrinks_and_cowers_each_creature_of_the_target_opponent() {
    cr!("613.1d", "613.1f", "613.4b", "611.2c");
    ruling!("Curious Colossus", "last indefinitely");
    ruling!("Curious Colossus", "Creatures they begin to control later won't be affected");
    assert_supported("Curious Colossus");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "White Knight");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let colossus = t.hand(P0, "Curious Colossus");
    t.lands(P0, "Plains", 7);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, colossus).go();
    t.resolve_all();
    assert_eq!(t.pt(knight), (1, 1));
    assert!(chars(&t, knight).has_subtype("Coward"));
    assert!(chars(&t, knight).has_subtype("Knight"));
    assert!(!chars(&t, knight).has_keyword(KeywordKind::FirstStrike));
    assert_eq!(t.pt(mine), (2, 2));
    // Indefinitely; and not creatures that player controls later.
    let later = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(knight), (1, 1));
    assert_eq!(t.pt(later), (2, 2));
}

#[test]
fn werewolf_pack_leader_has_base_pt_gains_trample_and_isnt_a_human() {
    cr!("613.4b", "613.1d", "205.1a");
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Werewolf Pack Leader");
    assert!(chars(&t, wolf).has_subtype("Human"));
    t.lands(P0, "Forest", 4);
    t.activate(P0, wolf, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(wolf), (5, 3));
    assert!(chars(&t, wolf).has_keyword(KeywordKind::Trample));
    assert!(!chars(&t, wolf).has_subtype("Human"));
    assert!(chars(&t, wolf).has_subtype("Werewolf"));
}

#[test]
fn start_your_engines_makes_vehicles_artifact_creatures_with_printed_pt() {
    cr!("301.7b", "205.1b");
    assert_supported("Start Your Engines");
    let mut t = TestGame::new(2);
    let car = t.battlefield(P0, "Smuggler's Copter");
    let rock = t.battlefield(P0, "Mind Stone");
    let spell = t.hand(P0, "Start Your Engines");
    t.lands(P0, "Mountain", 4);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(chars(&t, car).is_creature());
    assert!(chars(&t, car).card_types.contains(CardType::Artifact));
    assert!(chars(&t, car).has_subtype("Vehicle"));
    // 3/3 printed, +2/+0 from the second sentence.
    assert_eq!(t.pt(car), (5, 3));
    assert!(!chars(&t, rock).is_creature());
    t.advance_to(P1, Step::Upkeep);
    assert!(!chars(&t, car).is_creature());
}

#[test]
fn coward_cant_block_and_is_a_coward_until_end_of_turn() {
    cr!("205.1b", "509.1b");
    assert_supported("Coward // Killer");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let coward = t.hand(P0, "Coward // Killer");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, coward)
        .method(object::CastMethod::Half(0))
        .target(bear)
        .go();
    t.resolve();
    assert!(chars(&t, bear).has_subtype("Coward"));
    assert!(chars(&t, bear).has_subtype("Bear"));
    let attacker = t.battlefield(P0, "Craw Wurm");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bear, attacker)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn phantasmal_form_sets_base_pt_and_adds_blue_illusion() {
    cr!("613.4b", "613.4c", "613.7", "105.3", "205.1b");
    ruling!("Phantasmal Form", "overwrites all previous effects");
    assert_supported("Phantasmal Form");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Craw Wurm");
    // Earlier: a base power and toughness setting effect (7b) on the Wurm, and a pump
    // (7c) and a +1/+1 counter on the Bears.
    t.lands(P0, "Island", 5);
    t.lands(P0, "Forest", 1);
    let fractal = t.hand(P0, "Fractalize");
    t.cast(P0, fractal).x(0).target(b).go();
    t.resolve();
    assert_eq!(t.pt(b), (1, 1));
    t.g.add_counters(Entity::Object(a), "+1/+1", 1, None);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(a).go();
    t.resolve();
    let form = t.hand(P0, "Phantasmal Form");
    t.cast(P0, form)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    // 3/3 overwrites the earlier 1/1 (later timestamp in 7b); the pump and the counter
    // still apply on top of it (7c after 7b).
    assert_eq!(t.pt(a), (7, 7));
    assert_eq!(t.pt(b), (3, 3));
    for x in [a, b] {
        assert!(chars(&t, x).has_keyword(KeywordKind::Flying));
        assert!(chars(&t, x).has_subtype("Illusion"));
        assert!(chars(&t, x).colors.contains(Color::Blue));
        assert!(chars(&t, x).colors.contains(Color::Green));
    }
    assert!(chars(&t, a).has_subtype("Bear"));
    // A setting effect that starts to apply later overwrites it.
    let mut t2 = TestGame::new(2);
    let c = t2.battlefield(P0, "Grizzly Bears");
    t2.lands(P0, "Island", 5);
    let form = t2.hand(P0, "Phantasmal Form");
    t2.cast(P0, form).targets(&[Entity::Object(c)]).go();
    t2.resolve();
    let fractal = t2.hand(P0, "Fractalize");
    t2.cast(P0, fractal).x(0).target(c).go();
    t2.resolve();
    assert_eq!(t2.pt(c), (1, 1));
    // Fractalize sets the colors (green and blue) and creature types (Fractal only)
    // after Phantasmal Form added blue and Illusion.
    assert!(!chars(&t2, c).has_subtype("Illusion") && chars(&t2, c).has_subtype("Fractal"));
}

#[test]
fn touch_of_darkness_turns_any_number_of_targets_black() {
    cr!("105.3", "601.2c");
    assert_supported("Touch of Darkness");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Savannah Lions");
    let c = t.battlefield(P1, "Craw Wurm");
    let touch = t.hand(P0, "Touch of Darkness");
    t.lands(P0, "Swamp", 1);
    t.cast(P0, touch)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert_eq!(chars(&t, a).colors, ColorSet::single(Color::Black));
    assert_eq!(chars(&t, b).colors, ColorSet::single(Color::Black));
    assert_eq!(chars(&t, c).colors, ColorSet::single(Color::Green));
}

#[test]
fn mercurial_transformation_makes_the_chosen_creature_as_it_resolves() {
    cr!("608.2d", "613.1f", "613.4b");
    ruling!("Mercurial Transformation", "You choose Frog or Octopus as Mercurial Transformation resolves");
    assert_supported("Mercurial Transformation");
    let mut t = TestGame::new(2);
    let rock = t.battlefield(P1, "Mind Stone");
    let spell = t.hand(P0, "Mercurial Transformation");
    t.lands(P0, "Island", 2);
    t.cast(P0, spell).target(rock).go();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve();
    let c = chars(&t, rock);
    assert!(c.is_creature());
    assert!(!c.card_types.contains(CardType::Artifact));
    assert!(c.has_subtype("Octopus"));
    assert_eq!(c.colors, ColorSet::single(Color::Blue));
    assert!(c.has_no_abilities());
    assert_eq!(t.pt(rock), (4, 4));
}

#[test]
fn tundra_kavu_makes_a_land_the_chosen_basic_land_type() {
    cr!("305.7", "608.2d");
    assert_supported("Tundra Kavu");
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Tundra Kavu");
    let land = t.battlefield(P1, "Mountain");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, kavu, 0, &[Entity::Object(land)]).unwrap();
    t.resolve();
    assert!(chars(&t, land).has_subtype("Island"));
    assert!(!chars(&t, land).has_subtype("Mountain"));
}

#[test]
fn earnest_fellowship_gives_each_creature_protection_from_its_colors() {
    cr!("702.16b", "702.16e", "702.16g");
    assert_supported("Earnest Fellowship");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Earnest Fellowship");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let golem = t.battlefield(P1, "Ornithopter");
    let shock = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, shock).target(bear).go();
    t.resolve();
    assert!(!t.on_battlefield(bear));
    // A red creature can't be targeted by a red spell.
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.g.object_untargetable(goblin, P0, Some(bolt)));
    // A colorless creature is protected from nothing.
    assert!(!t.g.object_untargetable(golem, P0, Some(bolt)));
}

#[test]
fn tam_gives_other_creatures_hexproof_from_their_colors() {
    cr!("702.11d", "702.11g");
    ruling!("Tam, Mindful First-Year", "Colorless is not a color");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tam, Mindful First-Year");
    let goblin = t.battlefield(P0, "Raging Goblin");
    let bolt = t.hand(P1, "Lightning Bolt");
    let dark = t.hand(P1, "Disfigure");
    assert!(t.g.object_untargetable(goblin, P1, Some(bolt)));
    assert!(!t.g.object_untargetable(goblin, P1, Some(dark)));
    // Hexproof only stops opponents.
    let own = t.hand(P0, "Lightning Bolt");
    assert!(!t.g.object_untargetable(goblin, P0, Some(own)));
}

#[test]
fn samite_elder_locks_the_colors_as_it_resolves() {
    cr!("608.2h", "702.16b");
    ruling!("Samite Elder", "The color is determined on resolution");
    ruling!("Samite Elder", "the color of protection that was granted does not change");
    assert_supported("Samite Elder");
    let mut t = TestGame::new(2);
    let elder = t.battlefield(P0, "Samite Elder");
    let goblin = t.battlefield(P0, "Raging Goblin");
    t.activate(P0, elder, 0, &[Entity::Object(goblin)]).unwrap();
    t.resolve();
    let bolt = t.hand(P1, "Lightning Bolt");
    let dark = t.hand(P1, "Disfigure");
    assert!(t.g.object_untargetable(elder, P1, Some(bolt)));
    assert!(!t.g.object_untargetable(elder, P1, Some(dark)));
    // The goblin turning black doesn't change what the protection is from.
    let touch = t.hand(P0, "Touch of Darkness");
    t.lands(P0, "Swamp", 1);
    t.cast(P0, touch).target(goblin).go();
    t.resolve();
    assert!(t.g.object_untargetable(elder, P1, Some(bolt)));
    assert!(!t.g.object_untargetable(elder, P1, Some(dark)));
}

#[test]
fn leyline_of_the_guildpact_makes_nonland_permanents_all_colors() {
    cr!("105.3", "613.1e");
    assert_supported("Leyline of the Guildpact");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of the Guildpact");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let rock = t.battlefield(P0, "Mind Stone");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P0, "Wastes");
    assert_eq!(chars(&t, bear).colors, ColorSet::ALL);
    assert_eq!(chars(&t, rock).colors, ColorSet::ALL);
    assert_eq!(chars(&t, theirs).colors, ColorSet::single(Color::Green));
    assert_eq!(chars(&t, land).colors, ColorSet::NONE);
}

#[test]
fn melting_and_arcums_weathervane_change_the_snow_supertype() {
    cr!("205.4a", "613.1d");
    ruling!("Arcum's Weathervane", "Adding snow doesn’t overwrite other supertypes");
    assert_supported("Melting");
    assert_supported("Arcum's Weathervane");
    let mut t = TestGame::new(2);
    let snowy = t.battlefield(P0, "Snow-Covered Forest");
    let plain = t.battlefield(P0, "Forest");
    let vane = t.battlefield(P0, "Arcum's Weathervane");
    t.lands(P0, "Plains", 2);
    // {2}, {T}: Target nonsnow basic land becomes snow.
    t.activate(P0, vane, 1, &[Entity::Object(plain)]).unwrap();
    t.resolve();
    assert!(chars(&t, plain).has_supertype(Supertype::Snow));
    assert!(chars(&t, plain).has_supertype(Supertype::Basic));
    t.battlefield(P1, "Melting");
    assert!(!chars(&t, snowy).has_supertype(Supertype::Snow));
    assert!(!chars(&t, plain).has_supertype(Supertype::Snow));
    assert!(chars(&t, snowy).has_supertype(Supertype::Basic));
}

#[test]
fn mycosynth_lattice_makes_cards_in_every_zone_colorless() {
    cr!("611.3a", "105.2c");
    ruling!("Mycosynth Lattice", "makes everything, in every zone of the game, colorless");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mycosynth Lattice");
    let bear = t.battlefield(P1, "Grizzly Bears");
    let bolt = t.hand(P1, "Lightning Bolt");
    let dead = t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    for x in [bear, bolt, dead] {
        assert_eq!(chars(&t, x).colors, ColorSet::NONE);
    }
    assert!(chars(&t, bear).card_types.contains(CardType::Artifact));
}

#[test]
fn dune_chanter_makes_lands_everywhere_deserts() {
    cr!("611.3a", "205.1b");
    assert_supported("Dune Chanter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dune Chanter");
    let field = t.battlefield(P0, "Forest");
    let top = t.library_top(P0, "Mountain");
    let theirs = t.battlefield(P1, "Forest");
    t.g.recompute();
    assert!(chars(&t, field).has_subtype("Desert"));
    assert!(chars(&t, field).has_subtype("Forest"));
    assert!(chars(&t, top).has_subtype("Desert"));
    assert!(!chars(&t, theirs).has_subtype("Desert"));
}

#[test]
fn ever_after_returns_black_zombies() {
    cr!("611.2e", "105.3", "205.1b");
    ruling!("Ever After", "It won’t be both black and colorless");
    assert_supported("Ever After");
    let mut t = TestGame::new(2);
    let bear = t.graveyard(P0, "Grizzly Bears");
    let thopter = t.graveyard(P0, "Ornithopter");
    let spell = t.hand(P0, "Ever After");
    t.lands(P0, "Swamp", 6);
    t.cast(P0, spell)
        .targets(&[Entity::Object(bear), Entity::Object(thopter)])
        .go();
    t.resolve();
    let bears = t.named_on_battlefield("Grizzly Bears");
    let thopters = t.named_on_battlefield("Ornithopter");
    let (bear, thopter) = (bears[0], thopters[0]);
    assert!(chars(&t, bear).has_subtype("Zombie"));
    assert!(chars(&t, bear).has_subtype("Bear"));
    assert!(chars(&t, bear).colors.contains(Color::Green));
    assert!(chars(&t, bear).colors.contains(Color::Black));
    assert_eq!(chars(&t, thopter).colors, ColorSet::single(Color::Black));
}

#[test]
fn call_a_surprise_witness_returns_a_flying_spirit() {
    cr!("205.1b", "122.1b");
    assert_supported("Call a Surprise Witness");
    let mut t = TestGame::new(2);
    let bear = t.graveyard(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Call a Surprise Witness");
    t.lands(P0, "Plains", 2);
    t.cast(P0, spell).target(bear).go();
    t.resolve();
    let bear = t.named_on_battlefield("Grizzly Bears")[0];
    assert!(chars(&t, bear).has_subtype("Spirit"));
    assert!(chars(&t, bear).has_keyword(KeywordKind::Flying));
}

#[test]
fn fog_patch_and_choking_vines_make_attackers_blocked() {
    cr!("509.1h", "510.1c");
    ruling!("Choking Vines", "You may target a blocked creature with Choking Vines");
    assert_supported("Fog Patch");
    assert_supported("Choking Vines");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Craw Wurm");
    let b = t.battlefield(P0, "Grizzly Bears");
    let patch = t.hand(P1, "Fog Patch");
    t.lands(P1, "Forest", 2);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, Entity::Player(P1)), (b, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.cast(P1, patch).go();
    t.resolve();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);

    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Craw Wurm");
    let b = t.battlefield(P0, "Grizzly Bears");
    let vines = t.hand(P1, "Choking Vines");
    t.lands(P1, "Forest", 2);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, Entity::Player(P1)), (b, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.cast(P1, vines).x(1).target(b).go();
    t.resolve();
    t.advance_to(P0, Step::EndOfCombat);
    // The Wurm is unblocked; the Bears were blocked and dealt 1 damage.
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.obj_now(b).damage, 1);
}

#[test]
fn biorhythm_sets_each_life_total_to_that_players_creature_count() {
    cr!("119.5");
    ruling!("Biorhythm", "their life total will become zero");
    assert_supported("Biorhythm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Craw Wurm");
    let spell = t.hand(P0, "Biorhythm");
    t.lands(P0, "Forest", 8);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.life(P0), 3);
    assert!(t.has_lost(P1));
}

#[test]
fn secret_arcade_makes_your_permanent_spells_and_nonland_permanents_enchantments() {
    cr!("611.3a", "205.1b");
    assert_supported("Secret Arcade // Dusty Parlor");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let room = t.hand(P0, "Secret Arcade // Dusty Parlor");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P0, "Forest");
    t.lands(P0, "Plains", 7);
    // Secret Arcade's door is what has the ability: before it's on the battlefield
    // unlocked, nothing is an enchantment.
    assert!(!chars(&t, mine).card_types.contains(CardType::Enchantment));
    t.cast(P0, room).method(object::CastMethod::Half(0)).go();
    t.resolve();
    assert!(chars(&t, mine).card_types.contains(CardType::Enchantment));
    assert!(chars(&t, mine).is_creature());
    assert!(!chars(&t, theirs).card_types.contains(CardType::Enchantment));
    assert!(!chars(&t, land).card_types.contains(CardType::Enchantment));
    // A permanent spell you control on the stack is one too, but a card in hand isn't.
    let bear = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.g.recompute();
    assert!(!chars(&t, bear).card_types.contains(CardType::Enchantment));
    let spell = t.cast(P0, bear).go();
    t.g.recompute();
    assert!(chars(&t, spell).card_types.contains(CardType::Enchantment));
    assert!(chars(&t, spell).is_creature());
}

#[test]
fn curious_colossus_creatures_keep_abilities_and_pt_settings_gained_later() {
    cr!("613.1f", "613.4b", "613.7");
    ruling!("Curious Colossus", "If one of the affected creatures gains an ability after");
    ruling!(
        "Curious Colossus",
        "Any power- or toughness-setting effects that start to apply after the ability resolves will overwrite this effect"
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let colossus = t.hand(P0, "Curious Colossus");
    t.lands(P0, "Plains", 7);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, colossus).go();
    t.resolve_all();
    assert_eq!(t.pt(bear), (1, 1));
    // Later effects in the same layers: an ability gained (layer 6) and a base power and
    // toughness set (layer 7b) apply after the Colossus's, so they stick.
    t.lands(P0, "Island", 5);
    let form = t.hand(P0, "Phantasmal Form");
    t.cast(P0, form).targets(&[Entity::Object(bear)]).go();
    t.resolve();
    assert_eq!(t.pt(bear), (3, 3));
    assert!(chars(&t, bear).has_keyword(KeywordKind::Flying));
    assert!(chars(&t, bear).has_subtype("Coward"));
}
