//! roundtrip-pairs-3 (docs/agents/items/roundtrip-pairs-3.md): untriaged round-trip
//! mismatches from the long tail (the two-card clusters).

use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts the real Aura `name` onto the battlefield under `p`'s control attached to `to`.
fn attach_new(t: &mut TestGame, p: PlayerId, name: &str, to: ObjectId) -> ObjectId {
    let id = t.battlefield(p, name);
    assert!(t.g.attach(id, Entity::Object(to)), "{name} couldn't be attached");
    t.g.recompute();
    id
}

/// Activates the crew ability (the first activated ability) of `vehicle`, tapping `crew`.
fn crew(t: &mut TestGame, p: PlayerId, vehicle: ObjectId, crew: ObjectId) {
    t.answer_choose(p, &[Entity::Object(crew)]);
    let r = t.activate(p, vehicle, 0, &[]);
    t.clear_answers();
    assert!(r.is_ok(), "{r:?}");
    t.resolve_all();
}

#[test]
fn enchanted_creature_on_a_vehicle_applies_once_it_is_a_creature() {
    cr!("303.4m", "301.7a", "301.7b", "613.1d", "613.4c");
    // "Enchant creature or Vehicle ... Enchanted creature gets -4/-0." The static now
    // affects the enchanted permanent while it's a creature: the crew ability's type
    // change (layer 4) comes before the -4/-0 (layer 7c).
    for (aura, minus) in [("Aether Meltdown", 4), ("Mists of Littjara", 3)] {
        let mut t = TestGame::new(2);
        let copter = t.battlefield(P0, "Smuggler's Copter");
        let bear = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P1, aura, copter);
        assert!(!t.obj_now(copter).is(CardType::Creature));
        crew(&mut t, P0, copter, bear);
        assert!(t.obj_now(copter).is(CardType::Creature));
        assert_eq!(t.pt(copter), (3 - minus, 3), "{aura}");
        // On a creature.
        let wurm = t.battlefield(P1, "Craw Wurm");
        attach_new(&mut t, P0, aura, wurm);
        assert_eq!(t.pt(wurm), (6 - minus, 4), "{aura}");
    }
    // Aerial Modification makes the Vehicle it enchants a creature, which then gets
    // +2/+2 and flying.
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    attach_new(&mut t, P0, "Aerial Modification", copter);
    assert!(t.obj_now(copter).is(CardType::Creature));
    assert_eq!(t.pt(copter), (5, 5));
    assert!(t.obj_now(copter).has_keyword(KeywordKind::Flying));
}

#[test]
fn ydwen_efreet_losing_the_flip_unblocks_the_creature_it_blocked() {
    cr!("705.2", "506.4", "509.1h");
    // "Whenever ~ blocks, flip a coin. If you lose the flip, remove ~ from combat and it
    // can't block this turn. Creatures it was blocking that had become blocked by only ~
    // this combat become unblocked."
    for (won, life) in [(true, 20), (false, 18)] {
        let mut t = TestGame::new(2);
        let efreet = t.battlefield(P0, "Ydwen Efreet");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.dice.loaded_coins.push(won);
        t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
        t.attack(&[(bears, Entity::Player(P0))], &[(efreet, bears)]);
        assert_eq!(t.life(P0), life, "won: {won}");
        // The Efreet dealt combat damage only if it stayed in combat.
        assert_eq!(t.on_battlefield(bears), !won, "won: {won}");
    }
}

#[test]
fn abdel_adrian_makes_a_soldier_for_an_exiled_token_too() {
    cr!("111.7", "610.3");
    ruling!(
        "Abdel Adrian, Gorion's Ward",
        "If you exile a token nonland permanent this way, it ceases to exist. It won't be returned to the battlefield. You'll still create a Soldier creature token for it though."
    );
    // "When Abdel Adrian enters, exile any number of other nonland permanents you control
    // until Abdel Adrian leaves the battlefield. Create a 1/1 white Soldier creature token
    // for each permanent exiled this way."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let treasure = t.battlefield(P0, "Treasure");
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(treasure)]);
    t.enter(P0, "Abdel Adrian, Gorion's Ward");
    t.resolve_all();
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(treasure));
    let soldiers = t
        .g
        .permanents_controlled_by(P0)
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.has_subtype("Soldier"))
        .count();
    assert_eq!(soldiers, 2, "{}", t.dump_log());
}

