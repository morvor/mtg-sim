//! Rulings batch P047 — "if [a creature] would die this turn, exile it instead" (CR
//! 614.1a, 614.6, 700.4): the creature the replacement covers (the target, or a creature
//! actually dealt damage — redirection and prevention, CR 614.9, 615), dying for any
//! reason, regeneration, and the static versions on Kumano, Kumano's Pupils and
//! Frostwielder (which need their source on the battlefield).

use crate::r_p047_common::*;
use crate::r_s01_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Spells whose replacement covers their target creature: (card, damage dealt, modes).
const TARGET_EXILERS: &[(&str, i32, Option<usize>)] = &[
    ("Feed the Flames", 5, None),
    ("Fiery Annihilation", 5, None),
    ("Agate Assault", 4, Some(0)),
    ("Suplex", 3, Some(0)),
    ("Bot Bashing Time", 6, None),
    ("Reduce to Ashes", 5, None),
    ("Lava Coil", 4, None),
    ("Magma Spray", 2, None),
    ("Puncturing Blow", 5, None),
    ("Scorchmark", 2, None),
    ("Betrayer's Bargain", 5, None),
    ("Smite the Deathless", 3, None),
];

/// P0 casts `name` at `target` (choosing `mode`, paying Betrayer's Bargain's {2}).
fn cast_at(t: &mut TestGame, name: &str, mode: Option<usize>, target: ObjectId) {
    supported(name);
    let c = t.hand(P0, name);
    give_mana_for(t, P0, name);
    if name == "Betrayer's Bargain" {
        // "sacrifice a creature or enchantment or pay {2}": pay {2}.
        t.lands(P0, "Wastes", 2);
        t.answer(
            P0,
            DecisionKind::Option,
            mtg_engine::decision::Answer::Index(1),
        );
    }
    t.answer_targets(P0, &[obj(target)]);
    if name == "Fiery Annihilation" {
        // "up to one target Equipment attached to that creature": none.
        t.answer_targets(P0, &[]);
    }
    let mut b = t.cast(P0, c);
    if let Some(m) = mode {
        b = b.modes(&[m]);
    }
    b.go();
    t.resolve_all();
}

#[test]
fn target_is_exiled_if_it_dies_this_turn_for_any_reason() {
    cr!("614.1a", "700.4", "120.3e");
    ruling!(
        "Feed the Flames",
        "Feed the Flames's replacement effect will exile the target creature if it would die this turn for any reason, not just due to lethal damage."
    );
    ruling!(
        "Fiery Annihilation",
        "Fiery Annihilation's replacement effect will exile the target creature if it would die this turn for any reason, not just due to lethal damage."
    );
    ruling!(
        "Agate Assault",
        "The replacement effect from Agate Assault’s first mode will exile the target creature if it would die this turn for any reason"
    );
    ruling!(
        "Suplex",
        "The replacement effect from Suplex's first mode will exile the target creature if it would die this turn for any reason"
    );
    ruling!(
        "Bot Bashing Time",
        "The replacement effect of Bot Bashing Time will exile the target creature if it would die this turn for any reason, not just due to lethal damage."
    );
    ruling!(
        "Reduce to Ashes",
        "The replacement effect of Reduce to Ashes will exile the target creature if it would die this turn for any reason, not just due to lethal damage."
    );
    ruling!(
        "Lava Coil",
        "Lava Coil's replacement effect will exile the target creature if it would die this turn for any reason, not just due to lethal damage."
    );
    ruling!(
        "Magma Spray",
        "Magma Spray's replacement effect will exile the target creature if it would die this turn for any reason, not just due to lethal damage."
    );
    ruling!(
        "Puncturing Blow",
        "Puncturing Blow’s replacement effect will exile the target creature if it would die this turn for any reason, not just due to lethal damage."
    );
    ruling!(
        "Scorchmark",
        "Scorchmark’s replacement effect will exile the target creature if it would die this turn for any reason, not just due to lethal damage."
    );
    ruling!(
        "Betrayer's Bargain",
        "If Betrayer's Bargain resolves, its effect causes the creature to be exiled any time it would die that turn, not just if it would die as a result of damage dealt by Betrayer's Bargain."
    );
    for &(name, dmg, mode) in TARGET_EXILERS {
        // A 9/9 survives the damage, then Murder destroys it later this turn.
        let mut t = TestGame::new(2);
        let big = t.battlefield(P1, "Ancient Brontodon");
        cast_at(&mut t, name, mode, big);
        assert!(t.on_battlefield(big), "{name}");
        assert_eq!(t.g.obj(big).damage as i32, dmg, "{name}");
        murder(&mut t, P0, big);
        assert_eq!(t.zone(big), Zone::Exile, "{name}");
        // Next turn, the effect is over: a new one dies normally.
        let mut t = TestGame::new(2);
        let big = t.battlefield(P1, "Ancient Brontodon");
        cast_at(&mut t, name, mode, big);
        t.advance_to(P1, Step::PrecombatMain);
        spell(&mut t, P1, "Murder", &[obj(big)]);
        assert_eq!(t.zone(big), Zone::Graveyard(P1), "{name}: next turn");
    }
}

