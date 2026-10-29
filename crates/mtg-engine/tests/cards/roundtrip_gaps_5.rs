//! roundtrip-gaps-5 (docs/agents/items/roundtrip-gaps-5.md): the remaining gap clusters
//! (custom nodes) and the next 40 two-card clusters.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn spikeshell_harrier_slows_the_opponent_who_controlled_the_creature() {
    cr!("702.179b", "608.2h");
    // "return target creature or Vehicle an opponent controls to its owner's hand. If that
    // opponent's speed is greater than each other player's speed, reduce that opponent's
    // speed by 1": the opponent who controlled it, even when its owner is another player.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.g.objects[bears.0 as usize].controller = P1;
    t.g.objects[bears.0 as usize].base_controller = P1;
    t.g.players[0].speed = Some(1);
    t.g.players[1].speed = Some(3);
    t.g.players[2].speed = Some(2);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Spikeshell Harrier");
    t.resolve_all();
    assert!(t.in_hand(P2, "Grizzly Bears"));
    assert_eq!(t.g.players[1].speed, Some(2), "{}", t.dump_log());
    assert_eq!(t.g.players[2].speed, Some(2));
}

#[test]
fn bloodthirsty_adversary_casts_the_copies_its_reflexive_ability_made() {
    cr!("603.12", "707.12");
    // "When you pay this cost one or more times, put that many +1/+1 counters on ~, then
    // exile up to that many target instant and/or sorcery cards ... and copy them. You may
    // cast any number of the copies without paying their mana costs.": the copies are
    // made as the reflexive ability resolves, and cast then.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 3);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    // Pay {2}{R} once (there's no mana for more).
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    // Cast the copy, at the opponent.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let a = t.enter(P0, "Bloodthirsty Adversary");
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert!(t.in_exile("Lightning Bolt"));
    assert_eq!(t.life(P1), 17, "{}", t.dump_log());
}

/// Gives `p` control of `id` (its base controller, as a control-changing effect that ended
/// would leave it).
fn steal(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    t.g.objects[id.0 as usize].controller = p;
    t.g.objects[id.0 as usize].base_controller = p;
}

#[test]
fn faerie_slumber_party_counts_the_opponents_who_controlled_the_creatures() {
    cr!("608.2h", "400.7");
    // "For each opponent who controlled a creature returned this way, you create two 1/1
    // blue Faerie creature tokens": who controlled it, not who owns it.
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 6);
    let bears = t.battlefield(P1, "Grizzly Bears");
    steal(&mut t, bears, P2);
    let spell = t.hand(P0, "Faerie Slumber Party");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    let faeries = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.obj_now(**o).chars.name.starts_with("Faerie"))
        .count();
    assert_eq!(faeries, 2, "{}", t.dump_log());
}

#[test]
fn oversimplify_counts_the_power_of_creatures_each_player_controlled() {
    cr!("608.2h", "400.7");
    // "Each player creates a 0/0 green and blue Fractal creature token and puts a number of
    // +1/+1 counters on it equal to the total power of creatures they controlled that were
    // exiled this way."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Forest", 3);
    let giant = t.battlefield(P1, "Hill Giant");
    steal(&mut t, giant, P0);
    t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Oversimplify");
    t.cast(P0, spell).go();
    t.resolve_all();
    let fractal = |p: PlayerId| {
        t.g.battlefield
            .iter()
            .copied()
            .find(|o| t.obj_now(*o).controller == p && t.obj_now(*o).chars.name.contains("Fractal"))
    };
    let mine = fractal(P0).expect("P0's Fractal");
    let theirs = fractal(P1).expect("P1's Fractal");
    assert_eq!(t.counters(mine, "+1/+1"), 3, "{}", t.dump_log());
    assert_eq!(t.counters(theirs, "+1/+1"), 2);
}

#[test]
fn word_of_command_mana_can_be_spent_only_on_the_chosen_card() {
    cr!("723.7", "106.6");
    // "While doing so, the player can activate mana abilities only if they're from lands
    // that player controls and only if mana they produce is spent to activate other mana
    // abilities of lands the player controls and/or to play that card."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P1, "Ancient Tomb");
    let stone = t.battlefield(P1, "Mind Stone");
    let card = t.hand(P1, "Bonesplitter");
    let dismember = t.hand(P1, "Dismember");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let woc = t.hand(P0, "Word of Command");
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.cast(P0, woc).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bonesplitter").len(), 1, "{}", t.dump_log());
    // Ancient Tomb made {C}{C}: one paid for Bonesplitter, the other is still in the
    // pool. It can't pay for an artifact's ability ...
    assert_eq!(t.g.players[1].mana_pool.mana.len(), 1);
    assert!(t.activate(P1, stone, 1, &[]).is_err());
    assert!(t.on_battlefield(stone));
    // ... nor for a land's ability that isn't a mana ability (with no other mana
    // available) ...
    let factory = t.battlefield(P1, "Mishra's Factory");
    t.g.objects[factory.0 as usize].tapped = true;
    t.g.objects[stone.0 as usize].tapped = true;
    assert!(t.activate(P1, factory, 1, &[]).is_err());
    assert!(!t.g.obj(factory).chars.is_creature());
    // ... nor for another spell.
    assert!(t
        .cast(P1, dismember)
        .target(Entity::Object(bears))
        .try_go()
        .is_err());
    assert!(t.in_hand(P1, "Dismember"));
    assert_eq!(t.g.players[1].mana_pool.mana.len(), 1);
    // It can pay for another mana ability of a land that player controls.
    let grotto = t.battlefield(P1, "Shimmering Grotto");
    assert!(t.activate(P1, grotto, 1, &[]).is_ok(), "{}", t.dump_log());
    let pool = &t.g.players[1].mana_pool.mana;
    assert_eq!(pool.len(), 1);
    assert!(pool[0].restriction.is_none());
    assert!(t.g.obj(grotto).tapped);
}