#[test]
fn graveyard_triggers_dont_see_creatures_dying_with_them() {
    cr!("603.10a", "113.6k");
    ruling!(
        "Boneyard Scourge",
        "If Boneyard Scourge dies at the same time as another Dragon you control, its ability won't trigger."
    );
    ruling!(
        "Furious Forebear",
        "If Furious Forebear dies at the same time as one or more creatures you control, its ability won’t trigger."
    );
    // "Whenever a Dragon you control dies while ~ is in your graveyard, you may pay
    // {1}{B}. If you do, return ~ from your graveyard to the battlefield." / "Whenever a
    // creature you control dies while ~ is in your graveyard, you may pay {1}{W}. If you
    // do, return ~ from your graveyard to your hand."
    for (card, other) in [
        ("Boneyard Scourge", "Shivan Dragon"),
        ("Furious Forebear", "Grizzly Bears"),
    ] {
        // Dying together: no trigger.
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        t.battlefield(P0, other);
        t.lands(P0, "Plains", 4);
        t.lands(P0, "Swamp", 2);
        t.answer_yes(P0, true);
        let w = t.hand(P0, "Wrath of God");
        t.cast(P0, w).go();
        t.resolve_all();
        assert!(t.in_graveyard(P0, card), "{card}: {}", t.dump_log());
        // Already in the graveyard when another one dies: it returns.
        let mut t = TestGame::new(2);
        t.graveyard(P0, card);
        let o = t.battlefield(P0, other);
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Swamp", 1);
        t.answer_yes(P0, true);
        t.g.destroy(o, None);
        t.settle();
        t.resolve_all();
        assert!(!t.in_graveyard(P0, card), "{card}: {}", t.dump_log());
    }
}

#[test]
fn abdel_adrian_leaving_first_exiles_nothing_and_makes_no_soldiers() {
    cr!("610.3b");
    ruling!(
        "Abdel Adrian, Gorion's Ward",
        "If Abdel Adrian leaves the battlefield before its enters-the-battlefield ability resolves, you can't exile any nonland permanents. You won't create any Soldier creature tokens in this case."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let abdel = t.enter(P0, "Abdel Adrian, Gorion's Ward");
    t.settle();
    t.g.destroy(abdel, None);
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(bears), "{}", t.dump_log());
    let soldiers = t
        .g
        .permanents_controlled_by(P0)
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.has_subtype("Soldier"))
        .count();
    assert_eq!(soldiers, 0, "{}", t.dump_log());
}

#[test]
fn goblin_researcher_lets_you_play_only_its_own_card() {
    cr!("201.5");
    ruling!(
        "Goblin Researcher",
        "If you control more than one card named Goblin Researcher, each one's effect is independent from the other. Specifically, attacking with one will not allow you to play the card exiled by the other."
    );
    // "When ~ enters, exile the top card of your library. During any turn you attacked
    // with ~, you may play that card."
    use mtg_engine::turn::Step;
    let mut t = TestGame::new(2);
    let first = t.library_top(P0, "Ornithopter");
    let a = t.enter(P0, "Goblin Researcher");
    t.resolve_all();
    let second = t.library_top(P0, "Ornithopter");
    t.enter(P0, "Goblin Researcher");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(first)), mtg_engine::object::Zone::Exile);
    assert_eq!(t.zone(t.g.current(second)), mtg_engine::object::Zone::Exile);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    let can_cast = |t: &mut TestGame, card: ObjectId| {
        t.g.turn.priority = Some(P0);
        t.g.recompute();
        let card = t.g.current(card);
        t.g.legal_actions(P0)
            .iter()
            .any(|x| matches!(x, mtg_engine::decision::Action::Cast { card: c, .. } if *c == card))
    };
    // Not before attacking.
    assert!(!can_cast(&mut t, first) && !can_cast(&mut t, second));
    t.attack(&[(a, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_cast(&mut t, first), "{}", t.dump_log());
    assert!(!can_cast(&mut t, second));
}

#[test]
fn weathered_bodyguards_takes_only_unblocked_creatures_damage() {
    cr!("510.1c", "702.19b", "614.1b");
    ruling!(
        "Weathered Bodyguards",
        "Weathered Bodyguards affects only combat damage from unblocked creatures. It won’t affect noncombat damage, and it won’t affect trample damage."
    );
    // "As long as ~ is untapped, all combat damage that would be dealt to you by unblocked
    // creatures is dealt to ~ instead."
    use mtg_engine::turn::Step;
    let mut t = TestGame::new(2);
    let guards = t.battlefield(P0, "Weathered Bodyguards");
    let blocker = t.battlefield(P0, "Grizzly Bears");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.attack(
        &[(dreadmaw, Entity::Player(P0)), (bears, Entity::Player(P0))],
        &[(blocker, dreadmaw)],
    );
    // The trampler's excess damage reaches P0; the unblocked Bears' damage goes to the
    // Bodyguards.
    assert_eq!(t.life(P0), 20 - 4, "{}", t.dump_log());
    assert_eq!(t.obj_now(guards).damage, 2);
}
