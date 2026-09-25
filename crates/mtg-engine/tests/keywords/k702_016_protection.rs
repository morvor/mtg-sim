//! CR 702.16 Protection.

use crate::common_k702_011_017::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// Mother of Runes (untapped, controlled by `p`) gives `target` protection from `c`.
fn mother_of_runes(t: &mut TestGame, p: PlayerId, target: ObjectId, c: Color) {
    let mother = t.battlefield(p, "Mother of Runes");
    choose_color(t, p, c);
    t.activate(p, mother, 0, &[Entity::Object(target)]).unwrap();
    t.resolve_all();
}

#[test]
fn protection_from_a_color_stops_spells_and_abilities_with_it() {
    cr!("702.16", "702.16a", "702.16b");
    assert_supported("White Knight");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "White Knight");
    // Black spells and abilities from black sources can't target it — whoever controls
    // them.
    assert!(!spell_can_target(&mut t, P1, "Disfigure", knight));
    assert!(!spell_can_target(&mut t, P0, "Disfigure", knight));
    let assassin = t.battlefield(P1, "Royal Assassin");
    t.g.tap(knight);
    assert!(!ability_can_target(&mut t, assassin, 0, knight));
    // Others can.
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", knight));
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(ability_can_target(&mut t, pyromancer, 0, knight));
}

#[test]
fn protection_from_a_card_type_covers_permanents_and_other_zones() {
    cr!("702.16a", "702.16b", "702.16e");
    ruling!(
        "Petrified Wood-Kin",
        "\"Protection from instants\" means instant spells can't target it"
    );
    ruling!(
        "Petrified Wood-Kin",
        "It can be targeted by instant spells or abilities of instant cards while it's on the stack"
    );
    let mut t = TestGame::new(2);
    let woodkin = t.hand(P0, "Petrified Wood-Kin");
    t.lands(P0, "Forest", 7);
    let spell = t.cast(P0, woodkin).go();
    // On the stack it doesn't have protection yet (it works only on the battlefield).
    assert!(spell_targets(&mut t, P1, "Counterspell").contains(&Entity::Object(spell)));
    t.resolve_all();
    let woodkin = t.g.current(spell);
    assert!(t.on_battlefield(woodkin));
    // Instant spells (sources that aren't permanents) can't target it; sorceries can.
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", woodkin));
    assert!(spell_can_target(&mut t, P1, "Flame Slash", woodkin));
    // Damage from an instant is prevented (e.g. an instant that doesn't target it).
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.deal_damage(bolt, Entity::Object(woodkin), 3, false);
    assert_eq!(t.obj_now(woodkin).damage, 0);
    // Protection from artifacts: artifact permanents' abilities can't target it and
    // artifact creatures can't block it.
    let mut t = TestGame::new(2);
    let chosen = t.battlefield(P0, "Tel-Jilad Chosen");
    let rod = t.battlefield(P1, "Rod of Ruin");
    assert!(!ability_can_target(&mut t, rod, 0, chosen));
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(chosen, Entity::Player(P1))]);
    assert!(!t.g.can_block(thopter, chosen));
    assert!(t.g.can_block(bears, chosen));
    // Protection from creatures.
    let mut t = TestGame::new(2);
    let chaplain = t.battlefield(P0, "Beloved Chaplain");
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(!ability_can_target(&mut t, pyromancer, 0, chaplain));
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", chaplain));
}

