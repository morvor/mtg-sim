//! Rulings batch P160 — "shapechange" rulings beyond layer ordering: unblocked creatures,
//! exchanging power, abilities gained alongside or after a base-P/T effect, Vehicles and
//! lands that become creatures, copies made by token-copy effects, Equipment and Sagas
//! that become creatures, and triggers of the cards involved.

use crate::r_p160_common::*;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

#[test]
fn inkfathom_witch_affects_only_unblocked_creatures() {
    cr!("509.1h", "511.3", "611.2c");
    ruling!(
        "Inkfathom Witch",
        "An “unblocked creature” is a creature that attacked and wasn't blocked. Creatures aren't “blocked” or “unblocked” until the declare blockers step, so activating this ability before then (or after combat ends) will have no effect."
    );
    ruling!(
        "Inkfathom Witch",
        "If Inkfathom Witch is attacking and unblocked, it can make itself 4/1."
    );
    ruling!(
        "Inkfathom Witch",
        "Creatures stop being unblocked as the combat phase ends. However, they'll stay 4/1 until turn ends."
    );
    ruling!(
        "Inkfathom Witch",
        "Inkfathom Witch doesn't cause creatures to lose their abilities."
    );
    supported("Inkfathom Witch");
    let mut t = TestGame::new(2);
    let witch = t.battlefield(P0, "Inkfathom Witch");
    let pay = |t: &mut TestGame| {
        mana(t, P0, ManaType::U, 1);
        mana(t, P0, ManaType::B, 1);
        mana(t, P0, ManaType::C, 2);
    };
    // In the declare attackers step, nothing is unblocked yet.
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(witch, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    pay(&mut t);
    activate_resolve(&mut t, P0, witch, 0, &[]);
    assert_eq!(t.pt(witch), (1, 1));
    // In the declare blockers step, the unblocked Witch makes itself 4/1.
    t.advance_to(P0, Step::DeclareBlockers);
    pay(&mut t);
    activate_resolve(&mut t, P0, witch, 0, &[]);
    assert_eq!(t.pt(witch), (4, 1));
    assert!(t.obj(witch).has_keyword(KeywordKind::Fear));
    // After combat it's no longer unblocked, but it stays 4/1; activating again does
    // nothing to other creatures.
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.pt(witch), (4, 1));
    let bears = t.battlefield(P0, "Grizzly Bears");
    pay(&mut t);
    activate_resolve(&mut t, P0, witch, 0, &[]);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn mimics_keep_other_abilities_they_gained() {
    cr!("613.1f", "613.4b");
    ruling!(
        "Battlegate Mimic",
        "Any other abilities the Mimic may have gained are not affected."
    );
    ruling!(
        "Nightsky Mimic",
        "Any other abilities the Mimic may have gained are not affected."
    );
    ruling!(
        "Riverfall Mimic",
        "Any other abilities the Mimic may have gained are not affected."
    );
    ruling!(
        "Shorecrasher Mimic",
        "Any other abilities the Mimic may have gained are not affected."
    );
    ruling!(
        "Woodlurker Mimic",
        "Any other abilities the Mimic may have gained are not affected."
    );
    supported("Jump");
    // (Mimic, the spell of both its colors, that spell's targets, the base P/T, and the
    // keyword it gains.)
    let cases: [(&str, &str, (i32, i32), Option<KeywordKind>); 5] = [
        (
            "Battlegate Mimic",
            "Lightning Helix",
            (4, 2),
            Some(KeywordKind::FirstStrike),
        ),
        (
            "Nightsky Mimic",
            "Vindicate",
            (4, 4),
            Some(KeywordKind::Flying),
        ),
        ("Riverfall Mimic", "Electrolyze", (3, 3), None),
        (
            "Shorecrasher Mimic",
            "Growth Spiral",
            (5, 3),
            Some(KeywordKind::Trample),
        ),
        (
            "Woodlurker Mimic",
            "Putrefy",
            (4, 5),
            Some(KeywordKind::Wither),
        ),
    ];
    for (mimic, spell, pt, kw) in cases {
        supported(mimic);
        supported(spell);
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, mimic);
        let victim = t.battlefield(P1, "Grizzly Bears");
        // An ability gained earlier: flying (or vigilance via a different source isn't
        // needed — flying is enough to observe).
        cast_resolve(&mut t, P0, "Jump", &[Entity::Object(m)]);
        let targets: Vec<Entity> = match spell {
            "Lightning Helix" | "Electrolyze" => vec![Entity::Player(P1)],
            "Vindicate" | "Putrefy" => vec![Entity::Object(victim)],
            _ => vec![],
        };
        if spell == "Electrolyze" {
            t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2]));
        }
        cast_resolve(&mut t, P0, spell, &targets);
        assert_eq!(t.pt(m), pt, "{mimic}");
        assert!(t.obj(m).has_keyword(KeywordKind::Flying), "{mimic}");
        if let Some(k) = kw {
            assert!(t.obj(m).has_keyword(k), "{mimic}");
        }
    }
}

