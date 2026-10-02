//! Conditions about the game state and "unless [condition]": players compared with
//! players ("an opponent has more life than you", "a player has more life than each other
//! player", "each player has 10 or less life"), "there is no monarch", your hand, library
//! and life, attacks, groups ("a permanent of each color", "total toughness 10 or
//! greater"), the source's zone ("if ~ is in the command zone"), and effects or static
//! abilities that apply unless a condition holds ("sacrifice it unless {U} was spent to
//! cast it", "~ has hexproof unless it's attacking").

use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
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

// --- "unless [condition]" ----------------------------------------------------------------

#[test]
fn furnace_punisher_damages_players_without_two_basic_lands() {
    cr!("608.2h");
    assert_supported(&["Furnace Punisher"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Furnace Punisher");
    t.lands(P0, "Mountain", 2);
    t.lands(P1, "Forest", 1);
    t.battlefield(P1, "Wasteland");
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    // One basic land (the Wasteland isn't basic): 2 damage.
    assert_eq!(t.life(P1), 18);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn azorius_herald_is_sacrificed_unless_blue_was_spent() {
    cr!("601.2h", "608.2h");
    ruling!(
        "Azorius Herald",
        "If this enters in a way other than announcing it as a spell, then the appropriate mana can't have been paid, and you'll have to sacrifice it."
    );
    assert_supported(&["Azorius Herald", "Court Hussar", "Squealing Devil"]);
    // Cast with {U} among the mana spent: it stays.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 1);
    let herald = t.hand(P0, "Azorius Herald");
    let herald = t.cast(P0, herald).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Azorius Herald").len(), 1, "{herald:?}");
    // Cast with white mana only: sacrificed.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let herald = t.hand(P0, "Azorius Herald");
    t.cast(P0, herald).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Azorius Herald").is_empty());
    assert!(t.in_graveyard(P0, "Azorius Herald"));
    // Put onto the battlefield without being cast: sacrificed.
    let mut t = TestGame::new(2);
    let herald = t.enter(P0, "Azorius Herald");
    t.resolve_all();
    assert!(!t.on_battlefield(herald));
}

#[test]
fn war_elemental_stays_only_if_an_opponent_was_dealt_damage() {
    cr!("608.2h");
    assert_supported(&["War Elemental"]);
    let mut t = TestGame::new(2);
    let w = t.enter(P0, "War Elemental");
    t.resolve_all();
    assert!(!t.on_battlefield(w));
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    let w = t.enter(P0, "War Elemental");
    t.resolve_all();
    assert!(t.on_battlefield(w));
}

#[test]
fn tadeas_has_hexproof_unless_attacking() {
    cr!("611.3a", "702.11b");
    // (Its other ability isn't supported yet; the hexproof line is.)
    let mut t = TestGame::new(2);
    let tadeas = t.battlefield(P0, "Tadeas, Juniper Ascendant");
    t.g.recompute();
    assert!(t.obj_now(tadeas).has_keyword(KeywordKind::Hexproof));
    t.g.objects[tadeas.0 as usize].summoning_sick = false;
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(tadeas, Entity::Player(P1))]),
    );
    t.advance_to_step(Step::DeclareBlockers);
    t.g.recompute();
    assert!(!t.obj_now(tadeas).has_keyword(KeywordKind::Hexproof));
}

#[test]
fn arvinox_isnt_a_creature_unless_you_control_three_permanents_you_dont_own() {
    cr!("611.3a", "108.3");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Arvinox, the Mind Flail");
    t.g.recompute();
    assert!(!t.obj_now(a).is_creature());
    for _ in 0..3 {
        let land = t.battlefield(P1, "Forest");
        t.g.objects[land.0 as usize].base_controller = P0;
        t.g.objects[land.0 as usize].controller = P0;
    }
    t.g.recompute();
    assert!(t.obj_now(a).is_creature());
}

#[test]
fn fall_from_favor_untaps_only_for_the_monarch() {
    cr!("502.3", "725.1");
    assert_supported(&["Fall from Favor"]);
    let mut t = TestGame::new(2);
    let aura = t.battlefield(P0, "Fall from Favor");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.attach(aura, Entity::Object(bears));
    t.g.objects[bears.0 as usize].tapped = true;
    t.g.monarch = Some(P0);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped, "not the monarch: it doesn't untap");
    t.g.monarch = Some(P1);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped, "the monarch's creature untaps");
}

// --- Attack and block taxes ---------------------------------------------------------------

