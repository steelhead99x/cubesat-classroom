#!/usr/bin/env python3
"""Build the one-page printable quick start, output/pdf/cubesat-classroom-quick-start.pdf.

The installer address comes from site/src/hubs.mjs, so the printout always matches the site.
Needs ReportLab (`pip install reportlab`) and the DejaVu fonts, falling back to ReportLab's
bundled Vera fonts, which DejaVu is based on.

    python3 scripts/quick_start_pdf.py
"""
import re
from pathlib import Path

from reportlab.lib.colors import HexColor
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.pagesizes import letter
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.units import inch
from reportlab.lib.fonts import addMapping
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import ListFlowable, ListItem, Paragraph, SimpleDocTemplate, Spacer, Table, TableStyle

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "output" / "pdf" / "cubesat-classroom-quick-start.pdf"
REPO = "https://github.com/steelhead99x/cubesat-classroom"

INK = HexColor("#13284a")
ACCENT = HexColor("#c8102e")
LINK = HexColor("#1d4ed8")
CODE_BG = HexColor("#f2f5fa")
CODE_EDGE = HexColor("#c9d3e3")
MUTED = HexColor("#4a5872")


def installer_url() -> str:
    hubs = (ROOT / "site" / "src" / "hubs.mjs").read_text()
    host = re.search(r"export const classroomHost = '([^']+)'", hubs)
    if not host:
        raise SystemExit("could not find classroomHost in site/src/hubs.mjs")
    return f"https://{host.group(1)}/start.sh"


def register_fonts() -> tuple[str, str, str]:
    dejavu = Path("/usr/share/fonts/truetype/dejavu")
    faces = {"Sans": "DejaVuSans.ttf", "Sans-Bold": "DejaVuSans-Bold.ttf", "Mono": "DejaVuSansMono.ttf"}
    if all((dejavu / f).exists() for f in faces.values()):
        for name, file in faces.items():
            pdfmetrics.registerFont(TTFont(name, str(dejavu / file)))
        # Let <b> in paragraphs switch to the bold face.
        addMapping("Sans", 0, 0, "Sans")
        addMapping("Sans", 1, 0, "Sans-Bold")
        addMapping("Sans", 0, 1, "Sans")
        addMapping("Sans", 1, 1, "Sans-Bold")
        return "Sans", "Sans-Bold", "Mono"
    return "Helvetica", "Helvetica-Bold", "Courier"