#[test]
fn target_replacement_applies_without_damage_or_with_redirected_damage() {
    cr!("614.1a", "614.9", "615.1a");
    ruling!(
        "Lava Coil",
        "It applies to the target creature even if Lava Coil deals no damage to it (due to a prevention effect)."
    );
    ruling!(
        "Scorchmark",
        "It applies to the target creature even if Scorchmark deals no damage to it (perhaps due to a prevention effect)."
    );
    ruling!(
        "Magma Spray",
        "It applies to the target creature even if Magma Spray deals no damage to it (due to a prevention effect) or Magma Spray deals damage to a different creature (due to a redirection effect)."
    );
    ruling!(
        "Puncturing Blow",
        "It applies to the target creature even if Puncturing Blow deals no damage to it (due to a prevention effect) or Puncturing Blow deals damage to a different creature (due to a redirection effect)."
    );
    for name in ["Lava Coil", "Scorchmark", "Magma Spray", "Puncturing Blow"] {
        // Prevention: Shielded Passage on the target first.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        spell(&mut t, P1, "Shielded Passage", &[obj(bears)]);
        cast_at(&mut t, name, None, bears);
        assert_eq!(t.g.obj(bears).damage, 0, "{name}");
        murder(&mut t, P0, bears);
        assert_eq!(t.zone(bears), Zone::Exile, "{name}: prevented");
    }
    for name in ["Magma Spray", "Puncturing Blow"] {
        // Redirection: Palisade Giant takes the damage dealt to the Bears.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Palisade Giant");
        cast_at(&mut t, name, None, bears);
        assert_eq!(t.g.obj(bears).damage, 0, "{name}");
        assert!(t.g.obj(giant).damage > 0, "{name}");
        murder(&mut t, P0, bears);
        assert_eq!(t.zone(bears), Zone::Exile, "{name}: redirected");
    }
}

#[test]
fn dealt_damage_this_way_follows_the_damage() {
    cr!("614.1a", "614.9", "615.1a", "120.3e");
    ruling!(
        "Annihilating Fire",
        "A creature dealt damage by Annihilating Fire that dies this turn will be exiled even if it wasn’t the target of Annihilating Fire"
    );
    ruling!(
        "Pillar of Flame",
        "A creature dealt damage by Pillar of Flame that dies this turn will be exiled even if it wasn’t the target of Pillar of Flame (because the damage was redirected somehow). If the target creature isn’t dealt damage (most likely because the damage was prevented), it won’t be exiled if it dies that turn."
    );
    for name in ["Annihilating Fire", "Pillar of Flame"] {
        // Redirected from the Bears to Palisade Giant: the Giant is exiled when it dies,
        // the Bears (never dealt damage) go to the graveyard.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Palisade Giant");
        spell(&mut t, P0, name, &[obj(bears)]);
        assert!(t.g.obj(giant).damage > 0, "{name}");
        murder(&mut t, P0, giant);
        assert_eq!(t.zone(giant), Zone::Exile, "{name}");
        murder(&mut t, P0, bears);
        assert_eq!(t.zone(bears), Zone::Graveyard(P1), "{name}");
        // Redirected from the player to a creature (Pariah).
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let pariah = t.battlefield(P1, "Pariah");
        t.g.attach(pariah, obj(bears));
        spell(&mut t, P0, name, &[Entity::Player(P1)]);
        assert_eq!(t.life(P1), 20, "{name}");
        assert_eq!(t.zone(bears), Zone::Exile, "{name}: Pariah");
    }
    // Prevented: the target isn't exiled when it dies later this turn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    spell(&mut t, P1, "Shielded Passage", &[obj(bears)]);
    spell(&mut t, P0, "Pillar of Flame", &[obj(bears)]);
    murder(&mut t, P0, bears);
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
}