#[test]
fn brainwash_attack_costs_three() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Brainwash",
        "If there are multiple combat phases during the turn, the attack cost must be paid each time if you want to attack with the creature."
    );
    assert_supported(&["Brainwash", "Oppressive Rays", "Qal Sisma Behemoth"]);
    let mut t = TestGame::new(2);
    let aura = t.battlefield(P1, "Brainwash");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(aura, Entity::Object(bears));
    let cost = mtg_engine::combat::required_attack_cost(&t.g, bears, Entity::Player(P1));
    assert_eq!(cost.and_then(|c| c.mana).map(|m| m.mana_value()), Some(3));
    // Another creature attacks for free.
    let ogre = t.battlefield(P0, "Gray Ogre");
    assert!(mtg_engine::combat::required_attack_cost(&t.g, ogre, Entity::Player(P1)).is_none());
}

#[test]
fn brainwash_taxes_an_attack_on_a_battle_too() {
    cr!("508.1d", "310.5");
    // "can't attack unless ...": whatever it would attack, a battle included.
    let mut t = TestGame::new(2);
    t.answer_choose(P1, &[Entity::Player(P0)]);
    let battle = t.enter(P1, "Invasion of Segovia");
    t.resolve_all();
    let aura = t.battlefield(P1, "Brainwash");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(aura, Entity::Object(bears));
    let cost = mtg_engine::combat::required_attack_cost(&t.g, bears, Entity::Object(battle));
    assert_eq!(cost.and_then(|c| c.mana).map(|m| m.mana_value()), Some(3));
    // "can't attack you" (Elephant Grass) doesn't cover the battle.
    let mut t = TestGame::new(2);
    t.answer_choose(P1, &[Entity::Player(P0)]);
    let battle = t.enter(P1, "Invasion of Segovia");
    t.resolve_all();
    t.battlefield(P1, "Elephant Grass");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(
        mtg_engine::combat::required_attack_cost(&t.g, bears, Entity::Object(battle)).is_none()
    );
    assert!(
        mtg_engine::combat::required_attack_cost(&t.g, bears, Entity::Player(P1)).is_some()
    );
}

#[test]
fn myr_prototype_pays_for_each_counter_on_it() {
    cr!("508.1d", "509.1c", "509.1d");
    assert_supported(&["Myr Prototype", "Phyrexian Marauder"]);
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P0, "Myr Prototype");
    t.g.add_counters(Entity::Object(myr), "+1/+1", 2, None);
    let cost = mtg_engine::combat::required_attack_cost(&t.g, myr, Entity::Player(P1));
    assert_eq!(cost.and_then(|c| c.mana).map(|m| m.mana_value()), Some(2));
    let block = mtg_engine::combat::required_block_cost(&t.g, myr);
    assert_eq!(block.and_then(|c| c.mana).map(|m| m.mana_value()), Some(2));
    // Without counters, nothing to pay.
    let other = t.battlefield(P0, "Myr Prototype");
    assert!(mtg_engine::combat::required_attack_cost(&t.g, other, Entity::Player(P1)).is_none());
}

#[test]
fn sphere_of_safety_counts_its_controllers_enchantments() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Sphere of Safety",
        "If you control Sphere of Safety, your opponents can choose not to attack with a creature with an ability that says it attacks if able."
    );
    assert_supported(&["Sphere of Safety", "Collective Restraint", "Elephant Grass"]);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Safety");
    t.battlefield(P1, "Pacifism");
    // The attacker controls enchantments too: they don't count.
    t.battlefield(P0, "Pacifism");
    t.battlefield(P0, "Pacifism");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cost = mtg_engine::combat::required_attack_cost(&t.g, bears, Entity::Player(P1));
    assert_eq!(cost.and_then(|c| c.mana).map(|m| m.mana_value()), Some(2));
    // Attacking someone else is free.
    let mut t3 = TestGame::new(3);
    t3.battlefield(P1, "Sphere of Safety");
    let bears = t3.battlefield(P0, "Grizzly Bears");
    assert!(mtg_engine::combat::required_attack_cost(&t3.g, bears, Entity::Player(P2)).is_none());
}

#[test]
fn reclamation_black_attackers_cost_a_land_each() {
    cr!("508.1d", "508.1h");
    assert_supported(&["Reclamation", "Flooded Woodlands"]);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Reclamation");
    let rats = t.battlefield(P0, "Typhoid Rats");
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(mtg_engine::combat::required_attack_cost(&t.g, bears, Entity::Player(P1)).is_none());
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(rats, Entity::Player(P1))]),
    );
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.advance_to_step(Step::DeclareBlockers);
    assert!(!t.on_battlefield(forest), "the land was sacrificed to attack");
    assert!(t.g.combat.as_ref().is_some_and(|c| c.attacker(rats).is_some()));
}