#[test]
fn serene_master_exchanges_power_with_the_creature_it_blocks() {
    cr!("701.12g", "613.4b", "613.4c");
    ruling!(
        "Serene Master",
        "Any power-modifying effects, counters, Auras, or Equipment will apply to the creatures’ new powers. For example, say Serene Master is enchanted with Lightning Talons, which gives it +3/+0, and it blocks a 5/5 creature. After the exchange, Serene Master would be an 8/2 creature (its power became 5, which was then modified by Lightning Talons), and the other creature would be 3/5."
    );
    supported("Serene Master");
    supported("Lightning Talons");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Serene Master");
    cast_resolve(&mut t, P0, "Lightning Talons", &[Entity::Object(master)]);
    assert_eq!(t.pt(master), (3, 2));
    let saur = t.battlefield(P1, "Charging Monstrosaur");
    t.advance_to(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(saur, Entity::Player(P0))]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(master, saur)]),
    );
    t.answer_targets(P0, &[Entity::Object(saur)]);
    t.advance_to(P1, Step::DeclareBlockers);
    t.resolve_all();
    assert_eq!(t.pt(master), (8, 2));
    assert_eq!(t.pt(saur), (3, 5));
}

#[test]
fn serene_master_blocking_two_creatures_triggers_once() {
    cr!("509.3c", "603.2c");
    ruling!(
        "Serene Master",
        "If Serene Master blocks multiple creatures, its ability will trigger only once, and it will exchange power with only the target creature."
    );
    supported("Brave the Sands");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Serene Master");
    t.battlefield(P0, "Brave the Sands");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, Entity::Player(P0)), (b, Entity::Player(P0))]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(master, a), (master, b)]),
    );
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.advance_to(P1, Step::DeclareBlockers);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(master), (3, 2));
    assert_eq!(t.pt(a), (0, 3));
    assert_eq!(t.pt(b), (2, 2));
}

#[test]
fn chrome_host_hulk_overwrites_earlier_setting_effects() {
    cr!("613.4b", "613.4c", "613.7b");
    ruling!(
        "Gnottvold Hermit // Chrome Host Hulk",
        "Chrome Host Hulk’s ability overwrites all previous effects that set the affected creatures’ power and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after the ability resolves will overwrite that part of the effect."
    );
    supported("Gnottvold Hermit // Chrome Host Hulk");
    let mut t = TestGame::new(2);
    let hulk = t.battlefield(P0, "Gnottvold Hermit // Chrome Host Hulk");
    mtg_engine::dfc::transform(&mut t.g, hulk);
    let bears = t.battlefield(P0, "Grizzly Bears");
    plus_counters(&mut t, bears, 1);
    cast_resolve(&mut t, P0, "Relic's Roar", &[Entity::Object(bears)]);
    assert_eq!(t.pt(bears), (5, 4));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.attack(&[(t.g.current(hulk), Entity::Player(P1))], &[]);
    assert_eq!(t.pt(bears), (6, 6));
    cast_resolve(&mut t, P0, "Square Up", &[Entity::Object(bears)]);
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn dollmakers_shop_triggers_per_player_attacked_and_not_for_creatures_entering_attacking() {
    cr!("508.3a", "508.4");
    ruling!(
        "Dollmaker's Shop // Porcelain Gallery",
        "Creatures that enter attacking are never declared as attackers, and as such, they won't cause Dollmaker's Shop's ability to trigger."
    );
    ruling!(
        "Dollmaker's Shop // Porcelain Gallery",
        "Dollmaker's Shop's ability will trigger once for each player you attack with one or more non-Toy creatures."
    );
    supported("Dollmaker's Shop // Porcelain Gallery");
    supported("Hero of Bladehold");
    let toys = |t: &TestGame| {
        tokens_of(t, P0)
            .into_iter()
            .filter(|id| t.obj(*id).chars.subtypes.iter().any(|s| s == "Toy"))
            .count()
    };
    // Hero of Bladehold's Soldiers enter attacking: only Hero's declaration triggers.
    let mut t = TestGame::new(2);
    let shop = t.battlefield(P0, "Dollmaker's Shop // Porcelain Gallery");
    mtg_engine::rooms::unlock(&mut t.g, shop, 0, P0);
    let hero = t.battlefield(P0, "Hero of Bladehold");
    t.attack(&[(hero, Entity::Player(P1))], &[]);
    assert_eq!(tokens_of(&t, P0).len(), 3);
    assert_eq!(toys(&t), 1);
    // Attacking two players: one trigger each; two creatures at one player: one.
    let mut t = TestGame::new(3);
    let shop = t.battlefield(P0, "Dollmaker's Shop // Porcelain Gallery");
    mtg_engine::rooms::unlock(&mut t.g, shop, 0, P0);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.attack(
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P2)),
        ],
        &[],
    );
    assert_eq!(toys(&t), 2);
}

