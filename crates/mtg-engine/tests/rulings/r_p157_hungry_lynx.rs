//! Rulings batch P157 — Hungry Lynx: protection from Rats (CR 702.16), the Rat token its
//! end step trigger gives an opponent, and its "Whenever a Rat dies" trigger (leaves-the-
//! battlefield abilities look back, CR 603.10a; tokens go to the graveyard, CR 111.7).

use crate::r_p157_common::*;
use crate::r_p180_common::{can_block, plus};
use crate::r_s01_common::{attack_with, custom_card};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 controls Hungry Lynx; at P0's end step P1 creates the Rat. Returns (Lynx, Rat).
fn lynx_and_rat(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let lynx = t.battlefield(P0, "Hungry Lynx");
    t.answer_targets(P0, &[pl(P1)]);
    end_step(t);
    let rats = with_subtype(t, P1, "Rat");
    assert_eq!(rats.len(), 1);
    (lynx, rats[0])
}

/// A custom Rat creature for P1 with the given rules text.
fn custom_rat(t: &mut TestGame, text: &str) -> ObjectId {
    let def = custom_card("Test Rat", "Creature — Rat", "{B}", Some((1, 1)), text);
    t.custom(P1, def, Zone::Battlefield)
}

#[test]
fn hungry_lynx_the_opponent_who_creates_the_rat_owns_it() {
    cr!("111.2", "603.3d");
    ruling!(
        "Hungry Lynx",
        "The opponent who creates the Rat token is the owner of that token."
    );
    supported("Hungry Lynx");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Hungry Lynx");
    t.answer_targets(P0, &[pl(P2)]);
    end_step(&mut t);
    let rats = with_subtype(&t, P2, "Rat");
    assert_eq!(rats.len(), 1);
    let rat = t.obj_now(rats[0]);
    assert_eq!(rat.owner, P2);
    assert!(rat.has_keyword(mtg_engine::keywords::KeywordKind::Deathtouch));
    assert!(with_subtype(&t, P1, "Rat").is_empty());
}

