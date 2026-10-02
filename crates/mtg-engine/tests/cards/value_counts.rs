//! The value grammar (CR 107): counts and aggregates in instructions — "where X is the
//! number of ...", "equal to the number of ...", "for each ...", "the greatest toughness
//! among ...", "the number of basic land types among lands they control", amounts that
//! depend on each player ("each player loses 1 life for each creature they control"),
//! tokens and look-at-top-X with a defined X, and "draw cards equal to the difference".

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn hand_cards(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.hand(p, "Grizzly Bears");
    }
}

#[test]
fn iron_maiden_cards_in_their_hand_minus_four() {
    cr!("107.3c", "107.1b");
    assert_supported("Iron Maiden");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Iron Maiden");
    hand_cards(&mut t, P1, 6);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    // 6 cards in hand: 2 damage.
    assert_eq!(t.life(P1), 18);
    // With fewer than four cards, X would be negative: no damage (CR 107.1b).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Iron Maiden");
    hand_cards(&mut t, P1, 2);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn frantic_inventory_counts_cards_named_it_in_your_graveyard() {
    cr!("608.2h");
    assert_supported("Frantic Inventory");
    ruling!(
        "Frantic Inventory",
        "Because Frantic Inventory is still on the stack while it's resolving"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.graveyard(P0, "Frantic Inventory");
    t.graveyard(P0, "Frantic Inventory");
    t.graveyard(P1, "Frantic Inventory");
    let fi = t.hand(P0, "Frantic Inventory");
    let before = t.hand_size(P0);
    t.cast(P0, fi).go();
    t.resolve();
    // One card, then two more (the two in your graveyard; not the one on the stack nor
    // the opponent's).
    assert_eq!(t.hand_size(P0), before - 1 + 3);
}

#[test]
fn flame_burst_two_plus_cards_named_it_in_all_graveyards() {
    cr!("107.3c");
    assert_supported("Flame Burst");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.graveyard(P0, "Flame Burst");
    t.graveyard(P1, "Flame Burst");
    let fb = t.hand(P0, "Flame Burst");
    t.cast(P0, fb).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn gluttonous_troll_food_for_each_opponent() {
    cr!("102.3");
    assert_supported("Gluttonous Troll");
    let mut t = TestGame::new(3);
    t.enter(P0, "Gluttonous Troll");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Food Token").len(), 2);
}

#[test]
fn kin_tree_invocation_token_with_the_greatest_toughness() {
    cr!("107.3c", "111.3");
    assert_supported("Kin-Tree Invocation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Swamp", 1);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Wall of Wood");
    let k = t.hand(P0, "Kin-Tree Invocation");
    t.cast(P0, k).go();
    t.resolve();
    let tokens = t.named_on_battlefield("Spirit Warrior Token");
    assert_eq!(tokens.len(), 1);
    assert_eq!(t.pt(tokens[0]), (3, 3));
}

#[test]
fn wurmcalling_x_x_token_uses_the_spells_x() {
    cr!("107.3a");
    assert_supported("Wurmcalling");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let w = t.hand(P0, "Wurmcalling");
    t.cast(P0, w).x(3).go();
    t.resolve();
    let tokens = t.named_on_battlefield("Wurm Token");
    assert_eq!(tokens.len(), 1);
    assert_eq!(t.pt(tokens[0]), (3, 3));
}

#[test]
fn allied_strategies_counts_the_target_players_basic_land_types() {
    cr!("305.6");
    assert_supported("Allied Strategies");
    ruling!(
        "Allied Strategies",
        "You draw one card per basic land type, not for each basic land."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    // The target player's lands count, not yours.
    t.lands(P1, "Forest", 2);
    t.lands(P1, "Swamp", 1);
    let a = t.hand(P0, "Allied Strategies");
    let before = t.hand_size(P1);
    t.cast(P0, a).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), before + 2);
}

#[test]
fn stronghold_discipline_each_player_counts_their_own_creatures() {
    cr!("608.2h");
    assert_supported("Stronghold Discipline");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    t.battlefield(P0, "Grizzly Bears");
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
    }
    let s = t.hand(P0, "Stronghold Discipline");
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn price_of_progress_twice_each_players_nonbasic_lands() {
    cr!("120.3");
    assert_supported("Price of Progress");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.battlefield(P0, "Evolving Wilds");
    t.battlefield(P1, "Evolving Wilds");
    t.battlefield(P1, "Evolving Wilds");
    let p = t.hand(P0, "Price of Progress");
    t.cast(P0, p).go();
    t.resolve();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn generous_plunderer_artifacts_the_defending_player_controls() {
    cr!("508.5");
    assert_supported("Generous Plunderer");
    let mut t = TestGame::new(2);
    let gp = t.battlefield(P0, "Generous Plunderer");
    t.battlefield(P1, "Sol Ring");
    t.battlefield(P1, "Sol Ring");
    t.battlefield(P0, "Sol Ring");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(gp, Entity::Player(P1))], &[]);
    t.resolve_all();
    // The defending player's two artifacts, not the attacker's (and its combat damage).
    let power = t.pt(gp).0;
    assert_eq!(t.life(P1), 20 - 2 - power);
}

#[test]
fn shepherd_of_heroes_party_size() {
    cr!("700.8a");
    assert_supported("Shepherd of Heroes");
    ruling!(
        "Shepherd of Heroes",
        "If that creature is still on the battlefield when the ability resolves, it’ll be counted"
    );
    let mut t = TestGame::new(2);
    // A Wizard; Shepherd itself is the Cleric.
    t.battlefield(P0, "Merfolk Looter");
    t.battlefield(P0, "Prodigal Sorcerer");
    t.enter(P0, "Shepherd of Heroes");
    t.resolve_all();
    // Cleric (itself), Rogue, Wizard: three.
    assert_eq!(t.life(P0), 26);
}

#[test]
fn eldritch_pact_x_from_the_targets_graveyard_once_for_both_instructions() {
    cr!("107.3c", "608.2h");
    assert_supported("Eldritch Pact");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    for _ in 0..3 {
        t.graveyard(P1, "Forest");
    }
    let e = t.hand(P0, "Eldritch Pact");
    let before = t.hand_size(P1);
    t.cast(P0, e).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), before + 3);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn fomori_vault_looks_at_as_many_cards_as_artifacts() {
    cr!("107.3c");
    assert_supported("Fomori Vault");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    let vault = t.battlefield(P0, "Fomori Vault");
    t.battlefield(P0, "Sol Ring");
    t.battlefield(P0, "Ornithopter");
    t.hand(P0, "Grizzly Bears");
    t.activate(P0, vault, 1, &[]).unwrap();
    t.resolve();
    let offered = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.len()),
            _ => None,
        })
        .expect("no choice asked");
    assert_eq!(offered, 2);
}

