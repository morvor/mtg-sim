//! Mana grammar (CR 106): spending restrictions keyed to what the mana pays for (kicked
//! spells, equip abilities, cumulative upkeep costs, foretelling, unlocking doors, disturb
//! costs, costs that contain {C}, spells of the chosen type), mana production variants
//! ("two mana of different colors", "any color among ...", "the exiled card's colors",
//! "each player adds ..."), and rules about which mana may pay a cost ("Spend only mana
//! produced by basic lands to cast ~", "Spend only mana of the chosen color to activate
//! this ability", "Mana of any type can be spent to activate ~'s abilities").

use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::mana::{Mana, ManaRestriction, ManaType, SpendPurpose};
use mtg_engine::rooms;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

/// Adds `n` mana of type `ty` with restriction `r` to `p`'s pool.
fn add_restricted(t: &mut TestGame, p: PlayerId, ty: ManaType, n: usize, r: ManaRestriction) {
    for _ in 0..n {
        let mut m = Mana::new(ty);
        m.restriction = Some(r.clone());
        t.g.players[p.idx()].mana_pool.add(m);
    }
}

#[test]
fn mana_grammar_cards_compile() {
    assert_supported(&[
        "Elfhame Druid",
        "Throne of Makindi",
        "Volshe Tideturner",
        "Freya Crescent",
        "Ronin, Shadow Stalker",
        "Quinjet Technician",
        "Adarkar Unicorn",
        "Karfell Harbinger",
        "Niko Defies Destiny",
        "Unblinking Observer",
        "Cultivator Drone",
        "Titans' Nest",
        "Eclipsed Realms",
        "Secluded Courtyard",
        "Maelstrom of the Spirit Dragon",
        "Chandra's Embercat",
        "Fíli and Kíli, Joyous",
        "Séance Board",
        "Interplanar Beacon",
        "Guild Globe",
        "Firemind Vessel",
        "Component Pouch",
        "Open the Omenpaths",
        "Mox Amber",
        "Plaza of Heroes",
        "The Grey Havens",
        "Chrome Mox",
        "Pit of Offerings",
        "Selvala, Eager Trailblazer",
        "Stadium Vendors",
        "Yurlok of Scorch Thrash",
        "Belbe, Corrupted Observer",
        "Imperiosaur",
        "Myr Superion",
        "Throne of Eldraine",
        "Sharkey, Tyrant of the Shire",
        "Klaw, Master of Sound",
        "Blood Sun",
        "Break the Ice",
        "Umbra Stalker",
        "Raggadragga, Goreguts Boss",
        "Ice Cave",
        "Katilda, Dawnhart Prime",
        "Squandered Resources",
        "Smoky Lounge // Misty Salon",
    ]);
}

#[test]
fn kicked_spells_only_mana() {
    cr!("106.6", "702.33d");
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Elfhame Druid");
    t.activate(P0, druid, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G, ManaType::G]);
    // An unkicked creature spell can't be paid for with it.
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    assert_eq!(pool(&t, P0), vec![ManaType::G, ManaType::G]);
    // A kicked spell can: Gnarlid Colony with its kicker {2}{G} ({3}{G}{G} in all).
    t.lands(P0, "Forest", 3);
    let colony = t.hand(P0, "Gnarlid Colony");
    t.cast(P0, colony).kicked(true).go();
    assert!(pool(&t, P0).is_empty());
    t.resolve();
    let colony = t.named_on_battlefield("Gnarlid Colony")[0];
    assert_eq!(t.counters(colony, "+1/+1"), 2);
}

