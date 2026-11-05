//! Rulings batch P042 — the rest of the "damage prevention" group: protection from players
//! and from names (CR 702.16), choices made as a permanent enters (CR 614.12), targets and
//! modes (CR 115, 700.2, 601.2c), costs (CR 118, 602.2), copies of spells and permanents
//! (CR 707.10, 707.12), delayed returns (CR 603.7), and Sagas choosing modes at random.

use crate::r_p042_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- Protection from players --------------------------------------------------------------

#[test]
fn guardian_archon_protection_from_a_player() {
    cr!("702.16b", "702.16c", "702.16d", "702.16e", "702.16k", "109.4");
    ruling!(
        "Guardian Archon",
        "Protection from a player means all of the following: (1) Neither you nor the target permanent can be the target of spells or abilities controlled by the chosen player. (2) Neither you nor the target permanent can be enchanted or equipped by any Auras or Equipment controlled by the chosen players. (Okay, Fortifications too, if the target permanent is a land.) (3) If the target permanent is a creature, it can’t be blocked by creatures controlled by the chosen player. (4) All damage that would be dealt to you or the target permanent by sources controlled by the chosen player is prevented. The same is true for sources owned by the chosen player that don’t have controllers."
    );
    let mut t = TestGame::new(2);
    let archon = t.enter(P0, "Guardian Archon");
    t.g.flush_events();
    t.settle();
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_resolve(&mut t, P0, archon, 0, &[obj(bears)]);
    // (1) Targeting.
    let from = t.asked().len();
    t.set_step(P1, Step::PrecombatMain);
    let shock = cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    let offered = crate::r_p016_common::target_choices(&t, P1, from, "");
    assert!(!offered[0].contains(&pl(P0)) && !offered[0].contains(&obj(bears)));
    t.g.stack.retain(|s| *s != shock);
    // (2) P1's Aura and Equipment fall off.
    let rancor = t.battlefield(P1, "Rancor");
    let splitter = t.battlefield(P1, "Bonesplitter");
    t.g.attach(rancor, obj(bears));
    t.g.attach(splitter, obj(bears));
    t.g.recompute();
    t.settle();
    assert!(!t.on_battlefield(rancor), "the Aura is put into the graveyard");
    assert_eq!(t.obj_now(splitter).attached_to, None, "the Equipment is unattached");
    // (3) Blocking: P1's creatures can't block the Bears.
    let blocker = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, pl(P1))]);
    assert!(!t.g.can_block(blocker, bears));
    // (4) Damage from P1's sources, and from a card P1 owns that has no controller (a
    // card in P1's graveyard).
    let card = t.graveyard(P1, "Shock");
    damage(&mut t, blocker, pl(P0), 3);
    damage(&mut t, card, obj(bears), 2);
    assert_eq!((t.life(P0), damage_marked(&t, bears)), (20, 0));
    assert_eq!(prevented_events(&t).len(), 2);
    let mine = t.graveyard(P0, "Shock");
    damage(&mut t, mine, obj(bears), 1);
    assert_eq!(damage_marked(&t, bears), 1, "P0's own card isn't P1's");
}

