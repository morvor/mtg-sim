# ruling-conflicts-2: follow-up conflict cases found by rulings batches

Same method, conclusion types and actions as `ruling-conflicts-1.md` (an independent investigator and an adversarial skeptic for each case, at effort max; quote the sources with dates; conclusions A/B/C/D; at most two subagents at a time). Add a STATUS section at the end when done.

**Cases (Nov 5).**
1. **Ghost Council of Orzhova** against CR 400.7j. Reported by rulings-1 while landing P128/P091 (d948fbee, ccb35825). Find the ruling with `zcat data/rulings.jsonl.gz | grep -i "ghost council"`.
2. **Homicide Investigator.** The 2024-04-12 ruling says the ability "triggers for each of them". The Oracle text says "Whenever one or more nontoken creatures you control die, investigate. This ability triggers only once each turn." The engine follows the Oracle text (844da5ba, rulings-P005), and the ruling is uncited.
3. **Others from rulings-P005.** Any further rulings the rulings-2 worker left uncited as apparent conflicts, listed in docs/agents/engine-gaps.md under "Apparent conflicts (Nov 5)". If that section doesn't exist yet when you start, check `git log --grep '^rulings-P005:'` and the uncited rulings of batch P005 (`python3 scripts/rulings_batches.py show P005 --open`).

New cases reported by later batches are added here by the coordinator.

DONE when every case has a conclusion as in ruling-conflicts-1, the engine and tests follow A/B/C, the register and exemptions say why, and `cargo test --workspace` passes.
