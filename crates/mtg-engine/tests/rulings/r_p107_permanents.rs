//! Rulings batch P107 — triggers and characteristic changes of permanents: cast triggers
//! that resolve before their spell, "if it was a creature" leave-the-battlefield triggers,
//! type-changing statics that work only on the battlefield, turning face up and face
//! down, the legend rule, hellbent damage, and split second from a kicked spell.

use crate::r_p107_common::*;
use crate::r_s01_common::{attack_with, triggers_on_stack};
use crate::r_s02_common::create_token;
use crate::r_s06_common::damage;
use crate::r_s11_common::turn_face_up;
use crate::r_s12_common::morph;
use crate::r_s14_common::triggers_from;
use crate::r_s17_common::token_copy;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 casts Counterspell targeting the spell `spell`.
fn p1_counters(t: &mut TestGame, spell: ObjectId) {
    mana(t, P1, ManaType::U, 2);
    let c = t.hand(P1, "Counterspell");
    t.cast_with(P1, c, &[obj(spell)]).unwrap();
}

#[test]
fn tome_of_the_guildpact_draws_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Tome of the Guildpact",
        "Tome of the Guildpact’s ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tome of the Guildpact");
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::W, 1);
    let hand = t.hand_size(P0);
    let helix = cast_from_hand(&mut t, P0, "Lightning Helix", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    // Countered in response: the trigger (on top) still resolves first.
    p1_counters(&mut t, helix);
    t.resolve(); // Counterspell
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn rosnakhts_heroic_token_comes_even_if_the_spell_is_countered() {
    cr!("603.3", "702.37a");
    ruling!(
        "Rosnakht, Heir of Rohgahh",
        "Rosnakht, Heir of Rohgahh's last ability goes on the stack above the spell that caused it to trigger and resolves first. It resolves even if that spell is countered or otherwise left the stack."
    );
    let mut t = TestGame::new(2);
    let ros = t.battlefield(P0, "Rosnakht, Heir of Rohgahh");
    mana(&mut t, P0, ManaType::G, 1);
    let growth = cast_from_hand(&mut t, P0, "Giant Growth", &[obj(ros)]);
    t.settle();
    // The trigger is above the spell.
    assert_eq!(t.stack_len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), growth);
    p1_counters(&mut t, growth);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Kobolds of Kher Keep").len(), 1);
    assert!(t.in_graveyard(P0, "Giant Growth"));
}

#[test]
fn thorough_investigation_triggers_once_per_attack() {
    cr!("508.3a", "603.2c");
    ruling!(
        "Thorough Investigation",
        "Thorough Investigation's first ability can trigger only once per combat, no matter how many creatures you attack with."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thorough Investigation");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(
        &mut t,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Clue").len(), 1);
}

#[test]
fn eloise_surveils_for_any_sacrificed_token_but_not_the_legend_rule() {
    cr!("701.21a", "704.5j");
    ruling!(
        "Eloise, Nephalia Sleuth",
        "Eloise's last ability triggers whenever you sacrifice any token for any reason, not just whenever you sacrifice Clue tokens."
    );
    ruling!(
        "Eloise, Nephalia Sleuth",
        "This is not the same as sacrificing it. Putting the token into the graveyard won't cause nontoken Eloise's last ability to trigger."
    );
    let mut t = TestGame::new(2);
    let eloise = t.battlefield(P0, "Eloise, Nephalia Sleuth");
    let treasure = create_token(&mut t, P0, "Treasure");
    assert!(tap_for_mana(&mut t, P0, treasure, "Sacrifice"));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "surveil"), 1);
    t.resolve_all();
    // A token copy: the legend rule puts one into the graveyard; no surveil trigger.
    token_copy(&mut t, P0, eloise);
    assert_eq!(t.named_on_battlefield("Eloise, Nephalia Sleuth").len(), 1);
    assert_eq!(triggers_on_stack(&t, "surveil"), 0);
}