#[test]
fn aether_burst_number_of_targets_as_you_cast() {
    cr!("601.2c", "107.3c");
    assert_supported("Aether Burst");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.graveyard(P1, "Aether Burst");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let burst = t.hand(P0, "Aether Burst");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.cast(P0, burst).go();
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears") && t.in_hand(P1, "Hill Giant"));
}

#[test]
fn damia_draws_the_difference() {
    cr!("608.2h");
    assert_supported("Damia, Sage of Stone");
    ruling!(
        "Damia, Sage of Stone",
        "If you have fewer than seven cards in your hand, you’ll determine the difference"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Damia, Sage of Stone");
    hand_cards(&mut t, P0, 3);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn might_of_the_nephilim_for_each_of_its_colors() {
    cr!("105.2");
    assert_supported("Might of the Nephilim");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let c = t.battlefield(P0, "Watchwolf");
    let m = t.hand(P0, "Might of the Nephilim");
    t.cast(P0, m).target(Entity::Object(c)).go();
    t.resolve();
    // Watchwolf is green and white: +4/+4.
    assert_eq!(t.pt(c), (7, 7));
}

#[test]
fn phyresis_outbreak_each_creature_by_its_controllers_poison() {
    cr!("608.2h", "122.1");
    assert_supported("Phyresis Outbreak");
    ruling!(
        "Phyresis Outbreak",
        "The amount a creature's power and toughness are reduced is determined only once"
    );
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 3);
    t.g.player_mut(P1).counters.insert("poison".into(), 2);
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P2, "Hill Giant");
    let mine = t.battlefield(P0, "Hill Giant");
    let p = t.hand(P0, "Phyresis Outbreak");
    t.cast(P0, p).go();
    t.resolve();
    assert!(!t.on_battlefield(a), "P1 has 3 poison: -3/-3 kills a 3/3");
    assert_eq!(t.pt(b), (2, 2));
    assert_eq!(t.pt(mine), (3, 3));
}

