//! Rulings batch P159 — conditions and counts checked as a spell or ability resolves
//! (CR 608.2h): "instead" riders that look at what you control, numbers counted once, and
//! "creatures you control get ..." effects that lock in their set of creatures
//! (CR 611.2c).

use crate::r_p044_common::{cast_only, cast_spell, kill};
use crate::r_p046_common::{fill_pool, pool, tap_lands};
use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn hand_card(t: &mut TestGame, p: PlayerId) -> ObjectId {
    t.hand(p, "Grizzly Bears")
}

#[test]
fn instead_riders_check_what_you_control_as_they_resolve() {
    cr!("608.2h", "614.15", "611.2c");
    ruling!(
        "Gangrenous Zombies",
        "Whether it does 1 or 2 damage is checked on resolution."
    );
    ruling!(
        "Huatli's Spurring",
        "Whether you control a Huatli planeswalker is checked only as Huatli’s Spurring resolves."
    );
    ruling!(
        "Festering Newt",
        "Whether you control a creature named Bogbrew Witch is checked only as the triggered ability resolves."
    );
    ruling!(
        "Jace's Triumph",
        "Whether you control an appropriate planeswalker is checked only as the spell resolves."
    );
    ruling!(
        "Chandra's Triumph",
        "Whether you control an appropriate planeswalker is checked only as the spell resolves."
    );
    ruling!(
        "Liliana's Triumph",
        "Whether you control an appropriate planeswalker is checked only as the spell resolves."
    );
    ruling!(
        "Mine Worker",
        "You will gain 3 life instead of 1 if, at the time the ability resolves, you control at least one creature named Power Plant Worker and at least one creature named Tower Worker."
    );
    ruling!(
        "Zimone, Quandrix Prodigy",
        "Zimone's last ability counts the number of lands you control as it resolves, not as you activate it."
    );
    for c in [
        "Gangrenous Zombies",
        "Huatli's Spurring",
        "Festering Newt",
        "Bogbrew Witch",
        "Jace's Triumph",
        "Chandra's Triumph",
        "Liliana's Triumph",
        "Mine Worker",
        "Zimone, Quandrix Prodigy",
    ] {
        supported(c);
    }

    // Gangrenous Zombies: a snow Swamp arrives after activation: 2 damage.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Gangrenous Zombies");
    activate_containing(&mut t, P0, z, "Sacrifice").expect("activate");
    t.battlefield(P0, "Snow-Covered Swamp");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (18, 18));

    // Huatli's Spurring: Huatli arrives in response: +4/+0, which stays after she leaves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_only(&mut t, P0, "Huatli's Spurring", &[obj(bears)]);
    let huatli = t.battlefield(P0, "Huatli, Warrior Poet");
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 2));
    move_to(&mut t, huatli, Zone::Graveyard(P0));
    assert_eq!(t.pt(bears), (6, 2));

    // Festering Newt: Bogbrew Witch arrives before the trigger resolves: -4/-4, which
    // stays after the Witch leaves.
    let mut t = TestGame::new(2);
    let newt = t.battlefield(P0, "Festering Newt");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    t.answer_targets(P0, &[obj(dragon)]);
    kill(&mut t, newt);
    assert_eq!(t.stack_len(), 1);
    let witch = t.battlefield(P0, "Bogbrew Witch");
    t.resolve_all();
    assert_eq!(t.pt(dragon), (1, 1));
    move_to(&mut t, witch, Zone::Graveyard(P0));
    assert_eq!(t.pt(dragon), (1, 1));

    // Jace's Triumph: Jace arrives in response: three cards.
    let mut t = TestGame::new(2);
    cast_only(&mut t, P0, "Jace's Triumph", &[]);
    let hand = t.hand_size(P0);
    t.battlefield(P0, "Jace Beleren");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);

    // Chandra's Triumph: Chandra leaves in response: 3 damage, not 5.
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Torch of Defiance");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    cast_only(&mut t, P0, "Chandra's Triumph", &[obj(dragon)]);
    move_to(&mut t, chandra, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(dragon));
    assert_eq!(crate::r_s29_common::damage_marked(&t, dragon), 3);

    // Liliana's Triumph: Liliana arrives in response: the opponent also discards.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    hand_card(&mut t, P1);
    cast_only(&mut t, P0, "Liliana's Triumph", &[]);
    t.battlefield(P0, "Liliana of the Veil");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());

    // Mine Worker: Tower Worker arrives after activation (Power Plant Worker was there):
    // 3 life.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Mine Worker");
    t.battlefield(P0, "Power Plant Worker");
    activate_containing(&mut t, P0, mine, "gain").expect("activate");
    t.battlefield(P0, "Tower Worker");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);

    // Zimone: the eighth land arrives after activation: two cards.
    let mut t = TestGame::new(2);
    let zimone = t.battlefield(P0, "Zimone, Quandrix Prodigy");
    t.lands(P0, "Forest", 7);
    activate_containing(&mut t, P0, zimone, "Draw").expect("activate");
    t.battlefield(P0, "Island");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn tower_worker_adds_three_with_both_other_workers() {
    cr!("605.3b", "106.4");
    ruling!(
        "Tower Worker",
        "You will add {C}{C}{C} instead of {C} if you control at least one creature named Mine Worker and at least one creature named Power Plant Worker."
    );
    supported("Tower Worker");
    for (others, mana) in [
        (&["Mine Worker", "Power Plant Worker"][..], 3),
        (&["Mine Worker"][..], 1),
        (&["Power Plant Worker"][..], 1),
    ] {
        let mut t = TestGame::new(2);
        let tower = t.battlefield(P0, "Tower Worker");
        for o in others {
            t.battlefield(P0, o);
        }
        assert!(crate::r_s20_common::tap_for_mana(
            &mut t, P0, tower, "Add"
        ));
        assert_eq!(pool(&t, P0), mana, "{others:?}");
    }
}

