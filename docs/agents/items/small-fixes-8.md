# small-fixes-8: the payment planner's remaining gaps

Implementation item (effort high; review at max). Quote every CR rule and ruling you rely on verbatim; check every ruling of the cards involved, of every date, before calling anything provisional. Background: `docs/agents/engine-gaps.md`, the small-fixes-7 entry ("Residual: a repeatable mana ability is planned at most once per payment"), its "Left:" and "Still not planned at all" paragraphs, and register entries `automatic-mana-payment`, `repeatable-mana-abilities-planned-per-activation`, `castability-is-optimistic`.

Why it matters: `legal_actions` asks the planner whether a spell or ability can be paid for, so a planner gap hides castable spells from agents, and an over-optimistic plan offers a cast that is then reversed (CR 733.1).

1. **Mana abilities the planner never uses** (CR 605, 601.2g–h, 106.4):
   - costs that tap other untapped permanents ("Tap an untapped Elf you control": Birchlore Rangers, Heritage Druid, Urza, Lord High Artificer, Grand Architect, Seton, Relic of Legends, Meria, Baylen, Supportive Parents, The Cabbage Merchant);
   - discard costs (Skirge Familiar) and exile-from-hand costs (Cadaverous Bloom);
   - Squandered Resources, whose mana depends on the sacrificed land (`production_units` finds none while planning).
   Plan them, keeping the planner's preferences (`automatic-mana-payment`; ask before discarding or exiling a card from hand if the register says the player must choose), and keep the permanents tapped for one cost out of the others.
2. **A cost's own other parts:** the plan for a cost's mana doesn't keep what the cost's other parts sacrifice. Repros: Bone Splinters plus Phyrexian Altar with one creature; Fling plus Phyrexian Altar with two creatures. Both are offered now and then reversed.
3. **Convoke and improvise:** a creature convoked or improvised for the spell is still in the planner's sacrifice pools, and convoke's default choice doesn't count the sacrifices.
4. **Food Chain:** planned at one mana per activation; it adds X plus one, where X is the exiled creature's mana value.
5. **Optional, only if small:** consumer chains in an order other than `exec_key`'s.

For each part, add tests that pin the repro, then re-run the every-card sweep for the affected cards only (`--names`, CLAUDE.md's incremental refresh) and commit the tier changes. Report any card whose tier drops. Regenerate docs/STRUCTURE_COVERAGE.md if your changes touch structures (small-fixes-7 left it stale).

`cargo test --workspace` passes; yield row; follow worker.md's one-item-per-session rule and notify the coordinator.
