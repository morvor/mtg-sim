//! Rulings batch P131 — regeneration shields (CR 701.19, 614.8): what they replace and what
//! they don't, when they must be set up, how long they last, and regeneration abilities
//! whose costs or conditions involve other objects.

use crate::r_p131_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// How a permanent is put into the graveyard (or would be), and whether a regeneration
/// shield saves it.
#[derive(Clone, Copy, Debug)]
enum Kill {
    Destroy,
    LethalDamage,
    Deathtouch,
    ZeroToughness,
    Sacrifice,
}

fn kill_with(t: &mut TestGame, id: ObjectId, how: Kill) {
    match how {
        Kill::Destroy => destroy(t, id),
        Kill::LethalDamage => {
            let giant = t.battlefield(P1, "Hill Giant");
            damage(t, giant, 10, id);
        }
        Kill::Deathtouch => {
            let rats = t.battlefield(P1, "Typhoid Rats");
            damage(t, rats, 1, id);
        }
        Kill::ZeroToughness => {
            let n = t.pt(id).1 as u32;
            put_counters(t, id, counters::MINUS1, n);
            t.settle();
        }
        Kill::Sacrifice => {
            let c = t.obj_now(id).controller;
            t.g.sacrifice(t.g.current(id), c);
            t.settle();
        }
    }
}

/// `name` (with a regeneration ability whose cost P0 can pay from Swamps, Forests and a
/// card in hand) gets a shield and then is killed `how`: whether it survived.
fn shielded_survives(name: &str, how: Kill) -> bool {
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 2);
    let fodder = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(fodder)]);
    shield(&mut t, P0, c);
    kill_with(&mut t, c, how);
    let ok = regenerated(&t, c);
    assert_eq!(ok, !t.in_graveyard(P0, name), "{name} {how:?}");
    ok
}

#[test]
fn regeneration_shields_replace_destruction_only() {
    cr!("701.19a", "614.8", "704.5f", "704.5g", "704.5h");
    ruling!(
        "Cudgel Troll",
        "This works only if Cudgel Troll is dealt lethal damage, dealt damage from a source with deathtouch, or affected by a spell or ability that says to “destroy” it."
    );
    ruling!(
        "Patchwork Gnomes",
        "This shield works against effects that try to destroy Patchwork Gnomes or lethal damage that would be dealt to Patchwork Gnomes."
    );
    ruling!(
        "Spiritmonger",
        "This shield works against effects that try to destroy Spiritmonger or lethal damage that would be dealt to Spiritmonger."
    );
    for name in ["Cudgel Troll", "Patchwork Gnomes", "Spiritmonger"] {
        supported(name);
        assert!(shielded_survives(name, Kill::Destroy), "{name}");
        assert!(shielded_survives(name, Kill::LethalDamage), "{name}");
    }
    assert!(shielded_survives("Cudgel Troll", Kill::Deathtouch));
    assert!(!shielded_survives("Cudgel Troll", Kill::ZeroToughness));
    assert!(!shielded_survives("Cudgel Troll", Kill::Sacrifice));
}

#[test]
fn an_unused_regeneration_shield_goes_away_as_the_turn_ends() {
    cr!("701.19a");
    ruling!(
        "Cudgel Troll",
        "If it hasn’t been used, the regeneration shield goes away as the turn ends."
    );
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Cudgel Troll");
    t.lands(P0, "Forest", 1);
    shield(&mut t, P0, troll);
    t.advance_to(P1, Step::Upkeep);
    destroy(&mut t, troll);
    assert!(t.in_graveyard(P0, "Cudgel Troll"));
}

