//! Rulings batch P096 — Chain Lightning's copies (CR 707.10, 707.10c), Equipment that
//! creates a creature token to attach itself to (CR 702.92a, 701.3), and abilities that
//! compare your life total with your starting life total or with an opponent's (CR 119.1,
//! 810.9), including in Two-Headed Giant and Commander games.

use crate::r_p096_common::*;
use crate::r_s01_common::{supported, triggers_on_stack, with_subtype};
use crate::r_s25_common::{cast_new, change_copy_targets, spell_copies};
use crate::r_s29_common::cast_and_resolve;
use crate::r_s33_common::{set_life, two_headed_giant};
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::game::GameConfig;
use mtg_engine::object::{FaceState, ObjKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Chain Lightning
// ---------------------------------------------------------------------------------------

#[test]
fn chain_lightning_the_target_may_pay_to_copy_it_with_a_new_target() {
    cr!("707.10", "707.10c", "707.10a", "608.2c");
    ruling!(
        "Chain Lightning",
        "As Chain Lightning resolves, either the original or one of the copies, the targeted player or the controller of the targeted permanent may copy it. The copy has the same text, target, and color as the resolving spell, though the player creating the copy may choose a new target for it. Once that copy is created (or not), the Chain Lightning finishes resolving and leaves the stack."
    );
    ruling!(
        "Chain Lightning",
        "The copy of Chain Lightning is created on the stack, so it's not cast. Abilities that trigger when a player casts a spell won't trigger. Players may respond to that spell before it resolves."
    );
    ruling!(
        "Chain Lightning",
        "The player putting the copy of the spell on the stack controls that copy."
    );
    supported("Chain Lightning");
    // P0 casts Chain Lightning at P1's Grizzly Bears. P1 pays {R}{R} and copies it,
    // targeting P0. P1's Young Pyromancer ("Whenever you cast an instant or sorcery spell,
    // create a 1/1 red Elemental creature token.") doesn't trigger for the copy.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pyromancer = t.battlefield(P1, "Young Pyromancer");
    t.lands(P1, "Mountain", 2);
    let original = cast_new(&mut t, P0, "Chain Lightning", &[Entity::Object(bears)]);
    t.answer_yes(P1, true); // pay {R}{R}
    t.answer_yes(P1, true); // copy it
    change_copy_targets(&mut t, P1, &[Some(Entity::Player(P0))]);
    let from = t.g.turn_events.len();
    t.resolve();
    // The original finished resolving and left the stack; the copy is on the stack,
    // controlled by P1, red, with the same text and the new target.
    assert!(t.in_graveyard(P0, "Chain Lightning"));
    assert!(!t.on_battlefield(bears));
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    let copy = t.g.obj(copies[0]);
    assert_eq!(copy.kind, ObjKind::SpellCopy);
    assert_eq!(copy.controller, P1);
    assert_eq!(copy.chars.name.as_str(), "Chain Lightning");
    assert_eq!(copy.chars.colors, ColorSet::single(Color::Red));
    assert_eq!(
        copy.chars.rules_text,
        t.g.obj(t.g.current(original)).chars.rules_text
    );
    assert_eq!(
        crate::r_s25_common::targets_of(&t, copies[0]),
        vec![Entity::Player(P0)]
    );
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 2);
    // Not cast.
    assert!(t.g.turn_events[from..]
        .iter()
        .all(|e| !matches!(e, Event::SpellCast { .. })));
    assert_eq!(triggers_from(&t, pyromancer), 0);
    // P0 may respond before it resolves: P0 gets priority with the copy on the stack, and
    // casts Renewed Faith ("You gain 6 life.") in response.
    t.settle();
    cast_new(&mut t, P0, "Renewed Faith", &[]);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P0), 26);
    // The copy resolves: P0 is the targeted player and may pay {R}{R} to copy it (P0
    // declines).
    t.lands(P0, "Mountain", 2);
    let asked = t.asked().len();
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.life(P0), 23);
    assert!(t.asked()[asked..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    assert!(t.g.stack.is_empty());
}

#[test]
fn chain_lightning_with_an_illegal_target_cant_be_copied() {
    cr!("608.2b", "707.10");
    ruling!(
        "Chain Lightning",
        "If the targeted player or permanent is an illegal target as Chain Lightning tries to resolve, the spell doesn't resolve and none of its effects happen. It can't be copied."
    );
    // P1 returns the targeted Bears to their hand with Unsummon in response: P1 isn't asked
    // to pay or copy anything.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Mountain", 2);
    cast_new(&mut t, P0, "Chain Lightning", &[Entity::Object(bears)]);
    cast_new(&mut t, P1, "Unsummon", &[Entity::Object(bears)]);
    t.resolve();
    let from = t.asked().len();
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .all(|(p, d)| !(*p == P1 && matches!(d, Decision::YesNo { .. }))));
    assert!(spell_copies(&t).is_empty());
    // (Only Unsummon's Island was tapped; the Mountains weren't.)
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 1);
}

