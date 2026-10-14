//! Rulings batch P177 — assorted steal effects: "threaten" effects that may target an
//! untapped creature or one you already control, Treachery, Threads of Disloyalty,
//! Inevitable Betrayal, Illusory Gains, In Bolas's Clutches, Confiscate, Mass Mutiny with
//! umbra armor, Sarkhan Vol, The Nipton Lottery, Bond of Passion, Dominating Licid,
//! Varchild, Swooping Pteranodon, Soul Ransom and Enslave.

use crate::r_p177_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// How each card steals `target` for P0.
fn steal_with(t: &mut TestGame, name: &str, target: ObjectId) {
    supported(name);
    match name {
        "Smelt-Ward Ignus" | "Captivating Crew" => {
            let src = t.battlefield(P0, name);
            give_mana_for(t, P0, if name == "Smelt-Ward Ignus" { "Act of Treason" } else { "Captivating Crew" });
            t.activate(P0, src, 0, &[obj(target)]).unwrap();
            t.resolve_all();
        }
        "Enthralling Victor" => {
            t.answer_targets(P0, &[obj(target)]);
            t.enter(P0, name);
            t.resolve_all();
        }
        _ => {
            cast_resolve(t, P0, name, &[obj(target)]);
        }
    }
}

#[test]
fn threaten_effects_can_target_untapped_creatures_and_your_own() {
    cr!("115.1", "702.10b");
    ruling!(
        "Act of Treason",
        "Act of Treason can target any creature, even one that's untapped or one you already control."
    );
    ruling!(
        "Claim the Firstborn",
        "Claim the Firstborn can target any creature with mana value 3 or less, even one that's untapped or one you already control."
    );
    ruling!(
        "Mark of Mutiny",
        "Mark of Mutiny can target any creature, even one that you already control or that is already untapped."
    );
    ruling!(
        "Smelt-Ward Ignus",
        "Smelt-Ward Ignus’s ability can target any creature with power 3 or less, even one that’s untapped or one you already control."
    );
    ruling!(
        "Hijack",
        "You can target and gain control of an untapped artifact or creature this way. You can also untap an artifact or creature you already control and give it haste."
    );
    ruling!(
        "Enthralling Victor",
        "Enthralling Victor's ability can target a creature that's already untapped. You'll still gain control of it and it will gain haste."
    );
    ruling!(
        "Captivating Crew",
        "You can target and gain control of an untapped creature this way."
    );
    for (name, own) in [
        ("Act of Treason", true),
        ("Claim the Firstborn", true),
        ("Mark of Mutiny", true),
        ("Smelt-Ward Ignus", true),
        ("Hijack", true),
        ("Enthralling Victor", false),
        ("Captivating Crew", false),
    ] {
        // An opponent's untapped creature.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        steal_with(&mut t, name, bears);
        assert_eq!(controller(&mut t, bears), P0, "{name}");
        assert!(!tapped(&t, bears), "{name}");
        assert!(
            t.obj_now(bears).has_keyword(KeywordKind::Haste),
            "{name}"
        );
        if own {
            // A tapped, summoning-sick creature P0 already controls: untapped, with haste.
            let mut t = TestGame::new(2);
            let mine = t.battlefield_sick(P0, "Grizzly Bears");
            t.g.tap(mine);
            steal_with(&mut t, name, mine);
            assert_eq!(controller(&mut t, mine), P0, "{name}");
            assert!(!tapped(&t, mine), "{name}");
            t.set_step(P0, Step::BeginningOfCombat);
            assert!(can_attack(&mut t, mine), "{name}");
        }
    }
    // Hijack can take an artifact.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    t.g.tap(ring);
    steal_with(&mut t, "Hijack", ring);
    assert_eq!(controller(&mut t, ring), P0);
    assert!(!tapped(&t, ring));
}