#[test]
fn a_regeneration_shield_must_exist_before_the_destruction() {
    cr!("701.19a");
    ruling!(
        "Cudgel Troll",
        "To work, the regeneration shield must be created before Cudgel Troll is destroyed."
    );
    // In response to Murder: saved.
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Cudgel Troll");
    t.lands(P0, "Forest", 1);
    cast(&mut t, P1, "Murder", &[obj(troll)]);
    activate_containing(&mut t, P0, troll, "Regenerate").unwrap();
    t.resolve_all();
    assert!(regenerated(&t, troll));
    // After it's destroyed, it's too late: it's no longer on the battlefield.
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Cudgel Troll");
    t.lands(P0, "Forest", 1);
    destroy(&mut t, troll);
    assert!(t.in_graveyard(P0, "Cudgel Troll"));
    assert!(!can_activate(&mut t, P0, troll));
}

#[test]
fn regeneration_can_be_activated_when_not_at_risk() {
    cr!("701.19a");
    ruling!(
        "Patchwork Gnomes",
        "You can activate the regeneration ability even if Patchwork Gnomes isn't at risk of being destroyed."
    );
    ruling!(
        "Spiritmonger",
        "You can activate the regeneration ability even if Spiritmonger isn't at risk of being destroyed."
    );
    for name in ["Patchwork Gnomes", "Spiritmonger"] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        t.lands(P0, "Swamp", 1);
        let card = t.hand(P0, "Grizzly Bears");
        assert!(can_activate(&mut t, P0, c), "{name}");
        t.answer_choose(P0, &[obj(card)]);
        shield(&mut t, P0, c);
        // Untapped, undamaged and not in combat: nothing happens until it would be
        // destroyed later this turn.
        assert!(!t.obj(c).tapped, "{name}");
        destroy(&mut t, c);
        assert!(regenerated(&t, c), "{name}");
    }
}

#[test]
fn spiritmonger_becomes_only_the_chosen_color() {
    cr!("105.3");
    ruling!(
        "Spiritmonger",
        "Spiritmonger's last ability causes it to become only the chosen color. It will no longer be the colors it was previously."
    );
    let mut t = TestGame::new(2);
    let sm = t.battlefield(P0, "Spiritmonger");
    t.lands(P0, "Forest", 1);
    assert_eq!(
        t.obj(sm).chars.colors,
        [Color::Black, Color::Green]
            .into_iter()
            .collect::<ColorSet>()
    );
    t.answer(P0, DecisionKind::Any, Answer::Text("blue".into()));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    activate_containing(&mut t, P0, sm, "color of your choice").unwrap();
    t.resolve_all();
    let colors = t.obj(sm).chars.colors;
    assert_eq!(colors.count(), 1, "{colors:?}");
    assert!(!colors.contains(Color::Black) && !colors.contains(Color::Green));
}

#[test]
fn regenerating_a_planeswalker_doesnt_save_it_from_zero_loyalty() {
    cr!("701.19a", "704.5i");
    ruling!(
        "Reknit",
        "Regenerating a planeswalker will prevent it from being destroyed as a result of a “destroy” effect"
    );
    supported("Reknit");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    cast(&mut t, P0, "Reknit", &[obj(jace)]);
    t.resolve_all();
    destroy(&mut t, jace);
    assert!(t.g.is_live(jace) && t.on_battlefield(jace));
    // The shield would still be there for a second destruction, but loyalty 0 isn't one.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    cast(&mut t, P0, "Reknit", &[obj(jace)]);
    t.resolve_all();
    loyalty(&mut t, jace, 0);
    t.settle();
    assert!(t.in_graveyard(P0, "Jace Beleren"));
}

#[test]
fn draconian_cylix_discards_at_random_as_a_cost() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Draconian Cylix",
        "The card is discarded from your hand during announcement and as a cost."
    );
    supported("Draconian Cylix");
    let mut t = TestGame::new(2);
    let cylix = t.battlefield(P0, "Draconian Cylix");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    t.hand(P0, "Island");
    let from = t.asked().len();
    act(&mut t, P0, cylix, "Regenerate", &[obj(bears)]).unwrap();
    // Discarded already, with the ability still on the stack, and nobody chose the card.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Island"));
    assert!(t.asked()[from..].iter().all(|(_, d)| !matches!(
        d,
        mtg_engine::decision::Decision::ChooseEntities { prompt, .. } if prompt.contains("iscard")
    )));
    t.resolve_all();
    destroy(&mut t, bears);
    assert!(regenerated(&t, bears));
}

