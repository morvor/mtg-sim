//! Trigger timing (gap-trigger-timing): a resolving spell's or ability's instructions are
//! followed one after another and each one's events are checked for triggers before the
//! next one happens (CR 603.2, 603.10, 608.2c) — a token created and then given counters
//! or equipped enters as it was created — and a keyword's variable defined by the effect
//! that grants it is determined as the keyword's triggered ability resolves (CR 608.2h).

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// Triggered abilities waiting to be put on the stack or on it whose text contains `text`.
fn triggered(t: &TestGame, text: &str) -> usize {
    let pending =
        t.g.pending_triggers
            .iter()
            .filter(|p| p.ability.text.contains(text))
            .count();
    pending + triggers_on_stack(t, text)
}

/// An enchantment for `p`: "Whenever a creature you control with power 2 or greater
/// enters, draw a card."
fn power_two_watcher(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let def = custom_card(
        "Power Watcher",
        "Enchantment",
        "{G}",
        None,
        "Whenever a creature you control with power 2 or greater enters, draw a card.",
    );
    t.custom(p, def, Zone::Battlefield)
}

/// The tokens `p` controls.
fn the_token(t: &TestGame, p: PlayerId) -> ObjectId {
    let v = tokens(t, p);
    assert_eq!(v.len(), 1, "expected one token");
    v[0]
}

/// Puts an Equipment with "When this Equipment enters, [create a token], then attach this
/// Equipment to it" onto the battlefield and resolves that ability, answering `pay` to a
/// "you may pay" question. Returns the Equipment and the number of objects on the stack
/// seen when the player was asked whether to pay.
fn equipment_enters(t: &mut TestGame, name: &str, pay: bool) -> (ObjectId, Vec<usize>) {
    let seen = watch(
        t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.stack.len(),
    );
    t.answer_yes(P0, pay);
    let eq = t.enter(P0, name);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the enters ability is on the stack");
    t.resolve();
    let seen = seen.lock().unwrap().clone();
    (eq, seen)
}

#[test]
fn zaxara_hydra_enters_as_a_0_0_before_getting_counters() {
    cr!("603.2", "603.6a", "603.10", "608.2c");
    ruling!(
        "Zaxara, the Exemplary",
        "The Hydra token enters the battlefield as a 0/0 creature. Any abilities that modify or trigger on this event apply. After the token is on the battlefield but before any player can take actions, +1/+1 counters are put onto the token."
    );
    supported("Zaxara, the Exemplary");
    supported("Wild Hypothesis");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zaxara, the Exemplary");
    // "Whenever a creature you control with power 3 or greater enters, draw a card."
    t.battlefield(P0, "Elemental Bond");
    // "Whenever another creature you control with power 2 or less enters, you may pay {1}.
    // If you do, draw a card."
    t.battlefield(P0, "Mentor of the Meek");
    let spell = t.hand(P0, "Wild Hypothesis");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    t.cast(P0, spell).x(3).go();
    t.settle();
    assert_eq!(triggered(&t, "Hydra"), 1);
    // Zaxara's ability resolves before the spell.
    t.resolve();
    let hydra = the_token(&t, P0);
    assert_eq!(t.pt(hydra), (3, 3));
    // The Hydra entered as a 0/0: Mentor of the Meek saw it, Elemental Bond didn't.
    assert_eq!(triggered(&t, "power 2 or less"), 1);
    assert_eq!(triggered(&t, "power 3 or greater"), 0);
}

#[test]
fn grip_of_phyresis_germ_enters_as_a_0_0_before_the_equipment_is_attached() {
    cr!("603.2", "603.6a", "608.2c", "701.3a");
    ruling!(
        "Grip of Phyresis",
        "The Germ token exists on the battlefield as a 0/0 for a brief moment before the Equipment becomes attached to it."
    );
    supported("Grip of Phyresis");
    let mut t = TestGame::new(2);
    // "Equipped creature gets +10/+10 and loses flying."
    let hammer = t.battlefield(P1, "Colossus Hammer");
    t.battlefield(P0, "Elemental Bond");
    t.battlefield(P0, "Mentor of the Meek");
    let grip = t.hand(P0, "Grip of Phyresis");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.cast(P0, grip).target(hammer).go();
    t.resolve();
    let germ = the_token(&t, P0);
    assert_eq!(t.obj_now(hammer).controller, P0);
    assert_eq!(t.obj_now(hammer).attached_to, Some(Entity::Object(germ)));
    assert_eq!(t.pt(germ), (10, 10));
    // Abilities that trigger on it entering saw a 0/0, not the equipped 10/10.
    assert_eq!(triggered(&t, "power 2 or less"), 1);
    assert_eq!(triggered(&t, "power 3 or greater"), 0);
}