#[test]
fn instant_sorcery_or_kicked_spell() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    let volshe = t.battlefield(P0, "Volshe Tideturner");
    t.activate(P0, volshe, 0, &[]).unwrap();
    // Not for a blue creature spell.
    let drake = t.hand(P0, "Wind Drake");
    t.lands(P0, "Island", 2);
    assert!(t.cast(P0, drake).try_go().is_err());
    // For an instant.
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn equipment_spells_or_equip_abilities() {
    cr!("106.6", "702.6a");
    let mut t = TestGame::new(2);
    let freya = t.battlefield(P0, "Freya Crescent");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    t.activate(P0, freya, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    // Not for another ability's cost (Mind Stone's "{1}, {T}, Sacrifice").
    let stone = t.battlefield(P0, "Mind Stone");
    assert!(t.activate(P0, stone, 1, &[]).is_err());
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    // It pays Bonesplitter's equip {1}.
    t.activate(P0, splitter, 0, &[Entity::Object(bear)]).unwrap();
    assert!(pool(&t, P0).is_empty());
    t.resolve();
    assert_eq!(t.pt(bear), (4, 2));
}

#[test]
fn power_up_abilities_only() {
    cr!("106.6", "702.193a");
    let mut t = TestGame::new(2);
    add_restricted(
        &mut t,
        P0,
        ManaType::R,
        1,
        ManaRestriction::Purpose(SpendPurpose::Keyword(keywords::KeywordKind::PowerUp)),
    );
    // Equip isn't power-up: Bonesplitter's equip {1} can't use it.
    let bear = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert!(t.activate(P0, splitter, 0, &[Entity::Object(bear)]).is_err());
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    assert_supported(&["Quinjet Technician"]);
}

#[test]
fn cumulative_upkeep_only_mana() {
    cr!("106.6", "702.24a");
    let mut t = TestGame::new(2);
    let unicorn = t.battlefield(P0, "Adarkar Unicorn");
    let remora = t.battlefield(P0, "Mystic Remora");
    // Its mana can't cast a spell.
    let opt = t.hand(P0, "Opt");
    assert!(t.cast(P0, opt).try_go().is_err());
    assert!(!t.obj_now(unicorn).tapped);
    // It pays the cumulative upkeep {1}.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(remora));
    assert!(t.obj_now(unicorn).tapped);
}

#[test]
fn foretell_only_mana_pays_the_foretell_action() {
    cr!("106.6", "702.143a");
    ruling!(
        "Karfell Harbinger",
        "The mana can be spent to exile a card with foretell from your hand face down"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let harbinger = t.battlefield(P0, "Karfell Harbinger");
    let harbinger2 = t.battlefield(P0, "Karfell Harbinger");
    t.activate(P0, harbinger, 0, &[]).unwrap();
    t.activate(P0, harbinger2, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::U, ManaType::U]);
    // Not for a creature spell.
    let drake = t.hand(P0, "Wind Drake");
    t.battlefield(P0, "Island");
    assert!(t.cast(P0, drake).try_go().is_err());
    // It pays {2} to foretell Doomskar.
    let card = t.hand(P0, "Doomskar");
    t.g.turn.priority = Some(P0);
    let r = t.g.perform_action(P0, Action::Special(SpecialAction::Foretell { card }));
    assert!(r.is_ok(), "{r:?}");
    assert!(pool(&t, P0).is_empty());
    assert!(!t.in_hand(P0, "Doomskar"));
}

#[test]
fn disturb_cost_or_instant_or_sorcery() {
    cr!("106.6", "702.146a");
    let mut t = TestGame::new(2);
    add_restricted(
        &mut t,
        P0,
        ManaType::U,
        2,
        card("Unblinking Observer").faces[0]
            .chars
            .abilities
            .iter()
            .find_map(|a| match &a.kind {
                ability::AbilityKind::Activated(act) => match &act.body.effect {
                    ability::Effect::AddMana { restriction, .. } => restriction.clone(),
                    _ => None,
                },
                _ => None,
            })
            .unwrap(),
    );
    // A creature spell cast normally can't use it.
    let wind = t.hand(P0, "Wind Drake");
    t.battlefield(P0, "Island");
    assert!(t.cast(P0, wind).try_go().is_err());
    // An instant can.
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    assert_eq!(pool(&t, P0), vec![ManaType::U]);
}

#[test]
fn colorless_spell_ability_or_cost_with_c() {
    cr!("106.6");
    ruling!(
        "Cultivator Drone",
        "Mana produced by Cultivator Drone may be used to cast any colorless spell"
    );
    let mut t = TestGame::new(2);
    let drone = t.battlefield(P0, "Cultivator Drone");
    t.activate(P0, drone, 0, &[]).unwrap();
    // Not for a colored spell's generic mana.
    let bears = t.hand(P0, "Grizzly Bears");
    t.battlefield(P0, "Forest");
    assert!(t.cast(P0, bears).try_go().is_err());
    assert_eq!(pool(&t, P0), vec![ManaType::C]);
    // For a colorless artifact spell.
    let ring = t.hand(P0, "Sol Ring");
    t.cast(P0, ring).go();
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn spells_of_the_chosen_type_and_legendary_creature_spells() {
    cr!("106.6", "607.2d");
    let mut t = TestGame::new(2);
    // Secluded Courtyard: Elf chosen.
    let yard = t.battlefield(P0, "Secluded Courtyard");
    t.g.objects[yard.0 as usize].choices.creature_type = Some("Elf".into());
    t.answer(P0, DecisionKind::Option, Answer::Index(4)); // green
    t.activate(P0, yard, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
    // Not for a non-Elf creature.
    let bears = t.hand(P0, "Grizzly Bears");
    t.battlefield(P0, "Forest");
    assert!(t.cast(P0, bears).try_go().is_err());
    // For an Elf.
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn dragon_or_omen_spell() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    let maelstrom = t.battlefield(P0, "Maelstrom of the Spirit Dragon");
    t.answer(P0, DecisionKind::Option, Answer::Index(3)); // red
    t.activate(P0, maelstrom, 1, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    let goblin = t.hand(P0, "Raging Goblin");
    assert!(t.cast(P0, goblin).try_go().is_err());
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
}

#[test]
fn lists_of_spell_types() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    let fili = t.battlefield(P0, "Fíli and Kíli, Joyous");
    t.activate(P0, fili, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::R, ManaType::R]);
    // Not a Dwarf, Equipment or Saga.
    let goblin = t.hand(P0, "Raging Goblin");
    assert!(t.cast(P0, goblin).try_go().is_err());
    // An Equipment spell.
    let splitter = t.hand(P0, "Bonesplitter");
    t.cast(P0, splitter).go();
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
}

#[test]
fn two_mana_of_different_colors() {
    cr!("106.1", "106.5");
    let mut t = TestGame::new(2);
    let vessel = t.battlefield(P0, "Firemind Vessel");
    t.g.objects[vessel.0 as usize].tapped = false;
    // Asked for white first, then for white again: the second color must differ.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, vessel, 0, &[]).unwrap();
    let p = pool(&t, P0);
    assert_eq!(p.len(), 2);
    assert_ne!(p[0], p[1]);
    assert!(p.contains(&ManaType::W));
}

#[test]
fn two_of_one_color_and_two_of_another() {
    cr!("106.1");
    let mut t = TestGame::new(2);
    let omen = t.hand(P0, "Open the Omenpaths");
    t.lands(P0, "Mountain", 3);
    t.answer(P0, DecisionKind::Option, Answer::Index(4)); // green
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // then white
    t.cast(P0, omen).modes(&[0]).go();
    t.resolve();
    assert_eq!(
        pool(&t, P0),
        vec![ManaType::W, ManaType::W, ManaType::G, ManaType::G]
    );
    // Only for creature or enchantment spells.
    let growth = t.hand(P0, "Giant Growth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.cast(P0, growth).target(bears).try_go().is_err());
}

#[test]
fn any_color_among_legendary_permanents() {
    cr!("106.1", "106.5");
    let mut t = TestGame::new(2);
    let mox = t.battlefield(P0, "Mox Amber");
    // No legendary creature or planeswalker: no mana.
    t.activate(P0, mox, 0, &[]).unwrap();
    assert!(pool(&t, P0).is_empty());
    t.g.objects[mox.0 as usize].tapped = false;
    // With Isamaru, Hound of Konda (white): white mana.
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.activate(P0, mox, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::W]);
}

#[test]
fn the_exiled_cards_colors() {
    cr!("106.5", "607.2a");
    ruling!(
        "Chrome Mox",
        "If no card is exiled with this artifact, it can’t add mana"
    );
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let mox = t.hand(P0, "Chrome Mox");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, mox).go();
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    let mox = t.named_on_battlefield("Chrome Mox")[0];
    t.activate(P0, mox, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
}

#[test]
fn choose_a_color_and_add_that_much() {
    cr!("106.1");
    let mut t = TestGame::new(2);
    let selvala = t.battlefield(P0, "Selvala, Eager Trailblazer");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    // Powers 4 (Selvala), 2 and 3: three different powers.
    t.answer(P0, DecisionKind::Option, Answer::Index(1)); // blue
    t.activate(P0, selvala, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::U; 3]);
}

#[test]
fn chosen_player_adds_two_of_one_color() {
    cr!("106.4");
    let mut t = TestGame::new(2);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(2)); // black
    t.enter(P0, "Stadium Vendors");
    t.resolve_all();
    assert_eq!(pool(&t, P1), vec![ManaType::B, ManaType::B]);
}