#[test]
fn dealt_damage_this_way_dies_for_any_reason() {
    cr!("614.1a", "700.4", "704.5f", "704.5g");
    ruling!(
        "Annihilating Fire",
        "A creature dealt damage by Annihilating Fire will be exiled no matter why the creature would die. It could be destroyed by another spell or ability."
    );
    ruling!(
        "Pillar of Flame",
        "A creature dealt damage by Pillar of Flame will be exiled no matter why the creature would die that turn. It could have its toughness reduced to 0 or less or be destroyed by another spell or ability."
    );
    ruling!(
        "Anger of the Gods",
        "Creatures don't necessarily have to be dealt lethal damage by Anger of the Gods to be exiled. After being dealt damage, if they would die for any reason that turn, they'll be exiled instead."
    );
    ruling!(
        "Crush the Weak",
        "Creatures don’t necessarily have to be dealt lethal damage by Crush the Weak to be exiled. After being dealt damage, if they would die for any reason that turn, they’ll be exiled instead."
    );
    ruling!(
        "Incendiary Flow",
        "The damaged creature will be exiled if it would die for any reason that turn, not just if it dies due to damage from Incendiary Flow."
    );
    for name in [
        "Annihilating Fire",
        "Pillar of Flame",
        "Anger of the Gods",
        "Crush the Weak",
        "Incendiary Flow",
    ] {
        // Destroyed later this turn.
        let mut t = TestGame::new(2);
        let big = t.battlefield(P1, "Ancient Brontodon");
        let targets: &[Entity] = if name.contains("Gods") || name.contains("Crush") {
            &[]
        } else {
            &[obj(big)]
        };
        spell(&mut t, P0, name, targets);
        assert!(t.g.obj(big).damage > 0, "{name}");
        murder(&mut t, P0, big);
        assert_eq!(t.zone(big), Zone::Exile, "{name}");
    }
    // Pillar of Flame: toughness reduced to 0 (an indestructible creature survives the
    // damage, then Dead Weight's -2/-2 puts it into the graveyard).
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    spell(&mut t, P0, "Pillar of Flame", &[obj(myr)]);
    assert!(t.on_battlefield(myr));
    spell(&mut t, P0, "Dead Weight", &[obj(myr)]);
    assert_eq!(t.zone(myr), Zone::Exile);
}

#[test]
fn annihilating_fire_creature_can_still_regenerate() {
    cr!("701.19a", "614.1a");
    ruling!("Annihilating Fire", "The creature can still regenerate.");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    spell(&mut t, P1, "Regenerate", &[obj(bears)]);
    spell(&mut t, P0, "Annihilating Fire", &[obj(bears)]);
    assert!(t.on_battlefield(bears));
    assert!(t.g.obj(bears).tapped);
    assert_eq!(t.g.obj(bears).damage, 0);
}

#[test]
fn smite_and_burn_from_within_lose_indestructible_and_exile() {
    cr!("614.1a", "613.1f", "704.5g");
    ruling!(
        "Smite the Deathless",
        "If Smite the Deathless doesn't deal damage to the target creature (perhaps because that damage was prevented), the additional effects will still apply. It will still lose indestructible, and it will be exiled instead of dying that turn."
    );
    ruling!(
        "Smite the Deathless",
        "The damaged creature will be exiled if it would die for any reason that turn, not just if it dies due to damage from Smite the Deathless."
    );
    ruling!(
        "Burn from Within",
        "The damaged creature will be exiled if it would die for any reason that turn, not just if it dies due to damage from Burn from Within."
    );
    ruling!(
        "Smite the Deathless",
        "You can target a creature that doesn't have indestructible with Smite the Deathless. It will still be exiled if it would die this turn."
    );
    ruling!(
        "Burn from Within",
        "You can target a creature that doesn’t have indestructible with Burn from Within. It will still be exiled if it would die this turn."
    );
    // Indestructible, damage prevented: Smite still removes indestructible and exiles.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    spell(&mut t, P1, "Shielded Passage", &[obj(myr)]);
    spell(&mut t, P0, "Smite the Deathless", &[obj(myr)]);
    assert_eq!(t.g.obj(myr).damage, 0);
    assert!(!t.g.obj(myr).has_keyword(KeywordKind::Indestructible));
    murder(&mut t, P0, myr);
    assert_eq!(t.zone(myr), Zone::Exile);
    // Burn from Within's damaged Brontodon dies later this turn: exiled.
    let mut t = TestGame::new(2);
    let big = t.battlefield(P1, "Ancient Brontodon");
    spell_x(&mut t, P0, "Burn from Within", 2, &[obj(big)]);
    murder(&mut t, P0, big);
    assert_eq!(t.zone(big), Zone::Exile);
    // Non-indestructible targets killed by the damage: exiled.
    for name in ["Smite the Deathless", "Burn from Within"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        if name == "Burn from Within" {
            spell_x(&mut t, P0, name, 3, &[obj(bears)]);
        } else {
            spell(&mut t, P0, name, &[obj(bears)]);
        }
        assert_eq!(t.zone(bears), Zone::Exile, "{name}");
    }
}