// --- Players compared ------------------------------------------------------------------------

#[test]
fn sunset_revelry_compares_each_opponent_with_you() {
    cr!("608.2h");
    assert_supported(&["Sunset Revelry", "Timely Reinforcements"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    t.g.player_mut(P1).life = 25;
    t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    let revelry = t.hand(P0, "Sunset Revelry");
    t.cast(P0, revelry).go();
    t.resolve_all();
    // More life: gain 4. More creatures: two Humans. More cards in hand? P0 has two, P1
    // one: no card.
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.named_on_battlefield("Human Token").len(), 2);
    assert_eq!(t.hand_size(P0), 2);
    // With an empty hand against P1's one card, P0 draws.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    t.hand(P1, "Grizzly Bears");
    let revelry = t.hand(P0, "Sunset Revelry");
    t.cast(P0, revelry).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.life(P0), 20, "equal life: no life gained");
    assert!(t.named_on_battlefield("Human Token").is_empty());
}

#[test]
fn timely_reinforcements_both_bonuses_from_different_opponents() {
    cr!("608.2h");
    ruling!(
        "Timely Reinforcements",
        "If you have less life than one of your opponents and fewer creatures than one of your opponents (although this may be a different opponent), you’ll get both bonuses."
    );
    let mut t = TestGame::new(3);
    t.lands(P0, "Plains", 3);
    t.g.player_mut(P1).life = 25;
    t.battlefield(P2, "Grizzly Bears");
    let tr = t.hand(P0, "Timely Reinforcements");
    t.cast(P0, tr).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 3);
}

#[test]
fn wild_dogs_go_to_the_player_with_the_most_life() {
    cr!("603.4");
    ruling!(
        "Wild Dogs",
        "If no one player has more life than all other players at that time (i.e., if two or more players are tied for the most life), the ability won’t trigger at all."
    );
    assert_supported(&["Wild Dogs", "Sokenzan Renegade", "Wild Mammoth", "Thoughtbound Primoc"]);
    // Tied: nothing happens.
    let mut t = TestGame::new(2);
    let dogs = t.battlefield(P0, "Wild Dogs");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(dogs).controller, P0);
    // P1 has the most life: P1 gains control.
    t.g.player_mut(P1).life = 21;
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(dogs).controller, P1);
}

#[test]
fn sokenzan_renegade_goes_to_the_player_with_the_most_cards() {
    cr!("603.4");
    ruling!(
        "Sokenzan Renegade",
        "If multiple players are tied for the most cards in hand, either when Sokenzan Renegade’s ability triggers or as it resolves, the ability does nothing."
    );
    let mut t = TestGame::new(3);
    let r = t.battlefield(P0, "Sokenzan Renegade");
    t.hand(P2, "Grizzly Bears");
    t.hand(P2, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.set_step(P2, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(r).controller, P2);
}

#[test]
fn cryptolith_fragment_transforms_when_each_player_has_ten_or_less() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Cryptolith Fragment");
    t.g.player_mut(P0).life = 10;
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(f).chars.name, "Cryptolith Fragment", "P1 has 20");
    t.g.player_mut(P1).life = 9;
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(f).chars.name, "Aurora of Emrakul");
}

#[test]
fn kazuul_triggers_only_when_you_are_the_defending_player() {
    cr!("603.4", "508.5");
    ruling!(
        "Kazuul, Tyrant of the Cliffs",
        "In a multiplayer game, the ability checks whether you're the defending player for each individual attacking creature."
    );
    assert_supported(&["Kazuul, Tyrant of the Cliffs"]);
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Kazuul, Tyrant of the Cliffs");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Gray Ogre");
    t.answer_yes(P0, false);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P2))], &[]);
    // Only the creature attacking P1 triggered: one Ogre token for P1.
    let ogres: Vec<_> = t
        .named_on_battlefield("Ogre Token")
        .into_iter()
        .filter(|o| t.obj_now(*o).controller == P1)
        .collect();
    assert_eq!(ogres.len(), 1);
}

// --- Monarch, hand, library, life ----------------------------------------------------------

