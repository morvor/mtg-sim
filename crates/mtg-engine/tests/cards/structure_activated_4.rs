//! Structure-coverage tests (round 4): activated-ability structures shared by many cards
//! that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks that
//! a real card's ability does what its text says.

use super::structure_activated_2::{act, next_turn, target_choices, untapped};
use super::structure_seed_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color, ColorSet};
use mtg_engine::combat;
use mtg_engine::*;

#[test]
fn goblin_fireslinger_pings_a_player_or_planeswalker() {
    cr!("602.2", "115.1", "120.3a", "120.3c");
    // "{T}: This creature deals 1 damage to target player or planeswalker."
    supported("Goblin Fireslinger");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goblin Fireslinger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    act(&mut t, P0, g, "damage", &[Entity::Player(P1)]).unwrap();
    assert!(t.obj_now(g).tapped);
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Player(P1)) && c.contains(&Entity::Player(P0)));
    assert!(c.contains(&Entity::Object(garruk)));
    assert!(!c.contains(&Entity::Object(bears)), "not a creature");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
    // A planeswalker loses that much loyalty.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goblin Fireslinger");
    let garruk = t.battlefield(P1, "Garruk Wildspeaker");
    act(&mut t, P0, g, "damage", &[Entity::Object(garruk)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(garruk, "loyalty"), 2);
    assert_eq!(t.life(P1), 20);
}

/// Declares `attackers` attacking P1 and advances to the declare attackers step.
fn attack_with(t: &mut TestGame, attackers: &[ObjectId]) {
    t.set_step(P0, Step::BeginningOfCombat);
    let decl: Vec<_> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    declare(t, &decl);
    go_to(t, Step::DeclareAttackers);
}

#[test]
fn goblin_tunneler_makes_a_small_creature_unblockable() {
    cr!("602.2", "115.1", "509.1b", "611.2a");
    // "{T}: Target creature with power 2 or less can't be blocked this turn."
    supported("Goblin Tunneler");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Goblin Tunneler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    act(&mut t, P0, g, "blocked", &[Entity::Object(bears)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bears)) && c.contains(&Entity::Object(g)));
    assert!(!c.contains(&Entity::Object(giant)), "power 3");
    t.resolve_all();
    attack_with(&mut t, &[bears, giant]);
    assert!(!t.g.can_block(wall, bears), "it can't be blocked");
    assert!(t.g.can_block(wall, giant));
    // The next turn it can be blocked again.
    t.advance_to(P0, Step::PrecombatMain);
    attack_with(&mut t, &[bears]);
    assert!(t.g.can_block(wall, bears), "only this turn");
}

#[test]
fn golgari_guildmage_puts_a_counter_on_target_creature() {
    cr!("602.2", "122.1");
    // "{4}{G}: Put a +1/+1 counter on target creature."
    supported("Golgari Guildmage");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Golgari Guildmage");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    assert!(act(&mut t, P0, gm, "counter", &[Entity::Object(bears)]).is_err());
    t.clear_answers();
    t.lands(P0, "Forest", 1);
    act(&mut t, P0, gm, "counter", &[Entity::Object(bears)]).unwrap();
    assert_eq!(untapped(&t, P0, "Forest"), 0);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.counters(gm, "+1/+1"), 0);
}

#[test]
fn gravel_slinger_pings_an_attacking_creature() {
    cr!("602.2", "115.1", "120.3e");
    // "{T}: This creature deals 1 damage to target attacking or blocking creature."
    supported("Gravel Slinger");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Llanowar Elves");
    let idle = t.battlefield(P0, "Grizzly Bears");
    let s = t.battlefield(P1, "Gravel Slinger");
    attack_with(&mut t, &[attacker]);
    act(&mut t, P1, s, "damage", &[Entity::Object(attacker)]).unwrap();
    let c = target_choices(&t, P1).pop().unwrap();
    assert!(c.contains(&Entity::Object(attacker)));
    assert!(!c.contains(&Entity::Object(idle)), "not attacking");
    assert!(!c.contains(&Entity::Object(s)), "not blocking");
    t.resolve_all();
    assert!(!t.on_battlefield(attacker), "1 damage kills the 1/1");
    assert_eq!(t.obj_now(idle).damage, 0);
}