#[test]
fn life_goes_on_and_dega_sanctuary_never_gain_more() {
    cr!("614.15", "608.2c");
    ruling!(
        "Life Goes On",
        "You won't gain more than 8 life if more than one creature died this turn."
    );
    ruling!("Dega Sanctuary", "You can gain 2 or 4 life, but never 6.");
    supported("Life Goes On");
    supported("Dega Sanctuary");
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        let b = t.battlefield(P1, "Grizzly Bears");
        kill(&mut t, b);
    }
    cast_spell(&mut t, P0, "Life Goes On", &[]);
    assert_eq!(t.life(P0), 28);
    // Dega Sanctuary: a black and a red permanent: 4; a black one only: 2; a permanent
    // that's both black and red (Rakdos Cackler): 4.
    for (perms, life) in [
        (&["Typhoid Rats", "Raging Goblin"][..], 24),
        (&["Typhoid Rats"][..], 22),
        (&["Rakdos Cackler"][..], 24),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dega Sanctuary");
        for p in perms {
            t.battlefield(P0, p);
        }
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.life(P0), life, "{perms:?}");
    }
}

#[test]
fn zimones_land_ability_isnt_playing_a_land() {
    cr!("305.4", "305.2");
    ruling!(
        "Zimone, Quandrix Prodigy",
        "Zimone's first ability isn't the same as playing a land."
    );
    // On P1's turn, with P0's land play for P0's own turn irrelevant: the land enters
    // tapped, and no land play is counted.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let zimone = t.battlefield(P0, "Zimone, Quandrix Prodigy");
    t.lands(P0, "Wastes", 1);
    let land = t.hand(P0, "Forest");
    activate_containing(&mut t, P0, zimone, "land card").expect("activate");
    t.answer_choose(P0, &[obj(land)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(land));
    assert!(t.obj_now(land).tapped);
    assert_eq!(t.g.history.lands_played.get(&P0).copied().unwrap_or(0), 0);
    // On P0's turn, after P0 played a land: still allowed.
    let mut t = TestGame::new(2);
    let zimone = t.battlefield(P0, "Zimone, Quandrix Prodigy");
    t.lands(P0, "Wastes", 1);
    let first = t.hand(P0, "Island");
    t.play_land(P0, first).expect("land play");
    let land = t.hand(P0, "Forest");
    activate_containing(&mut t, P0, zimone, "land card").expect("activate");
    t.answer_choose(P0, &[obj(land)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(land));
}

#[test]
fn pumps_count_and_choose_their_creatures_once() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Elder of Laurels",
        "Once the ability has resolved, the bonus won’t change if the number of creatures you control changes later in the turn."
    );
    ruling!(
        "Kamahl, Fist of Krosa",
        "Kamahl’s last ability affects only creatures you control as it resolves."
    );
    ruling!(
        "Ezuri, Renegade Leader",
        "Only Elf creatures you control when the last ability resolves will get +3/+3 and gain trample until end of turn."
    );
    ruling!(
        "Squirrel Wrangler",
        "Only affects Squirrels on the battlefield when the ability resolves."
    );
    for c in [
        "Elder of Laurels",
        "Kamahl, Fist of Krosa",
        "Ezuri, Renegade Leader",
        "Squirrel Wrangler",
    ] {
        supported(c);
    }
    // Elder of Laurels: two creatures as it resolves: +2/+2, even after a third arrives.
    let mut t = TestGame::new(2);
    let elder = t.battlefield(P0, "Elder of Laurels");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, elder, "+X/+X").expect("activate");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(bears), (4, 4));

    // Kamahl: a land animated afterwards is a 1/1.
    let mut t = TestGame::new(2);
    let kamahl = t.battlefield(P0, "Kamahl, Fist of Krosa");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 6);
    let target_land = t.battlefield(P0, "Plains");
    activate_containing(&mut t, P0, kamahl, "+3/+3").expect("activate");
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(kamahl), (7, 6));
    t.answer_targets(P0, &[obj(target_land)]);
    activate_containing(&mut t, P0, kamahl, "becomes").expect("activate");
    t.resolve_all();
    assert!(t.obj_now(target_land).is(CardType::Creature));
    assert_eq!(t.pt(target_land), (1, 1));

    // Ezuri: an Elf that enters afterwards doesn't get the bonus; Ezuri itself does.
    let mut t = TestGame::new(2);
    let ezuri = t.battlefield(P0, "Ezuri, Renegade Leader");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 5);
    activate_containing(&mut t, P0, ezuri, "+3/+3").expect("activate");
    t.resolve_all();
    assert_eq!((t.pt(ezuri), t.pt(elf)), ((5, 5), (4, 4)));
    let late = t.battlefield(P0, "Elvish Mystic");
    assert_eq!(t.pt(late), (1, 1));

    // Squirrel Wrangler: the pump, then the tokens: the tokens are 1/1.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Squirrel Wrangler");
    t.lands(P0, "Forest", 6);
    activate_containing(&mut t, P0, w, "Squirrel creatures get").expect("activate");
    t.resolve_all();
    activate_containing(&mut t, P0, w, "Create").expect("activate");
    t.resolve_all();
    let squirrels = with_subtype(&t, P0, "Squirrel");
    assert_eq!(squirrels.len(), 2);
    for s in squirrels {
        assert_eq!(t.pt(s), (1, 1));
    }
}

