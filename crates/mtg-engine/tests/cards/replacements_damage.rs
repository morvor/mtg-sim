//! "If [a source] would deal damage to [something], prevent N of that damage / it deals
//! double that damage instead / ... plus N instead" (patterns in
//! `src/oracle/patterns/replacements_damage.rs`).

use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn damage_replacement_cards_compile() {
    assert_compiles(&[
        "Sphere of Grace",
        "Sphere of Reason",
        "Sphere of Duty",
        "Sphere of Law",
        "Sphere of Truth",
        "Sphere of Purity",
        "Orbs of Warding",
        "Urza's Armor",
        "Daunting Defender",
        "Djeru, With Eyes Open",
        "Shield of the Realm",
        "Rem Karolus, Stalwart Slayer",
        "Thunderstaff",
        "Calamity Bearer",
        "Gratuitous Violence",
        "Uncivil Unrest",
        "Fire Servant",
        "Fiendish Duo",
        "Curse of Bloodletting",
        "Goldnight Castigator",
        "Twinflame Tyrant",
        "Solphim, Mayhem Dominus",
        "Inquisitor's Flail",
        "The Sound of Drums",
        "Charging Tuskodon",
        "Embermaw Hellion",
        "Insult // Injury",
        "Blind Fury",
    ]);
}

/// P1 casts Lightning Bolt at `target`.
fn bolt(t: &mut TestGame, caster: PlayerId, target: impl Into<Entity>) {
    t.lands(caster, "Mountain", 1);
    let b = t.hand(caster, "Lightning Bolt");
    t.cast(caster, b).target(target).go();
    t.resolve();
}

#[test]
fn sphere_of_law_prevents_two_of_each_red_sources_damage() {
    cr!("615.1a", "615.10");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphere of Law");
    bolt(&mut t, P1, P0);
    assert_eq!(t.life(P0), 19);
    // Two red sources dealing damage at the same time: 2 of each is prevented.
    let g1 = t.battlefield(P1, "Hill Giant");
    let g2 = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage_batch(
        vec![(g1, Entity::Player(P0), 3), (g2, Entity::Player(P0), 3)],
        true,
    );
    assert_eq!(t.life(P0), 17);
    // A green source isn't affected.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(bears, Entity::Player(P0), 2, true);
    assert_eq!(t.life(P0), 15);
    // Damage to P0's creatures isn't prevented.
    let mine = t.battlefield(P0, "Hill Giant");
    bolt(&mut t, P1, mine);
    assert!(!t.on_battlefield(mine));
}

