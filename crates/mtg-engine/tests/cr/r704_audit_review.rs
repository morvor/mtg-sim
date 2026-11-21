//! CR 704 audit (review): one check performs each applicable state-based action in full
//! (704.5r removes all the counters beyond the limit at once, not one batch per check),
//! and an Equipment becoming unattached in the same check as its creature dies doesn't
//! change the creature's last known information (704.8).

use crate::r603_audit_common::*;
use mtg_engine::ability::*;
use mtg_engine::eval::Ctx;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn one_check_removes_every_counter_beyond_rasputins_limit() {
    cr!("704.5r", "704.3", "707.4");
    ruling!(
        "Rasputin Dreamweaver",
        "If a permanent with eight or more dream counters on it becomes a copy of Rasputin Dreamweaver (or just gains its last ability), all dream counters except seven are removed from it as a state-based action."
    );
    supported("Rasputin Dreamweaver");
    supported("Unstable Shapeshifter");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Unstable Shapeshifter");
    t.g.add_counters(Entity::Object(shifter), "dream", 9, None);
    t.settle();
    // Rasputin enters: the Shapeshifter becomes a copy of it, with nine dream counters.
    t.enter(P1, "Rasputin Dreamweaver");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.resolve_top();
    assert_eq!(t.obj_now(shifter).name(), "Rasputin Dreamweaver");
    assert_eq!(t.counters(shifter, "dream"), 9);
    // One check removes all but seven; nothing is left for the next one.
    assert!(t.g.check_sbas());
    assert_eq!(t.counters(shifter, "dream"), 7);
    assert!(!t.g.check_sbas());
}

/// Gives `id` protection from artifacts until end of turn.
fn protect_from_artifacts(t: &mut TestGame, id: ObjectId) {
    let mut prot =
        mtg_engine::keywords::Keyword::new(mtg_engine::keywords::KeywordKind::Protection);
    prot.filter = Some(Filter::Type(CardType::Artifact));
    let mut ctx = Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(prot)],
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
}

#[test]
fn a_creature_dying_as_its_equipment_becomes_unattached_dies_equipped() {
    cr!("704.3", "704.8", "704.5g", "704.5n", "702.16c");
    supported("Bonesplitter");
    supported("Death's Presence");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Death's Presence");
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(splitter, Entity::Object(bears)));
    t.settle();
    assert_eq!(t.pt(bears), (4, 2));
    // In one check: the Bears have lethal damage, and Bonesplitter is attached to a
    // creature with protection from artifacts. The Bears die as a 4/2.
    let source = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(source, Entity::Object(bears), 2, false);
    protect_from_artifacts(&mut t, bears);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.settle();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.g.obj(bears).power(), 4, "last known information");
    assert!(t.on_battlefield(splitter));
    assert_eq!(t.obj_now(splitter).attached_to, None);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 4);
}
