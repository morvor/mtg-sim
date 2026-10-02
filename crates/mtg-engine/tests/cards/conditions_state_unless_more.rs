//! More state conditions: what a referenced player has ("unless they control a
//! commander", "unless they have exactly three or exactly four cards in hand"), parity
//! ("unless it has an even number of counters on it"), the greatest power or mana value,
//! two conditions with one subject ("if you control no permanents other than ~ and have no
//! cards in hand"), "[effect] instead if [condition]", several conditional parts of one
//! ability ("{1} less to cast if ... and {1} less to cast if ...", "has lifelink if ...,
//! deathtouch if ..."), "skip your upkeep step if ...", eminence statics, and "any player
//! may pay ... If no one does, ...".

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

#[test]
fn crimson_honor_guard_spares_players_who_control_a_commander() {
    cr!("903.3", "608.2h");
    ruling!(
        "Crimson Honor Guard",
        "The player could control any player's commander to satisfy Crimson Honor Guard's ability."
    );
    ruling!(
        "Crimson Honor Guard",
        "Crimson Honor Guard will deal 4 damage to you, too, if you don't control a commander."
    );
    assert_supported(&["Crimson Honor Guard"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crimson Honor Guard");
    // P1 controls P0's commander.
    let cmdr = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.set_step(P0, Step::Upkeep);
    t.advance_to_step(Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 16, "no commander: 4 damage to you");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn skullcage_spares_exactly_three_or_four_cards() {
    cr!("608.2h");
    ruling!(
        "Skullcage",
        "Skullcage’s ability deals damage if the player has zero, one, two, five, or more than five cards in hand when its ability resolves."
    );
    assert_supported(&["Skullcage"]);
    for (cards, damaged) in [(2, true), (3, false), (4, false), (5, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Skullcage");
        t.g.player_mut(P1).hand.clear();
        for _ in 0..cards {
            t.hand(P1, "Grizzly Bears");
        }
        t.set_step(P0, Step::End);
        t.advance_to(P1, Step::Upkeep);
        t.resolve_all();
        let expected = if damaged { 18 } else { 20 };
        assert_eq!(t.life(P1), expected, "{cards} cards");
    }
}

#[test]
fn sab_sunen_attacks_only_with_an_even_number_of_counters() {
    cr!("611.3a", "508.1c");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sab-Sunen, Luxa Embodied");
    t.set_step(P0, Step::BeginningOfCombat);
    t.g.recompute();
    // Zero counters is even: it can attack.
    assert!(mtg_engine::combat::attack_options(&t.g)
        .iter()
        .any(|(c, _)| *c == s));
    t.g.add_counters(Entity::Object(s), "+1/+1", 1, None);
    t.g.recompute();
    assert!(!mtg_engine::combat::attack_options(&t.g)
        .iter()
        .any(|(c, _)| *c == s));
    t.g.add_counters(Entity::Object(s), "+1/+1", 1, None);
    t.g.recompute();
    assert!(mtg_engine::combat::attack_options(&t.g)
        .iter()
        .any(|(c, _)| *c == s));
}

#[test]
fn pipsqueak_can_attack_alone_only_with_a_counter() {
    cr!("506.5", "611.3a");
    assert_supported(&["Pipsqueak, Rebel Strongarm"]);
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Pipsqueak, Rebel Strongarm");
    t.set_step(P0, Step::BeginningOfCombat);
    t.g.recompute();
    let alone = [(p, Entity::Player(P1))];
    let options = mtg_engine::combat::attack_options(&t.g);
    assert!(!mtg_engine::combat::attack_declaration_legal(&t.g, &options, &alone));
    t.g.add_counters(Entity::Object(p), "+1/+1", 1, None);
    t.g.recompute();
    let options = mtg_engine::combat::attack_options(&t.g);
    assert!(mtg_engine::combat::attack_declaration_legal(&t.g, &options, &alone));
}

#[test]
fn might_makes_right_needs_every_greatest_power_creature() {
    cr!("603.4");
    ruling!(
        "Might Makes Right",
        "If another player controls a creature with the greatest power or tied for the greatest power at that time, the ability won’t trigger at all."
    );
    assert_supported(&["Might Makes Right"]);
    // Tied for the greatest power with an opponent's creature: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Might Makes Right");
    t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.obj_now(theirs).controller, P1);
    // The only creature with the greatest power is yours: gain control.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Might Makes Right");
    t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.obj_now(theirs).controller, P0);
}

#[test]
fn padeem_draws_with_the_greatest_mana_value_artifact() {
    cr!("603.4");
    ruling!(
        "Padeem, Consul of Innovation",
        "The artifact you control has to have the highest mana value only among artifacts on the battlefield, not among all permanents on the battlefield."
    );
    assert_supported(&["Padeem, Consul of Innovation"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Padeem, Consul of Innovation");
    t.battlefield(P0, "Sol Ring");
    t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::End);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "Sol Ring is the only artifact");
    // An opponent's artifact with a greater mana value: no card.
    t.battlefield(P1, "Mind Stone");
    t.set_step(P1, Step::End);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn barren_glory_needs_both_conditions() {
    cr!("603.4", "104.2b");
    ruling!(
        "Barren Glory",
        "Both conditions need to be true for Barren Glory’s ability to trigger"
    );
    assert_supported(&["Barren Glory"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Barren Glory");
    t.battlefield(P0, "Plains");
    t.g.player_mut(P0).hand.clear();
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.has_lost(P1), "another permanent");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Barren Glory");
    t.g.player_mut(P0).hand.clear();
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn teachings_of_the_archaics_draws_more_with_four_more_cards() {
    cr!("608.2h");
    ruling!(
        "Teachings of the Archaics",
        "The number of cards in each player’s hand is determined as Teachings of the Archaics resolves."
    );
    assert_supported(&["Teachings of the Archaics"]);
    for (theirs, drawn) in [(0, 0), (3, 2), (4, 3)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 3);
        t.g.player_mut(P1).hand.clear();
        for _ in 0..theirs {
            t.hand(P1, "Grizzly Bears");
        }
        let spell = t.hand(P0, "Teachings of the Archaics");
        t.cast(P0, spell).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), drawn, "{theirs} cards in P1's hand");
    }
}

#[test]
fn assassins_ink_two_reductions_from_one_artifact_enchantment() {
    cr!("601.2f");
    ruling!(
        "Assassin's Ink",
        "If you control an artifact that is also an enchantment, Assassin's Ink costs {2} less to cast."
    );
    assert_supported(&["Assassin's Ink"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ink = t.hand(P0, "Assassin's Ink");
    assert!(t.cast(P0, ink).target(bears).try_go().is_err(), "{{2}}{{B}}{{B}}");
    // A legendary enchantment artifact counts for both: {B}{B}.
    t.battlefield(P0, "Spear of Heliod");
    assert!(t.cast(P0, ink).target(bears).try_go().is_ok());
    // An artifact alone: {1}{B}{B}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let ring = t.battlefield(P0, "Sol Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ink = t.hand(P0, "Assassin's Ink");
    t.g.objects[ring.0 as usize].tapped = true;
    assert!(t.cast(P0, ink).target(bears).try_go().is_err());
}

#[test]
fn multiclass_baldric_grants_each_keyword_for_its_class() {
    cr!("611.3a", "613.1f");
    assert_supported(&["Multiclass Baldric"]);
    let mut t = TestGame::new(2);
    let baldric = t.battlefield(P0, "Multiclass Baldric");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(baldric, Entity::Object(bears));
    t.g.recompute();
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
    // A Wizard: flying only.
    t.battlefield(P0, "Prodigal Sorcerer");
    t.g.recompute();
    let o = t.obj_now(bears);
    assert!(o.has_keyword(KeywordKind::Flying));
    assert!(!o.has_keyword(KeywordKind::Deathtouch));
    assert!(!o.has_keyword(KeywordKind::Haste));
    assert!(!o.has_keyword(KeywordKind::Lifelink));
    // A Rogue too: deathtouch.
    t.battlefield(P0, "Merfolk Looter");
    t.g.recompute();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Deathtouch));
}

#[test]
fn gibbering_descent_skips_your_upkeep_with_an_empty_hand() {
    cr!("614.10", "611.3a");
    ruling!(
        "Gibbering Descent",
        "If you have no cards in hand at the time your upkeep step would start, instead that step is skipped and your draw step starts."
    );
    assert_supported(&["Gibbering Descent"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gibbering Descent");
    t.g.player_mut(P0).hand.clear();
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    // No upkeep: Gibbering Descent's own upkeep trigger didn't make P0 lose life.
    assert_eq!(t.life(P0), 20);
    // With a card in hand the upkeep happens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gibbering Descent");
    t.hand(P0, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn the_ur_dragon_reduces_dragon_spells_from_the_command_zone() {
    cr!("113.6b", "601.2f");
    assert_supported(&["The Ur-Dragon"]);
    let mut t = TestGame::new(2);
    t.command(P0, "The Ur-Dragon");
    t.lands(P0, "Mountain", 4);
    // Shivan Dragon costs {4}{R}{R}: with the reduction, {3}{R}{R}.
    let dragon = t.hand(P0, "Shivan Dragon");
    assert!(t.cast(P0, dragon).try_go().is_err());
    t.lands(P0, "Mountain", 1);
    assert!(t.cast(P0, dragon).try_go().is_ok());
    // In the graveyard, no reduction.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "The Ur-Dragon");
    t.lands(P0, "Mountain", 5);
    let dragon = t.hand(P0, "Shivan Dragon");
    assert!(t.cast(P0, dragon).try_go().is_err());
}

#[test]
fn rhystic_circle_prevents_unless_any_player_pays() {
    cr!("118.12", "615.7");
    ruling!(
        "Rhystic Circle",
        "Can’t be used to prevent damage to your creatures, just to you."
    );
    assert_supported(&["Rhystic Circle"]);
    let mut t = TestGame::new(2);
    let circle = t.battlefield(P0, "Rhystic Circle");
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let bolt_spell = {
        t.cast(P1, bolt).target(Entity::Player(P0)).go()
    };
    // Nobody pays: choose the Bolt as the source; its damage is prevented.
    t.answer_yes(P0, false);
    t.answer_yes(P1, false);
    t.answer(P0, DecisionKind::Any, Answer::Entities(vec![Entity::Object(bolt_spell)]));
    t.activate(P0, circle, 0, &[]).unwrap();
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn rhystic_lightning_deals_less_if_paid() {
    cr!("118.12a");
    ruling!(
        "Rhystic Lightning",
        "The player gets the option to pay when this spell resolves."
    );
    assert_supported(&["Rhystic Lightning"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P1, "Island", 2);
    let rl = t.hand(P0, "Rhystic Lightning");
    t.answer_yes(P1, true);
    t.cast(P0, rl).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // A creature target: its controller may pay; doesn't.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let wall = t.battlefield(P1, "Wall of Stone");
    let rl = t.hand(P0, "Rhystic Lightning");
    t.answer_yes(P1, false);
    t.cast(P0, rl).target(wall).go();
    t.resolve_all();
    assert_eq!(t.obj_now(wall).damage, 4);
}

#[test]
fn barbarian_bully_any_player_may_take_the_damage() {
    cr!("118.12a");
    assert_supported(&["Barbarian Bully"]);
    let mut t = TestGame::new(2);
    let bully = t.battlefield(P0, "Barbarian Bully");
    t.hand(P0, "Grizzly Bears");
    // P0 declines; P1 takes 4 damage: no +2/+2.
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.activate(P0, bully, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.pt(bully), (2, 2));
}

#[test]
fn nils_taxes_each_creature_by_its_own_counters() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Nils, Discipline Enforcer",
        "All counters on a creature are considered when determining the value of X, not only +1/+1 counters."
    );
    assert_supported(&["Nils, Discipline Enforcer"]);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Nils, Discipline Enforcer");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Gray Ogre");
    let c = t.battlefield(P0, "Hill Giant");
    t.g.add_counters(Entity::Object(a), "+1/+1", 1, None);
    t.g.add_counters(Entity::Object(b), "-1/-1", 1, None);
    t.g.add_counters(Entity::Object(b), "stun", 2, None);
    let mv = |t: &TestGame, o| {
        mtg_engine::combat::required_attack_cost(&t.g, o, Entity::Player(P1))
            .and_then(|c| c.mana)
            .map(|m| m.mana_value())
    };
    assert_eq!(mv(&t, a), Some(1));
    assert_eq!(mv(&t, b), Some(3));
    assert_eq!(mv(&t, c), None, "no counters: no tax");
}

#[test]
fn mishras_war_machine_taps_only_if_it_dealt_damage() {
    cr!("118.12a", "120.4b");
    ruling!(
        "Mishra's War Machine",
        "You can't avoid taking damage if you have no cards to discard."
    );
    assert_supported(&["Mishra's War Machine"]);
    // No cards to discard: 3 damage, and it taps.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mishra's War Machine");
    t.g.player_mut(P0).hand.clear();
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert!(t.obj_now(m).tapped);
    // Discards: no damage, stays untapped.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mishra's War Machine");
    let card = t.hand(P0, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(!t.obj_now(m).tapped);
}

#[test]
fn whipgrass_entangler_cost_is_per_cleric_and_cumulative() {
    cr!("508.1d", "509.1c");
    ruling!(
        "Whipgrass Entangler",
        "If you use this ability on the same creature more than once, the cost is cumulative."
    );
    assert_supported(&["Whipgrass Entangler"]);
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Whipgrass Entangler");
    t.lands(P0, "Plains", 4);
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.activate(P0, w, 0, &[Entity::Object(ogre)]).unwrap();
    t.resolve_all();
    // One Cleric on the battlefield (the Entangler): {1}.
    let mv = |t: &TestGame| {
        mtg_engine::combat::required_attack_cost(&t.g, ogre, Entity::Player(P0))
            .and_then(|c| c.mana)
            .map(|m| m.mana_value())
    };
    assert_eq!(mv(&t), Some(1));
    t.activate(P0, w, 0, &[Entity::Object(ogre)]).unwrap();
    t.resolve_all();
    assert_eq!(mv(&t), Some(2));
    let block = mtg_engine::combat::required_block_cost(&t.g, ogre);
    assert_eq!(block.and_then(|c| c.mana).map(|m| m.mana_value()), Some(2));
}

#[test]
fn reaper_of_night_flies_against_a_small_hand() {
    cr!("603.4", "508.5");
    ruling!(
        "Reaper of Night // Harvest Fear",
        "If the defending player has three or more cards in hand as Reaper of Night attacks, its ability won't trigger at all."
    );
    for (cards, flies) in [(2, true), (3, false)] {
        let mut t = TestGame::new(2);
        let r = t.battlefield(P0, "Reaper of Night // Harvest Fear");
        t.g.player_mut(P1).hand.clear();
        for _ in 0..cards {
            t.hand(P1, "Grizzly Bears");
        }
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(r, Entity::Player(P1))]),
        );
        t.set_step(P0, Step::BeginningOfCombat);
        t.advance_to_step(Step::DeclareBlockers);
        t.resolve_all();
        t.g.recompute();
        assert_eq!(t.obj_now(r).has_keyword(KeywordKind::Flying), flies, "{cards} cards");
    }
}

#[test]
fn aerial_surveyor_searches_when_the_defender_has_more_lands() {
    cr!("603.4", "508.5");
    ruling!(
        "Aerial Surveyor",
        "the ability won't trigger at all unless the defending player controls more lands than you"
    );
    assert_supported(&["Aerial Surveyor"]);
    // A Vehicle: crewed by the Bears before combat.
    let crewed = |t: &mut TestGame| {
        let s = t.battlefield(P0, "Aerial Surveyor");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_choose(P0, &[Entity::Object(bears)]);
        t.activate(P0, s, 0, &[]).unwrap();
        t.resolve_all();
        s
    };
    let mut t = TestGame::new(2);
    let s = crewed(&mut t);
    let plains = t.library_top(P0, "Plains");
    t.lands(P1, "Forest", 1);
    t.answer_choose(P0, &[Entity::Object(plains)]);
    t.attack(&[(s, Entity::Player(P1))], &[]);
    assert_eq!(t.named_on_battlefield("Plains").len(), 1);
    // Equal lands: nothing.
    let mut t = TestGame::new(2);
    let s = crewed(&mut t);
    t.library_top(P0, "Plains");
    t.attack(&[(s, Entity::Player(P1))], &[]);
    assert!(t.named_on_battlefield("Plains").is_empty());
}

#[test]
fn chrome_replicator_needs_two_permanents_sharing_a_name() {
    cr!("603.4", "201.2");
    assert_supported(&["Chrome Replicator"]);
    // Two differently named nonland, nontoken permanents: no token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Gray Ogre");
    t.lands(P0, "Forest", 2);
    t.enter(P0, "Chrome Replicator");
    t.resolve_all();
    assert!(t.named_on_battlefield("Construct Token").is_empty());
    // Two Bears share a name (lands don't count): a Construct.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Chrome Replicator");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Construct Token").len(), 1);
}

#[test]
fn beza_compares_with_each_opponent_separately() {
    cr!("608.2h");
    ruling!(
        "Beza, the Bounding Spring",
        "If you have fewer lands than one opponent and less life than another, for example, you'll get both of those bonuses."
    );
    assert_supported(&["Beza, the Bounding Spring"]);
    let mut t = TestGame::new(3);
    t.lands(P1, "Forest", 1);
    t.g.player_mut(P2).life = 25;
    t.enter(P0, "Beza, the Bounding Spring");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 1);
    assert_eq!(t.life(P0), 24);
    assert!(t.named_on_battlefield("Fish Token").is_empty());
}
