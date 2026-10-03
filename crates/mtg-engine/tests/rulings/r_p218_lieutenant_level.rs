//! Rulings batch P218 — lieutenant (an ability word, CR 207.2c), level up (CR 702.87),
//! learn (CR 701.48) and living weapon (CR 702.92).

use crate::r_s01_common::{attack_with, supported};
use crate::r_s04_common::{activate_named, spell_targets};
use crate::r_s05_common::{enter, move_to, tokens_with_subtype};
use crate::r_s06_common::{damage, give_control};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts the real card `name` onto the battlefield as `p`'s commander.
fn commander(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.dirty = true;
    t.settle();
    id
}

#[test]
fn a_stolen_thunderfoot_baloth_checks_its_controllers_commander() {
    cr!("903.3", "611.3a", "109.5");
    ruling!(
        "Thunderfoot Baloth",
        "If you gain control of a creature with a lieutenant ability owned by another player, that ability will check to see if you control your commander and will apply if you do. It won't check whether its owner controls their commander."
    );
    supported("Thunderfoot Baloth");
    let mut t = TestGame::new(2);
    let baloth = t.battlefield(P1, "Thunderfoot Baloth");
    commander(&mut t, P1, "Hill Giant");
    assert_eq!(t.pt(baloth), (7, 7));
    give_control(&mut t, baloth, P0);
    assert_eq!(t.pt(baloth), (5, 5));
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
    commander(&mut t, P0, "Gray Ogre");
    assert_eq!(t.pt(baloth), (7, 7));
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Trample));
}

#[test]
fn thunderfoot_baloth_only_your_own_commander_counts() {
    cr!("903.3", "611.3a");
    ruling!(
        "Thunderfoot Baloth",
        "Lieutenant abilities refer only to whether you control your commander, not any other player's commander."
    );
    // P0 controls P1's commander: no bonus.
    let mut t = TestGame::new(2);
    let baloth = t.battlefield(P0, "Thunderfoot Baloth");
    let theirs = commander(&mut t, P1, "Hill Giant");
    give_control(&mut t, theirs, P0);
    assert_eq!(t.obj_now(theirs).controller, P0);
    assert_eq!(t.pt(baloth), (5, 5));
    assert_eq!(t.pt(theirs), (3, 3));
}

#[test]
fn thunderfoot_baloth_bonus_ends_at_once_when_the_commander_is_lost() {
    cr!("903.3", "704.5g", "611.3a");
    ruling!(
        "Thunderfoot Baloth",
        "If you lose control of your commander, lieutenant abilities of creatures you control will immediately stop applying. If this causes a creature's toughness to become less than or equal to the amount of damage marked on it, the creature will be destroyed."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thunderfoot Baloth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cmdr = commander(&mut t, P0, "Gray Ogre");
    assert_eq!(t.pt(bears), (4, 4));
    damage(&mut t, cmdr, 3, bears);
    assert!(t.on_battlefield(bears));
    give_control(&mut t, cmdr, P1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn siege_gang_lieutenant_triggers_once_with_several_commanders() {
    cr!("903.3", "603.4");
    ruling!(
        "Siege-Gang Lieutenant",
        "If you have multiple commanders, the lieutenant effect will happen as long as you control at least one commander. It will happen only once even if you control multiple commanders."
    );
    supported("Siege-Gang Lieutenant");
    for (on_battlefield, goblins) in [(2, 2), (1, 2), (0, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Siege-Gang Lieutenant");
        for (i, name) in ["Grizzly Bears", "Hill Giant"].into_iter().enumerate() {
            if i < on_battlefield {
                commander(&mut t, P0, name);
            } else {
                let c = t.command(P0, name);
                t.g.objects[c.0 as usize].is_commander = true;
            }
        }
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        t.resolve_all();
        assert_eq!(tokens_with_subtype(&t, P0, "Goblin").len(), goblins);
    }
}

#[test]
fn loyal_unicorn_effects_persist_after_losing_the_commander() {
    cr!("611.2a", "615.3", "903.3");
    ruling!(
        "Loyal Unicorn",
        "Once Loyal Unicorn’s lieutenant ability has resolved while you control your commander, its effects persist this turn even if you lose control of your commander or Loyal Unicorn."
    );
    supported("Loyal Unicorn");
    let mut t = TestGame::new(2);
    let unicorn = t.battlefield(P0, "Loyal Unicorn");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let cmdr = commander(&mut t, P0, "Gray Ogre");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Vigilance));
    // P0 loses both the commander and the Unicorn.
    move_to(&mut t, cmdr, Zone::Command);
    give_control(&mut t, unicorn, P1);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Vigilance));
    // The Bears attack and are blocked by the Hill Giant: no combat damage to them.
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!t.obj_now(bears).tapped);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(giant, bears)]);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn hexdrinker_level_abilities_count_level_counters() {
    cr!("711.2a", "711.2b", "702.87a");
    ruling!(
        "Hexdrinker",
        "Level abilities count the number of level counters on the creature. Hexdrinker starts with zero level counters."
    );
    supported("Hexdrinker");
    // (level counters, P/T, Lightning Bolt can target it, Flame Slash can target it)
    let cases = [
        (0, (2, 1), true, true),
        (2, (2, 1), true, true),
        (3, (4, 4), false, true),
        (7, (4, 4), false, true),
        (8, (6, 6), false, false),
        (11, (6, 6), false, false),
    ];
    for (n, pt, bolt, slash) in cases {
        let mut t = TestGame::new(2);
        let hex = t.battlefield(P1, "Hexdrinker");
        assert_eq!(t.obj_now(hex).counter(counters::LEVEL), 0);
        if n > 0 {
            t.g.add_counters(Entity::Object(hex), counters::LEVEL, n, None);
        }
        t.g.recompute();
        assert_eq!(t.pt(hex), pt, "level {n}");
        let h = Entity::Object(hex);
        assert_eq!(spell_targets(&mut t, P0, "Lightning Bolt").contains(&h), bolt);
        assert_eq!(spell_targets(&mut t, P0, "Flame Slash").contains(&h), slash);
        // It always has level up {1}.
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Forest", 1);
        activate_named(&mut t, P1, hex, "Level Up", 0).expect("level up");
        t.resolve();
        assert_eq!(t.obj_now(hex).counter(counters::LEVEL), n + 1);
    }
}