#[test]
fn masters_councillors_graveyards_with_seven_cards() {
    cr!("404.1");
    assert_supported("Master's Councillors");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Master's Councillors");
    for _ in 0..7 {
        t.graveyard(P1, "Forest");
    }
    for _ in 0..6 {
        t.graveyard(P0, "Forest");
    }
    t.settle();
    assert_eq!(t.pt(m), (3, 3));
}

#[test]
fn mana_echoes_counts_the_entering_creature_itself() {
    cr!("205.3m");
    assert_supported("Mana Echoes");
    ruling!(
        "Mana Echoes",
        "The creature entering the battlefield shares a creature type with itself"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Echoes");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    // Two Bears share a creature type with the entering Bear (including itself).
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

#[test]
fn repay_in_kind_lowest_life_total() {
    cr!("119.5");
    assert_supported("Repay in Kind");
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 7);
    t.g.player_mut(P1).life = 7;
    t.g.player_mut(P2).life = 12;
    let r = t.hand(P0, "Repay in Kind");
    t.cast(P0, r).go();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (7, 7, 7));
}

#[test]
fn lichs_tomb_sacrifices_a_permanent_for_each_life_lost() {
    cr!("119.3");
    assert_supported("Lich's Tomb");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lich's Tomb");
    for _ in 0..3 {
        t.battlefield(P0, "Grizzly Bears");
    }
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Shock");
    t.g.turn.priority = Some(P1);
    t.answer_choose(P0, &[]);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    // Lost 2 life: sacrificed two permanents.
    let left = t.named_on_battlefield("Grizzly Bears").len()
        + t.named_on_battlefield("Lich's Tomb").len();
    assert_eq!(left, 2);
}

#[test]
fn peer_into_the_abyss_rounds_up_each_time() {
    cr!("107.1a");
    assert_supported("Peer into the Abyss");
    ruling!(
        "Peer into the Abyss",
        "The life lost is rounded up, not the remaining life total."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    t.g.player_mut(P1).life = 7;
    let lib = t.library_size(P1); // 30: draws 15
    let p = t.hand(P0, "Peer into the Abyss");
    let before = t.hand_size(P1);
    t.cast(P0, p).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), before + (lib + 1) / 2);
    assert_eq!(t.life(P1), 3);
}

#[test]
fn light_from_within_counts_white_symbols_in_each_creatures_cost() {
    cr!("107.4e");
    assert_supported("Light from Within");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Light from Within");
    let knight = t.battlefield(P0, "White Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.settle();
    // {W}{W}: +2/+2; {1}{G}: nothing.
    assert_eq!(t.pt(knight), (4, 4));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn eidolon_of_countless_battles_two_bonuses() {
    cr!("613.4c");
    assert_supported("Eidolon of Countless Battles");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Eidolon of Countless Battles");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pacifism = t.battlefield(P0, "Pacifism");
    assert!(t.g.attach(pacifism, Entity::Object(bears)));
    t.settle();
    // Two creatures and one Aura: +3/+3 on a 0/0.
    assert_eq!(t.pt(e), (3, 3));
}

#[test]
fn baleful_stare_counts_cards_in_the_revealed_hand() {
    cr!("701.20a");
    assert_supported("Baleful Stare");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.hand(P1, "Mountain");
    t.hand(P1, "Shock");
    t.hand(P1, "Forest");
    let b = t.hand(P0, "Baleful Stare");
    let before = t.hand_size(P0);
    t.cast(P0, b).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), before - 1 + 2);
}

#[test]
fn hydra_broodmaster_x_tokens_of_size_x() {
    cr!("701.37c", "107.3a");
    assert_supported("Hydra Broodmaster");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let h = t.battlefield(P0, "Hydra Broodmaster");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, h, 0, &[]).unwrap();
    t.resolve_all();
    let tokens = t.named_on_battlefield("Hydra Token");
    assert_eq!(tokens.len(), 2);
    assert!(tokens.iter().all(|x| t.pt(*x) == (2, 2)));
}

#[test]
fn netherborn_phalanx_each_opponent_counts_their_own_creatures() {
    cr!("608.2h");
    assert_supported("Netherborn Phalanx");
    let mut t = TestGame::new(3);
    for _ in 0..2 {
        t.battlefield(P1, "Grizzly Bears");
    }
    t.battlefield(P2, "Grizzly Bears");
    t.enter(P0, "Netherborn Phalanx");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 18, 19));
}