#[test]
fn urza_prince_of_kroog_copies_only_copiable_values() {
    cr!("707.2", "707.9b", "707.3");
    ruling!(
        "Urza, Prince of Kroog",
        "For Urza, Prince of Kroog's activated ability, the token copies exactly what was printed on the original permanent, with the listed exceptions, and nothing else (unless that permanent is copying something else or is a token; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    supported("Urza, Prince of Kroog");
    supported("Ornithopter");
    let mut t = TestGame::new(2);
    let urza = t.battlefield(P0, "Urza, Prince of Kroog");
    let thopter = t.battlefield(P0, "Ornithopter");
    plus_counters(&mut t, thopter, 2);
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(thopter)]);
    cast_resolve(&mut t, P0, "Relic's Roar", &[Entity::Object(thopter)]);
    t.g.tap(thopter);
    // Ornithopter: 4/3 (Relic's Roar) +2 counters +3/+3 +2/+2 (Urza) = 11/10.
    assert_eq!(t.pt(thopter), (11, 10));
    t.lands(P0, "Wastes", 6);
    activate_resolve(&mut t, P0, urza, 0, &[Entity::Object(thopter)]);
    let token = tokens_of(&t, P0)[0];
    let o = t.obj(token);
    assert_eq!(o.chars.name.as_str(), "Ornithopter");
    assert!(!o.tapped);
    assert_eq!(t.counters(token, counters::PLUS1), 0);
    assert!(o.chars.subtypes.iter().any(|s| s == "Soldier"));
    assert!(!o.chars.subtypes.iter().any(|s| s == "Dinosaur"));
    assert!(o.has_keyword(KeywordKind::Flying));
    // A 1/1 Soldier, plus Urza's +2/+2.
    assert_eq!(t.pt(token), (3, 3));
}