#[test]
fn weatherseed_totem_returns_only_if_it_was_a_creature() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Weatherseed Totem",
        "Weatherseed Totem will be returned to its owner's hand if it was a creature at the time it was put into a graveyard from the battlefield."
    );
    let mut t = TestGame::new(2);
    let totem = t.battlefield(P0, "Weatherseed Totem");
    destroy(&mut t, totem);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Weatherseed Totem"));
    let mut t = TestGame::new(2);
    let totem = t.battlefield(P0, "Weatherseed Totem");
    mana(&mut t, P0, ManaType::G, 3);
    mana(&mut t, P0, ManaType::C, 2);
    act(&mut t, P0, totem, "Treefolk", &[]).unwrap();
    t.resolve_all();
    assert!(is_creature(&t, totem));
    destroy(&mut t, totem);
    t.resolve_all();
    assert!(t.in_hand(P0, "Weatherseed Totem"));
}

#[test]
fn the_warring_triad_is_a_creature_off_the_battlefield_and_enters_as_one_only_with_eight_cards() {
    cr!("611.3a", "603.6a", "113.6");
    ruling!(
        "The Warring Triad",
        "The type-changing ability that can make The Warring Triad not be a creature functions only on the battlefield."
    );
    ruling!(
        "The Warring Triad",
        "When The Warring Triad enters the battlefield, the number of cards in your graveyard will determine if a creature entered the battlefield or not"
    );
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    let triad = t.hand(P0, "The Warring Triad");
    assert!(is_creature(&t, triad));
    let gy = t.graveyard(P0, "The Warring Triad");
    assert!(is_creature(&t, gy));
    mana(&mut t, P0, ManaType::C, 3);
    let spell = t.cast_with(P0, triad, &[]).unwrap();
    assert!(t.obj(spell).is(CardType::Creature));
    t.resolve_all();
    let perm = t.g.current(spell);
    assert!(t.on_battlefield(perm) && !is_creature(&t, perm));
    assert_eq!(t.life(P0), 20);
    // With eight cards in the graveyard, it enters as a creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    for _ in 0..8 {
        t.graveyard(P0, "Plains");
    }
    let triad = t.enter(P0, "The Warring Triad");
    t.resolve_all();
    assert!(is_creature(&t, triad));
    assert_eq!(t.life(P0), 21);
}

#[test]
fn zoetic_cavern_turned_face_up_sheds_auras_and_equipment_but_keeps_counters() {
    cr!("708.8", "704.5m", "704.5n");
    ruling!(
        "Zoetic Cavern",
        "When Zoetic Cavern is turned face up, any Auras on it that can't legally enchant it will be put into their owners' graveyards."
    );
    let mut t = TestGame::new(2);
    let cavern = morph(&mut t, P0, "Zoetic Cavern");
    let pacifism = attach_new(&mut t, P1, "Pacifism", cavern);
    let sword = attach_new(&mut t, P0, "Bonesplitter", cavern);
    put_counters(&mut t, cavern, counters::PLUS1, 1);
    mana(&mut t, P0, ManaType::C, 2);
    assert!(turn_face_up(&mut t, P0, cavern));
    t.settle();
    assert!(t.obj_now(cavern).is(CardType::Land));
    assert!(!is_creature(&t, cavern));
    assert!(t.in_graveyard(P1, "Pacifism"));
    assert!(!t.g.is_live(pacifism));
    assert!(t.on_battlefield(sword));
    assert_eq!(t.obj_now(sword).attached_to, None);
    assert_eq!(t.counters(cavern, counters::PLUS1), 1);
}

#[test]
fn maelstrom_djinn_turned_face_up_twice_has_two_vanishings() {
    cr!("702.63a", "708.2", "113.2c");
    ruling!(
        "Maelstrom Djinn",
        "If Maelstrom Djinn turns face up and then is somehow turned face down, it will still have vanishing and time counters."
    );
    let mut t = TestGame::new(2);
    let djinn = morph(&mut t, P0, "Maelstrom Djinn");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    assert!(turn_face_up(&mut t, P0, djinn));
    t.resolve_all();
    assert_eq!(t.counters(djinn, counters::TIME), 2);
    assert!(has_kw(&t, djinn, KeywordKind::Vanishing));
    // Turned face down: a 2/2 with no printed abilities, but the effect that gave it
    // vanishing still applies, and its counters stay.
    let now = t.g.current(djinn);
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, now));
    t.g.recompute();
    assert!(t.obj_now(djinn).face_down);
    assert!(!has_kw(&t, djinn, KeywordKind::Flying));
    assert!(has_kw(&t, djinn, KeywordKind::Vanishing));
    assert_eq!(t.counters(djinn, counters::TIME), 2);
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    assert!(turn_face_up(&mut t, P0, djinn));
    t.resolve_all();
    assert_eq!(t.counters(djinn, counters::TIME), 4);
    // Both vanishing abilities trigger in its controller's upkeep.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_from(&t, t.g.current(djinn)), 2);
    t.resolve_all();
    assert_eq!(t.counters(djinn, counters::TIME), 2);
}