// ---------------------------------------------------------------------------------------
// Equipment that creates its own creature
// ---------------------------------------------------------------------------------------

#[test]
fn doubling_season_two_tokens_the_equipment_attaches_to_one() {
    cr!("614.1a", "701.3a", "301.5c");
    ruling!(
        "Headsplitter",
        "If the triggered ability causes two Assassins to be created (due to an effect such as that of Doubling Season), Headsplitter becomes attached to one of them."
    );
    ruling!(
        "Ancestral Blade",
        "If the triggered ability causes two Soldiers to be created (due to an effect such as that of Doubling Season), Ancestral Blade becomes attached to one of them."
    );
    ruling!(
        "Field-Tested Frying Pan",
        "If the triggered ability causes more than one creature token to be created (due to an effect such as that of Doubling Season), Field-Tested Frying Pan becomes attached to one of them of your choice."
    );
    for (name, subtype) in [
        ("Headsplitter", "Assassin"),
        ("Ancestral Blade", "Soldier"),
        ("Field-Tested Frying Pan", "Halfling"),
    ] {
        supported(name);
        for pick in [0usize, 1] {
            let mut t = TestGame::new(2);
            t.battlefield(P0, "Doubling Season");
            let eq = crate::r_s05_common::enter(&mut t, P0, name);
            // P0 chooses which token (answered once the tokens exist).
            crate::r_s03_common::respond(
                &mut t,
                P0,
                if pick == 0 { first_token } else { last_token },
            );
            let from = t.asked().len();
            t.resolve_all();
            let made = with_subtype(&t, P0, subtype);
            assert_eq!(made.len(), 2, "{name}");
            let attached: Vec<ObjectId> = made
                .iter()
                .copied()
                .filter(|c| t.obj_now(eq).attached_to == Some(Entity::Object(*c)))
                .collect();
            assert_eq!(attached.len(), 1, "{name}");
            let asked = t.asked()[from..]
                .iter()
                .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. }))
                .count();
            assert_eq!(asked, 1, "{name}: P0 chooses the token");
            let mut sorted = made.clone();
            sorted.sort();
            assert_eq!(attached[0], sorted[pick], "{name}: the chosen one");
        }
    }
}

#[test]
fn living_weapon_with_doubling_season_equips_the_chosen_germ() {
    cr!("702.92a", "614.1a", "704.5f");
    // Flayer Husk (living weapon: "When this Equipment enters, create a 0/0 black
    // Phyrexian Germ creature token, then attach this to it."; "Equipped creature gets
    // +1/+1.") with Doubling Season: two Germs, P0 chooses which one to equip, and the
    // other, a 0/0, dies.
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Doubling Season");
        let husk = crate::r_s05_common::enter(&mut t, P0, "Flayer Husk");
        crate::r_s03_common::respond(&mut t, P0, if pick == 0 { first_token } else { last_token });
        let from = t.g.turn_events.len();
        t.resolve_all();
        let mut made: Vec<ObjectId> = t.g.turn_events[from..]
            .iter()
            .filter_map(|e| match e {
                Event::TokenCreated { obj, .. } => Some(*obj),
                _ => None,
            })
            .collect();
        made.sort();
        assert_eq!(made.len(), 2);
        let germs = with_subtype(&t, P0, "Germ");
        assert_eq!(germs, vec![made[pick]]);
        assert_eq!(
            t.obj_now(husk).attached_to,
            Some(Entity::Object(made[pick]))
        );
        assert_eq!(t.pt(made[pick]), (1, 1));
    }
}

/// Answers a choice among creature tokens with the one with the lowest id.
fn first_token(_g: &mtg_engine::game::Game, d: &Decision) -> Option<mtg_engine::decision::Answer> {
    match d {
        Decision::ChooseEntities { candidates, .. } if candidates.len() == 2 => {
            Some(mtg_engine::decision::Answer::Entities(vec![*candidates
                .iter()
                .min()?]))
        }
        _ => None,
    }
}

/// Answers a choice among creature tokens with the one with the highest id.
fn last_token(_g: &mtg_engine::game::Game, d: &Decision) -> Option<mtg_engine::decision::Answer> {
    match d {
        Decision::ChooseEntities { candidates, .. } if candidates.len() == 2 => {
            Some(mtg_engine::decision::Answer::Entities(vec![*candidates
                .iter()
                .max()?]))
        }
        _ => None,
    }
}