#[test]
fn haunted_dead_returns_from_the_graveyard_tapped() {
    cr!("602.2", "118.3", "701.9a");
    // "{1}{B}, Discard two cards: Return this card from your graveyard to the battlefield
    // tapped."
    supported("Haunted Dead");
    let mut t = TestGame::new(2);
    let hd = t.graveyard(P0, "Haunted Dead");
    t.lands(P0, "Swamp", 2);
    t.hand(P0, "Grizzly Bears");
    assert!(
        act(&mut t, P0, hd, "return", &[]).is_err(),
        "only one card to discard"
    );
    t.hand(P0, "Hill Giant");
    t.hand(P0, "Lightning Bolt");
    t.answer_choose(
        P0,
        &[
            Entity::Object(t.g.player(P0).hand[0]),
            Entity::Object(t.g.player(P0).hand[1]),
        ],
    );
    act(&mut t, P0, hd, "return", &[]).unwrap();
    assert_eq!(t.hand_size(P0), 1, "two cards discarded");
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P0, "Hill Giant"));
    t.resolve_all();
    let v = t.named_on_battlefield("Haunted Dead");
    assert_eq!(v.len(), 1);
    assert!(t.obj_now(v[0]).tapped, "returns tapped");
    assert_eq!(t.obj_now(v[0]).controller, P0);
}

#[test]
fn hidden_nursery_discovers_four_as_a_sorcery() {
    cr!("602.2", "602.5d", "701.57a");
    // "{4}{G}, {T}, Sacrifice this land: Discover 4. Activate only as a sorcery."
    supported("Hidden Nursery");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Hidden Nursery");
    t.lands(P0, "Forest", 5);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Craw Wurm"); // mana value 6
    t.library_top(P0, "Forest");
    t.set_step(P0, Step::Upkeep);
    assert!(
        act(&mut t, P0, n, "discover", &[]).is_err(),
        "only as a sorcery"
    );
    t.set_step(P0, Step::PrecombatMain);
    act(&mut t, P0, n, "discover", &[]).unwrap();
    assert!(!t.on_battlefield(n), "sacrificed as a cost");
    // Don't cast the discovered card: it goes to hand.
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Craw Wurm"));
    let lib = &t.g.player(P0).library;
    let bottom: Vec<String> = lib[..2]
        .iter()
        .map(|o| t.obj(*o).chars.name.to_string())
        .collect();
    assert!(bottom.contains(&"Craw Wurm".to_string()) && bottom.contains(&"Forest".to_string()));
}

#[test]
fn hoverstone_pilgrim_puts_a_graveyard_card_on_the_bottom() {
    cr!("602.2", "115.1");
    // "{2}: Put target card from a graveyard on the bottom of its owner's library."
    supported("Hoverstone Pilgrim");
    let mut t = TestGame::new(2);
    let hp = t.battlefield(P0, "Hoverstone Pilgrim");
    t.lands(P0, "Wastes", 2);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    let mine = t.graveyard(P0, "Grizzly Bears");
    let lib = t.library_size(P1);
    act(&mut t, P0, hp, "bottom", &[Entity::Object(bolt)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(bolt)) && c.contains(&Entity::Object(mine)));
    t.resolve_all();
    assert!(!t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.library_size(P1), lib + 1);
    let bottom = t.g.player(P1).library[0];
    assert_eq!(t.obj(bottom).chars.name.as_str(), "Lightning Bolt");
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn pump_target_creature_until_end_of_turn() {
    cr!("602.2", "611.2a", "613.4c");
    // Icatian Priest: "{1}{W}{W}: Target creature gets +1/+1 until end of turn."
    // Ghitu War Cry: "{R}: Target creature gets +1/+0 until end of turn."
    // Pradesh Gypsies: "{1}{G}, {T}: Target creature gets -2/-0 until end of turn."
    for (card, land, n, tap, (dp, dt)) in [
        ("Icatian Priest", "Plains", 3, false, (1, 1)),
        ("Ghitu War Cry", "Mountain", 1, false, (1, 0)),
        ("Pradesh Gypsies", "Forest", 2, true, (-2, 0)),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, card);
        let giant = t.battlefield(P1, "Hill Giant");
        let other = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, land, n);
        act(&mut t, P0, src, "target creature", &[Entity::Object(giant)]).unwrap();
        assert_eq!(untapped(&t, P0, land), 0, "{card}: the cost was paid");
        assert_eq!(t.obj_now(src).tapped, tap, "{card}");
        t.resolve_all();
        assert_eq!(t.pt(giant), (3 + dp, 3 + dt), "{card}");
        assert_eq!(t.pt(other), (2, 2), "{card}");
        next_turn(&mut t);
        assert_eq!(t.pt(giant), (3, 3), "{card}: until end of turn");
    }
}

