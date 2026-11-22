//! Rulings batch P046 — when a cost reduction is determined: after the spell moves to the
//! stack (CR 601.2a), locked in before mana abilities are activated (CR 601.2f–g), with no
//! player able to act in between (CR 601.2), and unaffected once the spell is cast.

use crate::r_p046_common::*;
use crate::r_s01_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Locked in before mana abilities: sacrificing what the reduction counted to pay for it.
// ---------------------------------------------------------------------------------------

/// P0 casts `spell` (targets `targets`) with only the mana sources set up: the cast
/// succeeds, every one of `sacrificed` (counted by the reduction) is gone, and the spell
/// is on the stack.
fn cast_sacrificing(t: &mut TestGame, spell: ObjectId, targets: &[Entity], sacrificed: &[ObjectId]) {
    let name = t.g.obj(spell).chars.name.to_string();
    let mut b = t.cast(P0, spell);
    for e in targets {
        b = b.target(*e);
    }
    b.go();
    assert_eq!(t.zone(spell), Zone::Stack, "{name}");
    for s in sacrificed {
        assert!(!t.on_battlefield(*s), "{name}: not sacrificed for mana");
    }
}

#[test]
fn stoic_rebuttal_counts_artifacts_before_mana_abilities() {
    cr!("601.2f", "601.2g");
    ruling!(
        "Stoic Rebuttal",
        "if you control three artifacts, you could determine that Stoic Rebuttal costs {U}{U} to cast, then sacrifice one of those artifacts to activate a mana ability"
    );
    supported("Stoic Rebuttal");
    supported("Lotus Petal");
    let mut t = TestGame::new(2);
    let target = p1_shocks(&mut t, Entity::Player(P0));
    // Three Lotus Petals ("{T}, Sacrifice this artifact: Add one mana of any color") are
    // P0's only mana sources: two of them pay {U}{U}.
    let petals: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Lotus Petal")).collect();
    let s = t.hand(P0, "Stoic Rebuttal");
    cast_sacrificing(&mut t, s, &[Entity::Object(target)], &[]);
    assert_eq!(petals.iter().filter(|p| t.on_battlefield(**p)).count(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn fungal_colossus_keeps_its_cost_after_sacrificing_a_land_for_mana() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Fungal Colossus",
        "If the number of differently named lands you control changes while activating mana abilities (probably because you sacrificed one or more lands), the cost to cast Fungal Colossus remains what you previously determined."
    );
    supported("Fungal Colossus");
    // Forest, Island and Crystal Vein ("{T}, Sacrifice this land: Add {C}{C}"): {3}{G}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    let vein = t.battlefield(P0, "Crystal Vein");
    let s = t.hand(P0, "Fungal Colossus");
    cast_sacrificing(&mut t, s, &[], &[vein]);
}

#[test]
fn orysa_keeps_its_cost_after_sacrificing_toughness_for_mana() {
    cr!("601.2f", "601.2g");
    ruling!(
        "Orysa, Tide Choreographer",
        "If the total toughness of creatures you control changes while activating mana abilities, the cost to cast Orysa remains what you previously determined."
    );
    supported("Orysa, Tide Choreographer");
    // Wall of Stone (0/8) and Grizzly Bears: total toughness 10, {1}{U}. An Island and
    // Ashnod's Altar ("Sacrifice a creature: Add {C}{C}") pay it.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Stone");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Ashnod's Altar");
    t.lands(P0, "Island", 1);
    let s = t.hand(P0, "Orysa, Tide Choreographer");
    cast_sacrificing(&mut t, s, &[], &[]);
    assert!(!t.on_battlefield(wall) || !t.on_battlefield(b));
}

#[test]
fn the_skullspore_nexus_keeps_its_cost_after_sacrificing_power_for_mana() {
    cr!("601.2f", "601.2g");
    ruling!(
        "The Skullspore Nexus",
        "If the greatest power among creatures you control changes while activating mana abilities, the cost to cast The Skullspore Nexus remains what you previously determined."
    );
    supported("The Skullspore Nexus");
    // Hill Giant (power 3): {3}{G}{G}, paid by two Forests, Wastes and the Giant
    // sacrificed to Ashnod's Altar.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Ashnod's Altar");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 1);
    let s = t.hand(P0, "The Skullspore Nexus");
    cast_sacrificing(&mut t, s, &[], &[giant]);
}

#[test]
fn draconic_lore_keeps_its_cost_after_sacrificing_the_dragon_for_mana() {
    cr!("601.2f", "601.2g");
    ruling!(
        "Draconic Lore",
        "if you sacrifice that Dragon as part of a mana ability, the cost won't be increased"
    );
    supported("Draconic Lore");
    // Shivan Dragon: {3}{U}, paid by an Island, Wastes and the Dragon sacrificed to
    // Ashnod's Altar.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Shivan Dragon");
    t.battlefield(P0, "Ashnod's Altar");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let s = t.hand(P0, "Draconic Lore");
    cast_sacrificing(&mut t, s, &[], &[dragon]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn ghalta_keeps_its_cost_after_sacrificing_a_creature_for_mana() {
    cr!("601.2f", "601.2g");
    ruling!(
        "Ghalta, Primal Hunger",
        "The total cost to cast Ghalta is locked in before you pay that cost. For example, if you control three 2/2 creatures"
    );
    supported("Ghalta, Primal Hunger");
    // Three Bears: {4}{G}{G}, paid by two Forests, two Wastes and a Bears sacrificed to
    // Ashnod's Altar.
    let mut t = TestGame::new(2);
    let bs = bears(&mut t, P0, 3);
    t.battlefield(P0, "Ashnod's Altar");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 2);
    let s = t.hand(P0, "Ghalta, Primal Hunger");
    cast_sacrificing(&mut t, s, &[], &[]);
    assert_eq!(bs.iter().filter(|b| t.on_battlefield(**b)).count(), 2);
}

#[test]
fn hour_of_revelation_keeps_its_cost_after_sacrificing_permanents_for_mana() {
    cr!("601.2f", "601.2g");
    ruling!(
        "Hour of Revelation",
        "You may sacrifice three Etherium Cells to pay this cost, even though there are no longer ten or more nonland permanents after you produce {W}{W}{W}."
    );
    supported("Hour of Revelation");
    // Ten nonland permanents: three Lotus Petals (the only mana sources) and seven Bears.
    let mut t = TestGame::new(2);
    let petals: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Lotus Petal")).collect();
    bears(&mut t, P1, 7);
    let s = t.hand(P0, "Hour of Revelation");
    cast_sacrificing(&mut t, s, &[], &petals);
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

// ---------------------------------------------------------------------------------------
// Determined after the spell moves to the stack (CR 601.2a, 601.2f).
// ---------------------------------------------------------------------------------------

#[test]
fn the_skullspore_nexus_uses_the_power_after_it_left_the_hand() {
    cr!("601.2a", "601.2f");
    ruling!(
        "The Skullspore Nexus",
        "The first step of casting a spell is to move it to the stack. If this causes the greatest power among creatures you control to change"
    );
    supported("Maro");
    // Maro ("power and toughness are each equal to the number of cards in your hand") is
    // 5/5 with the Nexus and four other cards in hand, 4/4 once the Nexus is on the stack:
    // {2}{G}{G}.
    let mut t = TestGame::new(2);
    let maro = t.battlefield(P0, "Maro");
    let s = t.hand(P0, "The Skullspore Nexus");
    for _ in 0..4 {
        t.hand(P0, "Grizzly Bears");
    }
    assert_eq!(t.pt(maro), (5, 5));
    assert_eq!(cost_to_cast(&mut t, P0, s, &[]), Some(4));
}

#[test]
fn orysa_uses_the_toughness_after_it_left_the_hand() {
    cr!("601.2a", "601.2f");
    ruling!(
        "Orysa, Tide Choreographer",
        "The first step of casting a spell is to move it to the stack. If this causes the total toughness of creatures you control to change"
    );
    // Ten cards in hand with Orysa: Maro is 10/10 but 9/9 as Orysa is cast: no reduction.
    let mut t = TestGame::new(2);
    let maro = t.battlefield(P0, "Maro");
    let s = t.hand(P0, "Orysa, Tide Choreographer");
    for _ in 0..9 {
        t.hand(P0, "Grizzly Bears");
    }
    assert_eq!(t.pt(maro), (10, 10));
    assert_eq!(cost_to_cast(&mut t, P0, s, &[]), Some(5));
    // Eleven: 10/10 on the stack: {3} less.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Maro");
    let s = t.hand(P0, "Orysa, Tide Choreographer");
    for _ in 0..10 {
        t.hand(P0, "Grizzly Bears");
    }
    assert_eq!(cost_to_cast(&mut t, P0, s, &[]), Some(2));
}

// ---------------------------------------------------------------------------------------
// No player can act while a spell is being cast (CR 601.2).
// ---------------------------------------------------------------------------------------

/// P0 casts `name` (set up by `setup`, which returns the targets) for `expected` mana,
/// and no player is given priority or any decision but P0's casting choices meanwhile.
fn nobody_acts_while_casting(
    name: &str,
    expected: usize,
    setup: impl FnOnce(&mut TestGame) -> Vec<Entity>,
) {
    supported(name);
    let mut t = TestGame::new(2);
    let targets = setup(&mut t);
    let s = t.hand(P0, name);
    let from = t.asked().len();
    assert_eq!(cost_to_cast(&mut t, P0, s, &targets), Some(expected), "{name}");
    let others = others_asked_since(&t, from, P0);
    assert!(others.is_empty(), "{name}: {others:?}");
}

/// P1's Shock on the stack (targeting P0).
fn shock(t: &mut TestGame) -> Vec<Entity> {
    vec![Entity::Object(p1_shocks(t, Entity::Player(P0)))]
}

#[test]
fn nobody_can_change_the_reduction_while_the_spell_is_cast() {
    cr!("601.2", "601.2f");
    ruling!(
        "Primeval Protector",
        "Once a player has announced that they are casting Primeval Protector, no player may take actions"
    );
    nobody_acts_while_casting("Primeval Protector", 8, |t| {
        bears(t, P1, 3);
        vec![]
    });
    ruling!(
        "The Lord of the Eagles",
        "Once a player has announced that they are casting The Lord of the Eagles, no player may take actions"
    );
    nobody_acts_while_casting("The Lord of the Eagles", 5, |t| {
        t.battlefield(P0, "Serra Angel");
        vec![]
    });
    ruling!(
        "Academy Journeymage",
        "Once you announce that you're casting Academy Journeymage, no player may take other actions until the spell's been paid for."
    );
    nobody_acts_while_casting("Academy Journeymage", 4, |t| {
        t.battlefield(P0, "Youthful Scholar");
        vec![]
    });
    ruling!(
        "Bolt Bend",
        "Once you announce that you're casting Bolt Bend, no player may take actions until the spell has been paid for."
    );
    nobody_acts_while_casting("Bolt Bend", 1, |t| {
        t.battlefield(P0, "Craw Wurm");
        shock(t)
    });
    ruling!(
        "Of One Mind",
        "Once you announce that you're casting Of One Mind, no player may take actions until the spell has been paid for."
    );
    nobody_acts_while_casting("Of One Mind", 1, |t| {
        t.battlefield(P0, "Order of the Sacred Bell");
        t.battlefield(P0, "Grizzly Bears");
        vec![]
    });
    ruling!(
        "Tentative Connection",
        "Once you announce that you're casting Tentative Connection, no player may take actions until the spell has been paid for."
    );
    nobody_acts_while_casting("Tentative Connection", 1, |t| {
        t.battlefield(P0, "Alley Strangler");
        vec![Entity::Object(t.battlefield(P1, "Grizzly Bears"))]
    });
    ruling!(
        "Winged Words",
        "Once you announce that you're casting Winged Words, no player may take actions until the spell has been paid for."
    );
    nobody_acts_while_casting("Winged Words", 2, |t| {
        t.battlefield(P0, "Serra Angel");
        vec![]
    });
    ruling!(
        "Cactarantula",
        "Once you announce that you’re casting Cactarantula, no player may take actions until the spell has been paid for."
    );
    nobody_acts_while_casting("Cactarantula", 5, |t| {
        t.lands(P0, "Desert of the Indomitable", 1);
        vec![]
    });
    ruling!(
        "Lookout's Dispersal",
        "Once you announce that you’re casting Lookout’s Dispersal, no player may take other actions until the spell’s been paid for."
    );
    nobody_acts_while_casting("Lookout's Dispersal", 2, |t| {
        t.battlefield(P0, "Swab Goblin");
        shock(t)
    });
    ruling!(
        "Wizard's Retort",
        "Once you announce that you’re casting Wizard’s Retort, no player may take other actions until the spell’s been paid for."
    );
    nobody_acts_while_casting("Wizard's Retort", 2, |t| {
        t.battlefield(P0, "Youthful Scholar");
        shock(t)
    });
    ruling!(
        "Fungal Colossus",
        "Once you announce you’re casting a spell, no player may take actions until after the spell has been paid for. Notably, opponents can’t try to remove lands you control"
    );
    nobody_acts_while_casting("Fungal Colossus", 4, |t| {
        for l in ["Forest", "Island", "Swamp"] {
            t.lands(P0, l, 1);
        }
        vec![]
    });
    ruling!(
        "Drag to the Roots",
        "Once you've announced that you're casting Drag to the Roots, players can't take any actions until you've finished doing so."
    );
    nobody_acts_while_casting("Drag to the Roots", 2, |t| {
        for c in ["Grizzly Bears", "Forest", "Shock", "Divination"] {
            t.graveyard(P0, c);
        }
        vec![Entity::Object(t.battlefield(P1, "Grizzly Bears"))]
    });
}

// ---------------------------------------------------------------------------------------
// Once cast, losing what the reduction counted doesn't matter.
// ---------------------------------------------------------------------------------------

#[test]
fn of_one_mind_is_unaffected_after_its_cast() {
    cr!("601.2f", "601.2i");
    ruling!(
        "Of One Mind",
        "Of One Mind is unaffected if you no longer control a Human and non-Human creature after you've cast it."
    );
    let mut t = TestGame::new(2);
    let human = t.battlefield(P0, "Order of the Sacred Bell");
    t.battlefield(P0, "Grizzly Bears");
    let s = t.hand(P0, "Of One Mind");
    assert_eq!(cost_to_cast(&mut t, P0, s, &[]), Some(1));
    let left = pool(&t, P0);
    bounce(&mut t, human);
    t.resolve_all();
    assert_eq!(pool(&t, P0), left);
    // The Human (returned to hand) and two cards drawn.
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn bolt_bend_is_unaffected_after_its_cast() {
    cr!("601.2f", "601.2i");
    ruling!(
        "Bolt Bend",
        "Once you've cast Bolt Bend, losing control of all creatures with power 4 or greater won't affect the spell or cause you to pay more mana."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let shock = p1_shocks(&mut t, Entity::Player(P0));
    let s = t.hand(P0, "Bolt Bend");
    assert_eq!(cost_to_cast(&mut t, P0, s, &[Entity::Object(shock)]), Some(1));
    let left = pool(&t, P0);
    bounce(&mut t, wurm);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(pool(&t, P0), left);
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
}

#[test]
fn tentative_connection_is_unaffected_after_its_cast() {
    cr!("601.2f", "601.2i");
    ruling!(
        "Tentative Connection",
        "Tentative Connection is unaffected if you no longer control a creature with menace after you've cast it. You won't have to pay an additional {3}."
    );
    let mut t = TestGame::new(2);
    let strangler = t.battlefield(P0, "Alley Strangler");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let s = t.hand(P0, "Tentative Connection");
    assert_eq!(cost_to_cast(&mut t, P0, s, &[Entity::Object(theirs)]), Some(1));
    let left = pool(&t, P0);
    bounce(&mut t, strangler);
    t.resolve_all();
    assert_eq!(pool(&t, P0), left);
    assert_eq!(t.obj_now(theirs).controller, P0);
}