#[test]
fn vagrant_plowbeasts_checks_power_only_on_activation_and_resolution() {
    cr!("608.2b", "701.19a");
    ruling!(
        "Vagrant Plowbeasts",
        "This ability checks the targeted creature’s power only at the time the ability is activated and at the time it resolves."
    );
    supported("Vagrant Plowbeasts");
    // The power drops after the shield is created: still regenerated.
    let mut t = TestGame::new(2);
    let pb = t.battlefield(P0, "Vagrant Plowbeasts");
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, pb, "Regenerate", &[obj(pb)]).unwrap();
    t.resolve_all();
    put_counters(&mut t, pb, counters::MINUS1, 3);
    assert_eq!(t.pt(pb), (3, 3));
    destroy(&mut t, pb);
    assert!(regenerated(&t, pb));
    // The power drops before it resolves: an illegal target, no shield.
    let mut t = TestGame::new(2);
    let pb = t.battlefield(P0, "Vagrant Plowbeasts");
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, pb, "Regenerate", &[obj(pb)]).unwrap();
    put_counters(&mut t, pb, counters::MINUS1, 3);
    t.resolve_all();
    destroy(&mut t, pb);
    assert!(t.in_graveyard(P0, "Vagrant Plowbeasts"));
}

#[test]
fn trollhides_shield_stays_after_trollhide_leaves() {
    cr!("701.19a", "113.7a");
    ruling!(
        "Trollhide",
        "the regeneration shield will be created and remain in effect even if Trollhide is no longer attached to the creature."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hide = attach_new(&mut t, P0, "Trollhide", bears);
    t.lands(P0, "Forest", 2);
    activate_containing(&mut t, P0, bears, "Regenerate").unwrap();
    // Trollhide leaves with the ability on the stack.
    destroy(&mut t, hide);
    assert!(t.in_graveyard(P0, "Trollhide"));
    t.resolve_all();
    destroy(&mut t, bears);
    assert!(regenerated(&t, bears));
}

#[test]
fn skeletonize_gives_you_the_token_whoever_controlled_the_creature() {
    cr!("603.7d");
    ruling!(
        "Skeletonize",
        "You get the token created by Skeletonize's delayed triggered ability regardless of who controlled the creature."
    );
    for owner in [P1, P0] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(owner, "Grizzly Bears");
        cast(&mut t, P0, "Skeletonize", &[obj(bears)]);
        t.resolve_all();
        assert!(t.in_graveyard(owner, "Grizzly Bears"));
        assert_eq!(tokens_with(&t, P0, "Skeleton"), 1, "{owner}");
        assert_eq!(tokens_with(&t, P1, "Skeleton"), 0, "{owner}");
    }
}

#[test]
fn skitterskins_condition_is_checked_only_on_activation() {
    cr!("602.5", "113.7a", "701.19a");
    ruling!(
        "Skitterskin",
        "Once you legally activate the ability, it doesn't matter what happens to the other colorless creature."
    );
    supported("Skitterskin");
    let mut t = TestGame::new(2);
    let sk = t.battlefield(P0, "Skitterskin");
    t.lands(P0, "Swamp", 2);
    assert!(!can_activate(&mut t, P0, sk));
    let thopter = t.battlefield(P0, "Ornithopter");
    assert!(can_activate(&mut t, P0, sk));
    activate_containing(&mut t, P0, sk, "Regenerate").unwrap();
    destroy(&mut t, thopter);
    t.resolve_all();
    destroy(&mut t, sk);
    assert!(regenerated(&t, sk));
}