def build() -> None:
    sans, bold, mono = register_fonts()
    body = ParagraphStyle("body", fontName=sans, fontSize=9.5, leading=13.2, textColor=INK, alignment=TA_LEFT, spaceAfter=6)
    title = ParagraphStyle("title", parent=body, fontName=bold, fontSize=22, leading=27, spaceAfter=10)
    h2 = ParagraphStyle("h2", parent=body, fontName=bold, fontSize=13, leading=17, spaceBefore=6, spaceAfter=4)
    code = ParagraphStyle("code", parent=body, fontName=mono, fontSize=9.5, leading=13, spaceAfter=0)

    def c(text: str) -> str:
        return f'<font name="{mono}" size="8.5">{text}</font>'

    def a(text: str, href: str) -> str:
        return f'<a href="{href}" color="#1d4ed8"><u>{text}</u></a>'

    def bullets(items: list[str]) -> ListFlowable:
        return ListFlowable(
            [ListItem(Paragraph(i, body), leftIndent=14, value="•") for i in items],
            bulletType="bullet", bulletFontName=sans, bulletFontSize=8, leftIndent=14, bulletOffsetY=-1,
        )

    command = Table([[Paragraph(f"curl -fsSL {installer_url()} | bash", code)]], colWidths=[6.9 * inch])
    command.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, -1), CODE_BG),
        ("BOX", (0, 0), (-1, -1), 0.6, CODE_EDGE),
        ("LINEBEFORE", (0, 0), (0, -1), 2.5, ACCENT),
        ("LEFTPADDING", (0, 0), (-1, -1), 10),
        ("TOPPADDING", (0, 0), (-1, -1), 8),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 8),
    ]))

    story = [
        Paragraph("CubeSat Classroom Quick Start", title),
        Paragraph("Before you build", h2),
        bullets([
            "<b>Computer.</b> Use macOS, Linux, or Windows Subsystem for Linux (WSL). Open a terminal on that computer.",
            f"<b>Tools.</b> Check {c('curl --version')} and {c('git --version')}. Ask a mentor to help set up your system’s C "
            f"compiler and linker (Xcode Command Line Tools on macOS, or build tools such as {c('build-essential')} on Ubuntu).",
            "<b>Internet.</b> The first run downloads the project and build dependencies. The starter can install Rust and "
            "Cargo. If you already have Rust, use version 1.91 or newer. A computer is enough for the first mission.",
        ]),
        Paragraph("1. Start the build", h2),
        Paragraph("Paste the command. The starter asks which hardware you have, recommends a quest, and explains each "
                  f"download. Press Enter to continue or type {c('n')} at a confirmation to stop.", body),
        command,
        Spacer(1, 8),
        Paragraph("<b>Check your work.</b> You should see a recommended quest path and a local web address.", body),
        Paragraph("2. Get a reading", h2),
        Paragraph("Open the address. Request a simulated sensor reading, write down its number and unit, then play the "
                  "practice video. Press Ctrl+C in the terminal to stop.", body),
        Paragraph("<b>Check your work.</b> Write one sentence another crew could use to repeat your reading.", body),
        Paragraph("3. Pick the next quest", h2),
        bullets([
            "<b>M0 · Try it now.</b> Choose computer only, Arduino, Seeed XIAO, or Raspberry Pi. Teachers help test "
            "instructions and shape the next lesson.",
            "<b>M1 · Plan and review.</b> Close design questions and write test plans with mentors and qualified reviewers.",
            "<b>M2 · Verify.</b> Run approved tests and record evidence for the exact hardware configuration.",
        ]),
        Paragraph(f"Open the {a('quest log', REPO + '/blob/main/quest/README.md')}. Read <b>Goal</b>, <b>Plan</b>, and "
                  "<b>Required</b> if present. Do the task and ask a person to review your evidence. Teachers piloting M0 "
                  "can share what worked, where students got stuck, and one change. Use a team nickname in public notes.", body),
        Paragraph(f"The Markdown format comes from {a('moq-dev/quest', 'https://github.com/moq-dev/quest')}, which grew out "
                  f"of MoQ. The CLI is optional. If installed, {c('quest unblocked')} lists ready tasks and {c('quest check')} "
                  "validates the log.", body),
        Paragraph("<b>With Codex, Claude Code, or another LLM.</b> Open the checkout or share the public quest. Ask it to "
                  "explain the first step and how to check it, then have it wait before editing files.", body),
    ]

    def frame(canvas, doc) -> None:
        width, height = letter
        canvas.saveState()
        canvas.setStrokeColor(ACCENT)
        canvas.setLineWidth(0.8)
        canvas.line(0.75 * inch, height - 0.5 * inch, width - 0.75 * inch, height - 0.5 * inch)
        canvas.setStrokeColor(CODE_EDGE)
        canvas.setLineWidth(0.5)
        canvas.line(0.75 * inch, 0.62 * inch, width - 0.75 * inch, 0.62 * inch)
        canvas.setFont(sans, 7.5)
        canvas.setFillColor(MUTED)
        canvas.drawString(0.75 * inch, 0.45 * inch, "CubeSat Classroom  /  Contributor quick start")
        canvas.drawRightString(width - 0.75 * inch, 0.45 * inch, f"{doc.page} / 1")
        canvas.restoreState()

    OUT.parent.mkdir(parents=True, exist_ok=True)
    doc = SimpleDocTemplate(
        str(OUT), pagesize=letter, title="CubeSat Classroom Quick Start", author="CubeSat Classroom",
        leftMargin=0.75 * inch, rightMargin=0.75 * inch, topMargin=0.7 * inch, bottomMargin=0.8 * inch,
    )
    doc.build(story, onFirstPage=frame, onLaterPages=frame)
    print(f"Wrote {OUT.relative_to(ROOT)} with installer {installer_url()}")


if __name__ == "__main__":
    build()