#[test]
fn spear_of_bashenga_makes_you_the_monarch_only_if_there_is_none() {
    cr!("603.4", "725.1");
    let mut t = TestGame::new(2);
    t.enter(P0, "The Spear of Bashenga");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    let mut t = TestGame::new(2);
    t.g.monarch = Some(P1);
    t.enter(P0, "The Spear of Bashenga");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P1));
}

#[test]
fn battle_of_wits_and_triskaidekaphile_count_cards() {
    cr!("603.4", "104.2b");
    ruling!(
        "Battle of Wits",
        "if you don’t have 200 or more cards in your library at the beginning of your upkeep, the ability won’t trigger"
    );
    assert_supported(&["Battle of Wits", "Triskaidekaphile", "Imaginary Pet"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Battle of Wits");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.has_lost(P1));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Triskaidekaphile");
    for _ in 0..13 {
        t.hand(P0, "Grizzly Bears");
    }
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn imaginary_pet_returns_only_with_a_card_in_hand() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    let pet = t.battlefield(P0, "Imaginary Pet");
    t.set_step(P1, Step::End);
    // The draw happens after the upkeep: an empty hand at the upkeep.
    t.g.player_mut(P0).hand.clear();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(pet));
    t.hand(P0, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(pet));
    assert!(t.in_hand(P0, "Imaginary Pet"));
}

// --- Groups and attacks --------------------------------------------------------------------

#[test]
fn spirit_of_resistance_needs_each_color() {
    cr!("611.3a", "105.2");
    ruling!(
        "Spirit of Resistance",
        "A permanent which is of multiple colors counts as each of its colors."
    );
    assert_supported(&["Spirit of Resistance", "Coalition Victory"]);
    let mut t = TestGame::new(2);
    // White, green, and blue-black (one multicolored permanent): no red yet.
    t.battlefield(P0, "Spirit of Resistance");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Dimir Guildmage");
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "no red permanent: not prevented");
    // A red permanent: all five colors, the damage is prevented.
    t.battlefield(P0, "Gray Ogre");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "all five colors: prevented");
}

#[test]
fn swat_away_costs_less_while_a_creature_attacks_you() {
    cr!("601.2f");
    assert_supported(&["Swat Away", "Nemesis Trap"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let swat = t.hand(P0, "Swat Away");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].summoning_sick = false;
    // Not attacked: {2}{U}{U} with two lands can't be cast.
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.cast(P0, swat).target(bears).try_go().is_err());
    // P1 attacks P0 with the Bears: {U}{U} is enough.
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.advance_to_step(Step::DeclareAttackers);
    assert!(t.cast(P0, swat).target(bears).try_go().is_ok());
}

#[test]
fn bull_rush_bruiser_needs_another_warrior_on_your_team() {
    cr!("102.4", "603.4");
    ruling!(
        "Bull-Rush Bruiser",
        "Whether your team controls another Warrior is checked as Bull-Rush Bruiser’s triggered ability triggers and again as it resolves."
    );
    assert_supported(&["Bull-Rush Bruiser", "Sickle Dancer", "Aurora Champion"]);
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Bull-Rush Bruiser");
    t.attack(&[(b, Entity::Player(P1))], &[]);
    assert!(!t.obj_now(b).has_keyword(KeywordKind::FirstStrike));
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Bull-Rush Bruiser");
    t.battlefield(P0, "Bull-Rush Bruiser");
    t.attack(&[(b, Entity::Player(P1))], &[]);
    assert!(t.obj_now(b).has_keyword(KeywordKind::FirstStrike));
}

// --- The source's zone -------------------------------------------------------------------------

#[test]
fn oloro_gains_life_from_the_command_zone() {
    cr!("113.6b", "603.4");
    ruling!(
        "Oloro, Ageless Ascetic",
        "If Oloro is your commander, its last ability will trigger at the beginning of the upkeep step on your first turn."
    );
    let mut t = TestGame::new(2);
    t.command(P0, "Oloro, Ageless Ascetic");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // On the battlefield only its first ability gains life (2, not 4).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oloro, Ageless Ascetic");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn nether_spirit_returns_only_as_the_only_creature_card() {
    cr!("603.4", "113.6b");
    ruling!(
        "Nether Spirit",
        "If you have two Nether Spirits in your graveyard, they stop each other from returning."
    );
    assert_supported(&["Nether Spirit"]);
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Nether Spirit");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Nether Spirit").len(), 1);
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Nether Spirit");
    t.graveyard(P0, "Nether Spirit");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.named_on_battlefield("Nether Spirit").is_empty());
}