#[test]
fn omen_of_the_sea_sacrifices_to_scry_two() {
    cr!("602.2", "701.21a", "701.22a");
    // "{2}{U}, Sacrifice this enchantment: Scry 2."
    supported("Omen of the Sea");
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Omen of the Sea");
    t.lands(P0, "Island", 3);
    act(&mut t, P0, o, "scry", &[]).unwrap();
    assert!(!t.on_battlefield(o));
    assert!(t.in_graveyard(P0, "Omen of the Sea"));
    t.resolve_all();
    assert_eq!(scries(&t), vec![2]);
    assert_eq!(t.hand_size(P0), 0, "no draw from this ability");
}

#[test]
fn caller_of_gales_grants_flying() {
    cr!("602.2", "611.2a", "702.9b");
    // "{1}{U}, {T}: Target creature gains flying until end of turn."
    supported("Caller of Gales");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Caller of Gales");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Island", 2);
    act(&mut t, P0, c, "flying", &[Entity::Object(bears)]).unwrap();
    assert!(t.obj_now(c).tapped);
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Flying));
    attack_with(&mut t, &[bears, other]);
    assert!(!t.g.can_block(blocker, bears), "a wall without reach can't block it");
    assert!(t.g.can_block(blocker, other));
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn wild_roads_makes_a_pilot_as_a_sorcery() {
    cr!("602.2", "602.5d", "111.1", "702.122a");
    // "{1}{G}, {T}, Sacrifice this land: Create a 1/1 colorless Pilot creature token with
    // "This token saddles Mounts and crews Vehicles as though its power were 2 greater."
    // Activate only as a sorcery."
    supported("Wild Roads");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Wild Roads");
    t.lands(P0, "Forest", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(act(&mut t, P0, r, "pilot", &[]).is_err(), "only as a sorcery");
    t.set_step(P0, Step::PrecombatMain);
    act(&mut t, P0, r, "pilot", &[]).unwrap();
    assert!(!t.on_battlefield(r));
    t.resolve_all();
    let pilots = subtype_ids(&t, P0, "Pilot");
    assert_eq!(pilots.len(), 1);
    let p = pilots[0];
    assert_eq!(t.pt(p), (1, 1));
    assert!(t.obj_now(p).is_token());
    assert!(t.obj_now(p).chars.colors.is_colorless(), "colorless");
    // Crew 3 with a power-1 Pilot: it crews as though its power were 3.
    let v = t.battlefield(P0, "Skysovereign, Consul Flagship");
    t.answer_choose(P0, &[Entity::Object(p)]);
    act(&mut t, P0, v, "crew", &[]).unwrap();
    assert!(t.obj_now(p).tapped);
    t.resolve_all();
    assert!(t.obj_now(v).is(CardType::Creature));
}

#[test]
fn felidar_cub_sacrifices_to_destroy_an_enchantment() {
    cr!("602.2", "701.21a", "701.8a", "115.1");
    // "Sacrifice this creature: Destroy target enchantment."
    supported("Felidar Cub");
    let mut t = TestGame::new(2);
    let cub = t.battlefield(P0, "Felidar Cub");
    let ench = t.battlefield(P1, "Ghitu War Cry");
    let other = t.battlefield(P1, "Dragon Roost");
    let art = t.battlefield(P1, "Golden Egg");
    act(&mut t, P0, cub, "destroy", &[Entity::Object(ench)]).unwrap();
    assert!(!t.on_battlefield(cub));
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&Entity::Object(other)));
    assert!(!c.contains(&Entity::Object(art)), "an artifact isn't an enchantment");
    t.resolve_all();
    assert!(!t.on_battlefield(ench));
    assert!(t.on_battlefield(other));
}