#[test]
fn courageous_resolve_protection_from_opponents_and_their_uncontrolled_objects() {
    cr!("702.16k", "702.16e", "109.4");
    ruling!(
        "Courageous Resolve",
        "Protection from your opponents means that the target creature has protection from each object controlled by those opponents. If an object has no controller (such as a card in a graveyard), its owner is considered its controller for this purpose."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Craw Wurm");
    cast_resolve(&mut t, P0, "Courageous Resolve", &[obj(bears)]);
    for p in [P1, P2] {
        let giant = t.battlefield(p, "Hill Giant");
        damage(&mut t, giant, obj(bears), 1);
        let card = t.graveyard(p, "Shock");
        damage(&mut t, card, obj(bears), 1);
    }
    assert_eq!(damage_marked(&t, bears), 0);
    assert_eq!(prevented_events(&t).len(), 4);
    let card = t.graveyard(P0, "Shock");
    damage(&mut t, card, obj(bears), 1);
    assert_eq!(damage_marked(&t, bears), 1);
}

#[test]
fn courageous_resolve_with_an_illegal_target_does_nothing_at_all() {
    cr!("608.2b");
    ruling!(
        "Courageous Resolve",
        "You can choose not to target a creature just to draw a card and, if you have 5 or less life, to get the additional effects listed. However, if you do choose a target, and that target is illegal at the time Courageous Resolve tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card, and the additional effects won't occur even if you have 5 or less life."
    );
    // No target: P0 draws and gets fateful hour.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[]);
    cast_resolve(&mut t, P0, "Courageous Resolve", &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    set_life(&mut t, P0, 0);
    t.settle();
    assert!(!t.has_lost(P0));
    // An illegal target: no draw, no fateful hour.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Courageous Resolve", &[obj(bears)]);
    exile_now(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    set_life(&mut t, P0, 0);
    t.settle();
    assert!(t.has_lost(P0));
}

// --- Names --------------------------------------------------------------------------------------

#[test]
fn runed_halo_protects_only_from_the_chosen_one_of_a_split_cards_names() {
    cr!("702.16a", "709.3b", "201.4b");
    ruling!(
        "Runed Halo",
        "Some cards can have alternative names, such as split cards, adventurer cards, double-faced cards, and so on. You may choose one of the two names the card can have; Runed Halo won't protect you from the other name."
    );
    supported("Fire // Ice");
    for name in ["Fire", "Ice"] {
        let mut t = TestGame::new(2);
        t.answer(P0, DecisionKind::Name, Answer::Text(name.to_string()));
        t.enter(P0, "Runed Halo");
        t.resolve_all();
        // P1 casts Fire at P0.
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Mountain", 2);
        let card = t.hand(P1, "Fire // Ice");
        let from = t.asked().len();
        t.answer_targets(P1, &[pl(P0)]);
        t.g.turn.priority = Some(P1);
        let r = t.g.cast_spell(P1, card, CastMethod::Half(0));
        let offered = crate::r_p016_common::target_choices(&t, P1, from, "");
        assert_eq!(
            offered.iter().any(|c| c.contains(&pl(P0))),
            name == "Ice",
            "chose {name}"
        );
        if name == "Ice" {
            r.unwrap();
            t.resolve_all();
            assert_eq!(t.life(P0), 18, "not protected from Fire");
        }
    }
}

#[test]
fn gideons_intervention_land_names_and_token_names() {
    cr!("201.4", "305.1", "111.4");
    ruling!(
        "Gideon's Intervention",
        "You may choose the name of a land card. Cards with that name can still be played (since lands are never cast), but damage they would deal to you and your permanents will be prevented."
    );
    ruling!(
        "Gideon's Intervention",
        "You must choose the name of a card, not the name of a token. For example, you can't choose \"Zombie Token.\" However, if a token happens to have the same name as a card (such as an embalmed creature token), you can choose it."
    );
    supported("Gideon's Intervention");
    supported("Mutavault");
    supported("Cackling Counterpart");
    // A land name: P1 still plays Mutavault; its damage to P0 is prevented.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Name, Answer::Text("Mutavault".into()));
    t.enter(P0, "Gideon's Intervention");
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    let vault = t.hand(P1, "Mutavault");
    t.play_land(P1, vault).unwrap();
    let vault = t.g.current(vault);
    assert!(t.on_battlefield(vault));
    damage(&mut t, vault, pl(P0), 2);
    assert_eq!(t.life(P0), 20);
    // A token's name that isn't a card's isn't a choice: "Soldier Token" is refused.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Name, Answer::Text("Soldier Token".into()));
    t.enter(P0, "Gideon's Intervention");
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Raise the Alarm", &[]);
    let soldiers = t.named_on_battlefield("Soldier Token");
    assert_eq!(soldiers.len(), 2);
    damage(&mut t, soldiers[0], pl(P0), 1);
    assert_eq!(t.life(P0), 19);
    // A token with a card's name: P1's token copy of Grizzly Bears.
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    t.enter(P0, "Gideon's Intervention");
    t.resolve_all();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Cackling Counterpart", &[obj(bears)]);
    let copies = t.named_on_battlefield("Grizzly Bears");
    let token = *copies.iter().find(|c| **c != bears).unwrap();
    assert!(t.obj_now(token).is_token());
    damage(&mut t, token, pl(P0), 2);
    assert_eq!(t.life(P0), 20);
}

// --- Serra's Emissary ----------------------------------------------------------------------------

#[test]
fn serra_emissary_chooses_as_it_enters_without_the_stack_among_card_types() {
    cr!("614.12", "205.2a", "702.16a");
    ruling!(
        "Serra's Emissary",
        "The card type choice is a replacement effect, not a triggered ability. It doesn't use the stack and can't be responded to. Notably, if you choose instant, your opponent can't try to cast an instant on the Emissary before it has protection."
    );
    ruling!(
        "Serra's Emissary",
        "You may choose any card type. The ones that may appear in a normal game of Magic are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Supertypes such as snow and basic may not be chosen. Subtypes such as Angel, Aura, and Forest also may not be chosen."
    );
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    respond_with(
        &mut t,
        P0,
        Box::new(|_, d| match d {
            Decision::ChooseOption { options, .. } => options
                .iter()
                .position(|o| o == "instant")
                .map(Answer::Index),
            _ => None,
        }),
    );
    let spell = cast_new(&mut t, P0, "Serra's Emissary", &[]);
    t.resolve();
    let emissary = t.g.current(spell);
    assert!(t.on_battlefield(emissary));
    assert_eq!(t.stack_len(), 0, "no trigger to respond to");
    // It already has protection from instants: P1's Shock can't target it or P0.
    let from2 = t.asked().len();
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Shock", &[obj(emissary)]);
    let offered = crate::r_p016_common::target_choices(&t, P1, from2, "");
    assert!(!offered[0].contains(&obj(emissary)) && !offered[0].contains(&pl(P0)));
    // The options were the card types only.
    let options: Vec<String> = t.asked()[from..from2]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .expect("a card type was chosen");
    for ty in [
        "artifact",
        "battle",
        "creature",
        "enchantment",
        "instant",
        "kindred",
        "land",
        "planeswalker",
        "sorcery",
    ] {
        assert!(options.iter().any(|o| o == ty), "{ty} in {options:?}");
    }
    for not in ["snow", "basic", "legendary", "angel", "aura", "forest"] {
        assert!(
            !options.iter().any(|o| o.eq_ignore_ascii_case(not)),
            "{not} in {options:?}"
        );
    }
}

// --- Targets and modes ----------------------------------------------------------------------------

#[test]
fn two_target_spells_may_target_the_same_creature_twice() {
    cr!("115.3", "601.2c");
    ruling!(
        "Agony Warp",
        "The two targets may be the same creature or they may be different creatures."
    );
    ruling!(
        "Azorius Ploy",
        "The two targets may be the same creature or they may be different creatures."
    );
    supported("Azorius Ploy");
    // Agony Warp twice on P1's Hill Giant: -3/-3, it dies.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Agony Warp", &[obj(giant), obj(giant)]);
    assert!(!t.on_battlefield(giant));
    // Azorius Ploy twice on P1's attacking Craw Wurm, blocked by P0's Hill Giant: neither
    // deals combat damage to the other.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P0, "Hill Giant");
    to_beginning_of_combat(&mut t, P1);
    cast_resolve(&mut t, P0, "Azorius Ploy", &[obj(wurm), obj(wurm)]);
    t.attack(&[(wurm, pl(P0))], &[(giant, wurm)]);
    assert!(t.on_battlefield(giant) && t.on_battlefield(wurm));
    assert_eq!((damage_marked(&t, giant), damage_marked(&t, wurm)), (0, 0));
    assert_eq!(prevented_events(&t).len(), 2);
}

#[test]
fn commands_modes_need_legal_targets_only_for_chosen_modes() {
    cr!("700.2a", "601.2c", "115.1");
    ruling!(
        "Atarka's Command",
        "You can choose a mode only if you can choose legal targets for that mode. Ignore the targeting requirements for modes that aren't chosen."
    );
    ruling!(
        "Dromoka's Command",
        "You can choose a mode only if you can choose legal targets for that mode. Ignore the targeting requirements for modes that aren't chosen."
    );
    ruling!(
        "Kolaghan's Command",
        "You can choose a mode only if you can choose legal targets for that mode. Ignore the targeting requirements for modes that aren't chosen."
    );
    /// The modes offered when `p` chose modes since decision `from`.
    fn available(t: &TestGame, from: usize) -> Vec<usize> {
        t.asked()[from..]
            .iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseModes { available, .. } => Some(available.clone()),
                _ => None,
            })
            .expect("modes chosen")
    }
    // Dromoka's Command: no instant or sorcery spell on the stack, so mode 1 can't be
    // chosen (asking for it falls back to available modes); modes 2 and 3 can.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Pacifism");
    lands_for_cost(&mut t, P0, "Dromoka's Command");
    let card = t.hand(P0, "Dromoka's Command");
    let from = t.asked().len();
    t.cast(P0, card)
        .modes(&[1, 2])
        .target(pl(P1))
        .target(obj(bears))
        .go();
    assert_eq!(available(&t, from), vec![1, 2]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert!(t.in_graveyard(P1, "Pacifism"));
    // With a sorcery on the stack, mode 1 is available.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Divination", &[]);
    lands_for_cost(&mut t, P0, "Dromoka's Command");
    let card = t.hand(P0, "Dromoka's Command");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    t.answer_targets(P0, &[]);
    let _ = t.cast_with(P0, card, &[]);
    assert_eq!(available(&t, from), vec![0, 1, 2]);
    // Kolaghan's Command: an empty graveyard and no artifact, so modes 1 and 3 aren't
    // available.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Kolaghan's Command");
    let card = t.hand(P0, "Kolaghan's Command");
    t.hand(P1, "Hill Giant");
    let from = t.asked().len();
    t.cast(P0, card)
        .modes(&[1, 3])
        .target(pl(P1))
        .target(pl(P1))
        .go();
    assert_eq!(available(&t, from), vec![1, 3]);
    t.resolve_all();
    assert_eq!((t.hand_size(P1), t.life(P1)), (0, 18));
    // Atarka's Command: modes without targets are always available.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Atarka's Command");
    let card = t.hand(P0, "Atarka's Command");
    t.cast(P0, card).modes(&[0, 1]).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn dromokas_command_sacrifice_mode_can_target_a_player_with_no_enchantment() {
    cr!("115.1", "701.21a");
    ruling!(
        "Dromoka's Command",
        "You can target any player with the second ability, including one that doesn't control an enchantment."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Dromoka's Command");
    let card = t.hand(P0, "Dromoka's Command");
    t.cast(P0, card)
        .modes(&[1, 2])
        .target(pl(P1))
        .target(obj(bears))
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Dromoka's Command"));
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn falling_timber_has_a_second_target_only_if_kicked() {
    cr!("702.33d", "601.2c");
    ruling!(
        "Falling Timber",
        "You pick a second target only if you choose to kick Falling Timber."
    );
    supported("Falling Timber");
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Hill Giant");
        let b = t.battlefield(P1, "Craw Wurm");
        t.lands(P0, "Forest", 3);
        let card = t.hand(P0, "Falling Timber");
        let from = t.asked().len();
        t.answer_targets(P0, &[obj(a)]);
        t.answer_targets(P0, &[obj(b)]);
        let spell = t.cast(P0, card).kicked(kicked).go();
        let slots = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
            .count();
        assert_eq!(slots, if kicked { 2 } else { 1 }, "kicked: {kicked}");
        assert_eq!(
            crate::r_s25_common::targets_of(&t, spell).len(),
            if kicked { 2 } else { 1 }
        );
        t.clear_answers();
    }
}

#[test]
fn zealot_il_vec_chooses_a_target_even_if_you_dont_have_it_deal_damage() {
    cr!("603.3d", "603.5");
    ruling!(
        "Zealot il-Vec",
        "When Zealot il-Vec attacks and isn’t blocked, you must choose a target for its ability even if you don’t use it."
    );
    supported("Zealot il-Vec");
    let mut t = TestGame::new(2);
    let zealot = t.battlefield(P0, "Zealot il-Vec");
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, false);
    to_beginning_of_combat(&mut t, P0);
    t.attack(&[(zealot, pl(P1))], &[]);
    let kinds: Vec<&str> = t.asked()[from..]
        .iter()
        .filter(|(p, _)| *p == P0)
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { .. } => Some("target"),
            Decision::YesNo { .. } => Some("may"),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, vec!["target", "may"], "target chosen first");
    assert_eq!(damage_marked(&t, giant), 0, "declined");
    assert_eq!(t.life(P1), 19, "its combat damage wasn't prevented");
}

#[test]
fn benalish_missionary_on_a_creature_blocked_by_an_effect() {
    cr!("509.1h", "702.19d");
    ruling!(
        "Benalish Missionary",
        "Works even if the creature is “blocked” by an effect rather than actual creatures."
    );
    supported("Benalish Missionary");
    supported("Curtain of Light");
    // P1's Colossal Dreadmaw (6/6 trample) attacks; P0's Curtain of Light makes it
    // blocked. Blocked by no creatures, a trampler assigns all its damage to P0.
    for missionary in [false, true] {
        let mut t = TestGame::new(2);
        let medic = t.battlefield(P0, "Benalish Missionary");
        let maw = t.battlefield(P1, "Colossal Dreadmaw");
        to_beginning_of_combat(&mut t, P1);
        attack_with(&mut t, &[(maw, pl(P0))]);
        // No blocks.
        t.advance_to(P1, Step::DeclareBlockers);
        cast_resolve(&mut t, P0, "Curtain of Light", &[obj(maw)]);
        if missionary {
            t.lands(P0, "Plains", 2);
            activate_resolve(&mut t, P0, medic, 0, &[obj(maw)]);
        }
        t.advance_to(P1, Step::EndOfCombat);
        assert_eq!(t.life(P0), if missionary { 20 } else { 14 }, "Missionary: {missionary}");
    }
}

// --- Lands, Auras and costs ----------------------------------------------------------------------

#[test]
fn flowering_field_the_lands_controller_activates() {
    cr!("602.2");
    ruling!(
        "Flowering Field",
        "The land’s controller gets to activate the ability."
    );
    // P1's Flowering Field on P0's Plains.
    let mut t = TestGame::new(2);
    let plains = t.battlefield(P0, "Plains");
    let field = t.battlefield(P1, "Flowering Field");
    assert!(t.g.attach(field, obj(plains)));
    t.g.recompute();
    t.g.turn.priority = Some(P1);
    // (Index 0 is the Plains' mana ability.)
    assert!(t.activate(P1, plains, 1, &[pl(P1)]).is_err(), "not the Aura's controller");
    activate_resolve(&mut t, P0, plains, 1, &[pl(P0)]);
    let giant = t.battlefield(P1, "Hill Giant");
    damage(&mut t, giant, pl(P0), 2);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn security_blockade_with_an_illegal_target_doesnt_enter() {
    cr!("608.3b", "303.4a");
    ruling!(
        "Security Blockade",
        "You must target a land to cast Security Blockade. If that land is an illegal target when Security Blockade tries to resolve, it won't resolve and won't enter. You won't get the Knight token."
    );
    supported("Security Blockade");
    supported("Stone Rain");
    for destroyed in [false, true] {
        let mut t = TestGame::new(2);
        let plains = t.battlefield(P0, "Plains");
        cast_new(&mut t, P0, "Security Blockade", &[obj(plains)]);
        if destroyed {
            exile_now(&mut t, plains);
        }
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Security Blockade").len(), usize::from(!destroyed));
        assert_eq!(t.named_on_battlefield("Knight Token").len(), usize::from(!destroyed));
    }
    // With no land on the battlefield, it can't be cast.
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Security Blockade");
    crate::r_p016_common::lots(&mut t, P0);
    assert!(t.cast(P0, card).try_go().is_err());
}

#[test]
fn glacial_chasm_with_no_other_land_sacrifices_itself() {
    cr!("701.21a", "603.3");
    ruling!(
        "Glacial Chasm",
        "When Glacial Chasm's \"enters\" ability resolves, if you don't control any other lands, you must sacrifice Glacial Chasm itself."
    );
    supported("Glacial Chasm");
    let mut t = TestGame::new(2);
    let chasm = t.hand(P0, "Glacial Chasm");
    t.play_land(P0, chasm).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Glacial Chasm"));
    assert!(t.named_on_battlefield("Glacial Chasm").is_empty());
}

#[test]
fn barrenton_medic_counter_is_a_cost_and_can_kill_it() {
    cr!("602.2b", "118.3", "704.5f", "400.7");
    ruling!(
        "Barrenton Medic",
        "You put the -1/-1 counter on Barrenton Medic as a cost. That means it happens when you activate the ability, not when it resolves. If paying the cost causes the creature to have 0 toughness, it’s put into the graveyard before you can untap it and before you can even pay the cost again."
    );
    let mut t = TestGame::new(2);
    let medic = t.battlefield(P0, "Barrenton Medic");
    t.g.tap(medic);
    activate(&mut t, P0, medic, 1, &[]);
    assert_eq!(t.counters(medic, "-1/-1"), 1, "on activation");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.obj_now(medic).tapped);
    // With three counters, the fourth makes it 0 toughness: it dies before its ability
    // resolves, and it can't be activated again.
    crate::r_s29_common::put_counters(&mut t, medic, "-1/-1", 2);
    t.g.tap(medic);
    activate(&mut t, P0, medic, 1, &[]);
    t.settle();
    assert!(t.in_graveyard(P0, "Barrenton Medic"));
    assert_eq!(t.stack_len(), 1, "its ability is still on the stack");
    // It can't be activated again: not the permanent that left (CR 400.7), nor the card
    // in the graveyard.
    let card = t.g.current(medic);
    assert_ne!(card, medic);
    assert!(t.activate(P0, medic, 1, &[]).is_err(), "the permanent is gone");
    assert!(t.activate(P0, card, 1, &[]).is_err(), "the card can't pay the cost again");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Barrenton Medic"));
}

#[test]
fn atalya_spends_only_white_mana_on_x_for_either_mode() {
    cr!("107.3a", "700.2");
    ruling!(
        "Atalya, Samite Master",
        "Spending only white mana applies to the ability regardless of which mode you choose."
    );
    for mode in [0usize, 1] {
        let mut t = TestGame::new(2);
        let atalya = t.battlefield(P0, "Atalya, Samite Master");
        let wurm = t.battlefield(P0, "Craw Wurm");
        t.lands(P0, "Island", 2);
        let targets = if mode == 0 { vec![obj(wurm)] } else { vec![] };
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![mode]));
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        assert!(t.activate(P0, atalya, 0, &targets).is_err(), "mode {mode}: blue");
        t.clear_answers();
        t.lands(P0, "Plains", 2);
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![mode]));
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        activate_resolve(&mut t, P0, atalya, 0, &targets);
        if mode == 0 {
            let giant = t.battlefield(P1, "Hill Giant");
            damage(&mut t, giant, obj(wurm), 3);
            assert_eq!(damage_marked(&t, wurm), 1);
        } else {
            assert_eq!(t.life(P0), 22);
        }
    }
}

