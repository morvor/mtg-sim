//! Rulings batch S20 — split cards with a shared type line: Rooms (CR 709.5). Casting a
//! door, the characteristics of a Room in each zone, unlocking doors (CR 116.2m, 709.5e),
//! and "when you unlock this door" / "whenever you fully unlock a Room" triggers
//! (CR 709.5h, 709.5i).

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use crate::r_s08_common::mana_value;
use crate::r_s20_common::sram_expertise;
use mtg_engine::ability::{AbilityKind, Cmp, Filter, PlayerRef, Value};
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::eval::Ctx;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::rooms;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const THEATER: &str = "Dazzling Theater // Prop Room";
const ANNEX: &str = "Unholy Annex // Ritual Chamber";

fn unlock_door(half: usize, room: ObjectId) -> SpecialAction {
    SpecialAction::Other {
        name: format!("{}{half}", rooms::UNLOCK_ACTION),
        obj: Some(room),
    }
}

/// The unlock special actions `p` could take now for `room`.
fn unlock_actions(t: &mut TestGame, p: PlayerId, room: ObjectId) -> Vec<SpecialAction> {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Special(s @ SpecialAction::Other { obj: Some(o), .. }) if o == room => Some(s),
            _ => None,
        })
        .collect()
}

/// `p` unlocks `half` of `room` by paying its mana cost (from mana added to the pool),
/// then triggered abilities are put on the stack.
fn pay_to_unlock(t: &mut TestGame, p: PlayerId, room: ObjectId, half: usize) {
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
    ] {
        add_mana(t, p, ty, 2);
    }
    add_mana(t, p, ManaType::C, 6);
    t.g.turn.priority = Some(p);
    t.g.perform_action(p, Action::Special(unlock_door(half, room)))
        .expect("unlock");
    t.g.players[p.idx()].mana_pool.mana.clear();
    t.g.recompute();
    t.settle();
}

/// P0 casts door `half` of the Room card `name` (paying from mana added to the pool) and
/// it resolves. Returns the Room permanent.
fn cast_door(t: &mut TestGame, name: &str, half: u8) -> ObjectId {
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
    ] {
        add_mana(t, P0, ty, 2);
    }
    add_mana(t, P0, ManaType::C, 6);
    let card = t.hand(P0, name);
    let spell = t.cast(P0, card).method(CastMethod::Half(half)).go();
    t.g.players[0].mana_pool.mana.clear();
    t.resolve_all();
    let room = t.g.current(spell);
    assert!(t.on_battlefield(room));
    room
}

fn count_abilities(c: &mtg_engine::object::Characteristics) -> usize {
    c.abilities
        .iter()
        .filter(|a| !matches!(a.kind, AbilityKind::Unsupported(_)))
        .count()
}

#[test]
fn a_room_is_cast_as_one_door_which_is_unlocked_as_it_enters() {
    cr!("709.3", "709.3a", "709.5d");
    ruling!(
        "Dazzling Theater // Prop Room",
        "To cast a Room spell, choose a half (or \"door\") to cast. There's no way to cast both halves of a Room card. When the Room spell resolves, the corresponding door becomes unlocked as the Room enters."
    );
    supported(THEATER);
    let mut t = TestGame::new(2);
    let card = t.hand(P0, THEATER);
    let methods: Vec<CastMethod> =
        t.g.cast_options(P0, card)
            .into_iter()
            .map(|o| o.method)
            .collect();
    assert_eq!(methods, vec![CastMethod::Half(0), CastMethod::Half(1)]);
    let room = cast_door(&mut t, THEATER, 1);
    assert_eq!(rooms::unlocked(&t.g, room), [false, true]);
    assert_eq!(t.obj(room).chars.name, "Prop Room");
}

#[test]
fn only_the_cast_doors_characteristics_exist_on_the_stack() {
    cr!("709.3b", "709.5a");
    ruling!(
        "Dazzling Theater // Prop Room",
        "Room cards have two card faces with a shared type line on a single card. The characteristics of the door you didn't cast are ignored while the spell is on the stack."
    );
    supported(THEATER);
    let mut t = TestGame::new(2);
    let card = t.hand(P0, THEATER);
    add_mana(&mut t, P0, ManaType::W, 4);
    let spell = t.cast(P0, card).method(CastMethod::Half(0)).go();
    let c = t.obj(spell).chars.clone();
    assert_eq!(c.name, "Dazzling Theater");
    assert!(!c.has_name("Prop Room"));
    // The shared type line: an Enchantment — Room spell.
    assert!(c.is(CardType::Enchantment) && c.has_subtype("Room"));
    assert_eq!(mana_value(&t, spell), 4);
    assert_eq!(
        count_abilities(&c),
        count_abilities(&card_face(THEATER, 0))
    );
}

