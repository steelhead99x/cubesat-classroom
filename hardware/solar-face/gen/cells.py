"""Solar cells the 1U face can carry, with their sources.

Every number here comes from the listed source. `verified` stays False until someone checks the
footprint's pads against the maker's drawing; export.sh refuses to write Gerbers while any cell used
by a board is unverified, so an unchecked land pattern can't be ordered by mistake.
"""

CELLS = {
    'CIC_3G30A_8x4': dict(
        name='AZUR SPACE 3G30A-Advanced 8 x 4 cm CIC',
        what='Triple-junction InGaP/GaAs/Ge space cell assembly: cell, built-in silicon bypass diode, '
             'welded interconnectors and coverglass',
        body=(80.15, 40.15),      # assembly outline, mm (satnow listing of the 8 x 4 assembly; confirm on AZUR drawing)
        voc=2.700,                # V, BOL, AM0, 28 C (AZUR 3G30-Advanced datasheets)
        vmp=2.409,                # V  (AZUR 3G30A 8 x 4 datasheet HNR 0003401, BOL)
        imp=0.5029,               # A  (same)
        sun='AM0, 1367 W/m2, 28 C, beginning of life',
        source='AZUR SPACE data sheet HNR 0003401-01-01 (3G30A 8 x 4 cm CIC); AZUR 3G30-Advanced family data sheets',
        verified=False,
        why='Rear-contact polarity and the position and length of the welded interconnectors are taken as '
            'rear = +, interconnectors along one long edge. Check both against the AZUR drawing that comes '
            'with the cells before bonding or ordering boards.',
    ),
    'KXOB25_05X3F': dict(
        name='ANYSOLAR KXOB25-05X3F SolarBIT',
        what='Surface-mount monocrystalline silicon cell, three cells in series inside, 25% class',
        body=(23.0, 8.0),         # mm (DigiKey listing: 23.00 x 8.00 x 1.80 mm)
        voc=2.07,                 # V (DigiKey listing)
        vmp=1.67,                 # V (DigiKey listing)
        imp=0.0184,               # A (DigiKey listing)
        sun='1000 W/m2 (maker rating, Earth sunlight)',
        source='DigiKey KXOB25-05X3F-TR listing; ANYSOLAR KXOB25-05X3F data sheet (land pattern still to check)',
        verified=False,
        why='The two pads are placed at the ends of the body as a placeholder. Copy the land pattern '
            'from the ANYSOLAR data sheet and set verified=True before ordering boards.',
    ),
}