#[test]
fn each_player_adds_and_mana_burn() {
    cr!("106.4", "500.5");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let yurlok = t.battlefield(P0, "Yurlok of Scorch Thrash");
    t.battlefield(P0, "Wastes");
    t.activate(P0, yurlok, 0, &[]).unwrap();
    let brg = vec![ManaType::B, ManaType::R, ManaType::G];
    assert_eq!(pool(&t, P0), brg);
    assert_eq!(pool(&t, P1), brg);
    // The mana empties as the phase ends: each player loses 3 life.
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn active_player_adds_for_each_opponent_who_lost_life() {
    cr!("106.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Belbe, Corrupted Observer");
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Mountain");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    t.advance_to(P0, Step::PostcombatMain);
    t.settle();
    t.resolve();
    assert_eq!(pool(&t, P0), vec![ManaType::C, ManaType::C]);
}

#[test]
fn only_mana_from_basic_lands_or_creatures() {
    cr!("601.2h");
    ruling!(
        "Myr Superion",
        "Mana produced by abilities of creature cards not on the battlefield"
    );
    let mut t = TestGame::new(2);
    let saurian = t.hand(P0, "Imperiosaur");
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Forest");
    t.battlefield(P0, "Brushland");
    // Three basic lands aren't enough; the nonbasic land's mana can't help.
    assert!(t.cast(P0, saurian).try_go().is_err());
    t.battlefield(P0, "Forest");
    let r = t.cast(P0, saurian).try_go();
    assert!(r.is_ok(), "{r:?}");
    t.resolve();
    // Myr Superion: only mana from creatures.
    let myr = t.hand(P0, "Myr Superion");
    t.lands(P0, "Wastes", 2);
    assert!(t.cast(P0, myr).try_go().is_err());
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Fyndhorn Elves");
    let r = t.cast(P0, myr).try_go();
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn only_mana_of_the_chosen_color_to_activate() {
    cr!("607.2d");
    let mut t = TestGame::new(2);
    let throne = t.battlefield(P0, "Throne of Eldraine");
    t.g.objects[throne.0 as usize].choices.color = Some(types::Color::Green);
    t.lands(P0, "Mountain", 3);
    assert!(t.activate(P0, throne, 1, &[]).is_err());
    t.lands(P0, "Forest", 3);
    let hand = t.hand_size(P0);
    t.activate(P0, throne, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);
    // The four green mana pay only for monocolored green spells.
    let throne2 = t.battlefield(P0, "Throne of Eldraine");
    t.g.objects[throne2.0 as usize].choices.color = Some(types::Color::Green);
    t.activate(P0, throne2, 0, &[]).unwrap();
    let elves = t.hand(P0, "Llanowar Elves");
    t.cast(P0, elves).go();
    assert_eq!(pool(&t, P0), vec![ManaType::G; 3]);
}

#[test]
fn mana_of_any_type_for_its_own_abilities() {
    cr!("118.14");
    let mut t = TestGame::new(2);
    let sharkey = t.battlefield(P0, "Sharkey, Tyrant of the Shire");
    // P1's Mishra's Factory gives Sharkey "{1}: ... becomes an Assembly-Worker" and
    // "{T}: Target Assembly-Worker gets +1/+1": the colorless cost; and a colored one
    // from an Ancient Den? Use Kher Keep's "{1}{R}, {T}: Create a 0/1 red Kobold".
    t.battlefield(P1, "Kher Keep");
    t.g.recompute();
    t.lands(P0, "Island", 2);
    // {1}{R} paid with blue mana.
    let n = t
        .obj_now(sharkey)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, ability::AbilityKind::Activated(_)))
        .count();
    assert!(n >= 1);
    t.activate(P0, sharkey, n - 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Kobolds of Kher Keep").len(), 1);
}

#[test]
fn lands_lose_all_abilities_except_mana_abilities() {
    cr!("613.1f", "605.1a");
    let mut t = TestGame::new(2);
    let factory = t.battlefield(P0, "Mishra's Factory");
    t.battlefield(P1, "Blood Sun");
    t.g.recompute();
    let n = t
        .obj_now(factory)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, ability::AbilityKind::Activated(_)))
        .count();
    assert_eq!(n, 1);
    t.activate(P0, factory, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::C]);
}