fn card_face(name: &str, i: usize) -> mtg_engine::object::Characteristics {
    mtg_engine::card::card(name).faces[i].chars.clone()
}

#[test]
fn on_the_battlefield_a_room_has_the_characteristics_of_its_unlocked_doors() {
    cr!("709.5", "709.5c");
    ruling!(
        "Dazzling Theater // Prop Room",
        "While on the battlefield, a Room's characteristics are a combination of the characteristics of its unlocked doors."
    );
    supported(THEATER);
    let mut t = TestGame::new(2);
    let room = cast_door(&mut t, THEATER, 0);
    let c = t.obj(room).chars.clone();
    assert_eq!(c.name, "Dazzling Theater");
    assert_eq!(mana_value(&t, room), 4);
    assert_eq!(count_abilities(&c), count_abilities(&card_face(THEATER, 0)));
    // Both doors unlocked: both names, mana value 4 + 3, each door's abilities.
    pay_to_unlock(&mut t, P0, room, 1);
    let c = t.obj(room).chars.clone();
    assert!(c.has_name("Dazzling Theater") && c.has_name("Prop Room"));
    assert_eq!(mana_value(&t, room), 7);
    assert!(c.is(CardType::Enchantment) && c.has_subtype("Room"));
    assert_eq!(
        count_abilities(&c),
        count_abilities(&card_face(THEATER, 0)) + count_abilities(&card_face(THEATER, 1))
    );
    // Prop Room's ability works: "Untap each creature you control during each other
    // player's untap step."
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn off_the_battlefield_and_stack_a_room_card_has_both_doors_characteristics() {
    cr!("709.4", "709.4b");
    ruling!(
        "Dazzling Theater // Prop Room",
        "While in any zone other than the stack or the battlefield, a Room card's characteristics are a combination of its two doors."
    );
    supported(THEATER);
    let mut t = TestGame::new(2);
    let card = t.library_top(P0, THEATER);
    t.g.recompute();
    assert_eq!(mana_value(&t, card), 7);
    assert!(t.obj(card).chars.has_name("Dazzling Theater"));
    assert!(t.obj(card).chars.has_name("Prop Room"));
    // "Search your library for a card with mana value 4 or less" can't find it.
    let small = Filter::ManaValue(Cmp::Le, Box::new(Value::c(4)));
    assert!(!t.g.matches(card, &small, &Ctx::new(None, P0)));
}

#[test]
fn each_room_card_is_a_single_card() {
    cr!("709.1", "709.4");
    ruling!(
        "Dazzling Theater // Prop Room",
        "Each Room card is a single card. For example, if you discard a Room card, you've discarded one card, not two. If an effect counts the number of enchantment cards in your graveyard, Bottomless Pool // Locker Room counts once, not twice."
    );
    supported(THEATER);
    let mut t = TestGame::new(2);
    let card = t.hand(P0, THEATER);
    t.hand(P0, "Forest");
    t.g.discard(P0, card, None);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.graveyard_size(P0), 1);
    t.g.recompute();
    let enchantments = t.g.eval_value(
        &Value::CardsInGraveyard(PlayerRef::You, Filter::Type(CardType::Enchantment)),
        &Ctx::new(None, P0),
    );
    assert_eq!(enchantments, 1);
}

#[test]
fn a_room_with_no_unlocked_door_has_no_name_and_no_abilities() {
    cr!("709.5", "709.5d");
    ruling!(
        "Unholy Annex // Ritual Chamber",
        "If neither door of a Room is unlocked, it's a Room enchantment with no name and no abilities."
    );
    supported(ANNEX);
    let mut t = TestGame::new(2);
    let room = t.enter(P0, ANNEX);
    t.settle();
    let c = t.obj(room).chars.clone();
    assert_eq!(c.name, "");
    assert!(c.abilities.is_empty());
    assert!(c.is(CardType::Enchantment) && c.has_subtype("Room"));
    // Unholy Annex's "At the beginning of your end step, draw a card. ..." doesn't
    // trigger.
    let hand = t.hand_size(P0);
    let life = t.life(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), life);
}

#[test]
fn a_room_entering_from_another_zone_than_the_stack_has_both_doors_locked() {
    cr!("709.5d");
    ruling!(
        "Unholy Annex // Ritual Chamber",
        "If a Room enters from any zone other than the stack, it will enter with both halves locked."
    );
    supported(ANNEX);
    supported("Replenish");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, ANNEX);
    // Replenish: "Return all enchantment cards from your graveyard to the battlefield."
    add_mana(&mut t, P0, ManaType::W, 4);
    let replenish = t.hand(P0, "Replenish");
    t.cast(P0, replenish).go();
    t.resolve_all();
    let room = t.g.current(card);
    assert!(t.on_battlefield(room));
    assert_eq!(rooms::unlocked(&t.g, room), [false, false]);
    // Ritual Chamber's "When you unlock this door" didn't trigger: no Demon.
    assert!(with_subtype(&t, P0, "Demon").is_empty());
}