#[test]
fn grip_of_phyresis_germ_without_a_toughness_boost_dies_but_you_keep_the_equipment() {
    cr!("704.5f", "608.2c");
    ruling!(
        "Grip of Phyresis",
        "If the Equipment doesn’t provide a toughness boost, your Germ token will have 0 toughness and die. You’ll still control the Equipment."
    );
    let mut t = TestGame::new(2);
    // "Equipped creature gets +2/+0."
    let splitter = t.battlefield(P1, "Bonesplitter");
    let grip = t.hand(P0, "Grip of Phyresis");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.cast(P0, grip).target(splitter).go();
    t.resolve();
    assert!(tokens(&t, P0).is_empty(), "the 2/0 Germ died");
    assert_eq!(t.obj_now(splitter).controller, P0);
    assert_eq!(t.obj_now(splitter).attached_to, None);
}

#[test]
fn ancestral_blade_soldier_enters_as_a_1_1_and_is_equipped_before_sbas() {
    cr!("603.2", "603.6a", "608.2c", "704.3");
    ruling!(
        "Ancestral Blade",
        "The Soldier token that you create enters the battlefield as a 1/1 creature. Any abilities that trigger when a creature with a certain power enters the battlefield will see the token enter as a 1/1 creature."
    );
    ruling!(
        "Ancestral Blade",
        "No player may take any actions between the time you create the Soldier token and the time Ancestral Blade becomes attached to it."
    );
    supported("Ancestral Blade");
    let mut t = TestGame::new(2);
    power_two_watcher(&mut t, P0);
    let asked = t.asked().len();
    let (blade, _) = equipment_enters(&mut t, "Ancestral Blade", true);
    let soldier = the_token(&t, P0);
    assert_eq!(t.obj_now(blade).attached_to, Some(Entity::Object(soldier)));
    assert_eq!(t.pt(soldier), (2, 2));
    // It entered as a 1/1.
    assert_eq!(triggered(&t, "power 2 or greater"), 0);
    // Nobody got priority while the ability resolved.
    assert!(!asked_since(&t, asked)
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
}

#[test]
fn ancestral_blade_soldier_survives_illness_in_the_ranks() {
    cr!("608.2c", "704.3", "704.5f");
    ruling!(
        "Ancestral Blade",
        "For example, if Illness in the Ranks (\"Creature tokens get -1/-1\") is on the battlefield, the Soldier will enter as a 0/0 creature, but then Ancestral Blade will make it a 1/1 and the token will survive."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Illness in the Ranks");
    // "Whenever another creature you control with power 2 or less enters, ..."
    t.battlefield(P0, "Mentor of the Meek");
    equipment_enters(&mut t, "Ancestral Blade", true);
    let soldier = the_token(&t, P0);
    assert_eq!(t.pt(soldier), (1, 1));
    assert_eq!(triggered(&t, "power 2 or less"), 1);
}

#[test]
fn headsplitter_assassin_enters_as_a_1_1() {
    cr!("603.2", "603.6a", "608.2c");
    ruling!(
        "Headsplitter",
        "The Assassin token that you create enters the battlefield as a 1/1 creature. Any abilities that trigger when a creature with a certain power enters the battlefield will see the token enter as a 1/1 creature."
    );
    ruling!(
        "Headsplitter",
        "No player may take any actions between the time you create the Assassin token and the time Headsplitter becomes attached to it."
    );
    supported("Headsplitter");
    let mut t = TestGame::new(2);
    power_two_watcher(&mut t, P0);
    let asked = t.asked().len();
    let (eq, _) = equipment_enters(&mut t, "Headsplitter", true);
    let assassin = the_token(&t, P0);
    assert_eq!(t.obj_now(eq).attached_to, Some(Entity::Object(assassin)));
    assert_eq!(t.pt(assassin), (2, 1));
    assert_eq!(triggered(&t, "power 2 or greater"), 0);
    assert!(!asked_since(&t, asked)
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
}

#[test]
fn kyoshi_battle_fan_ally_enters_as_a_1_1() {
    cr!("603.2", "603.6a", "608.2c");
    ruling!(
        "Kyoshi Battle Fan",
        "The Ally creature token enters as a 1/1 creature. Any abilities that trigger when a creature with a certain power or toughness enters will see the token enter as a 1/1 creature."
    );
    supported("Kyoshi Battle Fan");
    let mut t = TestGame::new(2);
    power_two_watcher(&mut t, P0);
    let (eq, _) = equipment_enters(&mut t, "Kyoshi Battle Fan", true);
    let ally = the_token(&t, P0);
    assert_eq!(t.obj_now(eq).attached_to, Some(Entity::Object(ally)));
    assert_eq!(t.pt(ally), (2, 1));
    assert_eq!(triggered(&t, "power 2 or greater"), 0);
}

#[test]
fn elven_bow_elf_warrior_enters_as_a_1_1() {
    cr!("603.2", "603.6a", "608.2c", "608.2d");
    ruling!(
        "Elven Bow",
        "The Elf Warrior creature token enters the battlefield as a 1/1 creature. Any abilities that trigger when a creature with a certain power or toughness enters the battlefield will see the token enter as a 1/1 creature."
    );
    ruling!(
        "Elven Bow",
        "You decide whether to pay {2} as the enters-the-battlefield ability resolves. If you do, you immediately create the Elf Warrior creature token and attach Elven Bow to it."
    );
    supported("Elven Bow");
    let mut t = TestGame::new(2);
    power_two_watcher(&mut t, P0);
    t.lands(P0, "Wastes", 2);
    let (eq, seen) = equipment_enters(&mut t, "Elven Bow", true);
    // Asked while the ability was resolving (still on the stack).
    assert_eq!(seen, vec![1]);
    let elf = the_token(&t, P0);
    assert_eq!(t.obj_now(eq).attached_to, Some(Entity::Object(elf)));
    assert_eq!(t.pt(elf), (2, 3));
    assert_eq!(triggered(&t, "power 2 or greater"), 0);
}

#[test]
fn valkyries_sword_angel_warrior_enters_as_a_4_4() {
    cr!("603.2", "603.6a", "608.2c", "608.2d");
    ruling!(
        "Valkyrie's Sword",
        "The Angel Warrior creature token enters the battlefield as a 4/4 creature. Any abilities that trigger when a creature with a certain power or toughness enters the battlefield will see the token enter as a 4/4 creature."
    );
    ruling!(
        "Valkyrie's Sword",
        "You decide whether to pay {4}{W} as the enters-the-battlefield ability resolves. If you do, you immediately create the Angel Warrior creature token and attach Valkyrie's Sword to it."
    );
    supported("Valkyrie's Sword");
    let mut t = TestGame::new(2);
    // "Whenever a creature you control with power 4 or greater enters, draw a card."
    t.battlefield(P0, "Garruk's Uprising");
    // "Whenever a creature you control with power 5 or greater enters, you may put a
    // +1/+1 counter on this creature."
    t.battlefield(P0, "Godtracker of Jund");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 4);
    let (eq, seen) = equipment_enters(&mut t, "Valkyrie's Sword", true);
    assert_eq!(seen, vec![1]);
    let angel = the_token(&t, P0);
    assert_eq!(t.obj_now(eq).attached_to, Some(Entity::Object(angel)));
    assert_eq!(t.pt(angel), (6, 5));
    assert_eq!(triggered(&t, "power 4 or greater"), 1);
    assert_eq!(triggered(&t, "power 5 or greater"), 0);
}

#[test]
fn draugrs_helm_zombie_berserker_enters_as_a_2_2() {
    cr!("603.2", "603.6a", "608.2c", "608.2d");
    ruling!(
        "Draugr's Helm",
        "The Zombie Berserker creature token enters the battlefield as a 2/2 creature. Any abilities that trigger when a creature with a certain power or toughness enters the battlefield will see the token enter as a 2/2 creature."
    );
    ruling!(
        "Draugr's Helm",
        "You decide whether to pay {2}{B} as the enters-the-battlefield ability resolves. If you do, you immediately create the Zombie Berserker creature token and attach Draugr's Helm to it."
    );
    supported("Draugr's Helm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Bond");
    t.battlefield(P0, "Mentor of the Meek");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    let (eq, seen) = equipment_enters(&mut t, "Draugr's Helm", true);
    assert_eq!(seen, vec![1]);
    let zombie = the_token(&t, P0);
    assert_eq!(t.obj_now(eq).attached_to, Some(Entity::Object(zombie)));
    assert_eq!(t.pt(zombie), (4, 4));
    assert_eq!(triggered(&t, "power 2 or less"), 1);
    assert_eq!(triggered(&t, "power 3 or greater"), 0);
}

#[test]
fn dwarven_hammer_dwarf_berserker_enters_as_a_2_1() {
    cr!("603.2", "603.6a", "608.2c", "608.2d");
    ruling!(
        "Dwarven Hammer",
        "The Dwarf Berserker creature token enters the battlefield as a 2/1 creature. Any abilities that trigger when a creature with a certain power enters the battlefield will see the token enter as a 2/1 creature."
    );
    ruling!(
        "Dwarven Hammer",
        "You decide whether to pay {2} as the enters-the-battlefield ability resolves. If you do, you immediately create the Dwarf Berserker creature token and attach Dwarven Hammer to it."
    );
    supported("Dwarven Hammer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Bond");
    t.battlefield(P0, "Mentor of the Meek");
    t.lands(P0, "Wastes", 2);
    let (eq, seen) = equipment_enters(&mut t, "Dwarven Hammer", true);
    assert_eq!(seen, vec![1]);
    let dwarf = the_token(&t, P0);
    assert_eq!(t.obj_now(eq).attached_to, Some(Entity::Object(dwarf)));
    assert_eq!(t.pt(dwarf), (5, 1));
    assert_eq!(triggered(&t, "power 2 or less"), 1);
    assert_eq!(triggered(&t, "power 3 or greater"), 0);
}

#[test]
fn cori_steel_cutter_monk_enters_as_a_1_1_and_you_may_attach_the_cutter() {
    cr!("603.2", "603.6a", "608.2c", "608.2d");
    ruling!(
        "Cori-Steel Cutter",
        "The Monk token that you create enters the battlefield as a 1/1 creature. Any abilities that trigger when a creature with a certain power enters the battlefield will see the token enter as a 1/1 creature."
    );
    ruling!(
        "Cori-Steel Cutter",
        "No player may take any actions between the time you create the Monk token and the time you choose whether to attach Cori-Steel Cutter to it."
    );
    supported("Cori-Steel Cutter");
    let mut t = TestGame::new(2);
    let cutter = t.battlefield(P0, "Cori-Steel Cutter");
    power_two_watcher(&mut t, P0);
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    // Flurry: the second spell this turn.
    assert_eq!(t.stack_len(), 2);
    // When asked whether to attach the Cutter, the Monk is already there.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| {
            g.permanents()
                .filter(|o| o.is_token() && o.chars.has_subtype("Monk"))
                .count()
        },
    );
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(*seen.lock().unwrap(), vec![1]);
    let monk = the_token(&t, P0);
    assert_eq!(t.obj_now(cutter).attached_to, Some(Entity::Object(monk)));
    assert_eq!(t.pt(monk), (2, 2));
    assert_eq!(triggered(&t, "power 2 or greater"), 0);
}