#[test]
fn protection_from_a_subtype_supertype_or_mana_value() {
    cr!("702.16a", "702.16f");
    ruling!(
        "Mistmeadow Skulk",
        "Mistmeadow Skulk can't be blocked by creatures with mana value 3 or greater."
    );
    // Protection from Goblins.
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Warren-Scourge Elf");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let fanatic = t.battlefield(P1, "Mogg Fanatic");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!ability_can_target(&mut t, fanatic, 0, elf));
    attack_with(&mut t, &[(elf, Entity::Player(P1))]);
    assert!(!t.g.can_block(goblin, elf));
    assert!(t.g.can_block(bears, elf));
    // Protection from snow.
    let mut t = TestGame::new(2);
    let hulk = t.battlefield(P0, "Ronom Hulk");
    let snow = t.battlefield(P1, "Snow-Covered Swamp");
    let swamp = t.battlefield(P1, "Swamp");
    t.g.deal_damage(snow, Entity::Object(hulk), 2, false);
    assert_eq!(t.obj_now(hulk).damage, 0);
    t.g.deal_damage(swamp, Entity::Object(hulk), 2, false);
    assert_eq!(t.obj_now(hulk).damage, 2);
    // Protection from mana value 3 or greater.
    let mut t = TestGame::new(2);
    let skulk = t.battlefield(P0, "Mistmeadow Skulk");
    assert!(!spell_can_target(&mut t, P1, "Murder", skulk));
    assert!(spell_can_target(&mut t, P1, "Disfigure", skulk));
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(skulk, Entity::Player(P1))]);
    assert!(!t.g.can_block(giant, skulk));
    assert!(t.g.can_block(bears, skulk));
}

#[test]
fn protection_from_a_card_name_only_when_it_says_name() {
    cr!("702.16a", "702.16b");
    ruling!(
        "Runed Halo",
        "Runed Halo can't protect you from tokens unless those tokens have the same name"
    );
    assert_supported("Runed Halo");
    let mut t = TestGame::new(2);
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Lightning Bolt".to_string()),
    );
    t.enter(P0, "Runed Halo");
    // "You have protection from the chosen card name."
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", P0));
    assert!(spell_can_target(&mut t, P1, "Shock", P0));
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.deal_damage(bolt, Entity::Player(P0), 3, false);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn protection_for_players_stops_targeting() {
    cr!("702.16b");
    assert_supported("Absolute Virtue");
    let mut t = TestGame::new(2);
    // "You have protection from each of your opponents."
    t.battlefield(P0, "Absolute Virtue");
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", P0));
    assert!(!ability_can_target(&mut t, pyromancer, 0, P0));
    assert!(spell_can_target(&mut t, P0, "Lightning Bolt", P0));
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", P1));
}

#[test]
fn protection_keeps_auras_with_the_quality_off() {
    cr!("702.16c", "704.5m");
    assert_supported("Black Knight");
    let mut t = TestGame::new(2);
    // A white Aura can't enchant a creature with protection from white.
    let knight = t.battlefield(P0, "Black Knight");
    assert!(!spell_can_target(&mut t, P1, "Pacifism", knight));
    let pacifism = t.battlefield(P1, "Pacifism");
    assert!(!t.g.attach(pacifism, Entity::Object(knight)));
    // One already attached is put into its owner's graveyard when the creature gains
    // protection from its color.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(pacifism, Entity::Object(bears)));
    mother_of_runes(&mut t, P0, bears, Color::White);
    assert!(t.in_graveyard(P1, "Pacifism"));
    // A player with protection can't be enchanted by such Auras either.
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P1, "Curse of the Pierced Heart");
    assert!(t.g.attach(curse, Entity::Player(P0)));
    t.battlefield(P0, "Absolute Virtue");
    t.settle();
    assert!(t.in_graveyard(P1, "Curse of the Pierced Heart"));
}