#[test]
fn a_room_has_two_names_and_only_one_can_be_chosen() {
    cr!("709.4a", "201.3");
    ruling!(
        "Glassworks // Shattered Yard",
        "Each Room card has two names. If an effect instructs you to choose a card name, you may choose one of those names, but not both."
    );
    supported("Glassworks // Shattered Yard");
    supported("Meddling Mage");
    let mut t = TestGame::new(2);
    let card = t.hand(P1, "Glassworks // Shattered Yard");
    t.g.recompute();
    let ctx = Ctx::new(None, P0);
    assert!(t.g.matches(card, &Filter::Named("Glassworks".into()), &ctx));
    assert!(t.g.matches(card, &Filter::Named("Shattered Yard".into()), &ctx));
    // Meddling Mage: "As this creature enters, choose a nonland card name. Spells with the
    // chosen name can't be cast." Naming both isn't a legal choice.
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Glassworks // Shattered Yard".into()),
    );
    let mage = t.enter(P0, "Meddling Mage");
    assert_eq!(t.obj_now(mage).choices.card_name.as_deref(), Some(""));
    // Naming Glassworks: Glassworks can't be cast, Shattered Yard can.
    t.answer(P0, DecisionKind::Name, Answer::Text("Glassworks".into()));
    let mage = t.enter(P0, "Meddling Mage");
    assert_eq!(
        t.obj_now(mage).choices.card_name.as_deref(),
        Some("Glassworks")
    );
    t.set_step(P1, Step::PrecombatMain);
    add_mana(&mut t, P1, ManaType::R, 2);
    add_mana(&mut t, P1, ManaType::C, 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P1, &[Entity::Object(bears)]);
    assert!(t
        .cast(P1, card)
        .method(CastMethod::Half(0))
        .try_go()
        .is_err());
    let card = t.g.current(card);
    let r = t.cast(P1, card).method(CastMethod::Half(1)).try_go();
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn sram_expertise_can_cast_prop_room_but_not_dazzling_theater() {
    cr!("601.3e", "709.3a");
    ruling!(
        "Dazzling Theater // Prop Room",
        "If an effect allows you to cast a spell with certain characteristics, consider only the characteristics of the door you're casting. For example, if an effect allows you to cast a permanent spell with mana value 3 or less from among cards in your graveyard, you could cast Bottomless Pool this way, but not Locker Room."
    );
    supported(THEATER);
    supported("Sram's Expertise");
    let mut t = TestGame::new(2);
    let card = t.hand(P0, THEATER);
    // Prop Room has mana value 3; Dazzling Theater has 4.
    let options = sram_expertise(&mut t, 1);
    assert_eq!(options, vec!["Don't cast a spell", "Cast Prop Room"]);
    t.resolve_all();
    let room = t.g.current(card);
    assert!(t.on_battlefield(room));
    assert_eq!(rooms::unlocked(&t.g, room), [false, true]);
}

#[test]
fn a_locked_doors_unlock_cost_is_paid_as_a_special_action() {
    cr!("116.2m", "709.5e");
    ruling!(
        "Unholy Annex // Ritual Chamber",
        "Any time you have priority during a main phase of your turn and the stack is empty, you may pay the mana cost of a locked door (also called its \"unlock cost\"). That door becomes unlocked. This is a special action. It doesn't use the stack and can't be responded to."
    );
    supported(ANNEX);
    let mut t = TestGame::new(2);
    let room = cast_door(&mut t, ANNEX, 0);
    // Ritual Chamber {3}{B}{B} can be unlocked in P0's main phase with the stack empty.
    add_mana(&mut t, P0, ManaType::B, 2);
    add_mana(&mut t, P0, ManaType::C, 3);
    assert_eq!(unlock_actions(&mut t, P0, room), vec![unlock_door(1, room)]);
    // Not in combat, and not while a spell is on the stack.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(unlock_actions(&mut t, P0, room).is_empty());
    t.set_step(P0, Step::PostcombatMain);
    let from = t.asked().len();
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(unlock_door(1, room)))
        .expect("unlock");
    // The door is unlocked at once; nothing used the stack and no player got priority to
    // respond before it happened. (Its "When you unlock this door" ability triggers.)
    assert_eq!(rooms::unlocked(&t.g, room), [true, true]);
    assert!(t.g.stack.is_empty());
    assert_eq!(
        crate::r_s08_common::priority_asks_of(&t, P1, from),
        0
    );
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Demon").len(), 1);
}