#[test]
fn urzas_armor_prevents_one_damage_to_you_only() {
    cr!("615.10");
    ruling!(
        "Urza's Armor",
        "If a spell or ability damages multiple things, divide up the damage before applying this effect, and prevent the 1 damage to you only."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Urza's Armor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage_batch(
        vec![(giant, Entity::Player(P0), 2), (giant, Entity::Object(bears), 1)],
        false,
    );
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn orbs_of_warding_prevents_one_damage_from_creatures_and_stacks() {
    cr!("615.10");
    ruling!(
        "Orbs of Warding",
        "The damage prevention effects of multiple Orbs of Warding are cumulative."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orbs of Warding");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 18);
    // A noncreature source isn't affected. (Orbs also gives P0 hexproof, so no Bolt.)
    let rock = t.battlefield(P1, "Sol Ring");
    t.g.deal_damage(rock, Entity::Player(P0), 3, false);
    assert_eq!(t.life(P0), 15);
    t.battlefield(P0, "Orbs of Warding");
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 14);
}

#[test]
fn thunderstaff_prevents_combat_damage_only_while_untapped() {
    cr!("615.10");
    ruling!(
        "Thunderstaff",
        "If Thunderstaff is untapped, it prevents 1 damage from each creature, each time it would deal combat damage to you."
    );
    let mut t = TestGame::new(2);
    let staff = t.battlefield(P0, "Thunderstaff");
    let g1 = t.battlefield(P1, "Hill Giant");
    let g2 = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(
        &[(g1, Entity::Player(P0)), (g2, Entity::Player(P0))],
        &[],
    );
    // 3 + 2, one prevented from each creature.
    assert_eq!(t.life(P0), 17);
    // Noncombat damage from a creature isn't prevented.
    t.g.deal_damage(g1, Entity::Player(P0), 3, false);
    assert_eq!(t.life(P0), 14);
    // Tapped: no prevention.
    t.g.tap(staff);
    t.recompute();
    t.g.deal_damage(g1, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 11);
}

#[test]
fn daunting_defender_and_shield_of_the_realm_protect_creatures() {
    cr!("615.10");
    ruling!(
        "Shield of the Realm",
        "If multiple sources would deal damage to the equipped creature at once (for example, several blocking creatures), 2 damage from each of those sources is prevented."
    );
    let mut t = TestGame::new(2);
    let defender = t.battlefield(P0, "Daunting Defender");
    bolt(&mut t, P1, defender);
    // A 3/3 Cleric: 1 of the 3 damage is prevented.
    assert!(t.on_battlefield(defender));
    assert_eq!(t.obj_now(defender).damage, 2);

    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let shield = t.battlefield(P0, "Shield of the Realm");
    assert!(t.g.attach(shield, Entity::Object(wurm)));
    t.recompute();
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage_batch(
        vec![(a, Entity::Object(wurm), 3), (b, Entity::Object(wurm), 2)],
        true,
    );
    assert_eq!(t.obj_now(wurm).damage, 1);
    // An unequipped creature isn't protected.
    let other = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(a, Entity::Object(other), 1, false);
    assert_eq!(t.obj_now(other).damage, 1);
}

#[test]
fn gratuitous_violence_doubles_damage_from_your_creatures() {
    cr!("701.10g", "614.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gratuitous Violence");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(bears, Entity::Player(P1), 2, true);
    assert_eq!(t.life(P1), 16);
    // Not a creature: Lightning Bolt deals 3.
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 13);
    // An opponent's creature isn't affected.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(theirs, Entity::Player(P0), 2, true);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn fiendish_duo_and_curse_of_bloodletting_double_damage_to_players() {
    cr!("701.10g", "614.1a");
    ruling!(
        "Curse of Bloodletting",
        "Curse of Bloodletting works with any damage, not just combat damage. It also doesn’t matter who controls the source of the damage that’s being dealt."
    );
    ruling!(
        "Curse of Bloodletting",
        "If more than one Curse of Bloodletting enchants the same player, damage dealt to that player will double for each one"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fiendish Duo");
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 14);
    // Damage to P0 isn't doubled.
    bolt(&mut t, P1, P0);
    assert_eq!(t.life(P0), 17);

    let mut t = TestGame::new(2);
    let c1 = t.battlefield(P0, "Curse of Bloodletting");
    assert!(t.g.attach(c1, Entity::Player(P1)));
    t.recompute();
    // P1's own source damaging P1 is doubled too.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(theirs, Entity::Player(P1), 2, false);
    assert_eq!(t.life(P1), 16);
    // The enchanting player isn't affected.
    t.g.deal_damage(theirs, Entity::Player(P0), 2, false);
    assert_eq!(t.life(P0), 18);
    let c2 = t.battlefield(P0, "Curse of Bloodletting");
    assert!(t.g.attach(c2, Entity::Player(P1)));
    t.recompute();
    t.g.deal_damage(theirs, Entity::Player(P1), 2, false);
    assert_eq!(t.life(P1), 8);
}

#[test]
fn solphim_doubles_only_noncombat_damage_to_opponents() {
    cr!("701.10g", "120.2b");
    let mut t = TestGame::new(2);
    let solphim = t.battlefield(P0, "Solphim, Mayhem Dominus");
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 14);
    let opp_bears = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(solphim, Entity::Object(opp_bears), 1, false);
    assert_eq!(t.obj_now(opp_bears).damage, 2);
    // Combat damage isn't doubled.
    t.g.deal_damage(solphim, Entity::Player(P1), 3, true);
    assert_eq!(t.life(P1), 11);
    // Damage to you isn't doubled.
    bolt(&mut t, P0, P0);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn charging_tuskodon_doubles_combat_damage_to_players_only() {
    cr!("701.10g");
    ruling!(
        "Charging Tuskodon",
        "The doubled damage Charging Tuskodon deals is still combat damage."
    );
    let mut t = TestGame::new(2);
    let tusk = t.battlefield(P0, "Charging Tuskodon");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(tusk, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 12);
    // One damage event of 8, and it's combat damage.
    let dealt: Vec<(u32, bool)> = t
        .g
        .turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target: Entity::Player(p),
                amount,
                combat,
            } if *source == tusk && *p == P1 => Some((*amount, *combat)),
            _ => None,
        })
        .collect();
    assert_eq!(dealt, vec![(8, true)]);
    // Noncombat damage isn't doubled, nor is damage to a creature.
    t.g.deal_damage(tusk, Entity::Player(P1), 1, false);
    assert_eq!(t.life(P1), 11);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.deal_damage(tusk, Entity::Object(wurm), 4, true);
    assert_eq!(t.obj_now(wurm).damage, 4);
}