#[test]
fn treachery_untaps_zero_to_five_lands() {
    cr!("107.1c");
    ruling!("Treachery", "You can untap 0 to 5 lands.");
    supported("Treachery");
    for n in [0usize, 3, 5] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        // Five Islands pay for Treachery; P0 untaps n of them.
        let islands = t.lands(P0, "Island", 5);
        let pick: Vec<_> = islands[..n].iter().map(|i| obj(*i)).collect();
        let treachery = t.hand(P0, "Treachery");
        t.cast_with(P0, treachery, &[obj(bears)]).unwrap();
        assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
        t.answer_choose(P0, &pick);
        t.resolve_all();
        assert_eq!(controller(&mut t, bears), P0);
        assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), n, "{n}");
    }
}

#[test]
fn threads_of_disloyalty_can_take_a_token() {
    cr!("111.6", "202.3");
    ruling!("Threads of Disloyalty", "Tokens usually have a mana value of 0.");
    supported("Threads of Disloyalty");
    let mut t = TestGame::new(2);
    let token = crate::r_s02_common::create_token(&mut t, P1, "Dragon");
    assert_eq!(t.obj_now(token).chars.mana_value(), 0);
    cast_resolve(&mut t, P0, "Threads of Disloyalty", &[obj(token)]);
    assert_eq!(controller(&mut t, token), P0);
}

#[test]
fn inevitable_betrayal_may_fail_to_find() {
    cr!("701.23b");
    ruling!(
        "Inevitable Betrayal",
        "You may fail to find a creature card in your opponent's library."
    );
    supported("Inevitable Betrayal");
    let mut t = TestGame::new(2);
    let bears = t.library_top(P1, "Grizzly Bears");
    // P1 draws an Island first.
    t.library_top(P1, "Island");
    // Suspended with one time counter; cast at P0's upkeep.
    let betrayal = t.exile(P0, "Inevitable Betrayal");
    t.g.add_counters(obj(betrayal), mtg_engine::types::counters::TIME, 1, None);
    t.g.recompute();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[]);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Inevitable Betrayal"), "it was cast");
    assert_eq!(t.zone(bears), Zone::Library(P1));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn illusory_gains_moves_onto_a_hexproof_creature() {
    cr!("303.4", "702.11b");
    ruling!(
        "Illusory Gains",
        "The last ability of Illusory Gains doesn't target the new creature. It will become attached to that creature even if that creature has hexproof or shroud."
    );
    supported("Illusory Gains");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gains = attach_new(&mut t, P0, "Illusory Gains", bears);
    assert_eq!(controller(&mut t, bears), P0);
    let scout = t.enter(P1, "Gladecover Scout");
    t.resolve_all();
    assert_eq!(attached_to(&t, gains), Some(obj(scout)));
    assert_eq!(controller(&mut t, scout), P0);
    assert_eq!(controller(&mut t, bears), P1);
}

#[test]
fn in_bolass_clutches_doesnt_take_auras_or_equipment() {
    cr!("303.4e", "301.5d");
    ruling!(
        "In Bolas's Clutches",
        "Gaining control of a permanent doesn’t cause you to gain control of any Auras or Equipment attached to it."
    );
    supported("In Bolas's Clutches");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let splitter = attach_new(&mut t, P1, "Bonesplitter", bears);
    let strength = attach_new(&mut t, P1, "Holy Strength", bears);
    cast_resolve(&mut t, P0, "In Bolas's Clutches", &[obj(bears)]);
    assert_eq!(controller(&mut t, bears), P0);
    assert_eq!(controller(&mut t, splitter), P1);
    assert_eq!(controller(&mut t, strength), P1);
    assert_eq!(attached_to(&t, splitter), Some(obj(bears)));
    assert_eq!(attached_to(&t, strength), Some(obj(bears)));
    assert_eq!(t.pt(bears), (5, 4));
}

#[test]
fn confiscating_an_aura_doesnt_move_it() {
    cr!("303.4e");
    ruling!(
        "Confiscate",
        "Gaining control of an Aura or Equipment doesn't change what it's attached to."
    );
    supported("Confiscate");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let strength = attach_new(&mut t, P1, "Holy Strength", bears);
    cast_resolve(&mut t, P0, "Confiscate", &[obj(strength)]);
    assert_eq!(controller(&mut t, strength), P0);
    assert_eq!(controller(&mut t, bears), P1);
    assert_eq!(attached_to(&t, strength), Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 4));
}