#[test]
fn urza_copying_a_token_or_a_copy() {
    cr!("707.2", "707.3", "111.4");
    ruling!(
        "Urza, Prince of Kroog",
        "If the copied permanent is a token, the token created with Urza copies the original characteristics of that token as stated by the effect that created that token, with the listed exceptions."
    );
    ruling!(
        "Urza, Prince of Kroog",
        "If the copied permanent is copying something else, then the token enters the battlefield as whatever that permanent copied, with the listed exceptions."
    );
    supported("Servo Exhibition");
    supported("Sculpting Steel");
    // A Servo token modified by Relic's Roar: the copy is a plain Servo, 1/1 Soldier.
    let mut t = TestGame::new(2);
    let urza = t.battlefield(P0, "Urza, Prince of Kroog");
    cast_resolve(&mut t, P0, "Servo Exhibition", &[]);
    let servo = tokens_of(&t, P0)[0];
    cast_resolve(&mut t, P0, "Relic's Roar", &[Entity::Object(servo)]);
    t.lands(P0, "Wastes", 6);
    activate_resolve(&mut t, P0, urza, 0, &[Entity::Object(servo)]);
    let copy = *tokens_of(&t, P0).last().unwrap();
    assert_ne!(copy, servo);
    let o = t.obj(copy);
    assert_eq!(o.chars.name.as_str(), "Servo Token");
    assert!(o.chars.subtypes.iter().any(|s| s == "Soldier"));
    assert!(!o.chars.subtypes.iter().any(|s| s == "Dinosaur"));
    assert_eq!(t.pt(copy), (3, 3));
    // Sculpting Steel copying Ornithopter: Urza's copy of it is an Ornithopter.
    let mut t = TestGame::new(2);
    let urza = t.battlefield(P0, "Urza, Prince of Kroog");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Wastes", 3);
    yes(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    cast_new(&mut t, P0, "Sculpting Steel", &[]);
    t.resolve_all();
    let steel =
        t.g.permanents()
            .filter(|o| o.controller == P0 && !o.is_token())
            .filter(|o| o.card.as_ref().is_some_and(|c| c.name == "Sculpting Steel"))
            .map(|o| o.id)
            .next()
            .expect("Sculpting Steel on the battlefield");
    assert_eq!(t.obj(steel).chars.name.as_str(), "Ornithopter");
    t.lands(P0, "Wastes", 6);
    activate_resolve(&mut t, P0, urza, 0, &[Entity::Object(steel)]);
    let copy = tokens_of(&t, P0)[0];
    assert_eq!(t.obj(copy).chars.name.as_str(), "Ornithopter");
    assert!(t.obj(copy).has_keyword(KeywordKind::Flying));
}

#[test]
fn humble_and_abilities_that_already_triggered_or_are_gained_later() {
    cr!("113.7a", "613.1f", "613.7b");
    ruling!(
        "Humble",
        "Humble doesn’t counter abilities that have already triggered or been activated."
    );
    ruling!(
        "Humble",
        "If the affected creature gains an ability after Humble resolves, it will keep that ability."
    );
    ruling!(
        "Trickster's Elk",
        "If the enchanted creature gains an ability after Trickster's Elk becomes attached to it, it will keep that ability."
    );
    supported("Elvish Visionary");
    supported("Humble");
    supported("Trickster's Elk");
    let mut t = TestGame::new(2);
    let visionary = t.enter(P0, "Elvish Visionary");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Humble", &[Entity::Object(visionary)]);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.pt(visionary), (0, 1));
    cast_resolve(&mut t, P0, "Jump", &[Entity::Object(visionary)]);
    assert!(t.obj(visionary).has_keyword(KeywordKind::Flying));

    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let elk = t.hand(P0, "Trickster's Elk");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, elk)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    cast_resolve(&mut t, P0, "Jump", &[Entity::Object(bears)]);
    assert!(t.obj(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn eidolon_of_astral_winds_triggers_for_each_enchantment_entering_with_it() {
    cr!("603.6a", "603.2c");
    ruling!(
        "Eidolon of Astral Winds",
        "If Eidolon of Astral Winds enters at the same time as one or more other enchantments you control, its last ability will trigger for each of those enchantments, including itself."
    );
    supported("Eidolon of Astral Winds");
    supported("Replenish");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Eidolon of Astral Winds");
    t.graveyard(P0, "Glorious Anthem");
    t.graveyard(P0, "Leyline of Anticipation");
    cast_new(&mut t, P0, "Replenish", &[]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 3);
}

#[test]
fn suit_up_on_a_vehicle_isnt_crewing_it() {
    cr!("702.122a", "702.122c", "613.4b");
    ruling!(
        "Suit Up",
        "If Suit Up causes a Vehicle to become an artifact creature, it doesn't count as “crewing” that Vehicle for any ability that would trigger off of a Vehicle becoming crewed."
    );
    supported("Suit Up");
    supported("Mobilizer Mech");
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mobilizer Mech");
    t.battlefield(P0, "Smuggler's Copter");
    cast_new(&mut t, P0, "Suit Up", &[Entity::Object(mech)]);
    t.resolve();
    t.settle();
    assert!(t.stack.is_empty());
    assert!(creature(&t, mech));
    assert_eq!(t.pt(mech), (4, 5));
}

#[test]
fn gigantomancer_overrides_a_levelers_level_power_and_toughness() {
    cr!("613.4b", "613.7a", "711.2a");
    ruling!(
        "Gigantomancer",
        "If a leveler is affected by Gigantomancer’s ability, then enough level counters are put on it to reach a level indicated by a new level symbol, it will still be 7/7, not the power and toughness indicated by that level symbol."
    );
    supported("Gigantomancer");
    supported("Student of Warfare");
    let mut t = TestGame::new(2);
    let mancer = t.battlefield(P0, "Gigantomancer");
    let student = t.battlefield(P0, "Student of Warfare");
    t.lands(P0, "Wastes", 1);
    activate_resolve(&mut t, P0, mancer, 0, &[Entity::Object(student)]);
    assert_eq!(t.pt(student), (7, 7));
    let s = t.g.current(student);
    t.g.add_counters(Entity::Object(s), counters::LEVEL, 7, None);
    t.g.recompute();
    assert_eq!(t.pt(student), (7, 7));
    assert!(t.obj(student).has_keyword(KeywordKind::DoubleStrike));
}

#[test]
fn a_permanent_that_becomes_a_creature_with_set_pt_keeps_it_but_crewed_vehicles_dont() {
    cr!("613.4b", "613.7a", "613.7b", "702.122a");
    ruling!(
        "Kudo, King Among Bears",
        "If an effect causes a noncreature permanent to become a creature and sets its power and toughness as it does so, that creature will have that power and toughness; it won't be 2/2. Notably, crewing a Vehicle does not set its power and toughness, so a Vehicle will be a 2/2 creature once crewed."
    );
    ruling!(
        "Harmonious Archon",
        "If an effect causes a noncreature permanent to become a creature and sets its power and toughness as it does so, that creature will have that power and toughness; it won't be 3/3. Notably, crewing a Vehicle does not set its power and toughness, so a Vehicle will be a 3/3 creature once crewed."
    );
    supported("Vengeant Earth");
    supported("Smuggler's Copter");
    for (lord, pt) in [("Kudo, King Among Bears", 2), ("Harmonious Archon", 3)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, lord);
        let forest = t.battlefield(P0, "Forest");
        cast_resolve(&mut t, P0, "Vengeant Earth", &[Entity::Object(forest)]);
        assert!(creature(&t, forest), "{lord}");
        assert_eq!(t.pt(forest), (4, 4), "{lord}");
        let copter = t.battlefield(P0, "Smuggler's Copter");
        let crew = t.battlefield(P0, "Llanowar Elves");
        t.answer_choose(P0, &[Entity::Object(crew)]);
        activate_resolve(&mut t, P0, copter, 0, &[]);
        assert!(creature(&t, copter), "{lord}");
        assert_eq!(t.pt(copter), (pt, pt), "{lord}");
    }
}

#[test]
fn astral_dragon_copies() {
    cr!("707.2", "707.9b", "302.6");
    ruling!(
        "Astral Dragon",
        "If the copied permanent is a token, the new token that's created copies the original characteristics of that token, with the exceptions noted above."
    );
    ruling!(
        "Astral Dragon",
        "Since the tokens are creatures, they can't attack or {T} until your next turn if they don't have haste."
    );
    supported("Astral Dragon");
    supported("Strike It Rich");
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Strike It Rich", &[]);
    let treasure = tokens_of(&t, P0)[0];
    t.answer_targets(P0, &[Entity::Object(treasure)]);
    t.enter(P0, "Astral Dragon");
    t.resolve_all();
    let dragons: Vec<_> = tokens_of(&t, P0)
        .into_iter()
        .filter(|id| *id != treasure)
        .collect();
    assert_eq!(dragons.len(), 2);
    for d in &dragons {
        let o = t.obj(*d);
        assert_eq!(o.chars.name.as_str(), "Treasure Token");
        assert!(o.chars.subtypes.iter().any(|s| s == "Dragon"));
        assert!(o.has_keyword(KeywordKind::Flying));
        assert_eq!(t.pt(*d), (3, 3));
    }
    // The copies' "{T}, Sacrifice this token: Add one mana" can't be activated yet.
    assert!(activate_containing(&mut t, P0, dragons[0], "Sacrifice").is_err());
    assert!(t.on_battlefield(dragons[0]));
    // On the controller's next turn they can.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(activate_containing(&mut t, P0, dragons[0], "Sacrifice").is_ok());
}

#[test]
fn astral_dragon_targeting_an_aura_creates_no_tokens() {
    cr!("303.4d", "303.4g");
    ruling!(
        "Astral Dragon",
        "If you target an Aura with Astral Dragon's ability, no tokens will be created."
    );
    supported("Pacifism");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Pacifism", &[Entity::Object(bears)]);
    let pacifism = t.named_on_battlefield("Pacifism")[0];
    t.answer_targets(P0, &[Entity::Object(pacifism)]);
    t.enter(P0, "Astral Dragon");
    t.resolve_all();
    assert!(tokens_of(&t, P0).is_empty());
}

#[test]
fn gigantiform_with_an_illegal_target_doesnt_resolve() {
    cr!("608.2b", "702.33d");
    ruling!(
        "Gigantiform",
        "If the creature targeted by the Gigantiform spell is an illegal target by the time Gigantiform resolves, the spell doesn't resolve. It won't enter, so you won't get to search your library for another Gigantiform."
    );
    supported("Gigantiform");
    supported("Unsummon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.library_top(P0, "Gigantiform");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 7);
    let g = t.hand(P0, "Gigantiform");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, g).kicked(true).go();
    cast_new(&mut t, P0, "Unsummon", &[Entity::Object(bears)]);
    yes(&mut t, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Gigantiform"));
    assert!(t.named_on_battlefield("Gigantiform").is_empty());
    assert_eq!(t.zone(other), Zone::Library(P0));
}

#[test]
fn vincents_limit_break_on_a_token() {
    cr!("111.7", "603.6c", "700.4");
    ruling!(
        "Vincent's Limit Break",
        "If the target creature is a token, the ability still triggers when it dies, but you won't return the token to the battlefield."
    );
    supported("Vincent's Limit Break");
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Servo Exhibition", &[]);
    let servo = tokens_of(&t, P0)[0];
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    cast_resolve(
        &mut t,
        P0,
        "Vincent's Limit Break",
        &[Entity::Object(servo)],
    );
    assert_eq!(t.pt(servo), (3, 2));
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Object(servo)]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1, "the dies trigger");
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 1);
}