#[test]
fn charging_tuskodon_doubles_trample_damage_only_as_its_dealt() {
    cr!("701.10g", "702.19b");
    ruling!(
        "Charging Tuskodon",
        "that damage is assigned based on its actual power and is doubled only as it’s dealt"
    );
    let mut t = TestGame::new(2);
    let tusk = t.battlefield(P0, "Charging Tuskodon");
    // A 3/3 blocker: 3 is assigned to it (lethal damage isn't doubled), 1 tramples over
    // to the player and is doubled to 2 as it's dealt.
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(tusk, Entity::Player(P1))], &[(giant, tusk)]);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn embermaw_hellion_adds_one_to_other_red_sources() {
    cr!("614.1a");
    ruling!(
        "Embermaw Hellion",
        "Multiple Embermaw Hellions are cumulative. If you control two of them, you’ll add 2 to the damage dealt by another red source you control."
    );
    let mut t = TestGame::new(2);
    let h1 = t.battlefield(P0, "Embermaw Hellion");
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 16);
    // Its own damage isn't increased.
    t.g.deal_damage(h1, Entity::Player(P1), 2, false);
    assert_eq!(t.life(P1), 14);
    // A red source an opponent controls isn't affected.
    bolt(&mut t, P1, P0);
    assert_eq!(t.life(P0), 17);
    // With two Hellions, each adds 1 to the other's damage.
    t.battlefield(P0, "Embermaw Hellion");
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 9);
    t.g.deal_damage(h1, Entity::Player(P1), 2, false);
    assert_eq!(t.life(P1), 6);
}

#[test]
fn inquisitors_flail_doubles_combat_damage_by_and_to_the_equipped_creature() {
    cr!("701.10g");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let flail = t.battlefield(P0, "Inquisitor's Flail");
    assert!(t.g.attach(flail, Entity::Object(bears)));
    t.recompute();
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(wurm, bears)]);
    // The Bears' 2 combat damage is doubled, destroying the 6/4 Wurm.
    assert!(!t.on_battlefield(wurm));
    assert!(!t.on_battlefield(bears));
    // Combat damage another creature deals to the equipped creature is doubled too.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let flail = t.battlefield(P0, "Inquisitor's Flail");
    assert!(t.g.attach(flail, Entity::Object(maw)));
    t.recompute();
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(maw, Entity::Player(P1))], &[(blocker, maw)]);
    assert!(!t.on_battlefield(blocker));
    assert_eq!(t.obj_now(maw).damage, 4);
    // Noncombat damage to the equipped creature isn't.
    let other = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(other, Entity::Object(maw), 1, false);
    assert_eq!(t.obj_now(maw).damage, 5);
    // Noncombat damage by the equipped creature isn't doubled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let flail = t.battlefield(P0, "Inquisitor's Flail");
    assert!(t.g.attach(flail, Entity::Object(bears)));
    t.recompute();
    t.g.deal_damage(bears, Entity::Player(P1), 2, false);
    assert_eq!(t.life(P1), 18);
    t.g.deal_damage(bears, Entity::Player(P1), 2, true);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn rem_karolus_prevents_spell_damage_and_adds_to_your_spells() {
    cr!("615.1a", "614.1a");
    ruling!(
        "Rem Karolus, Stalwart Slayer",
        "Rem Karolus, Stalwart Slayer's last two abilities apply only to damage dealt by spells on the stack."
    );
    let mut t = TestGame::new(2);
    let rem = t.battlefield(P0, "Rem Karolus, Stalwart Slayer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    bolt(&mut t, P1, P0);
    assert_eq!(t.life(P0), 20);
    bolt(&mut t, P1, bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    // "another permanent you control": damage from a spell to Rem Karolus itself isn't
    // prevented.
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, shock).target(rem).go();
    t.resolve();
    assert_eq!(t.obj_now(rem).damage, 2);
    // Damage from a creature isn't prevented.
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(giant, Entity::Player(P0), 3, true);
    assert_eq!(t.life(P0), 17);
    // P0's spells deal 1 more to opponents and their permanents; Rem's own damage isn't
    // increased.
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 16);
    t.g.deal_damage(rem, Entity::Player(P1), 2, true);
    assert_eq!(t.life(P1), 14);
    let theirs = t.battlefield(P1, "Colossal Dreadmaw");
    bolt(&mut t, P0, theirs);
    assert_eq!(t.obj_now(theirs).damage, 4);
    // P0's spell damaging P0's own creature is prevented, not increased.
    let mine = t.battlefield(P0, "Colossal Dreadmaw");
    bolt(&mut t, P0, mine);
    assert_eq!(t.obj_now(mine).damage, 0);
}

#[test]
fn blind_fury_doubles_combat_damage_to_creatures_this_turn() {
    cr!("701.10g", "611.2a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 4);
    let fury = t.hand(P0, "Blind Fury");
    t.cast(P0, fury).go();
    t.resolve();
    t.g.deal_damage(bears, Entity::Object(wurm), 2, true);
    assert_eq!(t.obj_now(wurm).damage, 4);
    // Combat damage to a player and noncombat damage to a creature aren't doubled.
    t.g.deal_damage(bears, Entity::Player(P1), 2, true);
    assert_eq!(t.life(P1), 18);
    t.g.deal_damage(bears, Entity::Object(wurm), 1, false);
    assert_eq!(t.obj_now(wurm).damage, 5);
    // The effect ends with the turn.
    t.advance_to(P1, Step::Upkeep);
    let later = t.battlefield(P1, "Craw Wurm");
    t.g.deal_damage(bears, Entity::Object(later), 2, true);
    assert_eq!(t.obj_now(later).damage, 2);
}