#[test]
fn protection_keeps_equipment_with_the_quality_off() {
    cr!("702.16d", "704.5n");
    assert_supported("Maul of the Skyclaves");
    let mut t = TestGame::new(2);
    // Maul of the Skyclaves is a white Equipment.
    let knight = t.battlefield(P0, "Black Knight");
    let maul = t.battlefield(P0, "Maul of the Skyclaves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let equip = t
        .obj_now(maul)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
        - 1;
    assert!(!ability_can_target(&mut t, maul, equip, knight));
    assert!(ability_can_target(&mut t, maul, equip, bears));
    assert!(!t.g.attach(maul, Entity::Object(knight)));
    // Attached, then the creature gains protection from white: it becomes unattached but
    // stays on the battlefield.
    assert!(t.g.attach(maul, Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    mother_of_runes(&mut t, P0, bears, Color::White);
    assert!(t.on_battlefield(maul));
    assert_eq!(t.obj_now(maul).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
    // Protection from artifacts: no Equipment can be attached.
    let chosen = t.battlefield(P0, "Tel-Jilad Chosen");
    let spear = t.battlefield(P0, "Shadowspear");
    assert!(!t.g.attach(spear, Entity::Object(chosen)));
}

#[test]
fn protection_prevents_damage_from_sources_with_the_quality() {
    cr!("702.16e", "615.1");
    assert_supported("Pestilence");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "White Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Pestilence (black): "{B}: This enchantment deals 1 damage to each creature and
    // each player."
    let pestilence = t.battlefield(P1, "Pestilence");
    t.lands(P1, "Swamp", 1);
    t.activate(P1, pestilence, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(knight).damage, 0);
    assert_eq!(t.obj_now(bears).damage, 1);
    assert_eq!(t.life(P0), 19);
    // Blocking a black creature: it deals no damage to the Knight.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "White Knight");
    let wraith = t.battlefield(P1, "Bog Wraith");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(wraith, Entity::Player(P0))], &[(knight, wraith)]);
    assert!(t.on_battlefield(knight));
    assert_eq!(t.obj_now(knight).damage, 0);
    assert_eq!(t.obj_now(wraith).damage, 2);
}

#[test]
fn protection_damage_prevention_respects_damage_that_cant_be_prevented() {
    cr!("702.16e", "615.12");
    ruling!(
        "The One Ring",
        "some damage can't be prevented. In this case, that damage reduces your life total as normal"
    );
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "White Knight");
    t.battlefield(P0, "Absolute Virtue");
    let wraith = t.battlefield(P1, "Bog Wraith");
    t.custom(
        P1,
        CardDef::custom(mtg_engine::object::Characteristics {
            name: "Unpreventable".into(),
            card_types: CardTypeSet::single(CardType::Enchantment),
            abilities: vec![AbilityDef::new(
                AbilityKind::Static(StaticAbility::new(StaticEffect::Restriction(
                    Restriction::DamageCantBePrevented,
                ))),
                "Damage can't be prevented.",
            )],
            ..Default::default()
        }),
        Zone::Battlefield,
    );
    t.g.deal_damage(wraith, Entity::Object(knight), 1, false);
    t.g.deal_damage(wraith, Entity::Player(P0), 2, false);
    assert_eq!(t.obj_now(knight).damage, 1);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn attacking_creatures_with_protection_cant_be_blocked_by_the_quality() {
    cr!("702.16f");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "White Knight");
    let wraith = t.battlefield(P1, "Bog Wraith");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(knight, Entity::Player(P1))]);
    assert!(!t.g.can_block(wraith, knight));
    assert!(t.g.can_block(bears, knight));
    block_and_finish(&mut t, P1, &[(wraith, knight)]);
    assert!(!is_blocking(&t, wraith));
    assert_eq!(t.life(P1), 18);
    // A creature with protection can still block creatures with the quality.
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "White Knight");
    let wraith = t.battlefield(P1, "Bog Wraith");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(wraith, Entity::Player(P0))]);
    assert!(t.g.can_block(knight, wraith));
}