#[test]
fn field_tested_frying_pan_attaches_before_anyone_can_act() {
    cr!("608.2c", "117.2e", "603.3");
    ruling!(
        "Field-Tested Frying Pan",
        "No player may take any actions between the time you create the Halfling token and the time Field-Tested Frying Pan becomes attached to it."
    );
    // Soul Warden ("Whenever another creature enters, you gain 1 life.") sees the Halfling
    // enter, but its trigger waits until the Frying Pan's ability has finished resolving:
    // the Halfling is already equipped when Soul Warden's trigger is put on the stack, and
    // it has
    // the granted "Whenever you gain life, this creature gets +X/+X until end of turn"
    // when Soul Warden's trigger gains P0 1 life. No player gets priority during the
    // resolution.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    let pan = crate::r_s05_common::enter(&mut t, P0, "Field-Tested Frying Pan");
    let asked = t.asked().len();
    t.resolve();
    // Nobody got priority during the resolution.
    assert!(t.asked()[asked..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    let halfling = with_subtype(&t, P0, "Halfling")[0];
    assert_eq!(t.obj_now(pan).attached_to, Some(Entity::Object(halfling)));
    assert_eq!(triggers_on_stack(&t, "gain 1 life"), 1);
    t.resolve();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.pt(halfling), (2, 2));
}

#[test]
fn barbed_spike_creates_the_token_even_if_it_cant_attach() {
    cr!("701.3b", "608.2c");
    ruling!(
        "Barbed Spike",
        "The token will still be created even if Barbed Spike is not able to be attached to it (perhaps because it was destroyed in response to the ability)."
    );
    // "When this Equipment enters, create a 1/1 colorless Thopter artifact creature token
    // with flying, then attach this Equipment to it." P1 destroys it with Disenchant in
    // response.
    supported("Barbed Spike");
    let mut t = TestGame::new(2);
    let spike = crate::r_s05_common::enter(&mut t, P0, "Barbed Spike");
    cast_new(&mut t, P1, "Disenchant", &[Entity::Object(spike)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Barbed Spike"));
    t.resolve_all();
    let thopters = with_subtype(&t, P0, "Thopter");
    assert_eq!(thopters.len(), 1);
    assert_eq!(t.pt(thopters[0]), (1, 1));
}

#[test]
fn scytheclaw_trigger_resolves_after_combat_damage() {
    cr!("510.2", "603.3", "107.1a");
    ruling!(
        "Scytheclaw",
        "Scytheclaw's triggered ability triggers and resolves after combat damage is dealt. For example, if the Germ token deals 1 combat damage to a player with 10 life, combat damage will reduce that player's life total to 9. Then Scytheclaw's ability will cause the player to lose 5 life, leaving the player at 4."
    );
    supported("Scytheclaw");
    let mut t = TestGame::new(2);
    crate::r_s05_common::enter(&mut t, P0, "Scytheclaw");
    t.resolve_all();
    let germ = with_subtype(&t, P0, "Germ")[0];
    t.g.objects[germ.0 as usize].summoning_sick = false;
    assert_eq!(t.pt(germ), (1, 1));
    set_life(&mut t, P1, 10);
    declare(&mut t, &[germ]);
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::CombatDamage && !g.stack.is_empty()
    });
    assert!(ok);
    assert_eq!(t.life(P1), 9);
    t.resolve_all();
    assert_eq!(t.life(P1), 4);
}

// ---------------------------------------------------------------------------------------
// Life totals compared
// ---------------------------------------------------------------------------------------

#[test]
fn cosmos_elixir_compares_as_it_resolves_and_never_does_both() {
    cr!("608.2h", "119.1");
    ruling!(
        "Cosmos Elixir",
        "You compare your current life total to your starting life total as Cosmos Elixir’s ability resolves to determine whether you draw a card or gain 2 life. There are no situations in which you’ll do both."
    );
    supported("Cosmos Elixir");
    // "At the beginning of your end step, draw a card if your life total is greater than
    // your starting life total. Otherwise, you gain 2 life." In response to the trigger,
    // P0 goes from 20 to 26 (Renewed Faith), or from 23 to 17 (two Lightning Bolts by P1).
    for (life, respond, drew, life_after) in [
        (20, true, 1, 26),
        (23, true, 0, 19),
        (20, false, 0, 22),
        (21, false, 1, 21),
    ] {
        let mut t = TestGame::new(2);
        let elixir = t.battlefield(P0, "Cosmos Elixir");
        set_life(&mut t, P0, life);
        to_end_step(&mut t, P0);
        assert_eq!(triggers_from(&t, elixir), 1);
        if respond {
            if life == 20 {
                cast_new(&mut t, P0, "Renewed Faith", &[]);
                t.resolve();
            } else {
                cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
                t.resolve();
                cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
                t.resolve();
            }
        }
        let hand = t.hand_size(P0);
        let from = t.g.turn_events.len();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + drew, "{life} {respond}");
        assert_eq!(t.life(P0), life_after, "{life} {respond}");
        assert_eq!(
            gains_since(&t, from, P0).len(),
            1 - drew,
            "{life} {respond}"
        );
    }
}