#[test]
fn ezuris_regeneration_cant_target_itself() {
    cr!("115.4", "115.1");
    ruling!(
        "Ezuri, Renegade Leader",
        "Ezuri's first ability can't affect itself, but its second ability does."
    );
    let mut t = TestGame::new(2);
    let ezuri = t.battlefield(P0, "Ezuri, Renegade Leader");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 1);
    let from = t.asked().len();
    activate_containing(&mut t, P0, ezuri, "Regenerate").expect("activate");
    let cands: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(cands, vec![vec![obj(elf)]]);
}

#[test]
fn self_counting_pumps_count_themselves() {
    cr!("611.2c", "205.4g");
    ruling!(
        "Conifer Wurm",
        "Because Conifer Wurm is itself a snow permanent, its ability will usually give it at least +1/+1."
    );
    ruling!(
        "Creeping Trailblazer",
        "Because Creeping Trailblazer is itself an Elemental, its last ability will usually give it at least +1/+1."
    );
    for (card, needle, before, after) in [
        ("Conifer Wurm", "+X/+X", (4, 4), (5, 5)),
        ("Creeping Trailblazer", "for each Elemental", (2, 2), (3, 3)),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, card);
        tap_lands(&mut t, P0);
        fill_pool(&mut t, P0, 4);
        assert_eq!(t.pt(c), before);
        activate_containing(&mut t, P0, c, needle).expect("activate");
        t.resolve_all();
        assert_eq!(t.pt(c), after, "{card}");
    }
}

#[test]
fn pallid_mycoderm_gives_a_fungus_saproling_one_bonus() {
    cr!("611.2c", "205.3");
    ruling!(
        "Pallid Mycoderm",
        "A creature that's both a Fungus and a Saproling gets just +1/+1 from Pallid Mycoderm's last ability."
    );
    supported("Pallid Mycoderm");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Pallid Mycoderm");
    let both = t.custom(
        P0,
        custom_card("Spore Twin", "Creature — Fungus Saproling", "{G}", Some((1, 1)), ""),
        Zone::Battlefield,
    );
    let fodder = t.custom(
        P0,
        custom_card("Sap Fodder", "Creature — Saproling", "{G}", Some((1, 1)), ""),
        Zone::Battlefield,
    );
    t.answer_choose(P0, &[obj(fodder)]);
    activate_containing(&mut t, P0, m, "Sacrifice a Saproling").expect("activate");
    t.resolve_all();
    assert!(!t.on_battlefield(fodder));
    assert_eq!(t.pt(both), (2, 2));
    assert_eq!(t.pt(m), (3, 5));
}

#[test]
fn heroes_bane_with_negative_power_adds_no_counters() {
    cr!("107.1b");
    ruling!(
        "Heroes' Bane",
        "If Heroes' Bane's power is less than 0 when its second ability resolves, X is considered to be 0."
    );
    supported("Heroes' Bane");
    let mut t = TestGame::new(2);
    let bane = t.enter(P0, "Heroes' Bane");
    assert_eq!(t.counters(bane, "+1/+1"), 4);
    cast_spell(&mut t, P0, "Disorient", &[obj(bane)]);
    assert_eq!(t.pt(bane), (-3, 4));
    t.lands(P0, "Forest", 4);
    activate_containing(&mut t, P0, bane, "Put X").expect("activate");
    t.resolve_all();
    assert_eq!(t.counters(bane, "+1/+1"), 4);
    assert_eq!(t.pt(bane), (-3, 4));
}
