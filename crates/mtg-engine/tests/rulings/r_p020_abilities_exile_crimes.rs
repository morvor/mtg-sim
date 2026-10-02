//! Rulings batch P020 — the other rulings of the batch's cards: countering activated and
//! loyalty abilities (CR 602.1, 605.1a, 606.3, 701.6a); "exile ... until" a permanent
//! leaves (CR 610.3, 400.7); crimes are committed by the initial targets only (CR 700.13,
//! 115.7); Glimmervoid Basin's copies and token copies (CR 707.10d, 707.2); triggers from
//! permanents turned face up with Fractured Realm (CR 603.2, 708.8); Tawnos's token copies
//! (CR 707.2, 603.6a).

use crate::r_p020_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::attach_new;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 activates Bonesplitter's equip ability targeting Grizzly Bears (in P0's main phase).
/// Returns (Bonesplitter, Bears, the ability on the stack).
fn equip_on_stack(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    supported("Bonesplitter");
    t.set_step(P0, Step::PrecombatMain);
    let blade = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    let ab = t
        .activate(P0, blade, 0, &[Entity::Object(bears)])
        .expect("equip")
        .expect("equip uses the stack");
    (blade, bears, ab)
}

#[test]
fn sublime_epiphany_counters_an_equip_ability() {
    cr!("602.1", "702.6a", "701.6a");
    ruling!(
        "Sublime Epiphany",
        "Activated abilities are written in the form \"[Cost]: [Effect].\" Some keyword abilities (such as equip) are activated abilities and will have colons in their reminder texts."
    );
    supported("Sublime Epiphany");
    let mut t = TestGame::new(2);
    let (blade, bears, ab) = equip_on_stack(&mut t);
    lands_for_cost(&mut t, P1, "Sublime Epiphany");
    let se = t.hand(P1, "Sublime Epiphany");
    // Mode 2: "Counter target activated or triggered ability."
    t.cast(P1, se).modes(&[1]).target(ab).go();
    t.resolve_all();
    assert!(t.g.stack.is_empty());
    assert_eq!(t.obj_now(blade).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn voidslime_counters_equip_and_loyalty_abilities() {
    cr!("602.1", "702.6a", "606.3", "701.6a", "701.6b");
    ruling!(
        "Voidslime",
        "Activated abilities are written in the form \"[Cost]: [Effect].\" Some keyword abilities, such as equip, are activated abilities and will have colons in their reminder texts. Loyalty abilities of planeswalkers are activated abilities."
    );
    supported("Voidslime");
    let mut t = TestGame::new(2);
    let (blade, _, ab) = equip_on_stack(&mut t);
    cast_new(&mut t, P1, "Voidslime", &[Entity::Object(ab)]);
    t.resolve_all();
    assert_eq!(t.obj_now(blade).attached_to, None);
    // Chandra's "+1: Add {R}{R}." is a loyalty ability: an activated ability Voidslime can
    // counter. Its cost (the loyalty counter) isn't refunded.
    let chandra = t.battlefield(P0, "Chandra, Torch of Defiance");
    let loyalty = t.counters(chandra, counters::LOYALTY);
    let ab = crate::r_s06_common::activate_containing(&mut t, P0, chandra, "{R}{R}")
        .unwrap()
        .expect("on the stack");
    assert_eq!(t.counters(chandra, counters::LOYALTY), loyalty + 1);
    cast_new(&mut t, P1, "Voidslime", &[Entity::Object(ab)]);
    t.resolve();
    assert!(t.g.stack.is_empty());
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
    assert_eq!(t.counters(chandra, counters::LOYALTY), loyalty + 1);
}

#[test]
fn voidslime_cant_target_a_mana_ability_but_a_loyalty_ability_that_adds_mana_isnt_one() {
    cr!("605.1a", "605.3a", "606.3");
    ruling!(
        "Voidslime",
        "An activated mana ability is one that adds mana as it resolves, doesn't have a target, and isn't a loyalty ability."
    );
    supported("Voidslime");
    let mut t = TestGame::new(2);
    // Llanowar Elves's mana ability doesn't use the stack: nothing to target.
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert_eq!(t.activate(P0, elves, 0, &[]).unwrap(), None);
    assert!(t.g.stack.is_empty());
    // Chandra's "+1: Add {R}{R}." adds mana but is a loyalty ability: it uses the stack and
    // can be countered.
    let chandra = t.battlefield(P0, "Chandra, Torch of Defiance");
    let ab = crate::r_s06_common::activate_containing(&mut t, P0, chandra, "{R}{R}")
        .unwrap()
        .expect("on the stack");
    cast_new(&mut t, P1, "Voidslime", &[Entity::Object(ab)]);
    t.resolve();
    assert!(t.g.stack.is_empty());
    let reds = t
        .g
        .player(P0)
        .mana_pool
        .mana
        .iter()
        .filter(|m| m.ty == mtg_engine::mana::ManaType::R)
        .count();
    assert_eq!(reds, 0);
}

#[test]
fn seal_away_exiles_the_creature_and_its_return_is_a_new_object() {
    cr!("610.3", "610.3a", "400.7", "704.5m", "704.5n", "122.2");
    ruling!(
        "Seal Away",
        "Auras attached to the exiled creature will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist. When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled."
    );
    // "Flash. When this enchantment enters, exile target tapped creature an opponent
    // controls until this enchantment leaves the battlefield."
    supported("Seal Away");
    supported("Holy Strength");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let aura = attach_new(&mut t, P1, "Holy Strength", giant);
    let blade = attach_new(&mut t, P1, "Bonesplitter", giant);
    t.g.tap(giant);
    t.g.add_counters(Entity::Object(giant), counters::PLUS1, 2, None);
    t.g.recompute();
    let seal = cast_new(&mut t, P0, "Seal Away", &[]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P1, "Holy Strength"));
    assert!(!t.g.is_live(aura) || !t.on_battlefield(aura));
    assert!(t.on_battlefield(blade));
    assert_eq!(t.obj_now(blade).attached_to, None);
    // Seal Away leaves: the Giant returns as a new object, untapped, without counters or
    // attachments.
    let seal = t.g.current(seal);
    t.g.destroy(seal, None);
    t.resolve_all();
    let back = t.named_on_battlefield("Hill Giant");
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], giant);
    assert!(crate::r_s26_common::fresh(&t, back[0]));
    assert_eq!(t.pt(back[0]), (3, 3));
    assert_eq!(t.obj_now(blade).attached_to, None);
}