#[test]
fn skitterskin_needs_another_colorless_creature_not_to_be_colorless() {
    cr!("602.5", "105.3");
    ruling!(
        "Skitterskin",
        "Skitterskin's activated ability doesn't depend on Skitterskin being colorless."
    );
    let mut t = TestGame::new(2);
    let sk = t.battlefield(P0, "Skitterskin");
    t.lands(P0, "Swamp", 2);
    cast(&mut t, P0, "Cerulean Wisps", &[obj(sk)]);
    t.resolve_all();
    assert!(t.obj(sk).chars.colors.contains(Color::Blue));
    // Skitterskin itself, now blue, doesn't count; another colorless creature does.
    assert!(!can_activate(&mut t, P0, sk));
    t.battlefield(P0, "Ornithopter");
    assert!(can_activate(&mut t, P0, sk));
    shield(&mut t, P0, sk);
    destroy(&mut t, sk);
    assert!(regenerated(&t, sk));
}

#[test]
fn kjeldoran_deads_sacrifice_is_a_triggered_ability() {
    cr!("603.3", "701.21a");
    ruling!(
        "Kjeldoran Dead",
        "The sacrifice is a triggered ability and not a cost."
    );
    ruling!(
        "Kjeldoran Dead",
        "When Kjeldoran Dead's \"enters\" ability resolves, if you don't control any other creatures, you must sacrifice Kjeldoran Dead itself."
    );
    supported("Kjeldoran Dead");
    // With another creature: cast without sacrificing anything; the trigger asks later.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dead = cast(&mut t, P0, "Kjeldoran Dead", &[]);
    assert!(t.on_battlefield(bears));
    t.resolve();
    assert!(t.on_battlefield(dead));
    assert!(t.stack_len() == 1 && is_trigger(&t, top(&t)));
    assert!(t.on_battlefield(bears));
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(dead));
    // Alone: it sacrifices itself.
    let mut t = TestGame::new(2);
    cast(&mut t, P0, "Kjeldoran Dead", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Kjeldoran Dead"));
}

#[test]
fn wake_of_vultures_sacrificed_to_its_own_ability_isnt_regenerated() {
    cr!("602.2b", "701.19a");
    ruling!(
        "Wake of Vultures",
        "You can sacrifice Wake of Vultures to activate its own ability. The ability will have no effect as it resolves."
    );
    supported("Wake of Vultures");
    let mut t = TestGame::new(2);
    let wake = t.battlefield(P0, "Wake of Vultures");
    t.lands(P0, "Swamp", 2);
    t.answer_choose(P0, &[obj(wake)]);
    activate_containing(&mut t, P0, wake, "Regenerate").unwrap();
    assert!(t.in_graveyard(P0, "Wake of Vultures"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Wake of Vultures"));
    assert!(t.named_on_battlefield("Wake of Vultures").is_empty());
}

#[test]
fn ink_eyes_can_take_a_creature_that_died_in_the_same_combat_damage_step() {
    cr!("510.2", "603.3d", "704.5g");
    ruling!(
        "Ink-Eyes, Servant of Oni",
        "If, during the combat damage step, a creature is destroyed during combat and Ink-Eyes also deals combat damage to a player, the destroyed creature can be a legal target for Ink-Eyes's ability."
    );
    supported("Ink-Eyes, Servant of Oni");
    let mut t = TestGame::new(2);
    let ink = t.battlefield(P0, "Ink-Eyes, Servant of Oni");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    attack_with(
        &mut t,
        &[(ink, Entity::Player(P1)), (mine, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(theirs, mine)]);
    t.resolve_all();
    // The blocker that died in the same step was offered as a target and put onto the
    // battlefield under P0's control.
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].len(), 1);
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.g.obj(bears[0]).owner, P1);
    assert_eq!(t.g.obj(bears[0]).controller, P0);
    assert_eq!(t.life(P1), 15);
}
