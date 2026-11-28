"""Purchasing specs for the CSC-R1P passive receiver and its controls.

Keep footprint, value and fit in the design. Assemblers return exact MPNs for
specification rows and obtain approval before substituting named RF parts.
"""

RESISTOR_0402 = 'R_0402_1005Metric'
CAPACITOR_0402 = 'C_0402_1005Metric'
CAPACITOR_0603 = 'C_0603_1608Metric'
CAPACITOR_0805 = 'C_0805_2012Metric'
CAPACITOR_1206 = 'C_1206_3216Metric'

# Values are the same as in the schematic. The PCB package stays in a separate
# column in the generated BOM.
CAPACITORS = {
    **{(CAPACITOR_0402, value): 'C0G, 50 V minimum, ±0.25 pF or tighter'
       for value in ('4p7', '6p8', '8p2')},
    **{(CAPACITOR_0402, value): 'C0G, 50 V minimum, ±5% or tighter'
       for value in ('10p', '12p', '18p', '22p', '33p', '680p', '1n', '2n2')},
    (CAPACITOR_0402, '3n9'): 'Murata GRM1555C1H392GE01D or C0G, 50 V, ±5% or tighter',
    (CAPACITOR_0402, '10n'): 'X7R, 25 V minimum, ±10%',
    (CAPACITOR_0402, '100n'): 'X7R, 16 V minimum, ±10%',
    (CAPACITOR_0603, '100n'): 'X7R, 25 V minimum, ±10%',
    (CAPACITOR_0603, '180n'): 'X7R, 25 V minimum, ±10%; confirm PLL loop filter bias',
    (CAPACITOR_0603, '1u'): 'X7R, 25 V minimum, ±10%',
    (CAPACITOR_0603, '2u2'): 'X7R, 16 V minimum, ±10%; check DAC charge-pump limits',
    (CAPACITOR_0805, '10u'): 'X7R, 16 V minimum, ±20%; check effective capacitance at rail bias',
    (CAPACITOR_0805, '22u'): 'X7R, 10 V minimum, ±20%; check effective capacitance at 5.09 V',
    (CAPACITOR_1206, '10u'): 'X7R, 25 V minimum, ±10%; check effective capacitance at 9 V',
}

# 2% ceramic-core wirewounds. Use the manufacturer model when checking a
# proposed alternate for Q, self-resonance and filtered response.
RF_INDUCTORS_0402 = {
    f'{nh}nH': f'Coilcraft 0402DC-{nh}NXGRW'
    for nh in (15, 22, 39, 47, 56)
}

LEDS_0603 = {
    'PWR green': 'Green LED, pad 1 cathode, rated for 2 mA operation',
    'BIAS-T amber': 'Amber LED, pad 1 cathode, rated for 3 mA operation',
}


HAND_FIT = {
    'PinSocket_1x07_P2.54mm_Vertical': '1×7 vertical through-hole socket, 2.54 mm pitch; fit to XIAO and check USB clearance',
    'PinHeader_2x04_P2.54mm_Vertical': '2×4 vertical through-hole header, 2.54 mm pitch; select with control socket',
    'PinSocket_2x04_P2.54mm_Vertical': '2×4 vertical through-hole socket, 2.54 mm pitch; underside fit, verify engagement at 15 mm board spacing',
}


def specification(part):
    """Return the sourcing line for a fitted pedal part with a blank MPN."""
    footprint = part['fp'].split(':')[1]
    value = part['value']
    if footprint == RESISTOR_0402:
        if value == '0R':
            return 'Zero-ohm jumper, 0.5 A minimum'
        if value == '100R':
            return 'Panasonic ERJ2RKF1000X or ±1%, 0.10 W minimum'
        return 'Thick-film resistor, ±1%, 0.063 W minimum, 25 V minimum'
    if footprint == 'L_0402_1005Metric':
        return RF_INDUCTORS_0402[value]
    if footprint == 'L_0603_1608Metric' and value == '470nH':
        return 'Coilcraft 0603DC-R47XGRW'
    if footprint == 'LED_0603_1608Metric':
        return LEDS_0603[value]
    if footprint in HAND_FIT:
        return HAND_FIT[footprint]
    return CAPACITORS[footprint, value]