/// P0 controls Vadmir, New Blood ("Whenever you commit a crime, put a +1/+1 counter on
/// Vadmir. This ability triggers only once each turn.").
fn vadmir_game() -> (TestGame, ObjectId) {
    supported("Vadmir, New Blood");
    supported("Swerve");
    let mut t = TestGame::new(2);
    let vadmir = t.battlefield(P0, "Vadmir, New Blood");
    (t, vadmir)
}

#[test]
fn changing_a_spells_target_to_an_opponents_creature_isnt_a_crime() {
    cr!("700.13", "115.7");
    ruling!(
        "Vadmir, New Blood",
        "Changing the target or targets of a spell or ability won't affect whether or not the controller of that spell or ability has committed a crime. Only the initial targets chosen for that spell or ability are used to determine whether or not its controller committed a crime."
    );
    let (mut t, vadmir) = vadmir_game();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    // Giant Growth on P0's own Bears: no crime.
    let growth = cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    // Swerve ("Change the target of target spell with a single target.") targets P0's own
    // spell; it moves Giant Growth onto the opponent's Hill Giant.
    cast_new(&mut t, P0, "Swerve", &[Entity::Object(growth)]);
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.resolve_all();
    assert_eq!(t.pt(theirs), (6, 6));
    assert_eq!(t.counters(vadmir, counters::PLUS1), 0);
}

#[test]
fn changing_a_spells_target_away_from_an_opponent_doesnt_undo_the_crime() {
    cr!("700.13", "115.7");
    ruling!(
        "Bandit's Haul",
        "Only the initial targets chosen for that spell or ability are used to determine whether or not its controller committed a crime."
    );
    let (mut t, vadmir) = vadmir_game();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    // Giant Growth on the opponent's Giant: a crime as it's cast.
    let growth = cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(theirs)]);
    cast_new(&mut t, P0, "Swerve", &[Entity::Object(growth)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.counters(vadmir, counters::PLUS1), 1);
}

#[test]
fn glimmervoid_basin_ignores_what_the_spell_couldnt_target() {
    cr!("707.10d", "115.1", "702.11b");
    ruling!(
        "Glimmervoid Basin",
        "Anything that couldn't be targeted by the original spell (due to shroud, protection abilities, targeting restrictions, or any other reason) is just ignored by Glimmervoid Basin's first ability."
    );
    use crate::r_s19_common::{planechase_game, start_planar_deck};
    use crate::r_s26_common::spell_copies;
    supported("Glimmervoid Basin");
    supported("Gladecover Scout");
    supported("White Knight");
    let mut t = planechase_game(2);
    start_planar_deck(&mut t, P0, &["Glimmervoid Basin"]);
    // Hexproof and protection from black, respectively: Terror can't target them; nor
    // can it target the artifact creature Ornithopter (a targeting restriction).
    t.battlefield(P1, "Gladecover Scout");
    t.battlefield(P1, "White Knight");
    t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // Terror: "Destroy target nonartifact, nonblack creature. It can't be regenerated."
    supported("Terror");
    cast_new(&mut t, P0, "Terror", &[Entity::Object(bears)]);
    t.settle();
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(elves));
    for n in ["Gladecover Scout", "White Knight", "Ornithopter"] {
        assert_eq!(t.named_on_battlefield(n).len(), 1, "{n}");
    }
}

