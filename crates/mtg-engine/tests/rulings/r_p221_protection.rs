//! Rulings batch P221 — protection (CR 702.16): what each quality covers (damage,
//! enchanting/equipping, blocking, targeting), and what it does not.

use crate::r_s01_common::*;
use crate::r_s03_common::respond;
use crate::r_s04_common::{ability_targets, crew, cycle};
use crate::r_s06_common::activate_containing;
use crate::r_s30_common::pick_replacement;
use mtg_engine::decision::Decision;
use mtg_engine::eval::Ctx;
use crate::r_s12_common::morph;
use crate::r_s13_common::add;
use crate::r_s02_common::target_candidates;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The legal choices for the first target of a new copy of spell `name` in `p`'s hand
/// (an Aura spell's target is what it will enchant, CR 303.4a).
fn spell_targets(t: &mut TestGame, p: PlayerId, name: &str) -> Vec<Entity> {
    let spell = t.hand(p, name);
    t.g.recompute();
    let chars = t.g.obj(spell).chars.clone();
    let spec = match t.g.spell_body(spell).targets.first() {
        Some(s) => s.clone(),
        None => mtg_engine::attach::aura_target_spec(&chars).expect("spell without targets"),
    };
    let ctx = Ctx::new(Some(spell), p);
    t.g.legal_target_candidates(&spec, &ctx, spell)
}

/// Picks the protection ability among the replacement/prevention effects to apply.
fn protection_first(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    pick_replacement(g, d, "Protection")
}

/// Whether `p` could target `what` with the first target of a new copy of spell `name`.
fn spell_can_target(t: &mut TestGame, p: PlayerId, name: &str, what: impl Into<Entity>) -> bool {
    spell_targets(t, p, name).contains(&what.into())
}

/// Whether the first activated ability of `source` could target `what`.
fn ability_can_target(t: &mut TestGame, source: ObjectId, what: impl Into<Entity>) -> bool {
    ability_targets(t, source, 0).contains(&what.into())
}

/// The damage marked on `to` after `source` deals it `n` noncombat damage.
fn damage_from(t: &mut TestGame, source: ObjectId, n: u32, to: ObjectId) -> u32 {
    let before = t.obj_now(to).damage;
    t.g.deal_damage(source, Entity::Object(t.g.current(to)), n, false);
    t.obj_now(to).damage - before
}

/// Declares `attacker` attacking P1 (in P0's turn).
fn attacks(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
}

#[test]
fn a_face_down_creature_has_mana_value_0() {
    cr!("702.16f", "702.16e", "708.2a", "202.3e");
    ruling!(
        "Mistmeadow Skulk",
        "A face-down creature has a mana value of 0, so Mistmeadow Skulk doesn't have protection from it"
    );
    supported("Mistmeadow Skulk");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    // A face-down Hill Giant... er, a face-down morph creature (Fallen Cleric, mana value
    // 5 face up) is a 2/2 with mana value 0.
    let fd = morph(&mut t, P1, "Fallen Cleric");
    let skulk = t.battlefield(P0, "Mistmeadow Skulk");
    t.set_step(P0, Step::BeginningOfCombat);
    attacks(&mut t, skulk);
    assert!(t.g.can_block(fd, skulk));
    assert_eq!(damage_from(&mut t, fd, 1, skulk), 1);
    // Face up (mana value 5), it couldn't block it.
    let up = t.battlefield(P1, "Fallen Cleric");
    assert!(!t.g.can_block(up, skulk));
}