#[test]
fn destroy_a_land_that_could_produce_colorless() {
    cr!("106.7");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Forest");
    let ice = t.hand(P0, "Break the Ice");
    t.lands(P0, "Swamp", 2);
    // A Forest isn't a legal target.
    assert!(t.cast(P0, ice).try_go().is_err());
    let wastes = t.battlefield(P1, "Wastes");
    t.cast(P0, ice).target(wastes).go();
    t.resolve();
    assert!(!t.on_battlefield(wastes));
}

#[test]
fn chroma_counts_symbols_of_cards_in_your_graveyard() {
    cr!("207.2c", "107.4e");
    let mut t = TestGame::new(2);
    let stalker = t.battlefield(P0, "Umbra Stalker");
    t.graveyard(P0, "Phyrexian Arena"); // {1}{B}{B}
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P1, "Dark Ritual"); // not yours
    t.g.recompute();
    assert_eq!(t.pt(stalker), (2, 2));
}

#[test]
fn cast_trigger_with_at_least_seven_mana_spent() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raggadragga, Goreguts Boss");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 8);
    let small = t.hand(P0, "Giant Growth");
    t.cast(P0, small).target(bear).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    let big = t.hand(P0, "Craw Wurm"); // {4}{G}{G}: six mana
    t.cast(P0, big).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    // Seven mana: it triggers.
    t.lands(P0, "Forest", 7);
    let tusker = t.hand(P0, "Krosan Tusker"); // {5}{G}{G}
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.cast(P0, tusker).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // Grizzly Bears 2/2 with Giant Growth's +3/+3, and +7/+7.
    assert_eq!(t.pt(bear), (12, 12));
}