#[test]
fn protection_from_two_qualities_is_two_abilities() {
    cr!("702.16g");
    assert_supported("Mask of Law and Grace");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Sword of War and Peace: "Equipped creature gets +2/+2 and has protection from red
    // and from white."
    let sword = t.battlefield(P0, "Sword of War and Peace");
    t.g.attach(sword, Entity::Object(bears));
    t.g.recompute();
    assert_eq!(keyword_count(&t, bears, KeywordKind::Protection), 2);
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", bears));
    assert!(!spell_can_target(&mut t, P1, "Swords to Plowshares", bears));
    assert!(!spell_can_target(&mut t, P1, "Pacifism", bears));
    assert!(spell_can_target(&mut t, P1, "Disfigure", bears));
    // A keyword line: "Protection from blue, from black, and from red".
    let oversoul = t.battlefield(P0, "Oversoul of Dusk");
    assert_eq!(keyword_count(&t, oversoul, KeywordKind::Protection), 3);
    assert!(!spell_can_target(&mut t, P1, "Disfigure", oversoul));
    assert!(spell_can_target(&mut t, P1, "Swords to Plowshares", oversoul));
}

#[test]
fn protection_from_each_color_is_one_ability_per_color() {
    cr!("702.16h");
    ruling!(
        "Iridescent Angel",
        "This card has Protection from Black, from Blue, from Red, from White, and from Green."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Iridescent Angel");
    assert_eq!(keyword_count(&t, angel, KeywordKind::Protection), 5);
    for spell in ["Swords to Plowshares", "Disfigure", "Lightning Bolt", "Giant Growth"] {
        assert!(!spell_can_target(&mut t, P1, spell, angel), "{spell}");
    }
    let rod = t.battlefield(P1, "Rod of Ruin");
    assert!(ability_can_target(&mut t, rod, 0, angel));
}

#[test]
fn protection_from_each_of_a_set_of_players() {
    cr!("702.16i", "702.16k");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Absolute Virtue");
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", P0));
    assert!(!spell_can_target(&mut t, P2, "Lightning Bolt", P0));
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    t.g.deal_damage(b1, Entity::Player(P0), 2, false);
    t.g.deal_damage(b2, Entity::Player(P0), 2, false);
    assert_eq!(t.life(P0), 20);
    // Not from its controller's own objects.
    let own = t.battlefield(P0, "Grizzly Bears");
    t.g.deal_damage(own, Entity::Player(P0), 2, false);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn protection_from_everything() {
    cr!("702.16j");
    ruling!(
        "Progenitus",
        "Progenitus can't be blocked, Progenitus can't be enchanted or equipped, Progenitus can't be the target of spells or abilities, and all damage that would be dealt to Progenitus is prevented."
    );
    ruling!(
        "Progenitus",
        "Progenitus can still be affected by effects that don't target it or deal damage to it"
    );
    let mut t = TestGame::new(2);
    let progenitus = t.battlefield(P0, "Progenitus");
    // Not even its controller's spells, Auras, or Equipment.
    assert!(!spell_can_target(&mut t, P0, "Giant Growth", progenitus));
    assert!(!spell_can_target(&mut t, P0, "Holy Strength", progenitus));
    let spear = t.battlefield(P0, "Shadowspear");
    assert!(!t.g.attach(spear, Entity::Object(progenitus)));
    let rod = t.battlefield(P1, "Rod of Ruin");
    assert!(!ability_can_target(&mut t, rod, 0, progenitus));
    // All damage is prevented.
    t.lands(P0, "Mountain", 2);
    let pyroclasm = t.hand(P0, "Pyroclasm");
    t.cast(P0, pyroclasm).go();
    t.resolve_all();
    assert_eq!(t.obj_now(progenitus).damage, 0);
    // No creature can block it.
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(progenitus, Entity::Player(P1))]);
    assert!(!t.g.can_block(thopter, progenitus));
    assert!(!t.g.can_block(bears, progenitus));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 10);
    // Effects that don't target it or damage it still affect it.
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve_all();
    assert!(!t.on_battlefield(progenitus));
}