#[test]
fn the_antiquities_war_and_equipment_that_becomes_a_creature() {
    cr!("301.5c", "704.5n", "714.2b");
    ruling!(
        "The Antiquities War",
        "An Equipment that becomes an artifact creature becomes unattached if it’s attached to a creature. Its equip ability can be activated, but it won’t become attached to the target creature."
    );
    supported("Bonesplitter");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bone = t.battlefield(P0, "Bonesplitter");
    t.g.attach(bone, Entity::Object(bears));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    let saga = t.battlefield(P0, "The Antiquities War");
    t.g.add_counters(Entity::Object(saga), counters::LORE, 3, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(creature(&t, bone));
    assert_eq!(t.obj(bone).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
    t.lands(P0, "Wastes", 1);
    activate_resolve(&mut t, P0, bone, 0, &[Entity::Object(bears)]);
    assert_eq!(t.obj(bone).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn the_antiquities_war_as_an_artifact_becomes_a_creature_then_is_sacrificed() {
    cr!("714.4", "613.1d");
    ruling!(
        "The Antiquities War",
        "If The Antiquities War somehow becomes an artifact enchantment prior to resolving its final chapter ability, it will become a 5/5 Saga artifact enchantment creature, and will then be sacrificed after that ability resolves."
    );
    supported("Liquimetal Torque");
    let mut t = TestGame::new(2);
    let torque = t.battlefield(P0, "Liquimetal Torque");
    let saga = t.battlefield(P0, "The Antiquities War");
    t.g.add_counters(Entity::Object(saga), counters::LORE, 2, None);
    t.g.flush_events();
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(saga)]);
    activate_containing(&mut t, P0, torque, "becomes an artifact").unwrap();
    t.resolve_all();
    assert!(t.obj(saga).is(CardType::Artifact));
    t.g.add_counters(Entity::Object(saga), counters::LORE, 1, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.resolve_top();
    t.g.recompute();
    assert!(creature(&t, saga));
    assert_eq!(t.pt(saga), (5, 5));
    t.settle();
    assert!(t.in_graveyard(P0, "The Antiquities War"));
}