#[test]
fn any_type_the_sacrificed_land_could_produce() {
    cr!("106.7");
    ruling!(
        "Squandered Resources",
        "Can sacrifice a land which can’t produce mana"
    );
    let mut t = TestGame::new(2);
    let res = t.battlefield(P0, "Squandered Resources");
    let forest = t.battlefield(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.activate(P0, res, 0, &[]).unwrap();
    assert!(!t.on_battlefield(forest));
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
}

#[test]
fn granted_mana_ability_of_its_own_colors() {
    cr!("106.1");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Katilda, Dawnhart Prime");
    let elder = t.battlefield(P0, "Thraben Inspector"); // white Human Soldier
    t.g.recompute();
    t.activate(P0, elder, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::W]);
}

#[test]
fn room_spells_and_unlocking_doors() {
    cr!("106.6", "709.5e");
    let mut t = TestGame::new(2);
    add_restricted(
        &mut t,
        P0,
        ManaType::U,
        4,
        ManaRestriction::Purpose(SpendPurpose::UnlockDoor),
    );
    // Not for a spell that isn't a Room.
    let drake = t.hand(P0, "Wind Drake");
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.cast(P0, drake).try_go().is_err());
    assert_eq!(t.g.player(P0).mana_pool.total(), 4);
    // It pays Misty Salon's unlock cost {3}{U}.
    let room = t.battlefield(P0, "Smoky Lounge // Misty Salon");
    rooms::unlock(&mut t.g, room, 0, P0);
    t.g.recompute();
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        Action::Special(SpecialAction::Other {
            name: format!("{}1", rooms::UNLOCK_ACTION),
            obj: Some(room),
        }),
    )
    .unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    assert_eq!(rooms::unlocked(&t.g, room), [true, true]);
}

#[test]
fn any_other_player_may_pay_that_spells_mana_cost() {
    cr!("118.12");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ice Cave");
    t.lands(P1, "Mountain", 1);
    t.battlefield(P0, "Mountain");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P1, true);
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn misty_salon_counts_unlocked_doors() {
    cr!("709.5h");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let room = t.battlefield(P0, "Smoky Lounge // Misty Salon");
    rooms::unlock(&mut t.g, room, 0, P0);
    t.g.recompute();
    rooms::unlock(&mut t.g, room, 1, P0);
    t.g.recompute();
    t.settle();
    t.resolve_all();
    let spirits: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).chars.has_subtype("Spirit"))
        .collect();
    assert_eq!(spirits.len(), 1);
    assert_eq!(t.pt(spirits[0]), (2, 2));
}