#[test]
fn retriever_phoenix_must_be_in_the_graveyard_as_you_are_told_to_learn() {
    cr!("614.1a", "701.48a", "614.12");
    ruling!(
        "Retriever Phoenix",
        "Retriever Phoenix must be in your graveyard at the moment you're instructed to learn if you want to use the last ability to return it to the battlefield."
    );
    supported("Retriever Phoenix");
    supported("Eyetwitch");
    // Eyetwitch ("When this creature dies, learn.") dies; the Phoenix reaches the
    // graveyard before the trigger resolves: it may return.
    let mut t = TestGame::new(2);
    let eye = t.battlefield(P0, "Eyetwitch");
    t.g.move_object(eye, Zone::Graveyard(P0), MoveCause::Destroy, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.graveyard(P0, "Retriever Phoenix");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Retriever Phoenix").len(), 1);
    // Leaving the graveyard before then (exiled in response): it can't.
    let mut t = TestGame::new(2);
    let phoenix = t.graveyard(P0, "Retriever Phoenix");
    let eye = t.battlefield(P0, "Eyetwitch");
    t.g.move_object(eye, Zone::Graveyard(P0), MoveCause::Destroy, None);
    t.settle();
    move_to(&mut t, phoenix, Zone::Exile);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.named_on_battlefield("Retriever Phoenix").is_empty());
    assert!(t.in_exile("Retriever Phoenix"));
}

#[test]
fn bonehoard_germ_dies_with_no_creature_cards_in_graveyards() {
    cr!("702.92a", "704.5f");
    ruling!(
        "Bonehoard",
        "If there are no creature cards in any graveyard when Bonehoard's living weapon ability resolves, the Germ will be 0/0 and put into its owner's graveyard."
    );
    supported("Bonehoard");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Forest");
    enter(&mut t, P0, "Bonehoard");
    t.resolve_all();
    assert!(tokens_with_subtype(&t, P0, "Germ").is_empty());
    // With creature cards in graveyards (any player's), it survives.
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    enter(&mut t, P0, "Bonehoard");
    t.resolve_all();
    let germs = tokens_with_subtype(&t, P0, "Germ");
    assert_eq!(germs.len(), 1);
    assert_eq!(t.pt(germs[0]), (2, 2));
}

#[test]
fn living_weapon_creates_phyrexian_germs() {
    cr!("702.92a");
    ruling!(
        "Nettlecyst",
        "The living weapon ability has been updated so that the triggered ability creates Phyrexian Germ tokens rather than Germ tokens."
    );
    supported("Nettlecyst");
    let mut t = TestGame::new(2);
    let cyst = enter(&mut t, P0, "Nettlecyst");
    t.resolve_all();
    let germs = tokens_with_subtype(&t, P0, "Germ");
    assert_eq!(germs.len(), 1);
    let germ = t.obj_now(germs[0]);
    assert!(germ.chars.has_subtype("Phyrexian"));
    // Equipped with Nettlecyst (an artifact): 0/0 +1/+1.
    assert_eq!(t.obj_now(cyst).attached_to, Some(Entity::Object(germs[0])));
    assert_eq!(t.pt(germs[0]), (1, 1));
}
