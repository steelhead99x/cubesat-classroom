# Private MoQ.pro development demos

Teachers, students, and volunteers can use the local classroom simulator. The hosted demos are optional, use generated video, and are for developers and adult mentors using their own private MoQ.pro project.

## Set up a private project

1. Create or sign in to your own account at [MoQ.pro](https://moq.pro/). Create a project with anonymous publishing, anonymous subscribing, and public discovery disabled. Check the current service terms and usage limits before testing. If the project cannot enforce these settings, use the local classroom simulator.
2. Create two short-lived credentials scoped to `cubesat-classroom-test.hang` in that project. Give the first publish permission and the second subscribe permission. Keep both in an adult-controlled environment.
3. Install the [MoQ CLI](https://doc.moq.dev/setup/install/) with `MOQ_CONNECT` support (such as 0.14.17) and FFmpeg with `libx264`. Install FFplay to watch the video. Run the commands below from the repository root and replace `YOUR_PRIVATE_PROJECT` with the dashboard project ID.

## Publish and watch

Run the publisher first. The script prompts for the publisher credential and generates a 45-second video pattern.

```sh
bash scripts/moq-pro-demo.sh publish YOUR_PRIVATE_PROJECT
```

Run the subscriber in another terminal while the publisher is running. Enter the subscribe-only credential.

```sh
bash scripts/moq-pro-demo.sh watch YOUR_PRIVATE_PROJECT
```

For a terminal-only smoke test, run `bash scripts/moq-pro-demo.sh check YOUR_PRIVATE_PROJECT`. It verifies that a client received and decoded video. Press Ctrl+C to stop an idle viewer or check.

## Verify access before a class demo

With the pattern running, use the MoQ CLI from a separate client with no credentials and confirm it cannot subscribe. Repeat with an expired credential. Check that the subscriber credential cannot publish and that the test broadcast is absent from unauthenticated discovery. Stop the exercise if any check fails. Share the date, client version and outcomes with your mentor without including tokens or test account identifiers.

The demo uses TLS-protected connections to the relay and authorization per role. The relay handles the media. Older MoQ versions send the short-lived token in the HTTPS URL, which the provider may log. Use generated video and keep student images, voices, and details out of hosted tests. CubeSat Classroom does not collect MoQ.pro credentials.

## Other MoQ providers

Pull requests for private, authenticated relay demos on other MoQ providers are welcome as future work. Include the CLI and protocol versions, exact test steps, and evidence of successful playback and denied unauthorized access. Start with the [contribution guide](../CONTRIBUTING.md).