// --- Copies ------------------------------------------------------------------------------------

#[test]
fn sevinne_copies_spells_without_targets_and_the_copy_isnt_cast() {
    cr!("707.10");
    ruling!(
        "Sevinne, the Chronoclasm",
        "Sevinne's ability will copy any instant or sorcery spell, not just one with targets."
    );
    ruling!(
        "Sevinne, the Chronoclasm",
        "The copy that Sevinne's ability creates is created on the stack, so it's not \"cast.\" Abilities that trigger when a player casts a spell won't trigger."
    );
    supported("Young Pyromancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sevinne, the Chronoclasm");
    t.battlefield(P0, "Young Pyromancer");
    t.lands(P0, "Island", 3);
    let hand = t.hand_size(P0);
    let tt = t.graveyard(P0, "Think Twice");
    t.cast(P0, tt)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback))
        .go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2, "Think Twice and its copy each drew");
    assert_eq!(
        t.named_on_battlefield("Elemental Token").len(),
        1,
        "Young Pyromancer triggered for the cast spell only"
    );
}

#[test]
fn kurbis_copies_put_onto_the_battlefield_without_being_cast_get_no_counters() {
    cr!("707.2", "614.1c", "704.5f");
    ruling!(
        "Kurbis, Harvest Celebrant",
        "Similarly, if you create a token that's a copy of Kurbis, or if a card enters the battlefield as a copy of Kurbis without being cast, it enters with zero +1/+1 counters on it."
    );
    supported("Kurbis, Harvest Celebrant");
    // A token copy (Cackling Counterpart).
    let mut t = TestGame::new(2);
    let kurbis = t.battlefield(P0, "Kurbis, Harvest Celebrant");
    t.g.add_counters(obj(kurbis), "+1/+1", 2, None);
    let deaths = |t: &TestGame| {
        t.g.turn_events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    mtg_engine::events::Event::ZoneChange {
                        from: Zone::Battlefield,
                        ..
                    }
                )
            })
            .count()
    };
    cast_resolve(&mut t, P0, "Cackling Counterpart", &[obj(kurbis)]);
    assert_eq!(t.named_on_battlefield("Kurbis, Harvest Celebrant"), vec![kurbis]);
    assert_eq!(deaths(&t), 1, "the 0/0 token copy died");
    // A Clone put onto the battlefield (not cast).
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(kurbis)]);
    t.enter(P0, "Clone");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Kurbis, Harvest Celebrant"), vec![kurbis]);
    assert_eq!(deaths(&t), 2, "the Clone died as a 0/0");
    assert!(t.in_graveyard(P0, "Clone"));
}