#[test]
fn mass_mutiny_a_stolen_creatures_umbra_armor_still_protects_it() {
    cr!("702.89a", "301.5d");
    ruling!(
        "Mass Mutiny",
        "if you gain control of a creature enchanted by an Aura with umbra armor and that creature would be destroyed that turn, instead the Aura will be destroyed and the creature will survive."
    );
    supported("Mass Mutiny");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let umbra = attach_new(&mut t, P1, "Hyena Umbra", bears);
    cast_resolve(&mut t, P0, "Mass Mutiny", &[obj(bears)]);
    assert_eq!(controller(&mut t, bears), P0);
    assert_eq!(controller(&mut t, umbra), P1);
    destroy(&mut t, bears);
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(umbra));
    assert!(t.in_graveyard(P1, "Hyena Umbra"));
}

#[test]
fn sarkhan_vols_plus_one_affects_only_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Sarkhan Vol",
        "Sarkhan Vol’s first ability affects only creatures you control when that ability resolves."
    );
    supported("Sarkhan Vol");
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan Vol");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, sarkhan, 0, &[]).unwrap();
    t.resolve_all();
    let giant = t.battlefield_sick(P0, "Hill Giant");
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.pt(giant), (3, 3));
    assert!(!t.obj_now(giant).has_keyword(KeywordKind::Haste));
}

#[test]
fn the_nipton_lottery_steals_one_creature_and_destroys_the_rest_at_once() {
    cr!("608.2c");
    ruling!(
        "The Nipton Lottery",
        "Players can’t take actions between the time the creature is randomly chosen and the time at which the rest of the effects occur."
    );
    supported("The Nipton Lottery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Serra Angel");
    cast_targeting(&mut t, P0, "The Nipton Lottery", &[]);
    t.g.resolve_top();
    let left: Vec<_> = t
        .g
        .permanents()
        .filter(|o| o.is(CardType::Creature))
        .map(|o| (o.id, o.controller))
        .collect();
    assert_eq!(left.len(), 1, "{left:?}");
    assert_eq!(left[0].1, P0);
    assert!(t.obj_now(left[0].0).has_keyword(KeywordKind::Haste));
}