#[test]
fn protection_from_enchantments_covers_enchantments_not_enchanted_creatures() {
    cr!("702.16b", "702.16c", "702.16e", "702.16f");
    ruling!(
        "Azorius First-Wing",
        "“Protection from enchantments” means that Azorius First-Wing can’t be enchanted, it can’t be targeted by Aura spells or by abilities of enchantments, all damage that would be dealt to it by enchantments is prevented, and it can’t be blocked by creatures that are also enchantments."
    );
    ruling!(
        "Azorius First-Wing",
        "Azorius First-Wing doesn’t have protection from creatures that are enchanted."
    );
    ruling!(
        "Polis Crusher",
        "“Protection from enchantments” means that Polis Crusher can't be enchanted, it can't be targeted by Aura spells or by abilities of enchantments, all damage that would be dealt to it by enchantments is prevented, and it can't be blocked by enchantment creatures. Notably, it doesn't have protection from enchanted creatures (unless those creatures are also enchantments)."
    );
    for (name, enchantment_blocker, other_blocker) in [
        ("Azorius First-Wing", "Lucent Liminid", "Wind Drake"),
        ("Polis Crusher", "Nyx-Fleece Ram", "Grizzly Bears"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let me = t.battlefield(P0, name);
        // Aura spells can't target it; an Aura can't be attached to it.
        assert!(!spell_can_target(&mut t, P1, "Pacifism", me), "{name}");
        let pacifism = t.battlefield(P1, "Pacifism");
        assert!(!t.g.attach(pacifism, Entity::Object(me)), "{name}");
        // Abilities of enchantments can't target it, and their damage is prevented.
        let seal = t.battlefield(P1, "Seal of Fire");
        assert!(!ability_can_target(&mut t, seal, me), "{name}");
        assert_eq!(damage_from(&mut t, seal, 2, me), 0, "{name}");
        // Enchantment creatures can't block it; an enchanted (non-enchantment) creature
        // can, and its abilities can target it and its damage isn't prevented.
        let ench = t.battlefield(P1, enchantment_blocker);
        let other = t.battlefield(P1, other_blocker);
        let strength = t.battlefield(P1, "Holy Strength");
        assert!(t.g.attach(strength, Entity::Object(other)));
        let pyro = t.battlefield(P1, "Prodigal Pyromancer");
        let rancor = t.battlefield(P1, "Rancor");
        assert!(t.g.attach(rancor, Entity::Object(pyro)));
        assert!(ability_can_target(&mut t, pyro, me), "{name}");
        assert_eq!(damage_from(&mut t, pyro, 1, me), 1, "{name}");
        attacks(&mut t, me);
        assert!(!t.g.can_block(ench, me), "{name}");
        assert!(t.g.can_block(other, me), "{name}");
    }
}

#[test]
fn phantom_centaur_can_apply_protection_first_and_keep_its_counter() {
    cr!("702.16e", "616.1", "616.1a", "615.1a");
    ruling!(
        "Phantom Centaur",
        "If this card is going to take damage from a black source, you can choose to apply Protection from Black's damage prevention ability prior to applying this card's built-in ability. This means you don't have to remove a counter."
    );
    supported("Phantom Centaur");
    let mut t = TestGame::new(2);
    let centaur = t.enter(P0, "Phantom Centaur");
    assert_eq!(t.counters(centaur, counters::PLUS1), 3);
    let knight = t.battlefield(P1, "Black Knight");
    respond(&mut t, P0, protection_first);
    let from = t.asked().len();
    t.g.deal_damage(knight, Entity::Object(centaur), 2, false);
    t.settle();
    // Its controller chose which to apply.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseReplacement { .. })));
    assert_eq!(t.obj_now(centaur).damage, 0);
    assert_eq!(t.counters(centaur, counters::PLUS1), 3);
    // From a nonblack source, its own ability applies: the damage is prevented and a
    // counter removed.
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.g.deal_damage(pyro, Entity::Object(centaur), 1, false);
    t.settle();
    assert_eq!(t.obj_now(centaur).damage, 0);
    assert_eq!(t.counters(centaur, counters::PLUS1), 2);
}

#[test]
fn minion_of_leshrac_cant_target_itself() {
    cr!("702.16b", "115.1b");
    ruling!(
        "Minion of Leshrac",
        "Minion of Leshrac can’t target itself with its activated ability because it has protection from black."
    );
    let mut t = TestGame::new(2);
    let minion = t.battlefield(P0, "Minion of Leshrac");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let targets = ability_targets(&mut t, minion, 0);
    assert!(!targets.contains(&Entity::Object(minion)));
    assert!(targets.contains(&Entity::Object(bears)));
}

#[test]
fn emrakuls_protection_from_instants_works_only_on_the_battlefield() {
    cr!("702.16a", "702.16b", "702.16e", "611.3a");
    ruling!(
        "Emrakul, the Promised End",
        "Protection abilities only apply while the object with the ability is on the battlefield. Notably, Emrakul may be the target of a spell that targets it while on the stack, such as Syncopate."
    );
    ruling!(
        "Emrakul, the Promised End",
        "Protection from instants means that Emrakul can’t be the target of instant spells or activated or triggered abilities from instant cards, and damage that would be dealt to it by instant spells or cards is prevented. Instant spells may still affect it in other ways; for example, it would still receive the bonus from Rally the Peasants."
    );
    supported("Emrakul, the Promised End");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Wastes", 13);
    let card = t.hand(P0, "Emrakul, the Promised End");
    let spell = t.cast(P0, card).target(P1).go();
    // On the stack, the instant Syncopate can target it.
    assert!(spell_can_target(&mut t, P1, "Syncopate", spell));
    t.resolve_all();
    let emrakul = t.g.current(spell);
    assert!(t.on_battlefield(emrakul));
    // On the battlefield, instant spells can't target it and their damage is prevented.
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", emrakul));
    assert!(spell_can_target(&mut t, P1, "Flame Slash", emrakul));
    let bolt = t.hand(P1, "Lightning Bolt");
    assert_eq!(damage_from(&mut t, bolt, 3, emrakul), 0);
    // An instant that doesn't target it still affects it.
    let rally = t.hand(P0, "Rally the Peasants");
    give_mana_for(&mut t, P0, "Rally the Peasants");
    t.cast(P0, rally).go();
    t.resolve_all();
    assert_eq!(t.pt(emrakul), (15, 13));
}

#[test]
fn hexdrinker_protection_from_instants_and_from_everything() {
    cr!("702.16b", "702.16c", "702.16d", "702.16e", "702.16f", "702.16j", "711.2a");
    ruling!(
        "Hexdrinker",
        "Protection from instants means that Hexdrinker can’t be the target of instant spells or abilities from instant cards (such as an ability of an instant card that triggers when that card is cycled), and all damage that an instant spell or instant card would deal to it is prevented. Nothing other than these events is prevented or illegal."
    );
    ruling!(
        "Hexdrinker",
        "Protection from everything means that Hexdrinker can’t be the target of any spells or abilities, it can’t be blocked, nothing can be attached to it, and all damage that would be dealt to it is prevented. It can still be affected in other ways"
    );
    supported("Hexdrinker");
    supported("Primal Boost");
    // Level 3: protection from instants.
    let mut t = TestGame::new(2);
    let hex = t.battlefield(P0, "Hexdrinker");
    add(&mut t, hex, counters::LEVEL, 3);
    assert_eq!(t.pt(hex), (4, 4));
    assert!(!spell_can_target(&mut t, P0, "Giant Growth", hex));
    assert!(spell_can_target(&mut t, P1, "Flame Slash", hex));
    let bolt = t.hand(P1, "Lightning Bolt");
    assert_eq!(damage_from(&mut t, bolt, 3, hex), 0);
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    assert_eq!(damage_from(&mut t, pyro, 1, hex), 1);
    // Primal Boost's cycling trigger ("When you cycle Primal Boost, you may have target
    // creature get +1/+1 until end of turn.") can't target it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let boost = t.hand(P0, "Primal Boost");
    t.lands(P0, "Forest", 3);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cycle(&mut t, P0, boost, 0).expect("cycle");
    t.resolve_all();
    let c = target_candidates(&t, P0, from);
    assert!(!c.is_empty());
    assert!(c.iter().all(|c| !c.contains(&Entity::Object(hex))));
    assert!(c[0].contains(&Entity::Object(bears)));
    // Level 8: protection from everything.
    let mut t = TestGame::new(2);
    let hex = t.battlefield(P0, "Hexdrinker");
    add(&mut t, hex, counters::LEVEL, 8);
    assert_eq!(t.pt(hex), (6, 6));
    assert!(!spell_can_target(&mut t, P0, "Giant Growth", hex));
    assert!(!spell_can_target(&mut t, P1, "Flame Slash", hex));
    let rancor = t.battlefield(P0, "Rancor");
    assert!(!t.g.attach(rancor, Entity::Object(hex)));
    let bonesplitter = t.battlefield(P0, "Bonesplitter");
    assert!(!t.g.attach(bonesplitter, Entity::Object(hex)));
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    assert_eq!(damage_from(&mut t, pyro, 1, hex), 0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attacks(&mut t, hex);
    assert!(!t.g.can_block(bears, hex));
    // It's still affected by things that don't target it, deal damage, or attach.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let hex = t.battlefield(P0, "Hexdrinker");
    add(&mut t, hex, counters::LEVEL, 8);
    let wrath = t.hand(P1, "Wrath of God");
    give_mana_for(&mut t, P1, "Wrath of God");
    t.cast(P1, wrath).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hexdrinker"));
}

#[test]
fn protection_from_humans_doesnt_cover_players_or_planeswalkers() {
    cr!("702.16a", "702.16b", "702.16e", "205.3m");
    ruling!(
        "Yawgmoth, Thran Physician",
        "Protection from Humans refers only to the creature type Human. As far as Yawgmoth is concerned, you, your opponents, and planeswalkers aren't Humans."
    );
    let mut t = TestGame::new(2);
    let yawg = t.battlefield(P0, "Yawgmoth, Thran Physician");
    // A Human Wizard's ability can't target it and its damage is prevented.
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(!ability_can_target(&mut t, pyro, yawg));
    assert_eq!(damage_from(&mut t, pyro, 1, yawg), 0);
    // A spell cast by a player (a Human, perhaps, but not a Human object) can.
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", yawg));
    // A planeswalker's ability can.
    let chandra = t.battlefield(P1, "Chandra Nalaar");
    assert!(ability_targets(&mut t, chandra, 1).contains(&Entity::Object(yawg)));
    assert_eq!(damage_from(&mut t, chandra, 2, yawg), 2);
}

#[test]
fn protection_from_clerics() {
    cr!("702.16b", "702.16e", "702.16f");
    ruling!(
        "Fallen Cleric",
        "Protection from clerics prevents damage from cleric sources, can't have any clerics assigned to block it, and can't be targeted by the abilities of clerics."
    );
    supported("Fallen Cleric");
    let mut t = TestGame::new(2);
    let cleric = t.battlefield(P0, "Fallen Cleric");
    // Loxodon Mystic (Elephant Cleric): "{W}, {T}: Tap target creature."
    let mystic = t.battlefield(P1, "Loxodon Mystic");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!ability_can_target(&mut t, mystic, cleric));
    assert!(ability_can_target(&mut t, mystic, bears));
    assert_eq!(damage_from(&mut t, mystic, 2, cleric), 0);
    assert_eq!(damage_from(&mut t, bears, 1, cleric), 1);
    attacks(&mut t, cleric);
    assert!(!t.g.can_block(mystic, cleric));
    assert!(t.g.can_block(bears, cleric));
}

#[test]
fn teysas_protection_from_creatures_covers_creature_cards_in_other_zones() {
    cr!("702.16b", "702.16e", "702.16f", "702.159a");
    ruling!(
        "Teysa, Envoy of Ghosts",
        "Protection from creatures means that Teysa, Envoy of Ghosts can’t be the target of abilities of creatures or abilities of creature cards in other zones, such as bloodrush abilities. Teysa also can’t be blocked by creatures, and all damage that would be dealt to Teysa by creatures is prevented."
    );
    supported("Teysa, Envoy of Ghosts");
    supported("Ghor-Clan Rampager");
    let mut t = TestGame::new(2);
    let teysa = t.battlefield(P0, "Teysa, Envoy of Ghosts");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    let rampager = t.hand(P0, "Ghor-Clan Rampager");
    attack_with(
        &mut t,
        &[(teysa, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    // Bloodrush ("Target attacking creature gets +4/+4") from the hand can't target it.
    let targets = ability_targets(&mut t, rampager, 0);
    assert!(targets.contains(&Entity::Object(bears)));
    assert!(!targets.contains(&Entity::Object(teysa)));
    assert!(!t.g.can_block(blocker, teysa));
    assert!(t.g.can_block(blocker, bears));
    assert_eq!(damage_from(&mut t, blocker, 3, teysa), 0);
    // A noncreature source's damage isn't prevented.
    let rod = t.battlefield(P1, "Rod of Ruin");
    assert_eq!(damage_from(&mut t, rod, 1, teysa), 1);
}

#[test]
fn teysas_trigger_doesnt_target() {
    cr!("702.16b", "603.2", "115.1");
    ruling!(
        "Teysa, Envoy of Ghosts",
        "Teysa’s last ability doesn’t target the creature. It will destroy a creature with hexproof or protection from white, for example."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa, Envoy of Ghosts");
    let knight = t.battlefield(P1, "Black Knight");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(knight, Entity::Player(P0))]);
    block_and_finish(&mut t, P0, &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert!(t.in_graveyard(P1, "Black Knight"));
    assert_eq!(t.named_on_battlefield("Spirit Token").len(), 1);
}

#[test]
fn protection_from_lands() {
    cr!("702.16b", "702.16e", "702.16f");
    ruling!(
        "Horizon Drake",
        "Protection from lands works like any other protection ability. Horizon Drake can’t be blocked by land creatures, all damage that would be dealt to it by lands (including combat damage from land creatures) is prevented, and it can’t be the target of activated or triggered abilities from lands (including your own Teetering Peaks, for example)."
    );
    supported("Horizon Drake");
    supported("Teetering Peaks");
    supported("Inkmoth Nexus");
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Horizon Drake");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Teetering Peaks: "When this land enters, target creature gets +2/+0 until end of
    // turn."
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Teetering Peaks");
    t.resolve_all();
    let c = target_candidates(&t, P0, from);
    assert_eq!(c.len(), 1);
    assert!(c[0].contains(&Entity::Object(bears)));
    assert!(!c[0].contains(&Entity::Object(drake)));
    // A land's damage is prevented.
    let mountain = t.battlefield(P1, "Mountain");
    assert_eq!(damage_from(&mut t, mountain, 2, drake), 0);
    // Inkmoth Nexus, animated into a flying land creature, can't block it; a flier can.
    let nexus = t.battlefield(P1, "Inkmoth Nexus");
    t.lands(P1, "Wastes", 1);
    activate_containing(&mut t, P1, nexus, "becomes").expect("animate");
    t.resolve_all();
    let drake2 = t.battlefield(P1, "Wind Drake");
    attacks(&mut t, drake);
    assert!(t.obj_now(nexus).is(CardType::Creature));
    assert!(!t.g.can_block(nexus, drake));
    assert!(t.g.can_block(drake2, drake));
}

#[test]
fn sigrid_has_no_protection_from_a_god_that_isnt_a_creature() {
    cr!("702.16b", "205.3", "700.6");
    ruling!(
        "Sigrid, God-Favored",
        "Some Gods in other sets have abilities that say they aren't creatures under certain conditions. If one of them isn't a creature (and thus also not a God), Sigrid won't have protection from it."
    );
    supported("Sigrid, God-Favored");
    supported("Thassa, God of the Sea");
    // Thassa: "{1}{U}: Target creature you control can't be blocked this turn." With
    // devotion to blue less than five, Thassa isn't a creature.
    let mut t = TestGame::new(2);
    let sigrid = t.battlefield(P0, "Sigrid, God-Favored");
    let thassa = t.battlefield(P0, "Thassa, God of the Sea");
    assert!(!t.obj_now(thassa).is(CardType::Creature));
    assert!(ability_targets(&mut t, thassa, 0).contains(&Entity::Object(sigrid)));
    // With devotion five, it's a God creature, and it can't.
    t.battlefield(P0, "Mahamoti Djinn");
    t.battlefield(P0, "Mahamoti Djinn");
    t.g.recompute();
    assert!(t.obj_now(thassa).is(CardType::Creature));
    assert!(!ability_targets(&mut t, thassa, 0).contains(&Entity::Object(sigrid)));
}

#[test]
fn reaver_titan_protection_from_mana_value_3_or_less() {
    cr!("702.16b", "702.16c", "702.16d", "702.16e", "702.16f");
    ruling!(
        "Reaver Titan",
        "The protection ability means the following: - Reaver Titan can't be blocked by creatures with mana value 3 or less."
    );
    supported("Reaver Titan");
    let mut t = TestGame::new(2);
    let titan = t.battlefield(P0, "Reaver Titan");
    // Crewed, so creature spells can target it.
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crew(&mut t, P0, titan, &[giant, bears]));
    t.resolve_all();
    assert!(t.obj_now(titan).is(CardType::Creature));
    // Spells with mana value 3 or less can't target it; others can.
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", titan));
    assert!(!spell_can_target(&mut t, P1, "Hero's Downfall", titan));
    assert!(spell_can_target(&mut t, P1, "Bake into a Pie", titan));
    // Auras and Equipment with mana value 3 or less can't be attached.
    let pacifism = t.battlefield(P1, "Pacifism");
    assert!(!t.g.attach(pacifism, Entity::Object(titan)));
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert!(!t.g.attach(splitter, Entity::Object(titan)));
    // Abilities of sources with mana value 3 or less can't target it, and their damage
    // is prevented.
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(!ability_can_target(&mut t, pyro, titan));
    assert_eq!(damage_from(&mut t, pyro, 1, titan), 0);
    let shivan = t.battlefield(P1, "Shivan Hellkite");
    assert!(ability_can_target(&mut t, shivan, titan));
    assert_eq!(damage_from(&mut t, shivan, 1, titan), 1);
    // Attacking, creatures with mana value 3 or less can't block it.
    let small = t.battlefield(P1, "Grizzly Bears");
    let big = t.battlefield(P1, "Craw Wurm");
    attacks(&mut t, titan);
    assert!(!t.g.can_block(small, titan));
    assert!(t.g.can_block(big, titan));
}

#[test]
fn protection_from_demons_and_from_dragons() {
    cr!("702.16b", "702.16e", "702.16f", "702.16g");
    ruling!(
        "Baneslayer Angel",
        "Baneslayer Angel can't be blocked by creatures with the creature type Demon or the creature type Dragon."
    );
    supported("Baneslayer Angel");
    supported("Shivan Hellkite");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Baneslayer Angel");
    // Shivan Hellkite (Dragon): "{1}{R}: deals 1 damage to any target."
    let hellkite = t.battlefield(P1, "Shivan Hellkite");
    assert!(!ability_can_target(&mut t, hellkite, angel));
    assert_eq!(damage_from(&mut t, hellkite, 1, angel), 0);
    // A Kindred spell with changeling (every creature type) can't target it.
    assert!(!spell_can_target(&mut t, P1, "Crib Swap", angel));
    assert!(spell_can_target(&mut t, P1, "Murder", angel));
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    assert_eq!(damage_from(&mut t, pyro, 1, angel), 1);
    // Dragons and Demons can't block it; other fliers can.
    let dragon = t.battlefield(P1, "Shivan Dragon");
    let drake = t.battlefield(P1, "Wind Drake");
    attacks(&mut t, angel);
    assert!(!t.g.can_block(dragon, angel));
    assert!(!t.g.can_block(hellkite, angel));
    assert!(t.g.can_block(drake, angel));
}

#[test]
fn protection_from_artifacts() {
    cr!("702.16b", "702.16d", "702.16e", "702.16f");
    ruling!(
        "Nacatl Savage",
        "“Protection from artifacts” means the following: -- Nacatl Savage can't be blocked by artifact creatures. -- Nacatl Savage can't be equipped."
    );
    ruling!(
        "Tel-Jilad Fallen",
        "“Protection from artifacts” means the following: -- Tel-Jilad Fallen can't be blocked by artifact creatures. -- Tel-Jilad Fallen can't be equipped."
    );
    for name in ["Nacatl Savage", "Tel-Jilad Fallen"] {
        supported(name);
        let mut t = TestGame::new(2);
        let me = t.battlefield(P0, name);
        let splitter = t.battlefield(P0, "Bonesplitter");
        assert!(!t.g.attach(splitter, Entity::Object(me)), "{name}");
        let rod = t.battlefield(P1, "Rod of Ruin");
        assert!(!ability_can_target(&mut t, rod, me), "{name}");
        let before = t.obj_now(me).counter(counters::MINUS1);
        t.g.deal_damage(rod, Entity::Object(me), 1, false);
        t.settle();
        assert!(t.on_battlefield(me), "{name}");
        assert_eq!(t.obj_now(me).damage, 0, "{name}");
        assert_eq!(t.obj_now(me).counter(counters::MINUS1), before);
        let thopter = t.battlefield(P1, "Ornithopter");
        let bears = t.battlefield(P1, "Grizzly Bears");
        attacks(&mut t, me);
        assert!(!t.g.can_block(thopter, me), "{name}");
        assert!(t.g.can_block(bears, me), "{name}");
    }
}

#[test]
fn corruption_counters_stay_and_a_later_dihada_has_protection_from_them() {
    cr!("702.16a", "702.16b", "702.16e", "122.1", "400.7");
    ruling!(
        "Geyadrone Dihada",
        "When Dihada leaves the battlefield, permanents keep their corruption counters. If you later control another Geyadrone Dihada, it will have protection from those permanents."
    );
    supported("Geyadrone Dihada");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let dihada = t.enter(P0, "Geyadrone Dihada");
    let corrupted = t.battlefield(P1, "Prodigal Pyromancer");
    let clean = t.battlefield(P1, "Prodigal Pyromancer");
    // +1: "... Put a corruption counter on up to one other target creature or
    // planeswalker."
    t.activate(P0, dihada, 0, &[Entity::Object(corrupted)])
        .expect("+1");
    t.resolve_all();
    assert_eq!(t.counters(corrupted, "corruption"), 1);
    assert!(!ability_can_target(&mut t, corrupted, dihada));
    assert!(ability_can_target(&mut t, clean, dihada));
    // Dihada leaves; the counter stays.
    crate::r_s02_common::destroy(&mut t, dihada);
    assert!(t.in_graveyard(P0, "Geyadrone Dihada"));
    assert_eq!(t.counters(corrupted, "corruption"), 1);
    // A later Dihada has protection from the corrupted permanent.
    let again = t.enter(P0, "Geyadrone Dihada");
    assert!(!ability_can_target(&mut t, corrupted, again));
    assert!(ability_can_target(&mut t, clean, again));
    let loyalty = t.counters(again, counters::LOYALTY);
    t.g.deal_damage(corrupted, Entity::Object(again), 1, false);
    t.settle();
    assert_eq!(t.counters(again, counters::LOYALTY), loyalty);
    t.g.deal_damage(clean, Entity::Object(again), 1, false);
    t.settle();
    assert_eq!(t.counters(again, counters::LOYALTY), loyalty - 1);
}