// --- Rakalite, Summon: Magus Sisters ----------------------------------------------------------

#[test]
fn rakalite_returns_only_if_it_is_still_on_the_battlefield() {
    cr!("603.7a", "400.7");
    ruling!(
        "Rakalite",
        "Only returns to your hand if it is still on the battlefield at the end of the turn. If it leaves the battlefield, it does not return."
    );
    supported("Rakalite");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let rakalite = t.battlefield(P0, "Rakalite");
        t.lands(P0, "Wastes", 2);
        activate_resolve(&mut t, P0, rakalite, 0, &[pl(P0)]);
        if leaves {
            exile_now(&mut t, rakalite);
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Rakalite"), !leaves, "left: {leaves}");
        assert_eq!(t.in_exile("Rakalite"), leaves, "left: {leaves}");
    }
}

#[test]
fn magus_sisters_chapter_mode_is_chosen_at_random_as_it_goes_on_the_stack() {
    cr!("714.2b", "700.2", "603.3c");
    ruling!(
        "Summon: Magus Sisters",
        "As you put one of Summon: Magus Sisters's chapter abilities on the stack, you choose a mode at random. Players can respond to the ability knowing which mode was chosen."
    );
    supported("Summon: Magus Sisters");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    let sisters = t.enter(P0, "Summon: Magus Sisters");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.counters(sisters, "lore"), 1);
    assert_eq!(t.stack_len(), 1, "chapter I");
    let ability = t.g.stack[0];
    let chosen = t.obj_now(ability).stack.as_ref().unwrap().chosen.clone();
    assert_eq!(chosen.len(), 1, "one mode, already chosen on the stack");
    assert!(chosen[0].mode.is_some());
    // No player chose it.
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
}


#[test]
fn palliation_accord_uses_palliation_counters_not_shield_counters() {
    cr!("122.1", "122.1c");
    ruling!(
        "Palliation Accord",
        "Palliation Accord has recieved an update to its original Oracle text. Notably, it now uses palliation counters instead of shield counters."
    );
    supported("Palliation Accord");
    let mut t = TestGame::new(2);
    let accord = t.battlefield(P0, "Palliation Accord");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(accord, "palliation"), 1);
    assert_eq!(t.counters(accord, "shield"), 0);
    // Removing it prevents the next 1 damage to P0 (not damage to the enchantment).
    activate_resolve(&mut t, P0, accord, 0, &[]);
    assert_eq!(t.counters(accord, "palliation"), 0);
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 18);
}