#[test]
fn glimmervoid_basin_chaos_tokens_copy_only_copiable_values() {
    cr!("707.2", "707.3", "311.7");
    ruling!(
        "Glimmervoid Basin",
        "As a token is created by the chaos ability, it checks the printed values of the creature it's copying, as well as any copy effects that have been applied to it. It won't copy counters on the creature, nor will it copy other effects that have changed the creature's power, toughness, types, color, and so on."
    );
    use crate::r_s19_common::{chaos, planechase_game, start_planar_deck};
    use crate::r_s26_common::{dress_up, fresh, new_tokens};
    supported("Glimmervoid Basin");
    let mut t = planechase_game(2);
    start_planar_deck(&mut t, P0, &["Glimmervoid Basin"]);
    // P1's Clone copying Hill Giant, tapped, with counters and other effects.
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[Entity::Object(giant)]);
    let clone = t.enter(P1, "Clone");
    dress_up(&mut t, clone);
    let clone = t.g.current(clone);
    t.answer_targets(P0, &[Entity::Object(clone)]);
    let before = t.g.battlefield.clone();
    chaos(&mut t, P0);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Hill Giant");
    assert!(o.chars.colors.contains(Color::Red) && !o.chars.colors.contains(Color::Green));
    assert_eq!(t.pt(toks[0]), (3, 3));
    assert!(fresh(&t, toks[0]));
}

#[test]
fn tawnos_token_copy_of_a_card_has_its_enters_abilities() {
    cr!("707.2", "707.9b", "603.6a", "614.1c");
    ruling!(
        "Tawnos, Solemn Survivor",
        "Any enters-the-battlefield abilities of the token you create will trigger when it enters the battlefield. Any “as [this permanent] enters the battlefield” or “[this permanent] enters the battlefield with” abilities of the copied token or card will also work."
    );
    // "{1}{W}{U}{B}, {T}, Sacrifice two artifact tokens, Exile an artifact or creature card
    // from your graveyard: Create a token that's a copy of the exiled card, except it's an
    // artifact in addition to its other types. Activate only as a sorcery."
    supported("Tawnos, Solemn Survivor");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tawnos = t.battlefield(P0, "Tawnos, Solemn Survivor");
    for land in ["Plains", "Island", "Swamp", "Forest"] {
        t.lands(P0, land, 1);
    }
    for _ in 0..2 {
        crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    }
    t.graveyard(P0, RIFTWATCHER);
    let before = crate::r_s01_common::tokens(&t, P0);
    let life = t.life(P0);
    t.activate(P0, tawnos, 1, &[]).unwrap();
    t.resolve_all();
    let toks = riftwatcher_tokens_entered(&t, P0, &before, life, 1);
    assert!(t.obj_now(toks[0]).is(CardType::Artifact));
}

#[test]
fn fractured_realm_makes_turned_face_up_triggers_trigger_twice() {
    cr!("603.2", "702.37e", "708.8", "709.5");
    ruling!(
        "Mirror Room // Fractured Realm",
        "Abilities that apply \"when [this creature] is turned face up\" will trigger an additional time due to Fractured Realm's ability."
    );
    // Fractured Realm: "If a triggered ability of a permanent you control triggers, that
    // ability triggers an additional time." Ruthless Ripper: morph—reveal a black card in
    // your hand; "When this creature is turned face up, target player loses 2 life."
    supported("Mirror Room // Fractured Realm");
    supported("Ruthless Ripper");
    let mut t = TestGame::new(2);
    let room = t.battlefield(P0, "Mirror Room // Fractured Realm");
    assert!(mtg_engine::rooms::unlock(&mut t.g, room, 1, P0));
    t.settle();
    assert!(t.g.stack.is_empty());
    let ripper = crate::r_s12_common::morph(&mut t, P0, "Ruthless Ripper");
    let black = t.hand(P0, "Walking Corpse");
    t.answer_choose(P0, &[Entity::Object(black)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, ripper));
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}
