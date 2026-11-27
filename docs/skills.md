# Public AI skills

These skills package what the project learned designing its printed hardware, so an AI assistant can follow the same rules. They're plain folders with a `SKILL.md` each, in [.claude/skills/](../.claude/skills/), under the repository's license.

| Skill | Use it when |
| --- | --- |
| [cubesat-printed-design](../.claude/skills/cubesat-printed-design/SKILL.md) | Designing, changing or reviewing any CubeSat Classroom 3D-printed part. A summary of the [hardware design guide](../hardware/DESIGN-GUIDE.md): no loose pieces where possible, rocket-launch ready and durable, easy enough for kids. |
| [printed-fit-coupons](../.claude/skills/printed-fit-coupons/SKILL.md) | Before printing a part with a new joint: a few-gram coupon from the same code, a fit ladder, and recording the result |
| [rust-cad-with-checks](../.claude/skills/rust-cad-with-checks/SKILL.md) | Building parametric print CAD in Rust: every rule a check, a broken-design self-test, reproducible output |
| [bambu-studio-projects](../.claude/skills/bambu-studio-projects/SKILL.md) | Writing Bambu Studio .3mf projects from code |

## Using them

- **Claude Code in this repository:** they load by themselves from `.claude/skills/`.
- **Another repository:** copy a skill's folder into that repository's `.claude/skills/`.
- **Claude apps:** zip a skill's folder and add it as a skill in Claude's settings, if your account offers that.
- **Other assistants:** the `SKILL.md` files are Markdown. [AGENTS.md](../AGENTS.md) points every assistant to the design guide.

The design guide is the source. When it changes, update `cubesat-printed-design` to match. Keep learner information out of skills, like everything else in the project.