#[test]
fn a_door_with_a_targeted_unlock_trigger_can_be_unlocked_without_a_target() {
    cr!("709.5e", "709.5h", "603.3d");
    ruling!(
        "Glassworks // Shattered Yard",
        "Some doors have abilities that trigger whenever you unlock that door and require one or more targets. You can unlock that door even if there would be insufficient legal targets for that triggered ability. The triggered ability won't go on the stack."
    );
    supported("Glassworks // Shattered Yard");
    let mut t = TestGame::new(2);
    // Glassworks: "When you unlock this door, this Room deals 4 damage to target creature
    // an opponent controls." P1 controls no creature.
    let room = cast_door(&mut t, "Glassworks // Shattered Yard", 1);
    assert_eq!(unlock_actions(&mut t, P0, room), vec![]);
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 2);
    assert_eq!(unlock_actions(&mut t, P0, room), vec![unlock_door(0, room)]);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(unlock_door(0, room)))
        .expect("unlock");
    t.settle();
    assert_eq!(rooms::unlocked(&t.g, room), [true, true]);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn unlock_this_door_triggers_when_the_door_is_unlocked_either_way_and_isnt_doubled_by_yarok() {
    cr!("709.5h", "603.2d");
    ruling!(
        "Unholy Annex // Ritual Chamber",
        "An ability that triggers \"when you unlock this door\" triggers when that door becomes unlocked. This can happen one of two ways: (1) the door becomes unlocked on the battlefield or (2) the door becomes unlocked as the Room enters the battlefield because you cast the corresponding half. In the latter case, since the door becoming unlocked is what causes the ability to trigger, effects that cause abilities that trigger when a permanent enters to trigger an additional time (such as that of Panharmonicon) won't apply."
    );
    supported(ANNEX);
    supported("Yarok, the Desecrated");
    let mut t = TestGame::new(2);
    // Yarok: "If a permanent entering causes a triggered ability of a permanent you
    // control to trigger, that ability triggers an additional time."
    t.battlefield(P0, "Yarok, the Desecrated");
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    t.battlefield(P0, "Soul Warden");
    let life = t.life(P0);
    // (2) Ritual Chamber is cast: "When you unlock this door, create a 6/6 black Demon
    // creature token with flying." One Demon: the door being unlocked, not the Room
    // entering, triggered it.
    cast_door(&mut t, ANNEX, 1);
    assert_eq!(with_subtype(&t, P0, "Demon").len(), 1);
    // (The Demon entering does trigger Soul Warden an additional time.)
    assert_eq!(t.life(P0), life + 2);
    // (1) A Room with both doors locked has Ritual Chamber unlocked on the battlefield.
    let locked = t.enter(P0, ANNEX);
    t.settle();
    t.resolve_all();
    pay_to_unlock(&mut t, P0, locked, 1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Demon").len(), 2);
}

#[test]
fn fully_unlocking_a_room_triggers_when_the_second_door_is_unlocked() {
    cr!("709.5i");
    ruling!(
        "Entity Tracker",
        "An ability that triggers \"whenever you fully unlock a Room\" triggers when a door becomes unlocked and the other door of that Room is already unlocked"
    );
    supported("Entity Tracker");
    supported(THEATER);
    supported("Keys to the House");
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    // Entity Tracker: "Eerie — Whenever an enchantment you control enters and whenever you
    // fully unlock a Room, draw a card."
    t.battlefield(P0, "Entity Tracker");
    // Dazzling Theater is cast: the Room enters (draw), but it isn't fully unlocked.
    let room = cast_door(&mut t, THEATER, 0);
    assert_eq!(t.hand_size(P0), 1);
    // Unlocking Prop Room fully unlocks it: draw.
    pay_to_unlock(&mut t, P0, room, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    // Keys to the House ("{3}, {T}, Sacrifice this artifact: Lock or unlock a door of
    // target Room you control.") locks Prop Room; another unlocks it again: the Room is
    // fully unlocked again (draw).
    let keys = t.battlefield(P0, "Keys to the House");
    add_mana(&mut t, P0, ManaType::C, 3);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[Entity::Object(room)]);
    activate_containing(&mut t, P0, keys, "Lock or unlock").expect("activate");
    t.resolve_all();
    assert_eq!(rooms::unlocked(&t.g, room), [true, false]);
    assert_eq!(t.hand_size(P0), 2);
    // (The options are locking Dazzling Theater and unlocking Prop Room.)
    let keys = t.battlefield(P0, "Keys to the House");
    add_mana(&mut t, P0, ManaType::C, 3);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[Entity::Object(room)]);
    activate_containing(&mut t, P0, keys, "Lock or unlock").expect("activate");
    t.resolve_all();
    assert_eq!(rooms::unlocked(&t.g, room), [true, true]);
    assert_eq!(t.hand_size(P0), 3);
}
