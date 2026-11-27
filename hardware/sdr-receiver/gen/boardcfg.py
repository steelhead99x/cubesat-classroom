"""Which board the generator scripts work on. Set CSC_BOARD (default csc-r1, the dev kit).

    CSC_BOARD=csc-r1p     ./build.sh     # pedal main board
    CSC_BOARD=csc-r1p-ctl ./build.sh     # pedal control board

Every script reads its design source, KiCad folder and fab folder from here, so one generator
serves all three boards and a fix in a shared script reaches all of them.
"""
import importlib
import os

BOARDS = {
    'csc-r1': dict(
        design='design', placer='gen_pcb.py', kicad='../kicad', fab='../fab',
        edge_ref='J3', edge_max=5,   # SMA edge-launch pads reach the board edge by design
        render=(0, 0, 120, 78), router_passes=8,
    ),
    'csc-r1p': dict(
        design='design_r1p', placer='gen_pcb_r1p.py', kicad='../pedal/kicad', fab='../pedal/fab/main',
        edge_ref=None, edge_max=0,
        render=(0, 0, 54, 104), router_passes=12,
    ),
    'csc-r1p-ctl': dict(
        design='design_r1p_ctl', placer='gen_pcb_r1p_ctl.py', kicad='../pedal/kicad', fab='../pedal/fab/controls',
        edge_ref=None, edge_max=0,
        render=(0, 0, 54, 72), router_passes=6,
        # Routed by hand with Freerouting; build.sh would replace that copper with the maze
        # router's, so it refuses unless CSC_REBUILD_ROUTED=1.
        hand_routed=True,
    ),
}

NAME = os.environ.get('CSC_BOARD', 'csc-r1')
if NAME not in BOARDS:
    raise SystemExit(f'CSC_BOARD={NAME!r}: expected one of {", ".join(BOARDS)}')
CFG = BOARDS[NAME]
PROJ = NAME
KICAD = CFG['kicad']
FAB = CFG['fab']
EDGE_REF = CFG['edge_ref']
EDGE_MAX = CFG['edge_max']
PCB = f'{KICAD}/{PROJ}.kicad_pcb'
SCH = f'{KICAD}/{PROJ}.kicad_sch'
BASE = f'{KICAD}/base.kicad_pcb' if NAME == 'csc-r1' else f'{KICAD}/{PROJ}-base.kicad_pcb'
NET = f'{KICAD}/{PROJ}-rt.net' if NAME != 'csc-r1' else f'{KICAD}/rt.net'


def design():
    return importlib.import_module(CFG['design'])


if __name__ == '__main__':
    # Lets shell scripts read settings: python3 boardcfg.py KEY
    import sys
    key = sys.argv[1]
    print({'design': CFG['design'], 'proj': PROJ, 'kicad': KICAD, 'fab': FAB, 'placer': CFG['placer'], 'pcb': PCB, 'sch': SCH, 'net': NET,
           'passes': CFG['router_passes'], 'hand_routed': int(CFG.get('hand_routed', False)), 'render': ' '.join(map(str, CFG['render']))}[key])
