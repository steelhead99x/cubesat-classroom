//! The one-page printable quick start, `output/pdf/cubesat-classroom-quick-start.pdf`.
//!
//! The installer address comes from `site/src/hubs.mjs` and the AI setup prompt
//! from `docs/quick-start.md`, so the printout always matches the site.

use std::path::Path;

use crate::layout::{Block, Rich, Style, flow};
use crate::pdf::{Face, Page, Rgb, document, text_width};

/// The repository the printout links to.
pub const REPO: &str = "https://github.com/steelhead99x/cubesat-classroom";

const INCH: f64 = 72.0;
const LETTER: [f64; 2] = [8.5 * INCH, 11.0 * INCH];

fn ink() -> Rgb {
    Rgb::hex(0x13_28_4a)
}
fn accent() -> Rgb {
    Rgb::hex(0xea_03_03)
}
fn link() -> Rgb {
    Rgb::hex(0x2f_3f_bf)
}
fn code_bg() -> Rgb {
    Rgb::hex(0xf2_f5_fa)
}
fn code_edge() -> Rgb {
    Rgb::hex(0xc9_d3_e3)
}
fn muted() -> Rgb {
    Rgb::hex(0x4a_58_72)
}

/// `https://<classroomHost>/start.sh`, from the site settings.
///
/// # Errors
/// If `hubs.mjs` has no `classroomHost`.
pub fn installer_url(hubs_mjs: &str) -> Result<String, String> {
    let marker = "export const classroomHost = '";
    let start = hubs_mjs
        .find(marker)
        .ok_or("could not find classroomHost in site/src/hubs.mjs")?
        + marker.len();
    let end = hubs_mjs[start..]
        .find('\'')
        .ok_or("could not find classroomHost in site/src/hubs.mjs")?;
    Ok(format!(
        "https://{}/start.sh",
        &hubs_mjs[start..start + end]
    ))
}

/// The AI setup prompt: the first ```` ```text ```` block in the guide.
///
/// # Errors
/// If the guide has no such block.
pub fn setup_prompt(guide: &str) -> Result<String, String> {
    let missing = || "could not find the AI setup prompt in docs/quick-start.md".to_string();
    let start = guide.find("\n```text\n").ok_or_else(missing)? + "\n```text\n".len();
    let end = guide[start..].find("\n```").ok_or_else(missing)?;
    Ok(guide[start..start + end].to_string())
}

fn body() -> Style {
    Style {
        face: Face::Sans,
        size: 9.5,
        leading: 13.2,
        color: ink(),
        space_before: 0.0,
        space_after: 6.0,
    }
}

fn para(text: Rich) -> Block {
    Block::Para {
        style: body(),
        text,
        indent: 0.0,
        bullet: None,
        boxed: None,
    }
}

fn h2(text: &str) -> Block {
    Block::Para {
        style: Style {
            face: Face::Bold,
            size: 13.0,
            leading: 17.0,
            space_before: 6.0,
            space_after: 4.0,
            ..body()
        },
        text: Rich::new().t(text),
        indent: 0.0,
        bullet: None,
        boxed: None,
    }
}

fn bullets(items: Vec<Rich>) -> Vec<Block> {
    items
        .into_iter()
        .map(|text| Block::Para {
            style: body(),
            text,
            indent: 14.0,
            bullet: Some(("•".into(), 8.0)),
            boxed: None,
        })
        .collect()
}