#[test]
fn mirror_gallery_turns_off_the_legend_rule_until_it_leaves() {
    cr!("704.5j", "704.3");
    ruling!(
        "Mirror Gallery",
        "If all Mirror Galleries leave the battlefield, the “Legend rule” will apply the next time state-based actions are performed."
    );
    let mut t = TestGame::new(2);
    let gallery = t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.settle();
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 2);
    destroy(&mut t, gallery);
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
}

#[test]
fn gathan_raiders_dies_when_a_card_reaches_an_empty_hand() {
    cr!("120.6", "704.5g");
    ruling!(
        "Gathan Raiders",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Gathan Raiders while your hand is empty may become lethal"
    );
    let mut t = TestGame::new(2);
    let raiders = t.battlefield(P0, "Gathan Raiders");
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.pt(raiders), (5, 5));
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 3, raiders);
    assert!(t.on_battlefield(raiders));
    t.hand(P0, "Plains");
    t.settle();
    assert!(!t.on_battlefield(raiders));
    assert!(t.in_graveyard(P0, "Gathan Raiders"));
}

#[test]
fn kicked_molten_disaster_has_split_second() {
    cr!("702.61a", "702.61b", "702.33d");
    ruling!(
        "Molten Disaster",
        "If Molten Disaster was kicked, Molten Disaster has split second as long as it's on the stack."
    );
    for kicked in [true, false] {
        let mut t = TestGame::new(2);
        mana(&mut t, P0, ManaType::R, 3);
        mana(&mut t, P0, ManaType::C, 2);
        let card = t.hand(P0, "Molten Disaster");
        let bear = t.battlefield(P1, "Grizzly Bears");
        t.cast(P0, card).x(2).kicked(kicked).go();
        t.g.recompute();
        mana(&mut t, P1, ManaType::G, 1);
        let growth = t.hand(P1, "Giant Growth");
        let r = t.cast_with(P1, growth, &[obj(bear)]);
        assert_eq!(r.is_ok(), !kicked, "kicked: {kicked}");
        t.resolve_all();
        assert_eq!(t.life(P1), 18);
        assert_eq!(t.on_battlefield(bear), !kicked);
    }
}

#[test]
fn summon_ixion_gone_before_chapter_one_resolves_exiles_nothing() {
    cr!("610.3c", "714.2b");
    ruling!(
        "Summon: Ixion",
        "If Summon: Ixion leaves the battlefield before its first chapter ability resolves, the target permanent won't be exiled."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    let ixion = t.enter(P0, "Summon: Ixion");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, ixion);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.zone(giant), Zone::Battlefield);
}

#[test]
fn red_herring_doesnt_attack_when_it_cant() {
    cr!("508.1d");
    ruling!(
        "Red Herring",
        "If Red Herring can't attack for any reason (such as being tapped), then it doesn't attack."
    );
    // Able to attack, it must (so the cases below aren't vacuous).
    let mut t = TestGame::new(2);
    let herring = t.battlefield(P0, "Red Herring");
    t.advance_to_step(Step::DeclareBlockers);
    assert!(t
        .g
        .combat
        .as_ref()
        .is_some_and(|c| c.attackers.iter().any(|a| a.id == herring)));
    // Tapped, and (in another game) with Ghostly Prison's {2} attack tax unpaid.
    let mut t = TestGame::new(2);
    let herring = t.battlefield(P0, "Red Herring");
    t.g.tap(herring);
    t.advance_to_step(Step::EndOfCombat);
    assert!(t.g.combat.as_ref().is_none_or(|c| c.attackers.is_empty()));
    assert_eq!(t.life(P1), 20);
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ghostly Prison");
    t.battlefield(P0, "Red Herring");
    t.lands(P0, "Mountain", 2);
    t.advance_to_step(Step::EndOfCombat);
    assert!(t.g.combat.as_ref().is_none_or(|c| c.attackers.is_empty()));
    assert_eq!(t.life(P1), 20);
}