#[test]
fn hungry_lynx_protection_from_rats_means_four_things() {
    cr!("702.16b", "702.16c", "702.16d", "702.16e", "704.5m");
    ruling!(
        "Hungry Lynx",
        "If a permanent has protection from Rats, it means four things: 1) Damage from a source that’s a Rat that would be dealt to that permanent is prevented. 2) Auras and Equipment that are somehow Rats can’t be attached to that permanent. 3) Rats can’t block that permanent. 4) That permanent can’t be the target of abilities of Rats or spells that are somehow Rats."
    );
    let mut t = TestGame::new(2);
    let (lynx, rat) = lynx_and_rat(&mut t);
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // 1) Damage from a Rat to a Cat P0 controls is prevented; to a non-Cat it isn't.
    damage(&mut t, rat, obj(lions), 1);
    damage(&mut t, rat, obj(bears), 1);
    assert_eq!(t.obj_now(lions).damage, 0);
    assert!(t.on_battlefield(lions));
    assert!(!t.on_battlefield(bears), "the Rat's deathtouch damage was dealt");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // 2) An Aura that's a Rat attached to the Lynx is put into the graveyard.
    let def = custom_card(
        "Rat Mantle",
        "Kindred Enchantment — Rat Aura",
        "{B}",
        None,
        "Enchant creature\nEnchanted creature gets +1/+1.",
    );
    let aura = t.custom(P1, def.clone(), Zone::Battlefield);
    t.g.attach(aura, obj(lynx));
    t.settle();
    assert!(!t.on_battlefield(aura), "a Rat Aura can't be attached to the Lynx");
    // (The Rat Aura "died": the Lynx's last ability triggers.)
    t.resolve_all();
    let aura = t.custom(P1, def.clone(), Zone::Battlefield);
    t.g.attach(aura, obj(bears));
    t.settle();
    assert!(t.on_battlefield(aura));
    // 4) Spells that are Rats can't target it: the Rat Aura spell.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let spell = t.custom(P1, def, Zone::Hand(P1));
    lots(&mut t, P1);
    let m = mark(&t);
    t.answer_targets(P1, &[obj(bears)]);
    t.cast(P1, spell).try_go().unwrap();
    let cands = target_choices(&t, P1, m, "");
    assert!(!cands.is_empty());
    assert!(cands.iter().all(|c| !c.contains(&obj(lynx)) && !c.contains(&obj(lions))));
    assert!(cands[0].contains(&obj(bears)));
    t.resolve_all();
    // 4) Abilities of Rats can't target it.
    let pinger = custom_rat(&mut t, "{T}: This creature deals 1 damage to target creature.");
    let m = mark(&t);
    act(&mut t, P1, pinger, 0, &[obj(bears)]);
    let cands = target_choices(&t, P1, m, "");
    assert!(!cands.is_empty());
    assert!(cands.iter().all(|c| !c.contains(&obj(lynx)) && !c.contains(&obj(lions))));
    assert!(cands[0].contains(&obj(bears)));
    // 3) Rats can't block it (the Bears can be blocked).
    let mut t = TestGame::new(2);
    let (lynx, rat) = lynx_and_rat(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    attack_with(&mut t, &[(lynx, pl(P1)), (bears, pl(P1))]);
    assert!(!can_block(&mut t, rat, lynx));
    assert!(can_block(&mut t, rat, bears));
}

#[test]
fn hungry_lynx_nothing_else_is_prevented() {
    cr!("702.16f", "509.1b");
    ruling!(
        "Hungry Lynx",
        "Nothing other than the specified events is prevented or illegal. Hungry Lynx can’t block a Rat with the ability “This creature can’t be blocked,” for example."
    );
    let mut t = TestGame::new(2);
    let lynx = t.battlefield(P0, "Hungry Lynx");
    let rat = custom_rat(&mut t, "");
    let sneaky = custom_rat(&mut t, "This creature can't be blocked.");
    t.set_step(P1, mtg_engine::turn::Step::BeginningOfCombat);
    attack_with(&mut t, &[(rat, pl(P0)), (sneaky, pl(P0))]);
    // The Lynx may block a Rat, but not one that can't be blocked.
    assert!(can_block(&mut t, lynx, rat));
    assert!(!can_block(&mut t, lynx, sneaky));
}

#[test]
fn hungry_lynx_dying_with_a_rat_triggers_but_gets_no_counter() {
    cr!("603.10a", "603.2", "608.2b");
    ruling!(
        "Hungry Lynx",
        "If Hungry Lynx dies at the same time as a Rat, its ability triggers, but Hungry Lynx won't be on the battlefield as that ability resolves. It can’t be saved by the +1/+1 counter that would have been put on it. The same is true of any other Cats you control that die at the same time. Your remaining Cats will each get a +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    let (lynx, rat) = lynx_and_rat(&mut t);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let lions = t.battlefield(P0, "Savannah Lions");
    let guardian = t.battlefield(P0, "Pride Guardian");
    // Pyroclasm: 2 damage to each creature. The Lynx, the Lions and the Rat die together.
    cast_at(&mut t, P0, "Pyroclasm", &[]);
    t.resolve();
    assert!(!t.on_battlefield(lynx));
    assert!(!t.on_battlefield(lions));
    assert!(!t.on_battlefield(rat));
    assert!(t.in_graveyard(P0, "Hungry Lynx"));
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hungry Lynx"));
    assert!(t.in_graveyard(P0, "Savannah Lions"));
    // The Pride Guardian (0/3) survived the Pyroclasm and gets the counter.
    assert!(t.on_battlefield(guardian));
    assert_eq!(plus(&t, guardian), 1);
    assert_eq!(t.pt(guardian), (1, 4));
}

#[test]
fn hungry_lynx_a_sacrificed_or_destroyed_rat_token_dies() {
    cr!("111.7", "700.4", "704.5d");
    ruling!(
        "Hungry Lynx",
        "Tokens that are sacrificed or destroyed are put into their owner’s graveyard before ceasing to exist. If the token was a Rat, Hungry Lynx’s last ability will trigger."
    );
    // Sacrificed.
    let mut t = TestGame::new(2);
    let (lynx, rat) = lynx_and_rat(&mut t);
    t.g.sacrifice(rat, P1);
    t.g.flush_events();
    t.resolve_all();
    assert!(!t.on_battlefield(rat));
    assert_eq!(plus(&t, lynx), 1);
    // Destroyed.
    let mut t = TestGame::new(2);
    let (lynx, rat) = lynx_and_rat(&mut t);
    kill(&mut t, rat);
    t.resolve_all();
    assert_eq!(plus(&t, lynx), 1);
    assert_eq!(t.pt(lynx), (3, 3));
}