/// A loyalty ability creating flying Dragon tokens.
#[test]
fn sarkhan_unbroken_minus_two_makes_a_dragon() {
    cr!("606.3", "606.4", "111.1", "701.7a");
    // "−2: Create a 4/4 red Dragon creature token with flying."
    supported("Sarkhan Unbroken");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sarkhan Unbroken");
    act(&mut t, P0, s, "dragon creature token", &[]).unwrap();
    assert_eq!(t.counters(s, "loyalty"), 2);
    t.resolve_all();
    let d = subtype_ids(&t, P0, "Dragon");
    assert_eq!(d.len(), 1);
    let o = t.obj_now(d[0]);
    assert!(o.is_token());
    assert_eq!(t.pt(d[0]), (4, 4));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Red));
    assert!(o.has_keyword(KeywordKind::Flying));
    assert_eq!(count_subtype(&t, P1, "Dragon"), 0);
}

#[test]
fn nivix_guildmage_loots() {
    cr!("602.2", "121.1", "701.9a");
    // "{1}{U}{R}: Draw a card, then discard a card."
    supported("Nivix Guildmage");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Nivix Guildmage");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    act(&mut t, P0, g, "draw a card, then discard", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib - 1);
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn wily_bandar_gains_indestructible() {
    cr!("602.2", "611.2a", "702.12b");
    // "{2}{G}: This creature gains indestructible until end of turn."
    supported("Wily Bandar");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wily Bandar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    act(&mut t, P0, w, "indestructible", &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(w).has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    t.g.destroy(w, None);
    t.g.deal_damage(bears, Entity::Object(w), 5, false);
    t.settle();
    assert!(t.on_battlefield(w), "not destroyed, not by lethal damage");
    next_turn(&mut t);
    assert!(!t.obj_now(w).has_keyword(KeywordKind::Indestructible));
}

#[test]
fn treefolk_healer_prevents_the_next_two_damage() {
    cr!("602.2", "615.1a", "615.7");
    // "{2}{W}, {T}: Prevent the next 2 damage that would be dealt to any target this turn."
    supported("Treefolk Healer");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Treefolk Healer");
    let src = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    act(&mut t, P0, h, "prevent", &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    t.g.deal_damage(src, Entity::Player(P0), 3, false);
    t.settle();
    assert_eq!(t.life(P0), 19, "2 of the 3 damage prevented");
    t.g.deal_damage(src, Entity::Player(P0), 3, false);
    t.settle();
    assert_eq!(t.life(P0), 16, "the shield is used up");
    t.g.deal_damage(src, Entity::Player(P1), 3, false);
    t.settle();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn loyalty_abilities_deal_damage_to_any_target() {
    cr!("606.3", "606.4", "120.3e", "120.3a");
    // Wrenn and Six: "−1: Wrenn and Six deals 1 damage to any target."
    // Chandra, the Firebrand: "+1: Chandra deals 1 damage to any target."
    for (card, text, loyalty) in [
        ("Wrenn and Six", "-1:", 2),
        ("Chandra, the Firebrand", "+1:", 4),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let pw = t.battlefield(P0, card);
        let elves = t.battlefield(P1, "Llanowar Elves");
        let bears = t.battlefield(P1, "Grizzly Bears");
        act(&mut t, P0, pw, text, &[Entity::Object(elves)]).unwrap();
        assert_eq!(t.counters(pw, "loyalty"), loyalty, "{card}");
        t.resolve_all();
        assert!(!t.on_battlefield(elves), "{card}");
        assert_eq!(t.obj_now(bears).damage, 0, "{card}");
        assert!(
            act(&mut t, P0, pw, text, &[Entity::Player(P1)]).is_err(),
            "{card}: one loyalty ability per turn"
        );
        t.clear_answers();
        next_turn(&mut t);
        t.advance_to(P0, Step::PrecombatMain);
        act(&mut t, P0, pw, text, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
        assert_eq!(t.life(P1), 19, "{card}");
    }
}

#[test]
fn self_pump_by_discarding_a_card() {
    cr!("602.2", "118.3", "701.9a", "611.2a");
    // Stalking Bloodsucker: "{1}{B}, Discard a card: This creature gets +2/+2 until end
    // of turn."
    supported("Stalking Bloodsucker");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Stalking Bloodsucker");
    t.lands(P0, "Swamp", 2);
    assert!(act(&mut t, P0, s, "discard", &[]).is_err(), "no card to discard");
    t.hand(P0, "Grizzly Bears");
    act(&mut t, P0, s, "discard", &[]).unwrap();
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.pt(s), (6, 6));
    next_turn(&mut t);
    assert_eq!(t.pt(s), (4, 4));
}

#[test]
fn azami_taps_an_untapped_wizard_to_draw() {
    cr!("602.2", "118.3", "302.6");
    // "Tap an untapped Wizard you control: Draw a card."
    supported("Azami, Lady of Scrolls");
    let mut t = TestGame::new(2);
    let azami = t.battlefield(P0, "Azami, Lady of Scrolls");
    let wiz = t.battlefield_sick(P0, "Puppeteer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Caller of Gales");
    t.answer_choose(P0, &[Entity::Object(wiz)]);
    act(&mut t, P0, azami, "draw", &[]).unwrap();
    assert!(t.obj_now(wiz).tapped, "a summoning-sick Wizard can be tapped for the cost");
    assert!(!t.obj_now(azami).tapped && !t.obj_now(bears).tapped);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // Azami is a Wizard too; then no untapped Wizard is left.
    act(&mut t, P0, azami, "draw", &[]).unwrap();
    assert!(t.obj_now(azami).tapped);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert!(
        act(&mut t, P0, azami, "draw", &[]).is_err(),
        "the opponent's Wizard and a non-Wizard can't pay"
    );
}

#[test]
fn escape_artist_discards_to_return_to_hand() {
    cr!("602.2", "118.3", "701.9a");
    // "{U}, Discard a card: Return this creature to its owner's hand."
    supported("Escape Artist");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Escape Artist");
    t.lands(P0, "Island", 1);
    assert!(act(&mut t, P0, e, "return", &[]).is_err(), "no card to discard");
    t.hand(P0, "Grizzly Bears");
    act(&mut t, P0, e, "return", &[]).unwrap();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert!(!t.on_battlefield(e));
    assert!(t.in_hand(P0, "Escape Artist"));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn blightspeaker_makes_target_player_lose_life() {
    cr!("602.2", "119.3", "115.1");
    // "{T}: Target player loses 1 life."
    supported("Blightspeaker");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Blightspeaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, b, "loses", &[Entity::Player(P1)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(!c.contains(&Entity::Object(bears)), "players only");
    assert!(t.obj_now(b).tapped);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn bramblesnap_taps_a_creature_to_grow() {
    cr!("602.2", "118.3", "611.2a");
    // "Tap an untapped creature you control: This creature gets +1/+1 until end of turn."
    supported("Bramblesnap");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Bramblesnap");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    act(&mut t, P0, b, "+1/+1", &[]).unwrap();
    assert!(t.obj_now(bears).tapped && !t.obj_now(b).tapped);
    t.resolve_all();
    assert_eq!(t.pt(b), (2, 2));
    assert_eq!(t.pt(bears), (2, 2));
    // Tapping itself.
    t.answer_choose(P0, &[Entity::Object(b)]);
    act(&mut t, P0, b, "+1/+1", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(b), (3, 3));
    assert!(
        act(&mut t, P0, b, "+1/+1", &[]).is_err(),
        "no untapped creature you control is left"
    );
    next_turn(&mut t);
    assert_eq!(t.pt(b), (1, 1));
}

#[test]
fn chittering_witch_sacrifices_a_creature_to_shrink_one() {
    cr!("602.2", "701.21a", "611.2a");
    // "{1}{B}, Sacrifice a creature: Target creature gets -2/-2 until end of turn."
    supported("Chittering Witch");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Chittering Witch");
    let fodder = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    act(&mut t, P0, w, "-2/-2", &[Entity::Object(giant)]).unwrap();
    assert!(!t.on_battlefield(fodder));
    assert!(t.on_battlefield(w));
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    assert_eq!(t.pt(bears), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn dimir_house_guard_sacrifices_a_creature_to_regenerate() {
    cr!("602.2", "701.19a", "701.21a");
    // "Sacrifice a creature: Regenerate this creature."
    supported("Dimir House Guard");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Dimir House Guard");
    let fodder = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    act(&mut t, P0, g, "regenerate", &[]).unwrap();
    assert!(!t.on_battlefield(fodder));
    t.resolve_all();
    assert!(!t.g.destroy(g, None), "the shield replaces destruction");
    t.settle();
    assert!(t.on_battlefield(g));
    assert!(t.obj_now(g).tapped, "regenerated: tapped");
    assert!(t.g.destroy(g, None), "the shield is used up");
    assert!(t.in_graveyard(P0, "Dimir House Guard"));
}

#[test]
fn dion_exiles_and_returns_transformed() {
    cr!("602.2", "602.5d", "701.27a", "400.7");
    // "{4}{W}{W}, {T}: Exile Dion, then return it to the battlefield transformed under its
    // owner's control. Activate only as a sorcery."
    let name = "Dion, Bahamut's Dominant // Bahamut, Warden of Light";
    supported(name);
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, name);
    t.lands(P0, "Plains", 6);
    t.set_step(P0, Step::EndOfCombat);
    assert!(act(&mut t, P0, d, "exile", &[]).is_err(), "only as a sorcery");
    t.set_step(P0, Step::PrecombatMain);
    act(&mut t, P0, d, "exile", &[]).unwrap();
    assert!(t.obj_now(d).tapped);
    t.resolve_all();
    assert!(!t.g.is_live(d), "a new object");
    let b = t.named_on_battlefield("Bahamut, Warden of Light");
    assert_eq!(b.len(), 1);
    assert_eq!(t.obj_now(b[0]).controller, P0);
    assert!(t.obj_now(b[0]).chars.has_subtype("Saga"));
    assert!(t.named_on_battlefield("Dion, Bahamut's Dominant").is_empty());
}

#[test]
fn endling_gains_menace() {
    cr!("602.2", "611.2a", "702.111b");
    // "{B}: This creature gains menace until end of turn."
    supported("Endling");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Endling");
    t.lands(P0, "Swamp", 1);
    act(&mut t, P0, e, "menace", &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(e).has_keyword(KeywordKind::Menace));
    assert!(!t.obj_now(e).has_keyword(KeywordKind::Deathtouch));
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[e]);
    let opts = vec![(b1, vec![e]), (b2, vec![e])];
    assert!(
        !combat::block_declaration_legal(&t.g, &opts, &[(b1, e)]),
        "a single creature can't block it"
    );
    assert!(combat::block_declaration_legal(
        &t.g,
        &opts,
        &[(b1, e), (b2, e)]
    ));
    next_turn(&mut t);
    assert!(!t.obj_now(e).has_keyword(KeywordKind::Menace));
}

#[test]
fn sacrifice_this_to_pump_target_creature() {
    cr!("602.2", "701.21a", "611.2a");
    // "Sacrifice this enchantment: Target creature gets +3/+3 until end of turn."
    supported("Seal of Strength");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Seal of Strength");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Llanowar Elves");
    act(&mut t, P0, s, "+3/+3", &[Entity::Object(bears)]).unwrap();
    assert!(!t.on_battlefield(s));
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(other), (1, 1));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn flailing_soldier_any_player_may_pump_it() {
    cr!("602.2", "611.2a");
    // "{1}: This creature gets +1/+1 until end of turn. Any player may activate this
    // ability."
    supported("Flailing Soldier");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Flailing Soldier");
    t.lands(P1, "Wastes", 1);
    t.lands(P0, "Wastes", 1);
    act(&mut t, P1, s, "+1/+1", &[]).unwrap();
    assert_eq!(untapped(&t, P1, "Wastes"), 0, "P1 paid");
    assert_eq!(untapped(&t, P0, "Wastes"), 1);
    t.resolve_all();
    assert_eq!(t.pt(s), (3, 3));
    act(&mut t, P0, s, "+1/+1", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(s), (4, 4));
    next_turn(&mut t);
    assert_eq!(t.pt(s), (2, 2));
}

#[test]
fn goldmeadow_nomad_exiles_itself_from_the_graveyard_for_a_token() {
    cr!("602.2", "602.5d", "111.1");
    // "{W}, Exile this card from your graveyard: Create a 1/1 green and white Kithkin
    // creature token. Activate only as a sorcery."
    supported("Goldmeadow Nomad");
    let mut t = TestGame::new(2);
    let on_bf = t.battlefield(P0, "Goldmeadow Nomad");
    let n = t.graveyard(P0, "Goldmeadow Nomad");
    t.lands(P0, "Plains", 1);
    assert!(
        act(&mut t, P0, on_bf, "kithkin", &[]).is_err(),
        "only from the graveyard"
    );
    t.set_step(P0, Step::Upkeep);
    assert!(act(&mut t, P0, n, "kithkin", &[]).is_err(), "only as a sorcery");
    t.set_step(P0, Step::PrecombatMain);
    act(&mut t, P0, n, "kithkin", &[]).unwrap();
    assert_eq!(t.zone(n), Zone::Exile);
    t.resolve_all();
    let k: Vec<_> = subtype_ids(&t, P0, "Kithkin")
        .into_iter()
        .filter(|id| t.obj_now(*id).is_token())
        .collect();
    assert_eq!(k.len(), 1);
    assert_eq!(t.pt(k[0]), (1, 1));
    let colors = t.obj_now(k[0]).chars.colors;
    assert!(colors.contains(Color::White) && colors.contains(Color::Green));
    assert_eq!(colors.count(), 2);
}

#[test]
fn crook_of_condemnation_exiles_a_graveyard_card() {
    cr!("602.2", "701.13a", "115.1");
    // "{1}, {T}: Exile target card from a graveyard."
    supported("Crook of Condemnation");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Crook of Condemnation");
    t.lands(P0, "Wastes", 1);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P1, "Grizzly Bears");
    act(&mut t, P0, c, "target card", &[Entity::Object(bolt)]).unwrap();
    assert!(t.obj_now(c).tapped);
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.graveyard_size(P1), 1);
}

#[test]
fn dragon_roost_makes_a_flying_dragon() {
    cr!("602.2", "111.1", "701.7a");
    // "{5}{R}{R}: Create a 5/5 red Dragon creature token with flying."
    supported("Dragon Roost");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Dragon Roost");
    t.lands(P0, "Mountain", 7);
    act(&mut t, P0, r, "dragon", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Mountain"), 0);
    t.resolve_all();
    let d = subtype_ids(&t, P0, "Dragon");
    assert_eq!(d.len(), 1);
    assert_eq!(t.pt(d[0]), (5, 5));
    assert!(t.obj_now(d[0]).is_token());
    assert_eq!(t.obj_now(d[0]).chars.colors, ColorSet::single(Color::Red));
    assert!(t.obj_now(d[0]).has_keyword(KeywordKind::Flying));
}

#[test]
fn puppeteer_taps_or_untaps_target_creature() {
    cr!("602.2", "701.26a", "701.26b");
    // "{U}, {T}: You may tap or untap target creature."
    supported("Puppeteer");
    // (tapped before, "you may" accepted)
    for (tapped, may) in [(false, true), (true, true), (true, false)] {
        let mut t = TestGame::new(2);
        let p = t.battlefield(P0, "Puppeteer");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let other = t.battlefield(P1, "Hill Giant");
        if tapped {
            t.g.tap(bears);
            t.g.tap(other);
        }
        t.lands(P0, "Island", 1);
        act(&mut t, P0, p, "tap or untap", &[Entity::Object(bears)]).unwrap();
        // Choose the action that changes it.
        t.answer(P0, DecisionKind::Option, Answer::Index(usize::from(tapped)));
        t.answer_yes(P0, may);
        t.resolve_all();
        assert_eq!(
            t.obj_now(bears).tapped,
            tapped != may,
            "tapped before: {tapped}, may: {may}"
        );
        assert_eq!(t.obj_now(other).tapped, tapped);
    }
}

#[test]
fn vraska_minus_three_destroys_target_creature() {
    cr!("606.3", "606.4", "701.8a");
    // "−3: Destroy target creature."
    supported("Vraska, Scheming Gorgon");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vraska, Scheming Gorgon");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, v, "destroy", &[Entity::Object(giant)]).unwrap();
    assert_eq!(t.counters(v, "loyalty"), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(bears));
}

#[test]
fn mount_doom_damages_each_opponent() {
    cr!("602.2", "120.3a");
    // "{1}{B}{R}, {T}: Mount Doom deals 1 damage to each opponent."
    supported("Mount Doom");
    let mut t = TestGame::new(3);
    let m = t.battlefield(P0, "Mount Doom");
    t.lands(P0, "Badlands", 3);
    act(&mut t, P0, m, "each opponent", &[]).unwrap();
    assert!(t.obj_now(m).tapped);
    assert_eq!(untapped(&t, P0, "Badlands"), 0);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(PlayerId(2)), 19);
}