#[test]
fn burn_from_within_without_damage_does_nothing_else() {
    cr!("614.1a", "608.2c");
    ruling!(
        "Burn from Within",
        "If Burn from Within doesn’t deal damage to the target creature (perhaps because that damage was prevented or X is 0), neither additional effect will apply. It won’t lose indestructible, and it won’t be exiled instead of dying that turn."
    );
    // Prevented: keeps indestructible; dying another way later this turn isn't replaced.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    spell(&mut t, P1, "Shielded Passage", &[obj(myr)]);
    spell_x(&mut t, P0, "Burn from Within", 2, &[obj(myr)]);
    assert!(t.g.obj(myr).has_keyword(KeywordKind::Indestructible));
    murder(&mut t, P0, myr);
    assert!(t.on_battlefield(myr));
    spell(&mut t, P0, "Dead Weight", &[obj(myr)]);
    assert_eq!(t.zone(myr), Zone::Graveyard(P1));
    // X is 0.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    spell_x(&mut t, P0, "Burn from Within", 0, &[obj(bears)]);
    murder(&mut t, P0, bears);
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
}

#[test]
fn fiery_annihilation_equipment_target() {
    cr!("115.1", "608.2b", "601.2c");
    ruling!(
        "Fiery Annihilation",
        "Fiery Annihilation can target a creature that has no Equipment attached to it."
    );
    // No Equipment: it can be cast and deals its damage.
    let mut t = TestGame::new(2);
    let big = t.battlefield(P1, "Ancient Brontodon");
    cast_at(&mut t, "Fiery Annihilation", None, big);
    assert_eq!(t.g.obj(big).damage, 5);
    // The creature leaves in response: the Equipment is no longer attached to it, so it's
    // no longer a legal target and stays (CR 608.2b: target legality is about what it's
    // attached to now, not as its host last existed). (Fiery Annihilation's ruling that the Equipment isn't exiled whenever the
    // creature is an illegal target — even still on the battlefield, with protection from
    // red — disagrees with Searing Blaze's on a dependent target and isn't cited here.)
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sword = t.battlefield(P1, "Bonesplitter");
    t.g.attach(sword, obj(bears));
    let fa = t.hand(P0, "Fiery Annihilation");
    give_mana_for(&mut t, P0, "Fiery Annihilation");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(sword)]);
    t.cast(P0, fa).go();
    let u = t.hand(P1, "Unsummon");
    give_mana_for(&mut t, P1, "Unsummon");
    t.cast(P1, u).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(sword));
    // Control: both legal, the Equipment is exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sword = t.battlefield(P1, "Bonesplitter");
    t.g.attach(sword, obj(bears));
    spell(&mut t, P0, "Fiery Annihilation", &[obj(bears), obj(sword)]);
    assert_eq!(t.zone(sword), Zone::Exile);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn static_exilers_need_their_source_on_the_battlefield() {
    cr!("614.1a", "611.3a", "603.10a");
    ruling!(
        "Frostwielder",
        "Frostwielder must be on the battlefield when the creature would be put into a graveyard in order to exile it."
    );
    ruling!(
        "Kumano, Master Yamabushi",
        "Kumano, Master Yamabushi must be on the battlefield when the creature would be put into a graveyard in order to exile it."
    );
    for name in ["Frostwielder", "Kumano, Master Yamabushi"] {
        supported(name);
        for gone in [false, true] {
            let mut t = TestGame::new(2);
            let src = t.battlefield(P0, name);
            let big = t.battlefield(P1, "Ancient Brontodon");
            if name == "Kumano, Master Yamabushi" {
                t.lands(P0, "Mountain", 2);
                t.activate(P0, src, 0, &[obj(big)]).unwrap();
            } else {
                t.activate(P0, src, 0, &[obj(big)]).unwrap();
            }
            t.resolve_all();
            assert_eq!(t.g.obj(big).damage, 1, "{name}");
            if gone {
                murder(&mut t, P0, src);
            }
            murder(&mut t, P0, big);
            let want = if gone { Zone::Graveyard(P1) } else { Zone::Exile };
            assert_eq!(t.zone(big), want, "{name} gone={gone}");
        }
    }
}