#[test]
fn bond_of_passion_still_damages_a_target_that_became_illegal_mid_resolution() {
    cr!("608.2b");
    ruling!(
        "Bond of Passion",
        "As long as the second target is a legal target as Bond of Passion begins to resolve (before the target creature changes control) it will attempt to deal damage to the second target."
    );
    supported("Bond of Passion");
    // True Believer: "You have shroud." P0 takes it and targets themself with the damage.
    let mut t = TestGame::new(2);
    let believer = t.battlefield(P1, "True Believer");
    cast_resolve(
        &mut t,
        P0,
        "Bond of Passion",
        &[obj(believer), Entity::Player(P0)],
    );
    assert_eq!(controller(&mut t, believer), P0);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn dominating_licid_becomes_only_an_aura() {
    cr!("205.1a", "613.1d");
    ruling!(
        "Dominating Licid",
        "It becomes an Enchantment - Aura and stops being all other permanent types."
    );
    supported("Dominating Licid");
    let mut t = TestGame::new(2);
    let licid = t.battlefield(P0, "Dominating Licid");
    t.lands(P0, "Island", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, licid, 0, &[obj(bears)]).unwrap();
    t.answer_yes(P0, false);
    t.resolve_all();
    let o = t.obj_now(licid);
    assert!(o.is(CardType::Enchantment));
    assert!(!o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Aura"));
    assert_eq!(attached_to(&t, licid), Some(obj(bears)));
    assert_eq!(controller(&mut t, bears), P0);
}

/// Varchild deals combat damage to P1 (P0's turn): P1 creates that many Survivors.
fn varchild_hits(t: &mut TestGame) -> ObjectId {
    supported("Varchild, Betrayer of Kjeldor");
    let varchild = t.battlefield(P0, "Varchild, Betrayer of Kjeldor");
    t.attack(&[(varchild, Entity::Player(P1))], &[]);
    t.resolve_all();
    varchild
}

fn survivors(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    crate::r_s01_common::with_subtype(t, p, "Survivor")
}

#[test]
fn varchilds_tokens_are_owned_by_the_player_who_creates_them() {
    cr!("111.2");
    ruling!(
        "Varchild, Betrayer of Kjeldor",
        "The player who creates the Survivor tokens owns them, not Varchild's controller."
    );
    let mut t = TestGame::new(2);
    varchild_hits(&mut t);
    let s = survivors(&t, P1);
    assert_eq!(s.len(), 3);
    for id in s {
        assert_eq!(t.obj_now(id).owner, P1);
    }
}

#[test]
fn varchilds_restriction_follows_its_current_controller() {
    cr!("611.3a", "508.1c");
    ruling!(
        "Varchild, Betrayer of Kjeldor",
        "Varchild's second ability isn't locked in as Survivor tokens are created."
    );
    let mut t = TestGame::new(2);
    let varchild = varchild_hits(&mut t);
    let s = survivors(&t, P1)[0];
    t.g.objects[s.0 as usize].summoning_sick = false;
    // P1's turn: P1's combat.
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(can_attack(&mut t, bears));
    assert!(!can_attack(&mut t, s), "can't attack Varchild's controller");
    give_control(&mut t, varchild, P1);
    assert!(can_attack(&mut t, s), "P1 now controls Varchild");
}

#[test]
fn varchild_affects_every_survivor_including_changelings() {
    cr!("702.73a", "205.3m");
    ruling!(
        "Varchild, Betrayer of Kjeldor",
        "Varchild's second ability affects any creatures your opponents control that happen to be Survivors"
    );
    let mut t = TestGame::new(2);
    let varchild = t.battlefield(P0, "Varchild, Betrayer of Kjeldor");
    let changeling = t.battlefield(P1, "Avian Changeling");
    let attacker = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
    assert!(!crate::r_s21_common::legal_blocks(
        &mut t,
        P1,
        &[(changeling, attacker)]
    ));
    // When Varchild leaves, P0 gains control of all Survivors.
    destroy(&mut t, varchild);
    t.resolve_all();
    assert_eq!(controller(&mut t, changeling), P0);
}

#[test]
fn swooping_pteranodons_land_is_chosen_when_the_delayed_trigger_triggers() {
    cr!("603.7c", "603.3d");
    ruling!(
        "Swooping Pteranodon",
        "You choose the target land (where the poor creature is going to drop) as the delayed triggered ability triggers at the beginning of the next end step."
    );
    supported("Swooping Pteranodon");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Swooping Pteranodon");
    t.resolve_all();
    assert_eq!(controller(&mut t, giant), P0);
    // A land that didn't exist as the ability resolved can be the target.
    let late = t.battlefield(P1, "Mountain");
    t.answer_targets(P0, &[obj(late)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn soul_ransom_draws_even_if_it_cant_be_sacrificed() {
    cr!("608.2b", "701.21a");
    ruling!(
        "Soul Ransom",
        "You'll draw two cards even if you can't sacrifice Soul Ransom"
    );
    supported("Soul Ransom");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ransom = attach_new(&mut t, P0, "Soul Ransom", bears);
    assert_eq!(controller(&mut t, bears), P0);
    // P1 activates it, discarding two cards; P0 destroys Soul Ransom in response.
    let discards = [t.hand(P1, "Island"), t.hand(P1, "Island")];
    t.answer_choose(P1, &discards.map(obj));
    t.activate(P1, ransom, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, ransom);
    let before = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 2);
    assert_eq!(controller(&mut t, bears), P1);
}

#[test]
fn enslave_damages_the_creatures_owner() {
    cr!("108.3");
    ruling!(
        "Enslave",
        "The owner of a card is the player who started the game with that card in their deck."
    );
    supported("Enslave");
    // P2 owns the Bears; P1 controls them; P0 enslaves them.
    let mut t = ffa(3);
    let bears = t.battlefield(P2, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    attach_new(&mut t, P0, "Enslave", bears);
    assert_eq!(controller(&mut t, bears), P0);
    t.advance_to(P2, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P2), 19);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}