#[test]
fn fumiko_bushido_bonus_is_calculated_as_the_trigger_resolves() {
    cr!("702.1b", "702.45a", "608.2h");
    ruling!(
        "Fumiko the Lowblood",
        "X is variable. The bushido bonus is calculated each time Fumiko's bushido trigger resolves, based on the number of attackers at that time."
    );
    let mut t = TestGame::new(2);
    // Fumiko the Lowblood: 3/2, "Fumiko has bushido X, where X is the number of attacking
    // creatures."
    let fumiko = t.battlefield(P1, "Fumiko the Lowblood");
    let b1 = t.battlefield(P0, "Grizzly Bears");
    let b2 = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(b1, Entity::Player(P1)), (b2, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(fumiko, b1)]),
    );
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::DeclareBlockers && g.turn.stage == Stage::Priority
    });
    assert!(ok);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the bushido trigger");
    // In response, the other attacker is destroyed: one attacking creature is left.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(b2).go();
    t.resolve();
    assert!(!t.on_battlefield(b2));
    t.resolve();
    assert_eq!(t.pt(fumiko), (4, 3));
}

#[test]
fn ulamog_annihilator_counts_counters_as_the_ability_resolves() {
    cr!("702.1b", "702.86a", "608.2h");
    ruling!(
        "Ulamog, the Defiler",
        "Use the number of +1/+1 counters on Ulamog at the time its annihilator ability resolves to determine how many permanents defending player should sacrifice."
    );
    ruling!(
        "Ulamog, the Defiler",
        "Annihilator abilities trigger and resolve during the declare attackers step."
    );
    let mut t = TestGame::new(2);
    let ulamog = t.battlefield(P0, "Ulamog, the Defiler");
    t.g.add_counters(Entity::Object(ulamog), "+1/+1", 2, None);
    t.g.recompute();
    t.lands(P1, "Plains", 5);
    attack_with(&mut t, &[(ulamog, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1, "annihilator 2 triggered");
    // In response, Ulamog gets another +1/+1 counter.
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Battlegrowth");
    t.cast(P0, growth).target(ulamog).go();
    t.resolve();
    assert_eq!(t.counters(ulamog, "+1/+1"), 3);
    t.resolve();
    let left = t.g.permanents().filter(|o| o.controller == P1).count();
    assert_eq!(left, 2, "the defending player sacrificed three permanents");
    // All of this happened in the declare attackers step, before blockers.
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
}

#[test]
fn volcano_hellion_echo_cost_is_your_life_total_as_the_ability_resolves() {
    cr!("702.1b", "702.30a", "608.2h");
    ruling!(
        "Volcano Hellion",
        "Volcano Hellion's echo cost constantly changes; it's not locked in when it enters. The echo cost you pay is equal to your life total as the echo triggered ability resolves."
    );
    let mut t = TestGame::new(2);
    let hellion = t.enter(P0, "Volcano Hellion");
    t.resolve_all();
    let lands = t.lands(P0, "Mountain", 20);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the echo trigger");
    // In response, P0 is dealt 3 damage: the echo cost becomes {17}.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 17);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(hellion));
    let tapped = lands.iter().filter(|l| t.obj_now(**l).tapped).count();
    assert_eq!(tapped, 17);
}

#[test]
fn giants_amulet_giant_wizard_enters_as_a_4_4() {
    cr!("603.2", "603.6a", "608.2c", "608.2d");
    ruling!(
        "Giant's Amulet",
        "The Giant Wizard creature token enters the battlefield as a 4/4 creature. Any abilities that trigger when a creature with a certain toughness enters the battlefield will see the token enter as a 4/4 creature."
    );
    ruling!(
        "Giant's Amulet",
        "You decide whether to pay {3}{U} as the enters-the-battlefield ability resolves. If you do, you immediately create the Giant Wizard creature token and attach Giant's Amulet to it."
    );
    supported("Giant's Amulet");
    let mut t = TestGame::new(2);
    let def = custom_card(
        "Toughness Watcher",
        "Enchantment",
        "{G}",
        None,
        "Whenever a creature you control with toughness 5 or greater enters, draw a card.",
    );
    t.custom(P0, def, Zone::Battlefield);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    let (eq, seen) = equipment_enters(&mut t, "Giant's Amulet", true);
    assert_eq!(seen, vec![1]);
    let giant = the_token(&t, P0);
    assert_eq!(t.obj_now(eq).attached_to, Some(Entity::Object(giant)));
    assert_eq!(t.pt(giant), (4, 5));
    assert_eq!(triggered(&t, "toughness 5 or greater"), 0);
}

#[test]
fn wolfriders_saddle_wolf_enters_as_a_2_2() {
    cr!("603.2", "603.6a", "608.2c");
    ruling!(
        "Wolfrider's Saddle",
        "The Wolf token that you create enters the battlefield as a 2/2 creature. Any abilities that trigger when a creature with a certain power enters the battlefield will see the token enter as a 2/2 creature before Wolfrider's Saddle becomes attached."
    );
    ruling!(
        "Wolfrider's Saddle",
        "No player may take any actions between the time you create the Wolf token and the time Wolfrider's Saddle becomes attached to it."
    );
    supported("Wolfrider's Saddle");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Bond");
    t.battlefield(P0, "Mentor of the Meek");
    let asked = t.asked().len();
    let (eq, _) = equipment_enters(&mut t, "Wolfrider's Saddle", true);
    let wolf = the_token(&t, P0);
    assert_eq!(t.obj_now(eq).attached_to, Some(Entity::Object(wolf)));
    assert_eq!(t.pt(wolf), (3, 3));
    assert_eq!(triggered(&t, "power 2 or less"), 1);
    assert_eq!(triggered(&t, "power 3 or greater"), 0);
    assert!(!asked_since(&t, asked)
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
}

#[test]
fn mask_of_immolation_attaches_before_anyone_can_act() {
    cr!("608.2c", "117.2e");
    ruling!(
        "Mask of Immolation",
        "No player may take any actions between the time you create the Elemental token and the time Mask of Immolation becomes attached to it."
    );
    supported("Mask of Immolation");
    let mut t = TestGame::new(2);
    let asked = t.asked().len();
    let (eq, _) = equipment_enters(&mut t, "Mask of Immolation", true);
    let elemental = the_token(&t, P0);
    assert_eq!(t.obj_now(eq).attached_to, Some(Entity::Object(elemental)));
    assert!(!asked_since(&t, asked)
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
}

#[test]
fn amassed_army_tokens_enter_as_0_0() {
    cr!("701.47a", "603.2", "603.6a");
    ruling!(
        "Lazotep Sliver",
        "If you don't control an Army, the Sliver Army token you create enters the battlefield as a 0/0 creature before receiving counters."
    );
    ruling!(
        "Mindless Conscription",
        "If you don't control an Army, the Zombie Army token that you create enters the battlefield as a 0/0 creature. Any abilities that trigger when a creature with a certain power enters the battlefield, such as that of Mentor of the Meek, will see the token enter as a 0/0 creature before it gets +1/+1 counters."
    );
    supported("Lazotep Sliver");
    supported("Mindless Conscription");
    // Mindless Conscription: "When this enchantment enters ..., amass Zombies 3."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elemental Bond");
    t.battlefield(P0, "Mentor of the Meek");
    t.enter(P0, "Mindless Conscription");
    t.resolve();
    let army = the_token(&t, P0);
    assert_eq!(t.pt(army), (3, 3));
    assert_eq!(triggered(&t, "power 2 or less"), 1);
    assert_eq!(triggered(&t, "power 3 or greater"), 0);
    // Lazotep Sliver (4/4): "Whenever a nontoken Sliver you control dies, amass Slivers 2."
    let mut t = TestGame::new(2);
    let sliver = t.battlefield(P0, "Lazotep Sliver");
    power_two_watcher(&mut t, P0);
    t.battlefield(P0, "Mentor of the Meek");
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(sliver).go();
    t.resolve();
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(sliver).go();
    t.resolve();
    assert!(!t.on_battlefield(sliver));
    t.resolve();
    let army = the_token(&t, P0);
    assert_eq!(t.pt(army), (2, 2));
    assert!(t.obj_now(army).chars.has_subtype("Sliver"));
    assert_eq!(triggered(&t, "power 2 or less"), 1);
    assert_eq!(triggered(&t, "power 2 or greater"), 0);
}

#[test]
fn kodama_soulshift_x_is_determined_as_it_dies_and_counts_itself() {
    cr!("702.46a", "603.10a", "702.1b");
    ruling!(
        "Kodama of the Center Tree",
        "Soulshift is a leaves the battlefield trigger, so the gamestate is referenced immediately before the Soulshift trigger to determine the value of X. Soulshift X includes Kodama of the Center Tree. So, X is always at least 1."
    );
    ruling!(
        "Kodama of the Center Tree",
        "Kodama of the Center Tree can return itself to its owner’s hand if you control five or more Spirits when it is put into a graveyard from the battlefield."
    );
    supported("Kodama of the Center Tree");
    let mut t = TestGame::new(2);
    // "Kodama of the Center Tree has soulshift X, where X is the number of Spirits you
    // control." Itself and four other Spirits: X is 5 as it dies (its mana value is 5),
    // although only four Spirits are left afterward.
    let kodama = t.battlefield(P0, "Kodama of the Center Tree");
    for _ in 0..4 {
        t.battlefield(P0, "Kami of Ancient Law");
    }
    assert_eq!(t.pt(kodama), (5, 5));
    t.lands(P1, "Swamp", 2);
    t.lands(P1, "Wastes", 1);
    let murder = t.hand(P1, "Murder");
    t.cast(P1, murder).target(kodama).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Kodama of the Center Tree"));
    t.settle();
    assert_eq!(t.stack_len(), 1, "soulshift 5 triggered and targeted Kodama itself");
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.in_hand(P0, "Kodama of the Center Tree"));
}