/// The page's blocks, top to bottom.
#[allow(clippy::too_many_lines)] // one handout, read top to bottom
#[must_use]
pub fn blocks(installer: &str, prompt: &str) -> Vec<Block> {
    let mut out = vec![Block::Para {
        style: Style {
            face: Face::Bold,
            size: 22.0,
            leading: 27.0,
            color: Rgb::hex(0x03_03_8c),
            space_after: 10.0,
            ..body()
        },
        text: Rich::new().t("CubeSat Classroom Quick Start"),
        indent: 0.0,
        bullet: None,
        boxed: None,
    }];
    out.push(h2("Before you build"));
    out.extend(bullets(vec![
        Rich::new()
            .b("Computer.")
            .t(" Use macOS, Linux, or Windows Subsystem for Linux (WSL). Open a terminal on that computer."),
        Rich::new()
            .b("Tools.")
            .t(" Check ")
            .code("curl --version")
            .t(" and ")
            .code("git --version")
            .t(". Ask a mentor to help set up your system’s C compiler and linker (Xcode Command Line Tools on macOS, or build tools such as ")
            .code("build-essential")
            .t(" on Ubuntu)."),
        Rich::new().b("Internet.").t(
            " The first run downloads the project and build dependencies. The starter can install Rust and \
             Cargo. If you already have Rust, use version 1.91 or newer. A computer is enough for the first mission.",
        ),
    ]));
    out.push(h2("1. Start the station"));
    out.push(para(Rich::new().b("With an AI assistant.").t(
        " Copy this prompt into Claude, Codex, Grok, or another assistant.",
    )));
    out.push(Block::Para {
        style: Style {
            size: 8.5,
            leading: 11.0,
            space_after: 10.0,
            ..body()
        },
        text: Rich::new().t(prompt),
        indent: 0.0,
        bullet: None,
        boxed: Some((code_bg(), code_edge(), 0.6, 6.0)),
    });
    out.push(para(Rich::new().b("With your terminal.")));
    out.push(para(
        Rich::new()
            .t("Paste the command. The starter asks which hardware you have, recommends a quest, and explains each download. Press Enter to continue or type ")
            .code("n")
            .t(" at a confirmation to stop."),
    ));
    out.push(Block::Callout {
        style: Style {
            face: Face::Mono,
            size: 9.5,
            leading: 13.0,
            space_after: 0.0,
            ..body()
        },
        text: Rich::new().face(&format!("curl -fsSL {installer} | bash"), Face::Mono),
        width: 6.9 * INCH,
        fill: code_bg(),
        border: (code_edge(), 0.6),
        accent: (accent(), 2.5),
        pad: [10.0, 6.0, 8.0, 8.0],
    });
    out.push(Block::Space(8.0));
    out.push(para(Rich::new().b("Check your work.").t(
        " You should see a recommended quest path and a local web address.",
    )));
    out.push(h2("2. Get a reading"));
    out.push(para(Rich::new().t(
        "Open the address. Request a simulated sensor reading, write down its number and unit, then play the \
         practice video. Press Ctrl+C in the terminal to stop.",
    )));
    out.push(para(Rich::new().b("Check your work.").t(
        " Write one sentence another crew could use to repeat your reading.",
    )));
    out.push(h2("3. Pick the next quest"));
    out.extend(bullets(vec![
        Rich::new().b("M0 · Try it now.").t(
            " Choose computer only, Arduino, Seeed XIAO, or Raspberry Pi. Teachers help test \
             instructions and shape the next lesson.",
        ),
        Rich::new().b("M1 · Plan and review.").t(
            " Close design questions and write test plans with mentors and qualified reviewers.",
        ),
        Rich::new()
            .b("M2 · Verify.")
            .t(" Run approved tests and record evidence for the exact hardware configuration."),
    ]));
    out.push(para(
        Rich::new()
            .t("Open the ")
            .a("quest log", &format!("{REPO}/blob/main/quest/README.md"), link())
            .t(". Read ")
            .b("Goal")
            .t(", ")
            .b("Plan")
            .t(", and ")
            .b("Required")
            .t(
                " if present. Do the task and ask a person to review your evidence. Teachers piloting M0 \
                 can share what worked, where students got stuck, and one change. Use a team nickname in public notes.",
            ),
    ));
    out.push(para(
        Rich::new()
            .t("The Markdown format comes from ")
            .a("moq-dev/quest", "https://github.com/moq-dev/quest", link())
            .t(", which grew out of MoQ. The CLI is optional. If installed, ")
            .code("quest unblocked")
            .t(" lists ready tasks and ")
            .code("quest check")
            .t(" validates the log."),
    ));
    out
}

/// Draw the page frame: a red rule at the top, a grey rule and footer at the bottom.
fn frame(page: &mut Page) {
    let [width, height] = LETTER;
    page.line(
        [0.75 * INCH, height - 0.5 * INCH],
        [width - 0.75 * INCH, height - 0.5 * INCH],
        0.8,
        accent(),
    );
    page.line(
        [0.75 * INCH, 0.62 * INCH],
        [width - 0.75 * INCH, 0.62 * INCH],
        0.5,
        code_edge(),
    );
    page.text(
        0.75 * INCH,
        0.45 * INCH,
        "CubeSat Classroom  /  Contributor quick start",
        Face::Sans,
        7.5,
        muted(),
    );
    let folio = "1 / 1";
    page.text(
        width - 0.75 * INCH - text_width(folio, Face::Sans, 7.5),
        0.45 * INCH,
        folio,
        Face::Sans,
        7.5,
        muted(),
    );
}

/// Build the PDF from the repository at `root`.
///
/// # Errors
/// If a source file is missing or the content runs past the bottom margin.
pub fn build(root: &Path) -> Result<(Vec<u8>, String), String> {
    let read =
        |rel: &str| std::fs::read_to_string(root.join(rel)).map_err(|e| format!("{rel}: {e}"));
    let installer = installer_url(&read("site/src/hubs.mjs")?)?;
    let prompt = setup_prompt(&read("docs/quick-start.md")?)?;
    let mut page = Page::default();
    frame(&mut page);
    let [width, height] = LETTER;
    let (left, right, top, bottom) = (0.75 * INCH, 0.75 * INCH, 0.7 * INCH, 0.8 * INCH);
    let end = flow(
        &mut page,
        &blocks(&installer, &prompt),
        left,
        height - top,
        width - left - right,
    );
    if end < bottom {
        return Err(format!(
            "the quick start runs {:.0} pt past the bottom margin; shorten it to keep one page",
            bottom - end
        ));
    }
    let pdf = document(
        &[page],
        LETTER,
        "CubeSat Classroom Quick Start",
        "CubeSat Classroom",
    );
    Ok((pdf, installer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_site_settings_and_the_prompt() {
        assert_eq!(
            installer_url("export const classroomHost = 'example.org';\n").unwrap(),
            "https://example.org/start.sh"
        );
        assert!(installer_url("nothing").is_err());
        assert_eq!(
            setup_prompt("# Q\n\n```text\nHelp me.\n```\n").unwrap(),
            "Help me."
        );
    }

    #[test]
    fn the_real_quick_start_fits_one_page() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (pdf, installer) = build(&root).expect("quick start builds");
        assert!(installer.ends_with("/start.sh"));
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("/URI (https://github.com/moq-dev/quest)"));
        assert_eq!(text.matches("/Type /Page ").count(), 1);
    }
}
