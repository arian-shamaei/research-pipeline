# Figure Design Rules for Research Pipeline

Synthesized from: Rougier et al. "Ten Simple Rules for Better Figures" (PLOS Comp Bio 2014),
IEEE Author Center guidelines, Simplified Science Publishing, Anna Clemens, and user feedback.

## Core Principles

1. **One message per figure.** If a figure tries to say two things, split it into two figures.
2. **Message at first glance.** A reader should understand the figure's point in under 5 seconds.
3. **Never trust defaults.** Every axis, font, color, margin, and tick must be intentionally set.

## Layout & Composition

4. **Fill the frame.** Data should occupy 70-80% of the figure area. Minimize dead whitespace.
5. **Flow left-to-right, top-to-bottom.** For block diagrams, the primary data path reads L->R.
6. **Align to an invisible grid.** All boxes, labels, and elements snap to consistent vertical/horizontal lines.
7. **Balance the composition.** No side should feel heavier than the other. Distribute elements evenly.
8. **Separate overlapping elements.** If two data series overlap, offset them vertically or use transparency. Never let one series completely obscure another.

## Text & Typography

9. **Font size 8-10pt in final print.** IEEE recommends 9-10pt. Nature: 8pt, Science: 9pt.
10. **Match figure fonts to paper body.** Use the same font family (Sans/Helvetica for figures in a Times body is acceptable per IEEE).
11. **All text must be horizontal.** Rotated y-axis labels are acceptable but minimize all other rotated text.
12. **Center text within boxes.** For block diagrams, text is horizontally AND vertically centered in its container.
13. **Spell out acronyms** on first use in figure, or keep to well-known ones (dB, Hz, MHz, etc.).

## Axes & Ticks

14. **Always label axes with units.** Format: "Quantity (Unit)" e.g., "Frequency (Hz)".
15. **Let data fill the axis range.** Don't waste 40% of y-axis on empty space above/below data.
16. **Reduce tick count.** 4-6 ticks per axis maximum. Remove minor ticks unless needed.
17. **Format numbers cleanly.** Use "$20k" not "$20000k". Use SI prefixes where appropriate.
18. **Dual-axis charts: separate visually.** Color-code each y-axis label to match its data series. Consider offsetting the secondary series above/below to prevent overlap.

## Color

19. **Okabe-Ito palette by default.** Color-blind friendly: #0072B2 #E69F00 #009E73 #CC79A7 #D55E00 #56B4E9 #F0E442.
20. **Use color to highlight, gray to recede.** Only the primary message gets saturated color; supporting elements are gray or muted.
21. **Verify in grayscale.** Print the figure in B&W -- can you still distinguish all elements?
22. **Limit to 4-5 colors maximum.** More than that creates visual confusion.

## Bar Charts

23. **Label bars with values.** Either inside the bar (if wide enough) or directly above/beside.
24. **Bars must have equal width.** Unequal bar widths mislead about data proportions.
25. **Start y-axis at zero** for bar charts (unlike line charts which can zoom).
26. **Sort bars meaningfully.** By value (ascending/descending) or by category logic, not randomly.
27. **Add subtle gridlines.** Horizontal gridlines at y-ticks help read bar heights. Keep them light (#e0e0e0).

## Line Charts

28. **Distinct line styles.** Use solid, dashed, dot-dashed -- not just color to differentiate.
29. **Line width 1.5-2.5pt** for data, 0.5-1pt for grid/reference lines.
30. **Add data point markers** for discrete measurements. Omit for continuous/simulated curves.
31. **Legend inside plot** in empty region, OR label lines directly if <=3 series.

## Block Diagrams / Architecture Figures

32. **All boxes same height** within a tier. All boxes same width within a column.
33. **Consistent padding.** Text-to-box-edge padding is the same for every box (e.g., 8% of box width).
34. **Arrows: straight lines only.** Avoid curved or diagonal arrows when a right-angle path works. Use consistent arrowhead size.
35. **Arrow routing: no overlap.** Arrows must not cross boxes or other arrows when avoidable. Route around obstacles.
36. **Group related elements** with background shading regions. Label each region.
37. **Consistent box colors** by function type: sensors=blue, processing=green, output=purple, etc.
38. **Minimum 3 elements per tier** to justify a full-width tier. Fewer than 3 should merge with adjacent tier.

## Dual-Axis / Combo Charts

39. **Color-code axis labels** to match their data series.
40. **Offset overlapping series.** If bars and lines overlap, place the line slightly above (small y-offset) or use transparency on bars.
41. **Never use dual-axis with same scale.** If both axes measure the same thing, use one axis.

## Verification Checklist (Stage 4: VERIFY)

Before accepting a figure, ALL must pass:

- [ ] Message readable in <5 seconds
- [ ] No clipped text or elements at edges
- [ ] All bars equal width (bar charts)
- [ ] All values labeled on bars (bar charts)
- [ ] Axes labeled with units
- [ ] Tick labels formatted cleanly (no "10kk", no overlapping)
- [ ] Legend doesn't overlap data
- [ ] All text >=8pt in final print size
- [ ] Colors distinguishable in grayscale
- [ ] Box text centered (block diagrams)
- [ ] Arrows don't cross boxes or each other (block diagrams)
- [ ] No more than 30% dead whitespace
- [ ] Figure fills its allocated column width (3.5in or 7.16in)
