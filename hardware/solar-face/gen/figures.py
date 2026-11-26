"""Desk figures for both strings, from cells.py. Printed for the README; nothing here is measured."""
import design
for v in ('si', 'tj'):
    design.load(v); f = design.string_figures()
    print(f"{v}: {f['cell_count']} cells, Voc {f['voc']:.2f} V, Vmp {f['vmp']:.2f} V, Imp {f['imp'] * 1000:.0f} mA, "
          f"Pmp {f['pmp']:.2f} W at {f['sun']} (desk figure from the maker's data)")
