# CubeSat Classroom Quick Start

## Run the ground station

Paste this into a terminal on macOS, Linux, or WSL. Then open the address it prints.

```sh
curl -fsSL https://steelhead99x.github.io/cubesat-classroom/start.sh | bash
```

Have Git and your system’s build tools installed. The first run downloads the project and installs Rust if needed. If you already have a copy of the project, run `bash docs/start.sh` from that folder.

Try a sensor reading, start the test video, then pause and restore contact. The station uses simulated data. Press Ctrl+C in the terminal to stop it.

## Try one quest

Open `quest/README.md` in the folder the starter prints. **Teach the two-relay lesson** is a first quest you can try without hardware.

- **Codex or Claude Code.** Open that folder in Codex, or run `codex` or `claude` there. Paste the prompt below.
- **Your own LLM.** Share the quest text and relevant public excerpts, then paste the same prompt.

```text
Read quest/m0/repeat-the-station-lesson.md.
Explain my first step and how I can check it.
Wait for me before editing files.
```

Repeat the lesson, write down what happened, and share your notes with a teacher or mentor. Use a team nickname and leave student details and passwords out of prompts and public notes. See [CONTRIBUTING.md](../CONTRIBUTING.md) for the project’s review standards.