#[test]
fn chalice_of_life_uses_the_starting_life_total_of_the_format() {
    cr!("119.1", "119.1a", "119.1c", "810.4");
    ruling!(
        "Chalice of Life // Chalice of Death",
        "Your starting life total is the life total you began the game with. For most two-player formats, this is 20. For Two-Headed Giant, it's the life total your team started with, usually 30. In Commander games, the starting life total is 40."
    );
    supported("Chalice of Life // Chalice of Death");
    // "{T}: You gain 1 life. Then if you have at least 10 life more than your starting life
    // total, transform this artifact."
    let games: [(&str, fn() -> TestGame, i32); 3] = [
        ("two-player", || TestGame::new(2), 30),
        ("Two-Headed Giant", two_headed_giant, 40),
        (
            "Commander",
            || TestGame::with_config(2, GameConfig::commander_game()),
            50,
        ),
    ];
    for (what, game, needed) in games {
        for (before, transforms) in [(needed - 2, false), (needed - 1, true)] {
            let mut t = game();
            let chalice = t.battlefield(P0, "Chalice of Life // Chalice of Death");
            set_team_life(&mut t, P0, before);
            t.activate(P0, chalice, 0, &[]).unwrap();
            t.resolve_all();
            assert_eq!(t.life(P0), before + 1);
            assert_eq!(
                t.obj_now(chalice).face == FaceState::Back,
                transforms,
                "{what} at {before}"
            );
        }
    }
}

#[test]
fn test_of_endurance_checks_as_upkeep_begins_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Test of Endurance",
        "Test of Endurance’s ability will check your life total as your upkeep begins, before any player has the opportunity to take any actions. If you don’t have 50 or more life at that time, the ability won’t trigger at all. If the ability does trigger, it will check again as it tries to resolve. If your life total has fallen below 50 at that point, the ability won’t have any effect."
    );
    // "At the beginning of your upkeep, if you have 50 or more life, you win the game."
    for (life, bolt, triggers, wins) in [
        (49, false, 0, false),
        (50, false, 1, true),
        (52, true, 1, false),
    ] {
        let mut t = TestGame::new(2);
        let test = t.battlefield(P0, "Test of Endurance");
        set_life(&mut t, P0, life);
        t.set_step(P0, Step::Untap);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(triggers_from(&t, test), triggers, "{life}");
        if bolt {
            cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
            t.resolve();
            assert_eq!(t.life(P0), 49);
        }
        t.resolve_all();
        assert_eq!(t.g.player(P0).has_won, wins, "{life}");
    }
}

#[test]
fn survival_cache_two_headed_giant_compares_team_life_totals() {
    cr!("810.9", "608.2h");
    ruling!(
        "Survival Cache",
        "In a Two-Headed Giant game, anything that cares about your life total checks the life total of your team. For example, say that your team has 10 life and your opponent’s team has 11 life when you cast Survival Cache. You’ll gain 2 life, so your team’s life total becomes 12. Since your life total (12) is greater than an opponent’s life total (11), you’ll draw a card."
    );
    supported("Survival Cache");
    // "You gain 2 life. Then if you have more life than an opponent, draw a card." From 9
    // life it's 11 against 11: no card.
    for (life, drew) in [(10, 1), (9, 0)] {
        let mut t = two_headed_giant();
        set_team_life(&mut t, P0, life);
        set_team_life(&mut t, P2, 11);
        let hand = t.hand_size(P0);
        cast_and_resolve(&mut t, P0, "Survival Cache", &[]);
        assert_eq!((t.life(P0), t.life(P1)), (life + 2, life + 2));
        assert_eq!(t.hand_size(P0), hand + drew, "{life}");
    }
}

#[test]
fn marina_vendrells_grimoire_no_maximum_hand_size_at_any_life_total() {
    cr!("402.2", "514.1");
    ruling!(
        "Marina Vendrell's Grimoire",
        "You have no maximum hand size regardless of what your life total is."
    );
    supported("Marina Vendrell's Grimoire");
    // At 20 life and at -5 life (the Grimoire keeps P0 in the game), P0 keeps ten cards
    // through the cleanup step.
    for life in [20, -5] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Marina Vendrell's Grimoire");
        set_life(&mut t, P0, life);
        for _ in 0..10 {
            t.hand(P0, "Grizzly Bears");
        }
        let hand = t.hand_size(P0);
        assert!(hand >= 10);
        t.advance_to(P1, Step::Upkeep);
        assert_eq!(t.hand_size(P0), hand, "{life}");
        assert!(!t.has_lost(P0));
    }
}