#[test]
fn player_protection_from_everything() {
    cr!("702.16j", "702.16b", "702.16c", "702.16e");
    ruling!(
        "The One Ring",
        "If a player has protection from everything, it means three things"
    );
    ruling!(
        "The One Ring",
        "Creatures can still attack you while you have protection from everything, although combat damage that they would deal to you will be prevented."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // "When The One Ring enters, if you cast it, you gain protection from everything
    // until your next turn."
    t.lands(P0, "Wastes", 4);
    let ring = t.hand(P0, "The One Ring");
    t.cast(P0, ring).go();
    t.resolve_all();
    // It can't be targeted, even by its own spells...
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", P0));
    assert!(!spell_can_target(&mut t, P0, "Lightning Bolt", P0));
    // ...can't be enchanted...
    let curse = t.battlefield(P1, "Curse of the Pierced Heart");
    assert!(!t.g.attach(curse, Entity::Player(P0)));
    // ...and all damage to it is prevented. Creatures can still attack it.
    let own = t.battlefield(P0, "Hill Giant");
    t.g.deal_damage(own, Entity::Player(P0), 3, false);
    assert_eq!(t.life(P0), 20);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    assert!(t.g.history.attackers.contains(&t.g.current(bears)));
    assert_eq!(t.life(P0), 20);
    // Until the player's next turn.
    t.advance_to(P0, Step::Upkeep);
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", P0));
}

#[test]
fn protection_from_a_player_covers_objects_they_control_or_own() {
    cr!("702.16k");
    ruling!(
        "Absolute Virtue",
        "If an object has no controller (such as a card in a graveyard), its owner is considered its controller for this purpose."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Absolute Virtue");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    // A card in the opponent's graveyard is treated as controlled by its owner.
    let card = t.graveyard(P1, "Hill Giant");
    t.g.deal_damage(card, Entity::Player(P0), 3, false);
    assert_eq!(t.life(P0), 20);
    // Their Auras can't enchant the player.
    let curse = t.battlefield(P1, "Curse of the Pierced Heart");
    assert!(!t.g.attach(curse, Entity::Player(P0)));
}

#[test]
fn multiple_instances_of_protection_from_the_same_quality_are_redundant() {
    cr!("702.16m");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "White Knight");
    // Mask of Law and Grace: "Enchanted creature has protection from black and from red."
    let mask = t.battlefield(P0, "Mask of Law and Grace");
    t.g.attach(mask, Entity::Object(knight));
    t.g.recompute();
    assert_eq!(keyword_count(&t, knight, KeywordKind::Protection), 3);
    assert!(!spell_can_target(&mut t, P1, "Disfigure", knight));
    let wraith = t.battlefield(P1, "Bog Wraith");
    t.g.deal_damage(wraith, Entity::Object(knight), 2, false);
    assert_eq!(t.obj_now(knight).damage, 0);
    // Without the Mask, the printed instance still protects it.
    let m = t.g.current(mask);
    t.g.unattach(m);
    t.settle();
    assert_eq!(keyword_count(&t, knight, KeywordKind::Protection), 1);
    assert!(!spell_can_target(&mut t, P1, "Disfigure", knight));
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", knight));
}

#[test]
fn protection_that_doesnt_remove_auras() {
    cr!("702.16n", "702.16c");
    ruling!(
        "Spectra Ward",
        "The protection granted by Spectra Ward won’t cause any Aura to be put into its owner’s graveyard, including Spectra Ward itself."
    );
    ruling!(
        "Spectra Ward",
        "if the enchanted creature gains protection from white in another way, Spectra Ward will be put into its owner’s graveyard"
    );
    ruling!(
        "Spectra Ward",
        "the enchanted creature can’t be the target of further Aura spells that have one or more colors"
    );
    assert_supported("Spectra Ward");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pacifism = t.battlefield(P1, "Pacifism");
    t.g.attach(pacifism, Entity::Object(bears));
    t.lands(P0, "Plains", 5);
    let ward = t.hand(P0, "Spectra Ward");
    t.cast(P0, ward).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(keyword_count(&t, bears, KeywordKind::Protection), 5);
    // Neither Pacifism nor Spectra Ward (both white) falls off.
    assert!(t.on_battlefield(pacifism));
    assert!(!t.in_graveyard(P0, "Spectra Ward"));
    // Further colored Aura spells can't target it.
    assert!(!spell_can_target(&mut t, P1, "Pacifism", bears));
    // Protection from white from another source removes white Auras: Sword of War and
    // Peace (colorless) gives protection from red and from white.
    let sword = t.battlefield(P0, "Sword of War and Peace");
    assert!(t.g.attach(sword, Entity::Object(bears)));
    t.settle();
    assert!(t.in_graveyard(P0, "Spectra Ward"));
    assert!(t.in_graveyard(P1, "Pacifism"));
}

#[test]
fn protection_that_doesnt_remove_what_is_already_attached() {
    cr!("702.16p", "702.16c", "702.16d");
    assert_supported("Benevolent Blessing");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let maul = t.battlefield(P0, "Maul of the Skyclaves");
    t.g.attach(maul, Entity::Object(bears));
    let strength = t.battlefield(P0, "Holy Strength");
    t.g.attach(strength, Entity::Object(bears));
    let pacifism = t.battlefield(P1, "Pacifism");
    t.g.attach(pacifism, Entity::Object(bears));
    // Benevolent Blessing (flash), choosing white: "Enchanted creature has protection
    // from the chosen color. This effect doesn't remove Auras and Equipment you control
    // that are already attached to it."
    t.lands(P0, "Plains", 2);
    let blessing = t.hand(P0, "Benevolent Blessing");
    choose_color(&mut t, P0, Color::White);
    t.cast(P0, blessing).target(bears).go();
    t.resolve_all();
    assert!(t.g.protected_from(t.g.current(bears), strength));
    // The white Auras and Equipment its controller controls stay, including the
    // Blessing itself; the opponent's Pacifism doesn't.
    assert_eq!(t.obj_now(maul).attached_to, Some(Entity::Object(bears)));
    assert_eq!(
        t.obj_now(strength).attached_to,
        Some(Entity::Object(bears))
    );
    let blessing_perm = t.named_on_battlefield("Benevolent Blessing");
    assert_eq!(blessing_perm.len(), 1);
    assert!(t.in_graveyard(P1, "Pacifism"));
    // Other white permanents can't become attached.
    let strength2 = t.battlefield(P0, "Holy Strength");
    assert!(!t.g.attach(strength2, Entity::Object(bears)));
    assert!(!spell_can_target(&mut t, P0, "Holy Strength", bears));
    // Unattached and reattached: no longer "already attached".
    t.g.unattach(maul);
    assert!(!t.g.attach(maul, Entity::Object(bears)));
    // Another instance of protection from white affects them as normal.
    let sword = t.battlefield(P0, "Sword of War and Peace");
    assert!(t.g.attach(sword, Entity::Object(bears)));
    t.settle();
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert!(t.in_graveyard(P0, "Benevolent Blessing"));
}

#[test]
fn losing_protection_removes_every_protection_ability() {
    cr!("702.16b", "702.16h");
    assert_supported("Shay Cormac");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "White Knight");
    let angel = t.battlefield(P0, "Iridescent Angel");
    // "{1}: Permanents your opponents control lose hexproof, indestructible, protection,
    // shroud, and ward until end of turn."
    let shay = t.battlefield(P1, "Shay Cormac");
    t.lands(P1, "Wastes", 1);
    t.activate(P1, shay, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(keyword_count(&t, knight, KeywordKind::Protection), 0);
    assert_eq!(keyword_count(&t, angel, KeywordKind::Protection), 0);
    assert!(spell_can_target(&mut t, P1, "Disfigure", knight));
    assert!(spell_can_target(&mut t, P1, "Lightning Bolt", angel));
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(!spell_can_target(&mut t, P1, "Disfigure", knight));
}