#[test]
fn kumano_exiles_creatures_dying_at_the_same_time_as_it() {
    cr!("614.1a", "614.6", "616.1");
    ruling!(
        "Kumano, Master Yamabushi",
        "If a creature dealt damage by Kumano this turn would be put into a graveyard, exile it, even if it's going to the graveyard at the same time as Kumano."
    );
    let mut t = TestGame::new(2);
    let kumano = t.battlefield(P0, "Kumano, Master Yamabushi");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, kumano, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    spell(&mut t, P0, "Wrath of God", &[]);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(kumano), Zone::Graveyard(P0));
}

#[test]
fn kumano_s_pupils_combat_damage() {
    cr!("614.1a", "510.2", "704.5g");
    ruling!(
        "Kumano's Pupils",
        "If a creature dealt damage by Kumano’s Pupils this turn would be put into a graveyard, exile it instead, even if it’s going to the graveyard at the same time as Kumano’s Pupils."
    );
    ruling!(
        "Kumano's Pupils",
        "Kumano’s Pupils must be on the battlefield when the creature would be put into a graveyard in order to exile it."
    );
    supported("Kumano's Pupils");
    // Pupils (3/3) and Hill Giant (3/3) trade: the Giant is exiled.
    let mut t = TestGame::new(2);
    let pupils = t.battlefield(P0, "Kumano's Pupils");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(pupils, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(giant, pupils)]);
    assert_eq!(t.zone(giant), Zone::Exile);
    assert_eq!(t.zone(pupils), Zone::Graveyard(P0));
    // Blocked by a 9/9 that survives, which dies after the Pupils are gone: graveyard.
    let mut t = TestGame::new(2);
    let pupils = t.battlefield(P0, "Kumano's Pupils");
    let big = t.battlefield(P1, "Ancient Brontodon");
    attack_with(&mut t, &[(pupils, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(big, pupils)]);
    assert_eq!(t.zone(pupils), Zone::Graveyard(P0));
    assert_eq!(t.g.obj(big).damage, 3);
    t.set_step(P0, Step::PostcombatMain);
    murder(&mut t, P0, big);
    assert_eq!(t.zone(big), Zone::Graveyard(P1));
}

#[test]
fn red_sun_s_zenith_that_doesnt_resolve_goes_to_the_graveyard() {
    cr!("608.2b", "701.6a", "608.2n");
    ruling!(
        "Red Sun's Zenith",
        "If this spell doesn’t resolve, none of its effects occur. In particular, it will go to the graveyard rather than to its owner’s library."
    );
    // Countered.
    let mut t = TestGame::new(2);
    let rsz = cast_x(&mut t, P0, "Red Sun's Zenith", Some(2), &[Entity::Player(P1)]);
    spell(&mut t, P1, "Counterspell", &[obj(rsz)]);
    assert_eq!(t.zone(rsz), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 20);
    // Its only target became illegal.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rsz = cast_x(&mut t, P0, "Red Sun's Zenith", Some(2), &[obj(bears)]);
    spell(&mut t, P1, "Unsummon", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(rsz), Zone::Graveyard(P0));
    // Resolving: it's shuffled into the library.
    let mut t = TestGame::new(2);
    let lib = t.library_size(P0);
    let rsz = spell_x(&mut t, P0, "Red Sun's Zenith", 2, &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 18);
    assert!(matches!(t.zone(rsz), Zone::Library(_)));
    assert_eq!(t.library_size(P0), lib + 1);
}
